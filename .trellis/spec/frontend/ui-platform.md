# UI Platform

> Cross-layer contracts for first-frame appearance, durable UI preferences,
> bundled localization, and application provider composition.

---

## Scenario: First-Frame And Runtime UI Preferences

### 1. Scope / Trigger

Apply this contract when changing `UiConfig`, native WebView construction,
appearance projection, configuration listeners or updates, localization, or
the application provider tree. The durable source is the Rust configuration
service; DOM attributes and React context are projections, not persistence.

### 2. Signatures

```rust
pub(crate) struct UiBootstrap { /* fixed script, native theme, background */ }

impl UiBootstrap {
    pub(crate) fn from_config(config: &UiConfig) -> Self;
}

pub(crate) fn apply_to_builder<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
    bootstrap: UiBootstrap,
) -> WebviewWindowBuilder<'a, R, M>;
```

```typescript
export interface UiConfigClient {
  fetchSnapshot(): Promise<ConfigSnapshot>;
  updateUi(ui: UiConfig): Promise<ConfigSnapshot>;
  listenChanged(
    listener: (snapshot: ConfigSnapshot) => void,
  ): Promise<() => void>;
}

export interface UiPreferencesContextValue {
  status: "loading" | "ready" | "failed";
  ui: Required<UiConfig>;
  warning: string | null;
  setTheme(theme: ThemePreference): Promise<void>;
  setDensity(density: Density): Promise<void>;
  setLocale(locale: AppLocale): Promise<void>;
  clearWarning(): void;
}
```

The wire types above come from `src/types/generated/bindings.ts`; do not
redeclare their unions or payload fields.

### 3. Contracts

- One startup `ConfigSnapshot` supplies both `WindowConfig` and `UiConfig`.
  Snapshot failure uses both defaults and the existing redacted visible notice
  path; it does not perform a second configuration read.
- `UiBootstrap` is a closed enum projection. At document start it guards the
  top-level frame, waits only for `documentElement`, sets `data-theme`,
  `data-density`, `lang`, resolved `.dark`, and `colorScheme`, then disconnects
  its observer. It never serializes paths, arbitrary locale text, notices, or
  other user-controlled data.
- Explicit light and dark preferences set the matching native Tauri theme and
  initial surface background. System preference leaves native theme and
  background unset and resolves through `prefers-color-scheme`.
- The React provider reads bootstrap attributes for its initial render,
  registers `config://changed` before fetching `config_snapshot`, and rejects
  snapshots whose revision is lower than the last confirmed revision. Equal
  revisions may replace the current authoritative snapshot.
- Preference setters optimistically project the DOM, then use one serialized
  queue to send complete `UiConfig` groups. Every queued job is based on the
  latest confirmed config plus its patch, so sibling fields are not lost.
- Failed updates remove only the failed pending job, restore the latest
  confirmed values plus any remaining jobs, and expose a warning bounded to
  512 Unicode scalar values. Listener, snapshot, native-theme, and update
  failures remain visible; configuration failure never blocks application
  startup.
- System-theme media listeners and asynchronously registered Tauri listeners
  have cleanup paths that are safe under React StrictMode and unmount races.
- Localization uses one synchronously initialized, bundled i18next instance.
  The current registry contains only `zh-CN`, split into `common`, `window`,
  `runtime`, and `ui` namespaces. React escapes interpolated values; no network
  loader is installed.
- Frontend-authored visible strings, accessible names, and fallback warnings
  are translation keys. Brand, version, codes, paths, and backend-provided
  message/detail fields remain data and are rendered without translation.
- Provider order is `I18nextProvider` -> `UiPreferencesProvider` ->
  `OverlayProvider` -> application. Do not mount a competing theme, locale, or
  tooltip provider below a route.
- No environment key, Web Storage entry, cookie, Vite flag, or second config
  file participates in this contract.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Config snapshot is unavailable at native startup | Use system/standard/zh-CN and default window config; emit one redacted visible notice |
| Document root does not yet exist | Observe until it exists, apply once, then disconnect |
| Script executes in a subframe | Return without modifying that frame |
| Newer event arrives before the initial snapshot | Keep the event revision and reject the stale snapshot |
| Two preference setters run concurrently | Project both immediately and persist full groups serially without sibling-field loss |
| Update fails while another job remains pending | Restore confirmed values, reapply remaining patches, and show a bounded warning |
| System color preference changes | Recompute `.dark` only while theme preference is `system` |
| Listener registration resolves after unmount | Invoke the returned unlisten function immediately; perform no state write |
| Translation key or locale is unsupported | Fall back to bundled `zh-CN`; do not fetch or invent an empty resource |

### 5. Good / Base / Bad Cases

- Good: a dark/compact snapshot is visible before React starts, a later
  density update preserves dark mode, and a newer configuration event wins
  over an older command response.
- Base: default system/standard/zh-CN attributes initialize synchronously and
  the first snapshot moves provider status from loading to ready.
- Bad: reading localStorage to prevent a flash, fetching the snapshot before
  listening, persisting concurrent field patches independently, interpolating
  arbitrary JSON into the native script, or translating backend error data.

### 6. Tests Required

- Rust unit tests cover all three themes, both densities, the current locale,
  defaults, fixed literals, subframe guard, missing-root observer, native theme,
  initial background, and exclusion of unrelated config fields.
- Appearance unit tests cover bootstrap normalization, light/dark/system root
  projection, native theme mapping, media changes, and listener cleanup.
- Provider tests assert listen-before-snapshot ordering, monotonic revisions,
  event-before-snapshot behavior, serialized sibling updates, rollback,
  warning bounds, native-theme failures, and both async cleanup timings.
- i18n tests assert synchronous initialization, namespaces, interpolation,
  locale fallback, and allowed locations for frontend-authored Chinese text.
- Run frontend typecheck, lint, Vitest, build, binding freshness, and the real
  Windows runtime smoke after changing this boundary.

### 7. Wrong vs Correct

```typescript
// Wrong: creates a second persistence source and can overwrite sibling fields.
localStorage.setItem("theme", theme);
void commands.configUpdateUi({ ...snapshot.config.ui, theme });

// Correct: one provider projects immediately and serializes complete updates.
await useUiPreferences().setTheme(theme);
```

## Sources

- [Native UI bootstrap](../../../src-tauri/src/ui_bootstrap.rs)
- [Application startup wiring](../../../src-tauri/src/lib.rs)
- [Typed configuration client](../../../src/lib/config-client.ts)
- [Appearance projection](../../../src/lib/ui-appearance.ts)
- [UI preferences provider](../../../src/app/providers/ui-preferences-provider.tsx)
- [Bundled i18n registry](../../../src/i18n/index.ts)
- [UI preferences tests](../../../src/app/providers/ui-preferences-provider.test.tsx)

