# Hook Guidelines

> React Hook boundaries before a project-specific Hook pattern exists.

---

## Current Status

**Pending implementation evidence.** The current product frontend has no custom
Hooks, data-fetching layer or runnable React root. React's own Hook rules apply,
but the repository has not established a project-specific file, export,
lifecycle or testing convention.

## Approved Boundary

- Hooks are frontend coordination boundaries only. File parsing, codec work,
  transformation, sorting, filtering and other domain behavior stay in Rust.
- Tauri commands and events use the generated IPC contracts. A Hook must not
  redefine or locally cast a raw payload shape.
- Large collections are requested by page or viewport. A Hook must not retain a
  second full lexicon or derive hundreds of thousands of row objects.
- Feature availability comes from the backend-populated feature store, not a
  Vite constant or command-presence probe.
- Subscriptions, timers and other effects introduced by a future Hook must have
  a testable cleanup path; cancellation/error behavior must remain visible to
  the caller rather than becoming silent success.

## Decisions Not Yet Established

The project has not selected or demonstrated:

- a custom-Hook directory, filename or export convention beyond React's
  language-level `use*` requirement;
- a Tauri command/event wrapper shape;
- React Query, SWR or another server-state/cache library;
- retry, deduplication, stale-data, cancellation or polling defaults;
- a Hook test harness, fake transport or subscription test helper.

## Forbidden Premature Assumptions

- Do not add a query/cache dependency only to create a generic data-fetching
  example.
- Do not treat the virtual-scroll benchmark controller as a product Hook.
- Do not put codec or business rules into a Hook to avoid a Rust boundary.
- Do not hide transport, cancellation or backend errors behind `undefined` or
  an empty collection.
- Do not create a second untyped IPC facade alongside generated bindings.

## Update Trigger

Update this guide when an approved S1 task implements the first reusable custom
Hook or IPC subscription/data-fetching boundary. Cite the concrete source and
tests, then document only the naming, cleanup, cancellation, error and cache
behavior that those tests establish.

## Sources

- [`docs/02-architecture.md` frontend responsibilities and IPC boundary](../../../docs/02-architecture.md)
- [Frontend directory structure](./directory-structure.md)
- [Frontend type safety](./type-safety.md)
- [Frontend quality guidelines](./quality-guidelines.md)
