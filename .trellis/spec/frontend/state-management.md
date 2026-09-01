# State Management

> Established Zustand ownership and selector conventions.

---

## Current Status

The S1 feature availability store in `src/stores/features.ts` is the first
reviewed Zustand convention. It uses Zustand 5.0.15's vanilla store, generated
Rust IPC types, an injected client, bounded invocation failures, and no browser
persistence.

## Scenario: Backend-Owned Feature Availability

### 1. Scope / Trigger

Apply this contract when a route, action, settings surface, or bootstrap step
needs to know whether a build includes a future capability.

### 2. Signatures

```typescript
export interface FeatureClient {
  fetchCatalog(): Promise<AppFeatureCatalog>;
}

export function createFeatureStore(client: FeatureClient): StoreApi<FeaturesState>;
export const featuresStore: StoreApi<FeaturesState>;
```

`FeaturesState` exposes `status`, `catalog`, `error`, `initialize`, `retry`,
`replace`, `feature`, and `isAvailable`. Selector factories accept the generated
`AppFeatureId`; frontend code must not define a duplicate feature union.

### 3. Contracts

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
- Initial status is `loading`. `initialize()` deduplicates one in-flight request
  and becomes idempotent after `ready`, so React StrictMode cannot duplicate the
  startup command. `retry()` starts a fresh request after failure.
- A successful fetch or `replace()` replaces the complete catalog; no action,
  event, or consumer patches individual feature records.
- Invocation errors are reduced to a user-readable message bounded to 512
  Unicode scalar values. The store retains no raw command payload or stack.
- The production store starts from `src/main.tsx`; the runtime shell does not
  invent route placeholders or infer a feature from command failure.

### 4. Validation & Error Matrix

| Condition | Required state |
|---|---|
| First or concurrent initialization | `loading`; exactly one client call |
| Fetch succeeds | `ready`, full returned catalog, `error = null` |
| Fetch fails | `failed`, prior bounded catalog unchanged, visible bounded error |
| Retry after failure | `loading`, then the new request result |
| Full replacement omits an old ID | Old record is absent from lookups and selectors |
| Generated ID absent from the current catalog | `undefined` / `false`, never inferred availability |

### 5. Good / Base / Bad Cases

- Good: two StrictMode-style calls share the exact promise and one backend invocation.
- Base: the S1 default catalog is ready with all 12 future capabilities unavailable.
- Bad: storing flags in Vite, persisting the catalog to localStorage, patching one record, or maintaining a handwritten ID array.

### 6. Tests Required

- Assert concurrent initialization returns the same promise and calls the client once.
- Assert loading, ready, failed, retry, enabled/disabled lookup, unknown lookup, and complete replacement.
- Compile selectors against generated `AppFeatureId` and catalog types.
- Search frontend production code for Vite feature flags, `localStorage`, `sessionStorage`, and Zustand persistence middleware.

### 7. Wrong vs Correct

```typescript
// Wrong: a second availability source that can drift from Cargo.
const enabled = import.meta.env.VITE_LEXICON_READ === "1";

// Correct: select the generated ID from the backend snapshot.
const enabled = useStore(featuresStore, selectFeatureAvailable("lexiconRead"));
```

## Remaining Boundaries

## Decisions Not Yet Established

The project has not yet established:

- criteria for promoting local component state to a shared store;
- middleware or devtools policy beyond their absence from the feature store;
- cancellation, optimistic-update, or server-state caching conventions;
- server-state caching, invalidation or synchronization behavior;
- selector helpers, equality functions, reset behavior or store test fixtures.

## Forbidden Patterns

- Do not create a second feature-flag source in frontend build configuration.
- Do not persist backend configuration or complete lexicon data in a browser
  store or WebView storage.
- Do not duplicate generated IPC models as store-specific wire types.
- Do not infer store organization from the empty `src/stores/` scaffold.

## Sources

- [`docs/02-architecture.md` frontend stack and D16](../../../docs/02-architecture.md)
- [Frontend directory structure](./directory-structure.md)
- [Frontend type safety](./type-safety.md)
- [Frontend quality guidelines](./quality-guidelines.md)
- [Feature store](../../../src/stores/features.ts)
- [Feature store tests](../../../src/stores/features.test.ts)
