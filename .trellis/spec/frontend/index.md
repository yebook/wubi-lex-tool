# Frontend Development Guidelines

> Project-specific contracts for the React, TypeScript, and Tauri IPC boundary.

---

## Status Model

- **Baseline; examples pending** means the rule is fixed by approved architecture, requirements, or the scaffolded repository layout. Real source examples must be added only after the relevant frontend surface exists.
- **Baseline with S0 binding evidence** means the boundary is implemented and verified by the Rust registry, generated TypeScript baseline, and repository gates, while real frontend consumers are still pending.
- **Pending implementation evidence** means no actual project convention has
  been established. The guide must still record approved boundaries,
  unselected decisions, forbidden assumptions and the event that will update
  it; do not turn generic template advice into a project rule.
- **Established** means the document already contains an approved, executable convention and examples grounded in the design system.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Route, shared UI, infrastructure, and generated-type placement | **Baseline with S0 binding evidence** |
| [Component Guidelines](./component-guidelines.md) | Component ownership, props, and composition | **Pending implementation evidence** |
| [Hook Guidelines](./hook-guidelines.md) | Custom hooks and effect boundaries | **Pending implementation evidence** |
| [State Management](./state-management.md) | Zustand store ownership and selector patterns | **Pending implementation evidence** |
| [Quality Guidelines](./quality-guidelines.md) | Frontend gates and Rust/frontend responsibility boundaries | **Baseline with S0 binding evidence** |
| [Type Safety](./type-safety.md) | Generated IPC contracts and frontend-owned type boundaries | **Baseline with S0 binding evidence** |
| [Virtualization And Performance](./virtualization-performance.md) | Bounded large-list ownership and visible-browser benchmark contracts | **Baseline with S0 risk-spike evidence** |
| [Tailwind v4 Token Convention](./tailwind-v4-tokens.md) | `@theme inline`, v3-to-v4 mapping, and named tokens | **Established** |

The S0 binding, CI, and virtual-scroll examples establish generated-type
placement, freshness, frontend toolchain gates, and bounded-DOM performance
measurement. Bootstrap completeness means every guide is non-placeholder and
honest about its evidence state; component, Hook, state, consumer and route
patterns remain pending until real S1 code and tests provide them.

---

**Language**: All documentation in this directory must be written in **English**.
