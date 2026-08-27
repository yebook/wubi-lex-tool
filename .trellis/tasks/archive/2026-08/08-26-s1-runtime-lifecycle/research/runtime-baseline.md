# Runtime Lifecycle Research Baseline

## Repository Evidence

- `src-tauri/src/lib.rs` currently exports only `bindings`; no desktop binary or runtime builder exists.
- `src-tauri/src/bindings/mod.rs` is the single generic command/event registry and exports with `MockRuntime`.
- `src-tauri/Cargo.toml` disables Tauri default features and enables only `test`, so the current crate cannot create a Wry window.
- `src-tauri/tauri.conf.json` contains identity fields only; it has no build, app window, security or capability wiring.
- `src/` contains only generated bindings and `vite-env.d.ts`; there is no `index.html` or React mount entry.
- Root `package.json` already contains React 19, Vite 8, Tailwind 4 and Tauri CLI 2.11.4; runtime work needs scripts, not an alternate package manager.

## Requirement Evidence

- `docs/modules/M7-app-shell.md` lines 20-46 define `M7-INST-001..006` and approve whole-process elevation for phase one.
- `docs/20-nonfunctional.md` defines the two-second startup target, elevation disclosure/detection, strict CSP/minimal capability, log redaction, abnormal-session evidence and rolling logs.
- Parent S1 planning limits this child to runnable entry, single instance, privilege, arguments, logging and abnormal session marker; window/tray, configuration and final feedback remain later children.
- Deprecated `M7-WIN-005` forbids all ImTip integration and is not a placeholder feature.

## Local Upstream Evidence

- `tauri-plugin-single-instance 2.4.3` supports Windows and Tauri 2. Its README requires registering the plugin first; its callback supplies `AppHandle`, argv and cwd.
- `tauri-build 2.6.3` exposes `WindowsAttributes::app_manifest`. Its default XML contains Common Controls v6, and its documented elevation example uses `requireAdministrator`.
- `tracing-appender 0.2.5` exposes daily rotation and `max_log_files`; a small owned-file cleanup pass is still required for exact seven-day retention.
- The installed Tauri capability schema confirms capabilities can target the exact `main` window and omit all remote origins.

## Resolved Technical Decisions

1. Use the official single-instance plugin instead of a custom mutex/IPC channel.
2. Keep launch parsing pure and shared by primary and secondary paths.
3. Preserve missed events through an authoritative runtime snapshot.
4. Put direct Windows token inspection in `wubilex-winime`, not Tauri commands or React.
5. Use a checked-in application manifest with both elevation and Common Controls.
6. Use structured `tracing` JSON with daily rolling, seven-day/seven-file retention and a non-blocking guard.
7. Give each session a unique `create_new` marker so normal cleanup cannot delete another session's evidence.
8. Treat the session marker as abnormal-process evidence only; never infer that system repair is required.
9. Keep the initial React page intentionally minimal so later UI tasks establish the durable design conventions.

## Deferred Verification

- Exact release startup p95 measurement is owned by `s1-integration`; this child only records a smoke baseline.
- Final `/tray` no-flash and delayed tray creation are owned by `s1-window-tray`.
- Final route vocabulary and navigation semantics are owned by `s1-routing-shell`.
- Full first-run persistence and recovery UI are owned by config and feedback children.
