# Routing And Application Shell

> Executable contracts for product paths, runtime launch navigation, safe
> in-app history, the desktop shell, and backend-owned feature placeholders.

## Scenario: Seven-Domain Routing Shell

### 1. Scope / Trigger

Apply this contract when changing `src/app/router/`, `AppRuntimeProvider`,
`AppShell`, Sidebar, StatusBar, a top-level route, launch-path handling, or
`FeatureGate`. Rust owns the launch transport envelope and native window
activation; the frontend owns canonical product paths, route history, focus,
warnings, and rendering.

### 2. Signatures

```typescript
export const routeCatalog: readonly RouteDefinition[];
export type RouteId = (typeof routeCatalog)[number]["id"];
export type CanonicalRoutePath = (typeof routeCatalog)[number]["path"];

export function validateProductPath(path: string): ProductPathResult;
export function createHashAppRouter(initial: InitialNavigation): DataRouter;
export function createMemoryAppRouter(
  initial: InitialNavigation,
  initialEntries?: string[],
): DataRouter;

export interface InitialNavigation {
  path: CanonicalRoutePath;
  warning: string | null;
  consumedLaunchSequence: number;
}
```

`NavigationProvider` exposes `canGoBack`, `warning`, `goBack`,
`navigateProductPath`, `rememberFocus`, and `clearWarning`. `FeatureGate`
accepts a generated `AppFeatureId`, one of `page | section | inline`, visible
copy, and real children.

### 3. Contracts

- `routeCatalog` is the only path/order/label/icon/feature source. It contains,
  in order, `/overview`, `/lexicons`, `/phrases`, `/lookup`, `/radicals`,
  `/learning`, and `/settings`. `/` is a replace-only alias; every other exact
  mismatch redirects to Overview with a 512-scalar visible warning.
- Production uses `react-router@8.3.1` hash history. Hash and memory factories
  build from the same route objects, which derive product paths from the
  catalog rather than repeating string literals.
- Before the first router, initial navigation prefers the latest startup event
  with a path, snapshot secondary path, primary path, current hash, then
  Overview. A secondary event without a path never resets an earlier target.
  Router creation occurs in an effect and disposes its listener during
  StrictMode cleanup; runtime and unknown-path bridges redirect in a layout
  effect so no wrong route is painted first.
- Live canonical launch events push once; same-path and pathless events do not
  grow history. Unknown product paths replace with Overview. The frontend does
  not call native window commands because Rust already restores and focuses the
  single window.
- Internal history is keyed by React Router `location.key`, not browser history
  length. Push truncates forward entries, replace keeps depth, known pop moves
  the index, and an unknown pop resets the safe boundary. `Alt+Left` is always
  intercepted; `Escape` returns only outside editable controls and active
  overlays. Route focus moves to `h1[data-route-heading]`, resets the route
  scroll container, and restores a still-mounted trigger after pop.
- The shell has one `WindowTitleBar` acting as the app bar, one Sidebar, one
  scrollable route main, and one persistent StatusBar. Status priority is
  navigation, UI preferences, runtime listener/refresh, window warning, newest
  merged backend notice, then loading/ready. No scheme, search, task, or IME
  value is invented without an authoritative source.
- Unknown Unicode hashes can reach the visible warning as a long percent-encoded
  path. Overview notice summary/detail text must wrap anywhere, while the
  persistent StatusBar may ellipsize its bounded accessible value; neither may
  widen the route scroller at 200% text size.
- Sidebar collapse and Settings use `UiPreferencesProvider`. Setters must not
  persist a complete `UiConfig` before an authoritative revision exists;
  controls stay natively disabled until ready. No Web Storage or second store
  participates.
- `FeatureGate` reads only the generated backend catalog: loading is busy,
  failure or a missing ready record is a retryable fail-closed alert,
  unavailable renders `FeaturePlaceholder` with the backend milestone, and
  available renders children. Route navigation remains enabled.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Root or exact canonical path | Replace root with Overview, or render the exact route |
| Query, fragment, case, trailing slash, nested, or unknown path | Replace with Overview and retain a bounded warning |
| Launch event races the initial snapshot | Preserve the newest local event and consume its path before the first route |
| An older refresh resolves after a newer request | Ignore the older result |
| Pop target is outside the tracked session stack | Reset to a one-entry safe boundary; never leave the WebView |
| Escape originates in an editor or while a portal is active | Let the editor/overlay own it; do not navigate |
| UI preferences are still loading or failed without a revision | Disable controls and reject full-group persistence |
| Feature catalog is failed or omits a generated ID | Show a bounded retryable error; never infer availability |
| Unknown Unicode hash expands into a long encoded path | Return to Overview, wrap the full notice, and keep every shell region free of horizontal overflow |

### 5. Good / Base / Bad Cases

- Good: a hidden primary starts at `/settings`, a secondary launch pushes
  `/lexicons`, `Alt+Left` restores the Settings trigger, and a later unknown
  target replaces Overview with a visible warning.
- Base: no launch path and no hash initializes Overview; disabled feature
  routes remain clickable and show their real milestone placeholder.
- Bad: duplicating path strings in the router and sidebar, using
  `window.history.length`, constructing a hash router during render, writing a
  full config group from bootstrap defaults, or probing command failure for a
  feature.

### 6. Tests Required

- Assert the seven catalog records, unique IDs/paths, order, mapping, exact
  validator behavior, warning bounds, root alias, and hash/memory parity.
- Exercise listener-before-snapshot, event/snapshot and refresh races, cleanup,
  initial precedence, live canonical/same/pathless/unknown navigation.
- Exercise push/replace/pop, forward truncation, trigger restoration, heading
  fallback, route scroll reset, top-level keys, editable targets, and real
  Dialog/Menu Escape ownership.
- Exercise one title bar, Sidebar expanded/collapsed semantics, current-link
  idempotence, status priority, Settings eight-group/native controls, and
  pre-snapshot config write rejection.
- Exercise FeatureGate loading, failed, missing, unavailable, available, retry,
  and all three placeholder variants. Run format, typecheck, lint, Vitest,
  build, bindings/docs checks, browser layout checks, and Windows runtime smoke.
  Browser checks include a 200%-text unknown Unicode hash so encoded warning
  text cannot regress route-scroller width.

### 7. Wrong vs Correct

```typescript
// Wrong: parallel path source and unsafe browser-owned back depth.
const routes = [{ path: "/overview" }, { path: "/settings" }];
if (window.history.length > 1) window.history.back();

// Correct: derive route objects from the catalog and use session-owned depth.
const routes = routeCatalog.map(({ id, path }) => ({
  path,
  element: createRouteElement(id),
}));
if (navigation.canGoBack) navigation.goBack();
```

## Sources

- [Route catalog](../../../src/app/router/catalog.ts)
- [Router factories](../../../src/app/router/router.tsx)
- [Runtime provider](../../../src/app/providers/app-runtime-provider.tsx)
- [Navigation provider](../../../src/app/router/navigation-provider.tsx)
- [Application shell](../../../src/app/layout/AppShell.tsx)
- [Feature gate](../../../src/components/feature-placeholder/FeatureGate.tsx)
