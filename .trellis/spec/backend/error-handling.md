# Error Handling

> Error ownership from pure libraries through the Tauri command boundary.

---

## Current Status

The library-side contract is established by `wubilex_codec::{CodecError, CodecErrorKind, SourceLocation}`. Repository automation also has established stage-preserving failures. S1 configuration commands establish the application-side generated `AppError` boundary in `src-tauri/src/error/mod.rs`; later modules must extend that shared contract instead of creating command-local payloads.

## Error Ownership

- Library crates define typed, domain-specific errors with `thiserror` and return `Result`; they must not serialize UI-facing errors themselves.
- `src-tauri` may use `anyhow` for application orchestration context, but every command must finish by returning `Result<T, AppError>`.
- The command boundary owns conversion into the shared serializable `AppError`. Command handlers remain thin adapters and do not duplicate lower-layer recovery logic.

The shared application error contract contains:

| Field | Contract |
|---|---|
| `code` | Stable camelCase operation identifier such as `configValidationFailed` or `configReplaceFailed` |
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

The `.lex` and EUDP decoders demonstrate the binary-parser contract: signed header and table offsets are preserved in `InvalidOffset`, every malformed wire field is attached to its zero-based field offset, and truncated records return `UnexpectedEof` without partial documents. EUDP tombstones are fully validated before omission, so a deleted flag cannot hide a malformed candidate, string, terminator, or offset. An unsupported EUDP `cbSize` uses `UnsupportedFormat`; canonical encoding reports model or arithmetic failures without inventing a source location for an in-memory value.

The community text decoder applies the same evidence split to text: malformed encoded bytes use `InvalidTextEncoding` at the original zero-based byte offset, while recognized invalid fields retain `InvalidInput` or `MalformedField` at the original one-based line and Unicode-scalar column. Unknown nonempty lines are compatibility diagnostics, not errors: `LexiconTextWarning` carries a structured kind, original location, bounded preview, and truncation flag. Warnings and entries share the expanded-output budget, and a nonempty body with no surviving entry remains an error rather than partial or empty success.

Phrase text follows the same strict byte and visible-warning contract through `phrase_text::decode`. Recognized P1-P6 shapes own their field errors, multiline state preserves the originating record location, unterminated comments report the opening delimiter, and array or automatic-candidate overflow returns a structured failure without a partial document. `PhraseTextWarning` previews are bounded to 160 Unicode scalar values and share one output budget with expanded entries.

The word-frequency and split-table decoders accept only BOM-less strict UTF-8 and exactly two nonempty Unicode-whitespace-delimited tokens per retained line. Unsupported BOMs report byte zero, malformed UTF-8 reports the original byte offset, and line structure or value failures report the original one-based line and Unicode-scalar column. Public auxiliary value objects use `ContainsWhitespace` to prevent construction of text that their canonical formatters could not read back unambiguously.

Repository fixture automation preserves the failing stage and entry in its command error chain: manifest loading/validation, cache verification, download, compressed integrity, LZMA decode, decoded integrity, strict `.lex` validation, and final placement remain distinguishable. `cargo xtask fixtures --check` never repairs or performs network work; it reports the invalid entry and the `cargo xtask fixtures` recovery command. Download cleanup is ownership-based: create the partial file successfully before arming its guard, and disarm only after validated placement. A failed `create_new` call must never authorize deletion of an existing or concurrently owned path.

## Scenario: Configuration Command Errors

### 1. Scope / Trigger

Apply this contract to configuration snapshot, grouped update, restore, import,
export, persistence, recovery, and blocking-task failures.

### 2. Signatures

```rust
pub struct AppError {
    pub code: AppErrorCode,
    pub kind: AppErrorKind,
    pub module: RequirementModule,
    pub message: String,
    pub detail: Option<String>,
    pub recoverable: bool,
}

pub async fn config_update_ui(...) -> Result<ConfigSnapshot, AppError>;
```

### 3. Contracts

- Rust owns `AppErrorCode`, `AppErrorKind`, `RequirementModule`, and `AppError`; bindings generate the TypeScript union.
- Configuration errors use module `m7`, a Chinese user message, and technical detail bounded to 1,024 Unicode scalar values.
- Detail may contain stage, error kind/code, disposition, and involved paths. It must never contain complete TOML, shortcut values, argv values, or other raw user payloads.
- A 1177 replace plus failed restore retains primary, restore, cleanup, target, staging, and backup evidence. Cleanup never replaces the primary failure.
- `spawn_blocking` join failures become `configStateFailed`; an event emit failure is logged after commit and is not returned as a false persistence failure.

