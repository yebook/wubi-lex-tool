# Logging Guidelines

> Structured application logging, retention, and redaction contracts.

---

## Current Status

**Established by the S1 runtime lifecycle.** `src-tauri` initializes one global
`tracing-subscriber` registry with an application-target filter, a daily JSONL
file sink, a bounded non-blocking worker, and a compact development stderr
layer. Library crates still return typed errors and never install application
logging policy.

## Runtime Contract

- `src-tauri/src/logging/mod.rs` is the only subscriber owner. Runtime startup
  retains its `LoggingGuard` for the complete Tauri process lifetime so queued
  records flush when the event loop returns.
- File logs use exact `tracing 0.1.44`, `tracing-subscriber 0.3.23`, and
  `tracing-appender 0.2.5`. Files are named `wubilex.YYYY-MM-DD.jsonl`, rotate
  daily, and are limited to seven files.
- Startup parses the owned filename date and removes only owned files older
  than seven UTC calendar days. File modification time is not retention truth,
  and unrelated or malformed near-match files remain untouched.
- File and development stderr layers accept only the `wubilex_app` target and
  its modules. Dependency events are excluded because their field safety is not
  controlled by the application.
- Every product record includes the formatter-owned timestamp, level and
  target plus explicit stable `event`, `stage`, `pid`, and `app_version`
  fields. Operation-specific bounded fields may be added when they contain no
  user or domain content.
- Logging setup failure becomes a visible `LoggingUnavailable` runtime notice
  and does not prevent the application window from starting.

## Redaction Contract

- Never record a complete argv vector, navigation target, working directory,
  panic payload, user input, lexicon entry, phrase content, credential, secret,
  or arbitrary frontend payload at any level.
- Launch diagnostics are projected to stable notice code and one-based argument
  position before logging. Summary, detail, and original argument values are
  excluded from the logging projection.
- The panic hook records only source location and payload type, then chains to
  the previous hook. It does not format or debug-print the payload into the
  structured sink.
- Native failures may record their stable operation stage and numeric HRESULT
  or Win32 code. Add readable system text only after confirming that the API
  cannot echo user-controlled content.
- Logging a failure never converts it to success or discards the typed error
  returned to the owning boundary.

## Level Rules

- `info`: lifecycle milestones and successful bounded control-plane events.
- `warn`: recoverable launch, event-delivery, window-activation, or degraded
  diagnostic conditions.
- `error`: panic evidence and failures during ownership-sensitive cleanup.
- `trace` and `debug` have no blanket payload exemption; the same redaction
  contract applies even in development builds.

## Tests Required

- Assert exact owned-name recognition, calendar-date validity, the seven-day
  boundary, and preservation of unrelated files.
- Assert logging projections cannot render launch summary, detail, raw argument,
  navigation target, or panic payload content.
- Review every new `tracing` call for the required stable fields and forbidden
  values. Keep a case-insensitive source search as a secondary audit, not as a
  substitute for typed projections.
- Run strict Clippy and the focused `wubilex-app` tests after changing subscriber
  composition, retention, fields, or redaction.

## Still Deferred

Correlation/task identifiers, TSF stage-duration events, exported diagnostic
bundles, and any telemetry policy remain unimplemented. Telemetry still
requires explicit user consent and must default to disabled.

## Sources

- [Application logging implementation](../../../src-tauri/src/logging/mod.rs)
- [Runtime logging call sites](../../../src-tauri/src/lib.rs)
- [`docs/20-nonfunctional.md` NFR-SEC-011 and NFR-OBS-001..007](../../../docs/20-nonfunctional.md)
- [Windows system integration](./windows-system-integration.md)
- [Backend error handling](./error-handling.md)
