# State Management

> Approved state ownership before the first Zustand store exists.

---

## Current Status

**Pending implementation evidence.** Architecture selects Zustand and selector
subscriptions, but Zustand is not installed and the current tree has no product
store. Store layout, actions, persistence and tests are therefore not yet
project conventions.

## Approved Boundary

- Product-wide frontend state uses Zustand once S1 introduces it. Consumers use
  selectors so unrelated state changes do not force broad React rerenders.
- Feature availability has one backend source: the `app_features` command
  populates a Zustand `features` store at startup. Frontend code does not infer
  availability from missing commands or duplicate it in Vite constants.
- Large lexicons remain Rust-owned. Frontend state holds only the current page,
  viewport, selection and other bounded presentation data, never a second full
  lexicon object array.
- Domain transformations, parsing, sorting and filtering remain in Rust.
  Cross-layer values use generated IPC types.
- Persistent application configuration is the backend D12 TOML concern. A
  frontend store is not an alternative configuration database.

## Decisions Not Yet Established

The project has not established:

- store filenames, slice boundaries, action naming or export conventions;
- criteria for promoting local component state to a shared store;
- middleware, devtools or frontend persistence policy;
- async action, loading/error/cancellation or optimistic-update shapes;
- server-state caching, invalidation or synchronization behavior;
- selector helpers, equality functions, reset behavior or store test fixtures.

## Forbidden Premature Assumptions

- Do not install Zustand or create an empty store solely to turn this pending
  guide into an implementation example.
- Do not create a second feature-flag source in frontend build configuration.
- Do not persist backend configuration or complete lexicon data in a browser
  store or WebView storage.
- Do not duplicate generated IPC models as store-specific wire types.
- Do not infer store organization from the empty `src/stores/` scaffold.

## Update Trigger

Update this guide in the task that implements the first reviewed Zustand store,
expected to include the backend-driven feature store in S1. Cite the real store,
selectors, consumers and tests before documenting action, slice, async or reset
patterns.

## Sources

- [`docs/02-architecture.md` frontend stack and D16](../../../docs/02-architecture.md)
- [Frontend directory structure](./directory-structure.md)
- [Frontend type safety](./type-safety.md)
- [Frontend quality guidelines](./quality-guidelines.md)
