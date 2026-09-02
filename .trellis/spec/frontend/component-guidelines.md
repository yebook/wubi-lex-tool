# Component Guidelines

> Product component boundaries established by the first native window control surface.

---

## Current Status

**Established by S1 window/tray.** `WindowTitleBar` is the first reviewed
product component with a colocated Testing Library test. It establishes only
the native window-control patterns below; it does not settle generic variants,
ref forwarding, route composition, or shadcn/ui conventions.

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

## Decisions Not Yet Established

The project has not established:

- a repository-wide file/export policy beyond the first named component;
- ref forwarding or a generic composition API;
- variant/class composition helpers or a component-generation command;
- loading, empty, failure, disabled and feature-placeholder component APIs;
- a shared accessibility test helper or snapshot policy.

## Forbidden Premature Assumptions

- Do not generalize the title-bar's native-window constraints into unrelated
  route or domain components.
- Do not copy shadcn/ui source before a product task needs and reviews that
  component.
- Do not invent props, variant or barrel-export rules from generic React
  practice.
- Do not put business/codec logic, raw IPC casts or a complete lexicon array in
  a component.
- Do not create a visible component that bypasses the theme, accessibility or
  feature-availability contracts.

## Update Trigger

Update this guide when the UI foundation establishes general tokens, primitives,
variants, ref forwarding, or route composition. Keep native title-bar rules
scoped to the window component.

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

## Sources

- [`docs/02-architecture.md` frontend stack, D8, D9 and D16](../../../docs/02-architecture.md)
- [`docs/21-ui-ux.md` tokens and component inventory](../../../docs/21-ui-ux.md)
- [`docs/20-nonfunctional.md` NFR-A11Y-001..007](../../../docs/20-nonfunctional.md)
- [Frontend directory structure](./directory-structure.md)
- [Virtualization and performance](./virtualization-performance.md)
- [Window title bar](../../../src/components/window-title-bar/WindowTitleBar.tsx)
- [Window title bar tests](../../../src/components/window-title-bar/WindowTitleBar.test.tsx)
