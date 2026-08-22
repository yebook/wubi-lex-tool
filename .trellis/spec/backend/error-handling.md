# Error Handling

> Error ownership from pure libraries through the Tauri command boundary.

---

## Current Status

The library-side contract is established by `wubilex_codec::{CodecError, CodecErrorKind, SourceLocation}`. The application-side `AppError` conversion remains pending until Tauri command work begins.

## Error Ownership

- Library crates define typed, domain-specific errors with `thiserror` and return `Result`; they must not serialize UI-facing errors themselves.
- `src-tauri` may use `anyhow` for application orchestration context, but every command must finish by returning `Result<T, AppError>`.
- The command boundary owns conversion into the shared serializable `AppError`. Command handlers remain thin adapters and do not duplicate lower-layer recovery logic.

The shared application error contract contains:

| Field | Contract |
|---|---|
| `kind` | One of the approved categories: I/O, parse, network, permission, system, validation, or cancelled |
| `module` | Owning requirement module, such as `M1` or `M4` |
| `message` | User-readable Chinese description |
| `detail` | Optional technical evidence such as a system error code, line number, byte offset, or path |
| `recoverable` | Whether retry or another documented recovery action is valid |

## Propagation Rules

- Production paths must not use `unwrap()` or `expect()`. Propagate a typed error or handle the branch explicitly.
- Malformed lexicon and phrase inputs return an error with a line number or byte offset and never crash the process.
- Windows API failures include both the native error code and readable system text in technical detail.
- TSF shutdown-window steps check every result. A failure stops forward progress and still permits the RAII recovery guard to restore the system.
- Functional resource failures are surfaced. Silent degradation is reserved for resources explicitly classified as decorative.
- Cancellation is represented as the approved cancelled error category, not disguised as I/O or an empty successful result.
- Preserve the original source and add context at layer boundaries; do not discard a lower-layer error and replace it with a generic message.

## Established Codec Pattern

Codec failures separate a stable, matchable kind from an optional source location. A parser records expected and actual wire evidence in the kind, then attaches the byte or text position independently:

```rust
use wubilex_codec::{CodecError, CodecErrorKind, FieldValue};

let error = CodecError::new(CodecErrorKind::MalformedField {
    field: "eudp.entry.cb_size",
    expected: FieldValue::Unsigned(16),
    actual: FieldValue::Unsigned(12),
})
.at_byte_offset(64);
```

Use the dedicated structured variants for magic bytes, EOF byte counts, malformed fields, offset ranges, invalid UTF-16 surrogates, selected text encoding, unsupported variants, overflow context, and resource limits. Tests inspect `kind()` and `location()`; they must not parse `Display` output. Model constructors return the same error type without a location, allowing a future parser to attach the position it owns.

The `.lex` decoder demonstrates the binary-parser contract: signed header and alpha-index offsets are preserved in `InvalidOffset`, every malformed wire field is attached to its zero-based field offset, and truncated records return `UnexpectedEof` without partial documents. Canonical encoding reports arithmetic failures through `IntegerOverflow` without inventing a source location for an in-memory model.

## Common Mistakes To Reject

- Returning `null`, an empty collection, or a Boolean after an operation failed.
- Showing only a generic retry message while dropping the cause, error code, path, line, or offset.
- Logging an error and then returning success.
- Constructing a different ad hoc error payload in each command.
- Letting library crates depend on Tauri only to create `AppError`.

## Sources

- [`docs/02-architecture.md` sections 3.3 and 5.1](../../../docs/02-architecture.md)
- [`NFR-REL-001..009`](../../../docs/20-nonfunctional.md)
- [`src-tauri/README.md`](../../../src-tauri/README.md)
- [`wubilex-codec` error contract](../../../crates/wubilex-codec/src/error.rs)
- [`wubilex-codec` `.lex` decoder](../../../crates/wubilex-codec/src/lex/decode.rs)

The codec enum is established. Add the `AppError` conversion example when the command boundary is implemented rather than inventing it in advance.
