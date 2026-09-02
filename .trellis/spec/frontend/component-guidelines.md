# Component Guidelines

> Product component boundaries and reviewed accessible UI primitives.

---

## Current Status

**Established through S1 UI foundation.** `WindowTitleBar` establishes the
native window-control contract. The project-owned Button, Input, Kbd, Dialog,
Dropdown Menu, Tooltip, and OverlayProvider establish the reviewed generic
primitive and overlay conventions below. Route composition remains pending.

## Approved Boundary

- Product UI uses React 19 and TypeScript. Project-owned shadcn/ui source belongs
  under `src/components/ui/`; reusable product components belong under their
  named directory in `src/components/`; route-specific UI stays with its route.
- Styling uses Tailwind CSS v4 with `src/styles/theme.css` as the token source.
  Components do not introduce literal color, spacing, radius, typography,
  shadow or motion values that bypass the token contract.
- Icons use Lucide components rather than icon fonts or hard-coded glyphs.
  Domain-specific radicals/key caps may use the approved PUA font or product
  rendering instead of an unrelated generic icon.
- Components do not parse files or perform codec/domain transformations. IPC
  payloads use the committed generated bindings.
- Accessibility requirements include keyboard access, visual-order tab flow,
  visible focus, required contrast, non-color-only meaning, accessible names
  and roles, reduced-motion support and resilient system font scaling.
- Large lexicon views must keep DOM and frontend ownership bounded. The S0
  virtual-scroll spike is performance evidence, not a component template.

## Established S1 Pattern

- A reusable product component lives in its named `src/components/` directory,
  uses a named export, declares a local props interface, and colocates its
  `*.test.tsx` file.
- Native window controls are semantic `button type="button"` elements with
  Chinese `aria-label` and `title`, visual-order DOM flow, visible focus, and a
  fixed 44 by 44 pixel target. Icons come through a narrow Lucide re-export.
- The drag attribute belongs only to the noninteractive brand region. Buttons
  and the control container do not carry `data-tauri-drag-region`.
- The component receives generated state and emits generated intent; transport,
  native state transitions, warning storage, and tray behavior stay outside it.

## Established UI Foundation Pattern

- Shared primitive source lives in `src/components/ui/`, uses named exports,
  accepts native/Radix component props, merges classes through the sole
  `src/lib/cn.ts` helper, and uses product tokens rather than a second palette.
- Button variants use local `class-variance-authority` definitions. The default
  element is a native `button type="button"`; `busy` sets `aria-busy` and native
  disabled state without changing dimensions. `asChild` uses Radix Slot only
  when the child owns valid interactive semantics.
- Input remains a native input and forwards native label, invalid, disabled,
  read-only, and ref behavior. Its consumer owns visible label/help/error
  composition; placeholder text is not a label.
- Kbd is semantic noninteractive shortcut text. It does not parse, register,
  record, or invoke shortcuts.
- Dialog, Dropdown Menu, and Tooltip are narrow wrappers around separately
  installed Radix packages. Radix retains ownership of focus trapping/restoring,
  Escape, roving focus, keyboard activation, and ARIA relationships.
- All reviewed overlays portal into the app-level `#overlay-root` obtained from
  one `OverlayProvider`. Tooltip delay configuration is mounted once there.
  Overlay elevation, scrim, and stacking use product tokens.
- Only icons required by reviewed components are re-exported from `src/icons/`.
  Icon-only controls still require an accessible name; tooltip content is
  supplementary and never substitutes for that name.

## Decisions Not Yet Established

The project has not established:

- route-level page and layout composition;
- a component-generation command or automated shadcn update policy;
- full form-field label/help/error composition;
- loading, empty, failure, confirmation, toast and feature-placeholder APIs;
- a shared accessibility test helper or snapshot policy.

## Forbidden Premature Assumptions

- Do not generalize the title-bar's native-window constraints into unrelated
  route or domain components.
- Do not copy additional shadcn/Radix source before a product task needs and
  reviews that component.
- Do not widen the reviewed barrel to re-export an entire Radix namespace.
- Do not put business/codec logic, raw IPC casts or a complete lexicon array in
  a component.
- Do not create a visible component that bypasses the theme, accessibility or
  feature-availability contracts.

## Update Trigger

Update this guide when a real route establishes page composition, a form task
establishes field composition, or the feedback task establishes confirmation,
toast, loading, empty, failure, or feature-placeholder APIs. Keep native
title-bar rules scoped to the window component.

## Scenario: Native Window Title Bar

### 1. Scope / Trigger

Apply this scenario when changing the frameless title bar, its controls, drag
region, icons, state props, or component tests.

### 2. Signatures

```typescript
interface WindowTitleBarProps {
  iconUrl: string;
  version: string;
  snapshot: WindowStateSnapshot | null;
  onControl: (intent: WindowControlIntent) => void;
}
```

### 3. Contracts

- Render product icon/name/version and minimize, maximize/restore, and close in
  one compact header; do not add routing, settings, theme, or tray ownership.
- Maximize label/icon derives only from `snapshot.maximized`; all controls are
  disabled while visibility is `exiting`.
- Use Lucide `Minus`, `Square`/`Copy`, and `X` through
  `src/icons/window-controls.ts`.
- Interactive descendants are outside the drag region and remain keyboard
  operable at 200 percent system text scaling.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Snapshot is not loaded | Render restore-safe non-maximized controls without inventing native state |
| Snapshot is maximized | Show `还原窗口` and the restore icon |
| Snapshot is exiting | Disable every native window control |
| Keyboard Enter or Space activates a button | Emit exactly the generated intent for that button |
| Control is inside a drag-marked element | Treat as a component defect; move it outside the drag region |

### 5. Good / Base / Bad Cases