### 4. Validation & Error Matrix

| Condition | Required error |
|---|---|
| Empty, relative, directory, or config-owned path | `configInvalidPath` or stage-specific import/export inspection failure |
| TOML/UTF-8/size decode failure | `configParseFailed` without document contents |
| Missing, zero, unsupported, or future version on import | `configUnsupportedVersion` |
| Bounded model validation failure | `configValidationFailed` with field/reason only |
| Staging write/sync/close/install failure | `configWriteFailed` with original stage |
| Backup selection or native replacement failure | `configBackupFailed` or `configReplaceFailed` |
| Poisoned state or blocking-task join failure | `configStateFailed` |

### 5. Good / Base / Bad Cases

- Good: failed 1177 restore reports every recovery stage and leaves last-valid bytes at the named owned backup.
- Base: a malformed import returns a bounded parse error and leaves snapshot, revision, and live bytes unchanged.
- Bad: logging a replace error and returning success, exposing TOML in detail, or returning only the cleanup error.

### 6. Tests Required

- Serialize representative codes, categories, module, nullable detail, and recoverability through generated bindings.
- Inject create, write, flush, sync, close, backup, replace, restore, and cleanup failures; assert the primary stage and secondary evidence.
- Assert invalid input values and shortcut contents never occur in error detail or recovery notices.
- Assert event-emission failure does not roll back a completed transaction.

### 7. Wrong vs Correct

```rust
// Wrong: loses the native stage and lies about rollback.
return Err(AppError::generic("保存失败"));

// Correct: preserve primary and recovery evidence; commit memory only on success.
return Err(AppError::config(code, kind, message, bounded_detail, true));
```

Binding and document automation follows the same stage-preserving rule. `cargo xtask bindings --check` reports missing and stale generated output with the exact regeneration command and never repairs it. Export, normalization, read, staging, synchronization, and replacement failures retain the path and stage. `cargo xtask check-docs` aggregates definition, count, dangling-reference, placeholder, and anchor failures; Python spawn failures and nonzero anchor output remain visible rather than becoming a generic validation error.

## Common Mistakes To Reject

- Returning `null`, an empty collection, or a Boolean after an operation failed.
- Showing only a generic retry message while dropping the cause, error code, path, line, or offset.
- Logging an error and then returning success.
- Constructing a different ad hoc error payload in each command.
- Letting library crates depend on Tauri only to create `AppError`.
- Arming a temporary-file cleanup guard before `create_new` succeeds, which can delete a partial file owned by another run.
- Returning success or silently skipping when an ignored real fixture is missing; report the fixture id and preparation command instead.
- Repairing generated bindings during `--check`, swallowing an anchor subprocess failure, or reporting only the first independently detectable documentation issue.

## Sources

- [`docs/02-architecture.md` sections 3.3 and 5.1](../../../docs/02-architecture.md)
- [`NFR-REL-001..009`](../../../docs/20-nonfunctional.md)
- [`src-tauri/README.md`](../../../src-tauri/README.md)
- [`wubilex-codec` error contract](../../../crates/wubilex-codec/src/error.rs)
- [`wubilex-codec` `.lex` decoder](../../../crates/wubilex-codec/src/lex/decode.rs)
- [`wubilex-codec` EUDP decoder](../../../crates/wubilex-codec/src/eudp/decode.rs)
- [`wubilex-codec` community text decoder](../../../crates/wubilex-codec/src/text/decode.rs)
- [`wubilex-codec` phrase text decoder](../../../crates/wubilex-codec/src/text/phrase/decode.rs)
- [`wubilex-codec` auxiliary text parser](../../../crates/wubilex-codec/src/text/auxiliary.rs)
- [`xtask` fixture failure stages and cleanup guards](../../../xtask/src/fixtures.rs)
- [`xtask` binding failure stages](../../../xtask/src/bindings.rs)
- [`xtask` document failure aggregation](../../../xtask/src/check_docs.rs)
- [Generated application error contract](../../../src-tauri/src/error/mod.rs)
- [Configuration error mapping](../../../src-tauri/src/config/mod.rs)

The codec enum and the first application `AppError` conversion are established. Extend the shared application enums when later command modules need new stable codes; do not fork the payload shape.
