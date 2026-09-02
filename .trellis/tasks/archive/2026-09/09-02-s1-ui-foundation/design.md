# Design - S1 UI 基础

## 1. Delivery Boundary

本任务把现有临时 runtime surface 收敛为可持续扩展的 UI 平台，但不交付最终 app shell：

```text
Rust ConfigService
  -> document-start appearance bootstrap
  -> generated config snapshot/event/update
  -> UiPreferencesProvider
  -> <html> theme/density/lang

bundled zh-CN resources
  -> I18nextProvider
  -> current runtime/titlebar + future product components

theme.css tokens
  -> Tailwind v4 utilities
  -> project-owned UI primitives
  -> current runtime/titlebar migration
```

父任务的 routing/actions/feedback 子任务只消费这些边界，不要求 UI foundation 建临时 route、设置页或 showcase。

## 2. Dependencies And Tooling

### Runtime dependencies

- `i18next@26.4.0`
- `react-i18next@17.0.13`
- `@radix-ui/react-dialog@1.1.23`
- `@radix-ui/react-dropdown-menu@2.1.24`
- `@radix-ui/react-tooltip@1.2.16`
- `@radix-ui/react-slot@1.3.3`
- `class-variance-authority@0.7.1`
- `clsx@2.1.1`
- `tailwind-merge@3.6.0`

### Development dependencies

- `prettier@3.9.6`
- `prettier-plugin-tailwindcss@0.8.1`

Versions are exact. `lucide-react`, Testing Library and jsdom are reused. `react-router`, shadcn CLI, `radix-ui`, animation/theme libraries and online fonts are excluded.

Prettier owns frontend source formatting but not generated bindings. `package.json` adds stable `format`/`format:check`; CI runs check mode only. No additional pnpm version source is introduced.

## 3. Native First-Frame Appearance

### 3.1 Snapshot ownership

`src-tauri/src/lib.rs` changes its initial config extraction from a window-only tuple to one snapshot projection:

```rust
struct InitialConfig {
    window: WindowConfig,
    ui: UiConfig,
    error_code: Option<AppErrorCode>,
}
```

The exact helper shape may remain private, but one `ConfigService::snapshot()` must feed both window placement and UI bootstrap. Failure uses both defaults and one redacted notice/log record.

### 3.2 Bootstrap module

`src-tauri/src/ui_bootstrap.rs` owns a pure projection from `UiConfig` to:

- a fixed document-start JavaScript string;
- optional native `tauri::Theme` for explicit light/dark;
- optional initial background color matching surface-1.

The script is a closed enum projection. It checks top-level context, waits only until `<html>` exists, writes `data-theme`, `data-density`, `lang`, `colorScheme`, and `.dark`, then disconnects its observer. It never interpolates a path, locale from arbitrary text, config serialization, or user content.

The builder applies initialization script before `.build()`. Normal visibility remains false until window placement/coordinator logic finishes, preserving prior no-flash and `/tray` guarantees.

### 3.3 Browser fallback

`theme.css` keeps system color media values as a fallback for ordinary Vite browser rendering where no native initialization script exists. Once `data-theme`/`.dark` is present, explicit theme tokens win. This fallback is presentation-only and does not persist state.

## 4. UI Preferences Runtime

### 4.1 Typed client

`src/lib/config-client.ts` wraps only generated bindings:

```typescript
interface UiConfigClient {
  fetchSnapshot(): Promise<ConfigSnapshot>;
  updateUi(ui: UiConfig): Promise<ConfigSnapshot>;
  listenChanged(listener: (snapshot: ConfigSnapshot) => void): Promise<() => void>;
}
```

No local wire interface duplicates `ConfigSnapshot` or `UiConfig`; the interface names methods for injection while signatures import generated types.

### 4.2 Controller/provider

`src/app/providers/ui-preferences-provider.tsx` owns:

```typescript
interface UiPreferencesContextValue {
  status: "loading" | "ready" | "failed";
  ui: Required<Pick<UiConfig, "theme" | "density" | "locale">> & UiConfig;
  warning: string | null;
  setTheme(theme: ThemePreference): Promise<void>;
  setDensity(density: Density): Promise<void>;
  clearWarning(): void;
}
```

Final types may use aliases from generated bindings, but no handwritten union. Context value is memoized. A small controller/reducer isolates revision merge and serialized updates for deterministic tests.

Initialization:

1. Read bootstrap attributes for a stable initial render.
2. Register config listener.
3. Fetch snapshot.
4. Merge by revision and apply authoritative UI config.
5. Register/remove system theme media listener based on current preference.

Update:

1. Construct a full UI group from latest pending state plus one patch.
2. Apply DOM optimistically in the same tick.
3. Append update to one promise queue.
4. Merge returned/event snapshots by revision.
5. On failure, restore latest confirmed config and expose a translated, bounded warning.

This queue prevents theme, density and future sidebar changes from overwriting one another through stale full-group updates.

### 4.3 Provider composition

```text
I18nProvider
  UiPreferencesProvider
    OverlayProvider / TooltipProvider
      RuntimeApp (later AppRouter)
```

Locale currently cannot change away from zh-CN, but root `lang` and i18next language still derive from generated `AppLocale`.

