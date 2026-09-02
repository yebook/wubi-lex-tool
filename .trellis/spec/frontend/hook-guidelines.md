# Hook Guidelines

> React Hook boundaries established by the first window IPC subscription.

---

## Current Status

**Established by S1 window/tray.** `useWindowControls` is the first reusable
custom Hook and listener-first IPC boundary with a renderHook test. Its
subscription, revision merge, cleanup, and visible-error rules are established;
generic query/cache, retry, polling, and server-state conventions remain open.

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

## Established S1 Pattern

- Reusable Hooks use `src/hooks/use-*.ts`, export a named `use*` function, and
  colocate `use-*.test.ts` when the lifecycle can be tested without a browser.
- IPC access is injected through a typed `src/lib/` client interface in tests.
  The production client imports commands, events, and payloads from generated
  bindings without local wire casts.
- Event listeners are registered before the initial snapshot. State merges by
  monotonic revision so a late snapshot cannot overwrite a newer event.
- Cleanup handles listeners that resolve before and after unmount. Listener,
  snapshot, and command failures remain visible to the component caller.

## Decisions Not Yet Established

The project has not selected or demonstrated:

- React Query, SWR or another server-state/cache library;
- retry, deduplication, stale-data, cancellation or polling defaults;
- a shared Hook fake-transport helper beyond local injected clients.

## Forbidden Premature Assumptions

- Do not add a query/cache dependency only to create a generic data-fetching
  example.
- Do not treat the virtual-scroll benchmark controller as a product Hook.
- Do not put codec or business rules into a Hook to avoid a Rust boundary.
- Do not hide transport, cancellation or backend errors behind `undefined` or
  an empty collection.
- Do not create a second untyped IPC facade alongside generated bindings.

## Update Trigger

Update this guide when another Hook establishes retry, polling, cancellation,
or shared server-state caching. Do not infer those defaults from the window
subscription.

## Scenario: Listener-First Window State Hook

### 1. Scope / Trigger

Apply this scenario when changing `useWindowControls`, its typed client,
window-state event/snapshot merging, live runtime notices, or cleanup behavior.

### 2. Signatures

```typescript
interface WindowClient {
  fetchState(): Promise<WindowStateSnapshot>;
  control(intent: WindowControlIntent): Promise<WindowStateSnapshot>;
  listenState(listener: (state: WindowStateSnapshot) => void): Promise<() => void>;
  listenNotice(listener: (notice: RuntimeNotice) => void): Promise<() => void>;
}
```

### 3. Contracts

- Register state and notice listeners before calling `fetchState()`.
- Merge snapshots by revision; equal revisions may accept the incoming
  authoritative value, while a lower revision never replaces current state.
- Keep at most eight distinct live notices, deduplicated by code plus detail.
- A successful control merges its returned snapshot and clears only the command
  warning. A failed control exposes a readable warning and retains prior state.
- Unmount calls every resolved unlisten function exactly once; a listener that
  resolves after disposal must immediately unlisten itself.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Event arrives while snapshot is in flight | Keep the higher event revision after snapshot resolves |
| State or notice listener rejects | Expose a listener warning and continue the remaining bootstrap steps |
| Snapshot rejects | Keep any event state and expose a visible fallback warning |
| Window command rejects | Preserve state/notices and expose the command error or fallback text |
| Duplicate live notice arrives | Keep one code/detail pair and do not grow the bounded list |
| Component unmounts during async registration | Stop every completed subscription without a post-unmount state write |

### 5. Good / Base / Bad Cases

- Good: revision 2 event arrives before revision 1 snapshot and revision 2
  remains rendered; both listeners are removed on unmount.
- Base: no events arrive, the initial snapshot becomes current, and warnings
  remain null.
- Bad: snapshot-first bootstrap loses an event, a raw payload is cast locally,
  command failure becomes `undefined`, or an asynchronously resolved listener
  remains registered after unmount.

### 6. Tests Required

- Record call order and assert state listener, notice listener, then snapshot.
- Resolve an old snapshot after a newer event and assert the newer state wins.
- Assert notice deduplication/bounds, command failure visibility, successful
  command merge, and cleanup of both listener timing paths.
- Run TypeScript, ESLint, Vitest, and `cargo xtask bindings --check` together.

### 7. Wrong vs Correct

```typescript
// Wrong: an event between these awaits is lost.
setState(await commands.windowState());
await events.windowStateChanged.listen(onState);

// Correct: listen first and reject stale snapshot revisions.
const unlisten = await client.listenState(onState);
const initial = await client.fetchState();
setState((current) => mergeWindowState(current, initial));
```

## Sources

- [`docs/02-architecture.md` frontend responsibilities and IPC boundary](../../../docs/02-architecture.md)
- [Frontend directory structure](./directory-structure.md)
- [Frontend type safety](./type-safety.md)
- [Window controls Hook](../../../src/hooks/use-window-controls.ts)
- [Window controls Hook tests](../../../src/hooks/use-window-controls.test.ts)
- [Frontend quality guidelines](./quality-guidelines.md)