- Good: a keyboard user can identify and invoke every 44 by 44 control, and
  maximize state changes its accessible name and Lucide icon.
- Base: a null bootstrap snapshot still renders stable controls and layout.
- Bad: icon-only controls have no accessible name, a button carries the drag
  attribute, or the component directly calls `@tauri-apps/api/window`.

### 6. Tests Required

- Query all controls by accessible role/name and assert matching `title` text.
- Invoke minimize with Enter, maximize with Space, and close with pointer input;
  assert the exact generated intents and order.
- Assert maximize/restore semantics, exiting disabled state, brand/version, and
  drag-region isolation.
- Keep browser viewport checks for fixed 44 by 44 controls and no horizontal
  overflow at the native minimum window size.

### 7. Wrong vs Correct

```tsx
// Wrong: a glyph div has no native keyboard or accessible behavior.
<div data-tauri-drag-region onClick={() => close()}>x</div>

// Correct: generated intent leaves native behavior in the coordinator.
<button type="button" aria-label="关闭窗口" title="关闭窗口"
  onClick={() => onControl("close")}><X aria-hidden="true" /></button>
```

## Scenario: Shared UI Primitives And Overlays

### 1. Scope / Trigger

Apply this scenario when changing the reviewed primitives, `cn`, icon exports,
overlay root/provider, portal behavior, component variants, or accessibility
tests in `src/components/ui/`.

### 2. Signatures

```typescript
export interface ButtonProps
  extends ComponentProps<"button">, VariantProps<typeof buttonVariants> {
  asChild?: boolean;
  busy?: boolean;
}

export function Button(props: ButtonProps): ReactNode;
export function Input(props: ComponentProps<"input">): ReactNode;
export function Kbd(props: ComponentProps<"kbd">): ReactNode;
export function OverlayProvider(props: {
  children: ReactNode;
  container?: HTMLElement | null;
}): ReactNode;
```

Dialog, Dropdown Menu, and Tooltip wrappers accept the corresponding Radix
primitive props and expose only the named exports in `src/components/ui/index.ts`.

### 3. Contracts

- Button variants are `primary`, `secondary`, `outline`, `ghost`, and `danger`;
  sizes are `default` and `icon`. Every size keeps `min-h-control`, and icon
  size keeps a square `size-control` target.
- Input and Button preserve native attributes and event behavior. Consumers do
  not simulate disabled or invalid state with CSS alone.
- Dialog content always renders through the shared portal with a scrim and a
  translated close button. Consumers provide a DialogTitle and, where needed,
  DialogDescription.
- Dropdown Menu content uses shared portal ownership and exposes real item
  disabled state, groups, labels, and separators. It does not implement custom
  arrow-key state.
- Tooltip opens for hover and keyboard focus through Radix and the app-level
  provider. Its default side offset is 8 px so the bubble does not cover the
  focused trigger.
- Custom classes may extend a primitive through `cn`, but may not remove focus,
  accessible state, stable hit area, or overlay stacking guarantees.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Button has no explicit type | Render native `type="button"` |
| Button is busy | Set `aria-busy=true`, disable activation, and keep dimensions stable |
| Input is invalid, disabled, or read-only | Preserve the matching native/ARIA attribute and distinct token state |
| Dialog closes by Escape or close button | Close and restore focus to its trigger |
| Menu item is disabled | Arrow navigation skips it, and Enter/click cannot select it |
| Overlay provider receives a test container | Portal into that container; production defaults to `#overlay-root` |
| Tooltip trigger has no accessible name | Treat as a consumer defect even if tooltip text exists |

### 5. Good / Base / Bad Cases

- Good: an icon button has a translated accessible name, a stable 44 px target,
  visible focus, and supplementary tooltip content portaled outside page
  overflow.
- Base: a native Input associates with a consumer-owned label and renders its
  default token states without wrapper-specific form logic.
- Bad: a clickable Kbd, a handwritten dialog focus trap, an emoji icon, a
  disabled-looking menu item that still selects, or a tooltip used as the only
  accessible name.

### 6. Tests Required

- Button tests assert default type, every variant/size, busy/disabled behavior,
  native activation, Slot composition, class merging, and ref forwarding.
- Input/Kbd tests assert native semantics, invalid/disabled/read-only classes,
  noninteractive Kbd behavior, and refs.
- Dialog tests assert portal target, title/description relationships, pointer
  close, Escape, focus trap, and trigger focus restoration.
- Menu tests assert groups/separators, arrows, Enter, Escape, disabled items,
  selection, focus restoration, and portal target.
- Tooltip tests assert one provider, hover/focus open, Escape close, spacing,
  accessible description behavior, and portal target.

### 7. Wrong vs Correct

```tsx
// Wrong: visual state replaces native semantics and focus behavior.
<div className="disabled" onClick={save}>Save</div>

// Correct: the reviewed primitive preserves native and accessible state.
<Button busy={saving} onClick={save}>{t("common:save")}</Button>
```

## Sources

- [`docs/02-architecture.md` frontend stack, D8, D9 and D16](../../../docs/02-architecture.md)
- [`docs/21-ui-ux.md` tokens and component inventory](../../../docs/21-ui-ux.md)
- [`docs/20-nonfunctional.md` NFR-A11Y-001..007](../../../docs/20-nonfunctional.md)
- [Frontend directory structure](./directory-structure.md)
- [Virtualization and performance](./virtualization-performance.md)
- [Window title bar](../../../src/components/window-title-bar/WindowTitleBar.tsx)
- [Window title bar tests](../../../src/components/window-title-bar/WindowTitleBar.test.tsx)
- [Shared UI primitives](../../../src/components/ui/index.ts)
- [Primitive interaction tests](../../../src/components/ui/primitives.test.tsx)
