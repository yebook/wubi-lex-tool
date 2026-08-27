# Design - S1 运行时与生命周期

## 1. Runtime Shape

`src-tauri/src/main.rs` 只设置 Windows release subsystem 并调用 `wubilex_app::run()`。Library entry 拥有 builder composition、runtime state 和可测试的纯逻辑；binding export 继续通过 `bindings::builder::<MockRuntime>()`，不维护第二份 command/event registry。

Desktop binary 通过 crate feature 启用 Wry，`xtask` 关闭该 default feature 后继续使用 mock export。这样真实应用与无窗口 bindings 生成共享合同，但不会把 Wry 强加给生成工具。

启动顺序为：

1. 从 `args_os` 解析 primary launch request，不因无效输入退出。
2. 构建 Tauri app，并首先注册 single-instance plugin。
3. 在 setup 中解析应用目录、初始化日志、安装脱敏 panic hook。
4. 检查并接管 session marker，探测实际 elevation，创建 runtime snapshot。
5. 根据 `/tray` 创建隐藏或可见的 baseline WebView window。
6. 注册唯一 IPC builder，并在正常 `RunEvent::Exit` 清理当前 marker。

## 2. Module Ownership

```text
src-tauri/src/
  main.rs                 desktop-only entry
  lib.rs                  builder composition and run loop
  launch/                 pure CLI parser and typed launch request
  runtime/                snapshot, notice queue and single-instance dispatch
  logging/                tracing subscriber, rolling files and panic hook
  recovery/               session marker ownership only
  commands/app/           runtime snapshot command
  bindings/               only command/event registry

crates/wubilex-winime/src/
  security/               Windows token elevation probe
```

The React bootstrap stays intentionally small: `src/main.tsx` mounts one runtime status view that consumes generated IPC types. It does not establish the final component, token, route or store conventions owned by later S1 tasks.

## 3. Launch Contract

The pure parser consumes `OsString` values after the executable path and produces:

```text
LaunchRequest
  startHidden: bool
  navigationPath: Option<String>

LaunchNotice
  code: stable enum
  summary: Chinese user text
  detail: bounded technical detail without the raw argv vector
```

Accepted syntax is no arguments, `/tray`, `--navigate <path>`, or both. `/tray` is case-insensitive because it is a Windows compatibility switch. Navigation path validation owns only the transport envelope: leading `/`, maximum 256 Unicode scalar values, no control characters, backslash, query, fragment, empty segment `//`, `.` or `..` segment. The routing task later maps valid paths to stable route IDs.

Invalid input returns a notice plus a safe normal-start request. This guarantees visible diagnosis without turning an accidental shell argument into an application outage.

## 4. Single-Instance Data Flow

```text
second argv
  -> shared launch parser
  -> RuntimeState pending snapshot
  -> typed app://launch-requested event
  -> request main window show/unminimize/focus
  -> frontend consumes snapshot + later events
```

The plugin is registered first as required upstream. Runtime state is authoritative because an event emitted before React subscribes can be lost. The initial snapshot command returns current elevation, previous abnormal-session state, primary request, most recent secondary request and bounded notices. Events carry the same Rust-owned payload type and only accelerate updates.

The managed runtime state is installed before the single-instance plugin. Its setup-time initialization replaces the privilege/session fields while preserving any secondary request received through the plugin's native window during the narrow interval before application setup completes. Window activation is queued until the product window exists and remains retryable after a failed show/unminimize/focus sequence.

No second-instance callback executes navigation or domain logic. It submits a validated request for the routing layer to consume later.

## 5. Privilege Boundary

`build.rs` uses `tauri_build::WindowsAttributes::app_manifest(include_str!(...))`. The checked-in XML starts from the Tauri default Common Controls v6 manifest and adds:

```xml
<requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
```

Actual elevation is independently detected in `wubilex-winime` using the current process token. The adapter returns a typed native error with operation stage and Win32 code; Tauri converts probe failure into a visible runtime notice rather than assuming the manifest succeeded.

