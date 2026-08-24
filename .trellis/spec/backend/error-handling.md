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

The `.lex` and EUDP decoders demonstrate the binary-parser contract: signed header and table offsets are preserved in `InvalidOffset`, every malformed wire field is attached to its zero-based field offset, and truncated records return `UnexpectedEof` without partial documents. EUDP tombstones are fully validated before omission, so a deleted flag cannot hide a malformed candidate, string, terminator, or offset. An unsupported EUDP `cbSize` uses `UnsupportedFormat`; canonical encoding reports model or arithmetic failures without inventing a source location for an in-memory value.

The community text decoder applies the same evidence split to text: malformed encoded bytes use `InvalidTextEncoding` at the original zero-based byte offset, while recognized invalid fields retain `InvalidInput` or `MalformedField` at the original one-based line and Unicode-scalar column. Unknown nonempty lines are compatibility diagnostics, not errors: `LexiconTextWarning` carries a structured kind, original location, bounded preview, and truncation flag. Warnings and entries share the expanded-output budget, and a nonempty body with no surviving entry remains an error rather than partial or empty success.

Phrase text follows the same strict byte and visible-warning contract through `phrase_text::decode`. Recognized P1-P6 shapes own their field errors, multiline state preserves the originating record location, unterminated comments report the opening delimiter, and array or automatic-candidate overflow returns a structured failure without a partial document. `PhraseTextWarning` previews are bounded to 160 Unicode scalar values and share one output budget with expanded entries.

The word-frequency and split-table decoders accept only BOM-less strict UTF-8 and exactly two nonempty Unicode-whitespace-delimited tokens per retained line. Unsupported BOMs report byte zero, malformed UTF-8 reports the original byte offset, and line structure or value failures report the original one-based line and Unicode-scalar column. Public auxiliary value objects use `ContainsWhitespace` to prevent construction of text that their canonical formatters could not read back unambiguously.

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
- [`wubilex-codec` EUDP decoder](../../../crates/wubilex-codec/src/eudp/decode.rs)
- [`wubilex-codec` community text decoder](../../../crates/wubilex-codec/src/text/decode.rs)
- [`wubilex-codec` phrase text decoder](../../../crates/wubilex-codec/src/text/phrase/decode.rs)
- [`wubilex-codec` auxiliary text parser](../../../crates/wubilex-codec/src/text/auxiliary.rs)

The codec enum is established. Add the `AppError` conversion example when the command boundary is implemented rather than inventing it in advance.
