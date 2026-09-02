# Window Coordinator

> Executable contracts for the native main-window, tray, placement, and window IPC boundary.

## Scenario: Native Window And Tray Lifecycle

### 1. Scope / Trigger

Apply this contract when changing `src-tauri/src/window/`, main-window setup,
window commands/events, close policy, tray ownership, second-instance restore,
or window placement persistence.

### 2. Signatures

```rust
pub enum WindowControlIntent { MinimizeToTray, ToggleMaximize, Close }
pub enum WindowVisibility { Visible, Hidden, Exiting }
pub struct WindowStateSnapshot {
    pub revision: u64, // generated as a JavaScript number
    pub visibility: WindowVisibility,
    pub maximized: bool,
}

window_state() -> WindowStateSnapshot
window_control(intent: WindowControlIntent) -> Result<WindowStateSnapshot, AppError>
```

Events are `window://state-changed` and `app://runtime-notice`. The fixed owned
tray ID is `wubilex-main-tray`; its only menu IDs are `tray.show` and
`tray.exit`.

### 3. Contracts

- `WindowCoordinator` is the only owner of native hide, restore, maximize,
  close, exit, and tray effects. Lifecycle operations are serialized, and
  `Exiting` is an irreversible terminal state.
- Hide creates the one owned tray before removing the window from the taskbar.
  Tray creation failure restores taskbar membership, unminimizes, shows, and
  focuses the main window so the application retains an entry point.
- Restore invalidates the delayed-tray generation, then independently attempts
  taskbar restore, unminimize, show, and focus. A partial failure is visible and
  does not create another window.
- A hidden `/tray` launch uses a cancellable three-second standard-thread wait.
  Its generation, hidden state, tray absence, and tray creation are checked
  under the same serialized lifecycle boundary. An early second instance may
  be consumed before a delay is scheduled; both that path and cancellation of
  an existing delay must leave no late tray creation.
- Only normal window bounds are sampled. Moved, resized, and scale changes are
  coalesced by the one placement worker after 250 ms quiet or two seconds
  maximum. Exit flush waits at most one second and never blocks exit forever.
- Placement updates mutate only bounds and maximized state inside the current
  config transaction, preserving a concurrently changed `closeAction`.
- The Rust binding registry is the only wire-type source. React sends generated
  intents and renders generated snapshots; it does not call native lifecycle
  methods directly.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Main window is unavailable | Return `windowUnavailable`, record a bounded runtime notice, and keep the process responsive |
| Tray creation fails | Return `windowOperationFailed`, fail open to a visible/taskbar window, and emit `trayUnavailable` once |
| One restore stage fails | Attempt all remaining stages, preserve the best native result, and return/report the first stable stage |
| Config snapshot or placement save fails | Use safe defaults or preserve native state and emit `windowPersistenceFailed` |
| Restore races delayed tray | The lifecycle serialization decides one order; restore-first cannot be followed by late tray creation |
| Hide or restore races exit | `Exiting` remains terminal and later controls cannot change its snapshot |
| Event emission fails | Log the stable event/stage only; do not roll back a successful native or config operation |

### 5. Good / Base / Bad Cases

- Good: close-to-tray creates one tray, a second instance restores the same
  window, the tray remains owned until exit, and the placement worker persists
  the latest normal bounds.
- Base: normal visible startup creates no tray; `/tray` stays hidden and either
  schedules its delay or consumes an early activation without scheduling one.
- Bad: an unconditional sleep creates a tray after restore, concurrent restore
  changes `Exiting` back to visible, tray failure leaves a minimized window
  without recovery, or a window event writes configuration synchronously.

### 6. Tests Required

- Pure bounds tests cover single/multiple monitors, negative coordinates,
  current monitor scale, removed/offscreen monitors, invalid values, and work
  areas smaller than the native minimum.
- Lifecycle tests assert idempotent revisions, irreversible `Exiting`, and stale
  delay-generation rejection.
- Placement tests assert quiet/latest-value coalescing, maximum-period flush,
  save-error reporting, and non-blocking stop; config tests assert preservation
  of the latest close action.
- Serialization and binding freshness tests assert command/event names, enum
  casing, and JavaScript-safe revision output.
- Elevated Windows smoke asserts hidden startup, both early/cancelled delay
  behavior, close-to-tray, second-instance restore, unique tray creation,
  abnormal marker evidence, close-action exit, and owned cleanup.

### 7. Wrong vs Correct

```rust
// Wrong: the timeout check and tray creation can be separated by restore.
if generation_is_current() {
    create_tray()?;
}

// Correct: final validation and creation share the serialized lifecycle order.
let _operation = self.lock_lifecycle_operation();
if self.delayed_tray_is_current(generation) {
    self.ensure_tray()?;
}
```

## Sources

- [`src-tauri/src/window/`](../../../src-tauri/src/window/)
- [Main application setup](../../../src-tauri/src/lib.rs)
- [Generated binding registry](../../../src-tauri/src/bindings/mod.rs)
- [Windows runtime smoke](../../../scripts/smoke-runtime.ps1)
