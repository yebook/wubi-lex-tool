# Frontend Development Guidelines

> Project-specific contracts for the React, TypeScript, and Tauri IPC boundary.

---

## Status Model

- **Baseline; examples pending** means the rule is fixed by approved architecture, requirements, or the scaffolded repository layout. Real source examples must be added only after the relevant frontend surface exists.
- **Baseline with S0 binding evidence** means the boundary is implemented and verified by the Rust registry, generated TypeScript baseline, and repository gates, while real frontend consumers are still pending.
- **Baseline with S1 runtime evidence** means the rule is exercised by the first generated command/event consumer and focused runtime-view tests without claiming final shell conventions.
- **Pending implementation evidence** means no actual project convention has
  been established. The guide must still record approved boundaries,
  unselected decisions, forbidden assumptions and the event that will update
  it; do not turn generic template advice into a project rule.
- **Established** means the document already contains an approved, executable convention and examples grounded in the design system.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Route, shared UI, infrastructure, and generated-type placement | **Established through S1 UI foundation** |
| [Component Guidelines](./component-guidelines.md) | Component ownership, props, composition, and accessible primitives | **Established through S1 UI foundation** |
| [Hook Guidelines](./hook-guidelines.md) | Custom hooks and effect boundaries | **Established by S1 window/tray** |
| [State Management](./state-management.md) | Zustand store ownership and selector patterns | **Established by S1 feature catalog** |
| [UI Platform](./ui-platform.md) | First-frame appearance, UI preferences, bundled i18n, and provider composition | **Established through S1 UI foundation** |
| [Quality Guidelines](./quality-guidelines.md) | Frontend gates and Rust/frontend responsibility boundaries | **Established through S1 UI foundation** |
| [Type Safety](./type-safety.md) | Generated IPC contracts and frontend-owned type boundaries | **Baseline with S1 runtime evidence** |
| [Virtualization And Performance](./virtualization-performance.md) | Bounded large-list ownership and visible-browser benchmark contracts | **Baseline with S0 risk-spike evidence** |
| [Tailwind v4 Token Convention](./tailwind-v4-tokens.md) | `@theme inline`, v3-to-v4 mapping, and named tokens | **Established** |

The S0 binding, CI, and virtual-scroll examples establish generated-type
placement, freshness, frontend toolchain gates, and bounded-DOM performance
measurement. The S1 runtime adds the first generated command/event consumer.
The S1 feature catalog establishes the first Zustand vanilla store, injected
client, async loading/retry shape, in-flight deduplication, and typed selectors.
The S1 window/tray task establishes the first accessible product component and
listener-first custom Hook. The S1 UI foundation establishes first-frame and
runtime appearance, bundled localization, shared tokens, accessible primitives,
overlay ownership, frontend formatting, and the production build gate.
Query/cache behavior and route patterns remain pending until their owning S1
tasks provide code and tests.

---

**Language**: All documentation in this directory must be written in **English**.
