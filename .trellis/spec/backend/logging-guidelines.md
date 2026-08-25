# Logging Guidelines

> Observability and redaction boundaries before product logging exists.

---

## Current Status

**Pending implementation evidence.** No production logging subscriber, file
sink, rotation policy or application logging call exists in the current tree.
The architecture names `tracing` and `tauri-plugin-log` as the starting stack,
but their versions and integration remain subject to implementation-time
review. They are not S0-established code conventions.

## Approved Boundary

- Product logs must be structured, leveled and written to rolling local files.
- The default retention requirement is seven days, with automatic cleanup.
- Every TSF shutdown-window stage and its duration must be observable. Native
  failures retain the operation stage, HRESULT or Win32 code and readable
  technical detail.
- Logs and exported diagnostics must never contain user input, lexicon entries,
  phrase content, credentials or other secrets.
- The product sends no telemetry. Any future telemetry requires explicit user
  consent and must be disabled by default.
- Library crates return structured errors to their caller. In particular,
  `wubilex-codec` must not initialize a logger or emit application policy.

## Decisions Not Yet Established

The project has not established:

- the exact `tracing` subscriber/plugin composition or dependency versions;
- required event field names, span hierarchy or correlation/task identifiers;
- the detailed mapping of `trace`, `debug`, `info`, `warn` and `error` to
  product events;
- log directory/file naming, rotation mechanics or cleanup implementation;
- the redaction helper/API, diagnostic bundle format or test capture strategy.

## Forbidden Premature Assumptions

- Do not add a logger, subscriber or Tauri plugin solely to populate this spec.
- Do not copy the plain-text risk-spike evidence format into product logging;
  those logs are task artifacts, not a runtime schema.
- Do not log raw lexicon/phrase/user-input fields even at debug or trace level.
- Do not swallow a structured error after logging it, and do not turn an error
  into a success because a log entry was written.
- Do not initialize global logging from a library crate.

## Update Trigger

Update this guide with real fields, level rules and source examples when the
first product logging/diagnostic task implements and tests the subscriber,
rolling files, retention and redaction behavior. A dependency selection alone
is insufficient evidence.

## Sources

- [`docs/20-nonfunctional.md` NFR-SEC-011 and NFR-OBS-001..007](../../../docs/20-nonfunctional.md)
- [`docs/02-architecture.md` dependency baseline](../../../docs/02-architecture.md)
- [Windows system integration](./windows-system-integration.md)
- [Backend error handling](./error-handling.md)