This task performs no privileged mutation. It does not add shell-open, process-launch, filesystem IPC or remote-content capabilities.

## 6. Session Marker

Each session owns one small JSON record under a dedicated Tauri application data subdirectory. The filename contains a unique session ID; the record contains schema version, session ID, process ID, app version and start timestamp. The service uses an injected directory, clock and ID source in tests.

Startup behavior:

1. Enumerate other owned marker files and report them as previous abnormal evidence.
2. Create the current session's unique marker with `create_new` and flush it.
3. Retain the exact owned path and session ID in managed state.

Normal exit removes only the exact marker path owned by the current managed state. A stale process or unrelated cleanup path therefore cannot delete another session's evidence. Panic and forced termination intentionally do not invoke cleanup. Old evidence is not deleted automatically until a later recovery flow explicitly acknowledges it.

The marker means “application session did not finish cleanly,” not “Windows system state is damaged.” S3 decides whether any real recovery is required.

## 7. Logging

The logging module builds a `tracing-subscriber` registry with JSON formatting and a non-blocking `tracing-appender` daily writer. Startup removes only owned log files older than seven days, and `max_log_files(7)` also bounds retained count. The worker guard lives for the whole Tauri process so exit flushes buffered entries. Development builds add a compact stderr layer.

Stable fields are `event`, `stage`, `pid`, `app_version` and optional error code. Launch notices log only stable code and argument position/type; the complete argv, navigation target, panic payload and user/domain content are forbidden. Panic hook records source location and payload type only, then chains to the previous hook while leaving the session marker armed.

## 8. Webview And Security

The Tauri config has no remote URL. Production CSP permits only bundled assets and Tauri IPC; development CSP adds the configured localhost dev server and its HMR WebSocket. Scripts and styles remain external, with no `unsafe-eval` or unrestricted inline source.

The main capability targets only the `main` window and grants exact `core:event:allow-listen` and `core:event:allow-unlisten` permissions. It does not grant frontend event emission. Custom runtime commands use the application-owned ACL path; no broad `core:default`, opener, shell, filesystem or process permission is added without evidence.

Vite injects development CSS through a style element, so serve mode emits a fixed development-only CSP nonce and `devCsp` allows that exact nonce. Production does not allow the nonce, `unsafe-inline`, or `unsafe-eval`; its compiled stylesheet remains external. Vite also ignores `target/**` so concurrent Rust artifact churn cannot make the Windows watcher traverse locked build files.

The baseline window remains native-decorated and otherwise minimal. `s1-window-tray` owns the later frameless coordinator, bounds restoration, close policy and tray.

## 9. Validation Architecture

- Rust pure tests: launch matrix, path bounds, non-Unicode input, session ownership, stale marker, atomic replacement failure, clean versus abnormal exit, redaction projection and privilege adapter outcomes.
- Tauri mock/contract tests: runtime command/event collected exactly once and serializable through generated bindings.
- Frontend Vitest: loading, elevated, permission failure, abnormal session and invalid argument notices.
- Static checks: manifest elevation/Common Controls, CSP/capability denial, metadata and forbidden integration search.
- Windows smoke: dev/release startup, second-instance handoff, `/tray` hidden creation, forced termination then restart warning, and clean exit marker removal.

## 10. Compatibility And Rollback

- Primary target is Windows 11 x64. The manifest, process-token APIs and single-instance plugin are compatible with the later Win10 1703 target; full release matrix remains deferred.
- `LaunchRequest`, launch notice codes and session marker schema become compatibility contracts once later tasks consume them. Changes require a central adapter or marker migration.
- If the official single-instance plugin fails the locked Tauri stack, record a minimal reproduction before evaluating a direct Windows primitive; do not create two live implementations.
- Runtime, logging, session marker and frontend bootstrap are kept in separable modules so any dependency or startup regression can be reverted without weakening CSP or elevation checks.
