# Directory Structure

> The approved React application layout and placement boundaries.

---

## Baseline Status

The `src/` directory tree matches the approved architecture and now contains a runnable S1 React bootstrap plus the committed generated IPC contract. The temporary runtime status surface does not establish the final route, component, hook, store, or design-token conventions owned by later S1 tasks. Generated bindings and compiled consumers are implementation evidence; `.gitkeep` files are not.

## Directory Layout

| Location | Ownership |
|---|---|
| `src/app/layout/` | Application bar, sidebar, status bar, and other shell layout |
| `src/app/providers/` | Application-wide providers for theme, i18n, queries, and shortcuts |
| `src/app/router/` | Route table and deep-link wiring |
| `src/routes/` | Seven top-level product domains: overview, lexicons, phrases, lookup, radicals, learning, and settings |
| `src/components/ui/` | Project-owned shadcn/ui source using the Tailwind v4 branch |
| `src/components/<named-component>/` | Approved reusable product components such as virtual table, command palette, and feature placeholder |
| `src/stores/` | Zustand application stores, including the feature availability store |
| `src/lib/` | Tauri IPC wrappers and frontend utilities |
| `src/styles/theme.css` | The only Tailwind v4 design-token source |
| `src/types/generated/` | Committed LF-normalized TypeScript generated from the Rust IPC registry; currently `bindings.ts` |
| `src/i18n/` | i18next resources |
| `src/icons/` | The centralized Lucide export boundary |

## Placement Rules

- Route-specific screens and behavior stay within the matching `src/routes/<domain>/` directory. The directory names follow the user-facing information architecture, not Rust module numbers.
- The lexicon library and editor are two states of `routes/lexicons/`, not separate top-level routes.
- Reusable UI shared across routes belongs under `src/components/`; generic frontend infrastructure belongs under `src/lib/`.
- Generated IPC files stay under `src/types/generated/`, use the narrow `.gitattributes` LF rule, and are updated only through `cargo xtask bindings`. Do not edit them or place handwritten look-alike contracts beside them.
- The initial `src/main.tsx` bootstrap consumes `commands` and `events` from generated bindings. Pure view projections may live beside it until the routing shell creates their durable owner; this temporary placement must not be copied as a route convention.
- Feature availability comes from the backend-driven feature store. Feature routes may render the shared placeholder, but must not create a second Vite-time flag source.
- ImTip is permanently excluded by deprecated `M7-WIN-005`: do not create a route, component, action, tray projection, settings entry, feature flag or placeholder for it.
- The frontend never owns file parsing, lexicon transformations, or a complete lexicon model. Those responsibilities remain in Rust; routes request paged view data through IPC.

Existing route and reusable-component directory names use lower-case kebab form. File naming and component export conventions remain pending until real implementation provides evidence.

## Sources

- [`docs/02-architecture.md` sections 6, D9, D11, D16, and 9](../../../docs/02-architecture.md)
- [`docs/21-ui-ux.md` information architecture and component inventory](../../../docs/21-ui-ux.md)
- The scaffolded [`src/`](../../../src/) directory
- [Generated TypeScript baseline](../../../src/types/generated/bindings.ts)
- [Rust-owned binding registry](../../../src-tauri/src/bindings/mod.rs)

Generated binding placement and the first runtime command/event consumer are established. Component, route, hook, store, and final shell examples remain pending.