## 5. Token Architecture

`src/styles/theme.css` is imported before component/application styles. It contains:

- Tailwind import and dark/compact variants;
- `@theme inline` semantic mappings;
- light `:root`, explicit `.dark`, and system fallback values;
- UI/mono/etymon font stacks;
- spacing/radius/control/elevation/z-index/motion tokens;
- focus/base body styles and reduced-motion policy.

The token names preserve the product vocabulary (`surface-1..3`, `text-1..3`, `border`, `border-strong`, `wubi-zone-1..5`) rather than shadcn's generic card/popover labels. Primitive wrappers map product tokens to their states; they do not create a second shadcn palette.

Current `runtime-status.css` either becomes a narrow layout stylesheet using these tokens or is replaced with Tailwind utilities. It must no longer define theme colors, font stacks, radius or motion constants. The title bar keeps its native-specific layout contract while consuming the same global tokens.

## 6. Internationalization

`src/i18n/resources/zh-CN.ts` exports a const resource split into `common`, `window`, `runtime`, and `ui` namespaces. `src/i18n/index.ts` initializes one bundled i18next instance synchronously and exports its resource types for module augmentation.

Rules:

- Frontend-authored visible text, aria labels, tooltip text and fallback warnings are keys.
- Dynamic values use interpolation; `escapeValue=false` only because React escapes.
- Backend-provided message/detail remains data and is rendered as-is.
- View projection helpers accept a narrow translator, avoiding an untestable singleton dependency.
- Tests may use the production zh-CN instance or an injected fixed translator; they do not duplicate Chinese literals across fixtures without purpose.

## 7. Primitive Architecture

### 7.1 Utilities

`src/lib/cn.ts` is the only `clsx + tailwind-merge` helper. Variants live beside the component that owns them; no global variant registry.

### 7.2 Button and Input

Button provides semantic variants and sizes, defaults `type="button"`, supports Slot composition, keeps a stable hit area, and combines `disabled` with `aria-busy` for loading. It does not render spinner text or invent async behavior.

Input is a styled native input. Consumers own visible label/help/error composition; Input exposes correct attributes/styles for invalid, disabled and read-only. No placeholder-only label pattern is provided.

### 7.3 Overlay primitives

Dialog, Dropdown Menu and Tooltip wrap Radix primitives with product tokens and named exports. The wrappers keep Radix ownership of focus management, Escape, roving keyboard selection and aria relationships.

`#overlay-root` is created in `index.html`. All project portals target it through a shared helper/context. TooltipProvider is mounted once at app level. Dialog requires title and visible/accessible close; Menu exposes group/separator; Tooltip remains supplementary and cannot be the accessible name.

### 7.4 Kbd

Kbd renders semantic shortcut text with the mono font and tabular layout. It has no click handler and no shortcut parsing/recording responsibilities.

## 8. Existing Surface Migration

- `main.tsx` becomes bootstrap/composition only; RuntimeApp may move to a named component/module to make providers testable without altering visible behavior.
- WindowTitleBar consumes translation keys and global tokens while retaining its generated state/intent props, drag-region isolation, icon states and 44x44 tests.
- Runtime status presentations externalize frontend strings, preserve runtime-generated notices, and merge UI preference warning into the existing visible warning section.
- No sidebar, app bar, route outlet, page placeholder or appearance setting is added.

## 9. Validation Design

### Rust

- Pure ui bootstrap tests for every enum combination and default.
- Assert fixed root attributes/classes, explicit native theme/background selection, top-level guard and no arbitrary config serialization.
- Existing desktop/window tests remain green; runtime smoke exercises the real WebView builder path.

### Frontend unit/component

- DOM projection and media listener tests for system/light/dark and cleanup.
- Listener-first/revision/event-before-snapshot/update queue/rollback/StrictMode cleanup tests.
- i18n initialization, interpolation and locale fallback tests.
- Button/Input/Kbd semantics and variants.
- Dialog open/close/Escape/focus restore; Menu arrows/Enter/Escape/disabled; Tooltip hover/focus/provider/portal.
- Existing RuntimeApp, titlebar, window Hook and feature store regression tests.

### Static and visual

- Parse CSS token blocks and calculate required contrast pairs.
- Search for Tailwind v3 files, Web Storage, online font imports, GSAP, extra icon libraries, hardcoded frontend Chinese, root resource reads and ImTip.
- Playwright/browser pixel checks for both themes/densities at native minimum and common desktop viewport, 200% text and reduced motion.
- Production build plus elevated runtime smoke.

## 10. Compatibility, Rollback And Trade-offs

- No config migration is required; rollback can remove provider/bootstrap while stored values remain valid.
- Native bootstrap and frontend provider are separate rollback points. If document-start injection fails, revert only native projection and retain token/provider work; do not introduce localStorage.
- Each Radix primitive is independently removable. Do not replace a failed primitive with a handwritten focus trap or broaden package selection to the whole bundle.
- Deferring React Router keeps this task focused and lockfile smaller; routing shell pays the installation/upgrade cost when it can test real navigation.
- The first component set deliberately omits feature placeholder, toast, confirmation and task feedback so later tasks define those contracts from real consumers.
