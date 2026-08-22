# Frontend Development Guidelines

> Project-specific contracts for the React, TypeScript, and Tauri IPC boundary.

---

## Status Model

- **Baseline; examples pending** means the rule is fixed by approved architecture, requirements, or the scaffolded repository layout. Real source examples must be added after the frontend shell exists.
- **Pending implementation evidence** means no actual project convention has been established. Do not turn generic template advice into a project rule.
- **Established** means the document already contains an approved, executable convention and examples grounded in the design system.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Route, shared UI, infrastructure, and generated-type placement | **Baseline; examples pending** |
| [Component Guidelines](./component-guidelines.md) | Component ownership, props, and composition | **Pending implementation evidence** |
| [Hook Guidelines](./hook-guidelines.md) | Custom hooks and effect boundaries | **Pending implementation evidence** |
| [State Management](./state-management.md) | Zustand store ownership and selector patterns | **Pending implementation evidence** |
| [Quality Guidelines](./quality-guidelines.md) | Frontend gates and Rust/frontend responsibility boundaries | **Baseline; examples pending** |
| [Type Safety](./type-safety.md) | Generated IPC contracts and frontend-owned type boundaries | **Baseline; examples pending** |
| [Tailwind v4 Token Convention](./tailwind-v4-tokens.md) | `@theme inline`, v3-to-v4 mapping, and named tokens | **Established** |

Keep `00-bootstrap-guidelines` active until real S0/S1 code provides component, hook, state, and source examples.

---

**Language**: All documentation in this directory must be written in **English**.
