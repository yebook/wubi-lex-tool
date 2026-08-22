# Quality Guidelines

> Rust quality gates and the S0 codec verification baseline.

---

## Required Gates

Backend changes must compile and pass formatting, Clippy with warnings denied, tests, and dependency license and vulnerability checks. CI obtains Rust from `rust-toolchain.toml`; workflow files must not hard-code another Rust version.

The approved CI sequence also checks generated IPC bindings and documentation contracts. A local check is not complete if it skips a gate affected by the change.

## Forbidden Patterns

- Do not map external `.lex` or EUDP bytes into `#[repr(C)]` structures with `transmute` or equivalent memory reinterpretation. Their variable lengths and alignment are not a Rust ABI contract; parse fields explicitly with bounds checks.
- Do not use `unwrap()` or `expect()` on production paths.
- Do not put domain transformations in Tauri command handlers or frontend callbacks.
- Do not add Tauri, Windows, network, or Tokio dependencies across the boundaries defined in the backend directory guide.
- Do not use machine-local fixtures or silently skip missing regression data.
- Do not treat the aardio implementation as a Rust implementation template. It is evidence for behavior only.

## Testing Requirements

| Area | Minimum evidence |
|---|---|
| `wubilex-codec` | Unit and property tests for round trips, dialects, version detection, boundaries, resource limits, and malformed input; measured coverage >= 90% |
| Codec fixtures | At least one reproducibly fetched real lexicon for each of 86, 98, 06, 091, 092, Zhengma, Xiaohe, and Biaoxingma |
| S0 regressions | Failure-to-pass tests for uppercase `XFXY`, asymmetric whitespace escaping, and the removed `codeWeight[0]` dead branch |
| `wubilex-core` | Input/output assertions for every implemented transform, slimming, weighting, and word-generation operation |
| `wubilex-winime` | Operation-sequence tests through recording dry-run behavior; real execution only in an isolated Windows CI environment |
| `wubilex-resource` | Mocked HTTP and hostile archive tests, including path traversal |
| `wubilex-app` | Serialization contract tests for commands and shared errors |

## Established S0 Codec Evidence

- `crates/wubilex-codec/tests/model_contracts.rs` fixes the validated model boundaries, ordered duplicate-preserving documents, nonzero weight/candidate ranges, UTF-16 code-unit behavior, scheme shape, and encoding/BOM contract.
- `crates/wubilex-codec/tests/error_and_limits.rs` fixes structured error evidence and the 64 MiB / 500,000 default limits. Assertions match error kinds and locations rather than rendered messages.
- `crates/wubilex-codec/tests/lex_binary.rs` uses hand-authored wire bytes to fix the `.lex` header, alpha-index, record, UTF-16, stable-order, implicit-weight, truncation, and resource-limit contracts independently of the production encoder. Its record-section truncation cases keep `fileSize` coherent so they exercise record bounds rather than stopping at header validation.
- The crate's only direct dependency at this stage is the exact `thiserror = 2.0.20` contract. Parser, encoding, platform, async, serialization, and network dependencies are introduced only by the task that uses them.
- Root-level user samples are not fixtures. Tests must use committed synthetic data or reproducibly fetched files under `crates/wubilex-codec/tests/fixtures/`; they must never depend on a machine-local `resource/` directory.

The remaining three known defect regressions belong to S4: EUDP drag-and-drop dispatch, duplicate Zhengma word generation, and the non-incrementing `unique()` loop.

## Scenario: Raw Microsoft Wubi `.lex` Codec

### 1. Scope / Trigger

Apply this contract whenever raw `imscwubi` bytes are decoded or a `LexiconDocument` is encoded. It covers the in-memory wire codec only. Filesystem access and `.lex.lzma` or `.lex.zst` containers remain in `wubilex-resource`.

### 2. Signatures

```rust
pub fn decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<LexiconDocument, CodecError>;

pub fn encode(document: &LexiconDocument) -> Result<Vec<u8>, CodecError>;
```

### 3. Contracts

- Decode checks the complete input limit before reading and the expanded-entry limit before appending each record.
- Magic and structural fields are strict: signed offsets must describe a complete index and record section, and declared file size must equal the input length. Version, marker, and reserved metadata are tolerated because they do not determine the documented layout.
- Decode preserves wire order and duplicates; every binary record produces an explicit nonzero weight.
- Encode emits canonical version 1.1 bytes, stably sorts by code, preserves equal-code order and duplicates, and resolves omitted weights from a per-code counter starting at zero.
- Decode errors carry the nearest zero-based wire offset. Encode arithmetic errors have no source location because the source is an in-memory model.

### 4. Validation & Error Matrix

| Condition | Required error |
|---|---|
| Input or expanded-entry ceiling exceeded | `ResourceLimitExceeded` with exact `limit` and `actual` |
| Magic differs from `imscwubi` | `MagicMismatch` at byte 0 with expected and actual bytes |
| Field bytes or a complete record are missing | `UnexpectedEof` at the field or record start |
| Signed header or alpha offset is negative or out of range | `InvalidOffset` preserving the signed wire value and valid range |
| Alpha index, record length, weight, code, padding, order, text, or terminator violates the wire contract | `MalformedField` at that field's byte offset |
| Text contains an unpaired UTF-16 surrogate | `InvalidUtf16` at the exact surrogate byte offset |
| Record length, file size, alpha offset, or implicit weight cannot be represented | `IntegerOverflow` with a stable operation name |

### 5. Good / Base / Bad Cases

- Good: a canonical sorted file decodes and re-encodes byte-for-byte, including duplicate records and non-BMP text.
- Base: an empty document encodes to the 168-byte canonical header and zero alpha index, then decodes to an empty document.
- Bad: a structurally valid alpha offset that points to the wrong record boundary is rejected even when all records themselves parse successfully.

### 6. Tests Required

- Hand-authored canonical bytes must assert decoded fields and exact encoded bytes independently of the production writer.
- Header, signed offset, semantic alpha-index, record field, strict UTF-16, implicit-weight reset, maximum record length, and resource-limit boundaries must assert both `kind()` and `location()`.
- Truncated record-section tests must update declared `fileSize`; otherwise they only exercise header validation and are tautological for record safety.
- Machine-local real samples may be used for a reported manual byte comparison, but automated tests must not reference the root `resource/` directory.

### 7. Wrong vs Correct

```rust
// Wrong: trusts indexes, performs lossy text conversion, and hides corruption.
let text = String::from_utf16_lossy(units);

// Correct: bounds-check every field, validate indexes against parsed record
// boundaries, and return InvalidUtf16 with the original byte offset.
let document = wubilex_codec::lex::decode(bytes, DecodeLimits::default())?;
```

## Review Checklist

- Does the change stay within its crate's dependency and responsibility boundary?
- Does every parser validate offsets, lengths, UTF-16 units, and resource limits before access or allocation?
- Are behavior contracts backed by round-trip, boundary, malformed-input, or regression tests as appropriate?
- Do failures preserve actionable context without panicking or becoming an empty success?
- If a Rust IPC contract changed, were generated TypeScript bindings regenerated and checked?
- Did the change preserve the requirement ID and documentation-anchor contracts?

## Sources

- [`docs/01-data-formats.md` implementation notes](../../../docs/01-data-formats.md)
- [`docs/02-architecture.md` sections 0, 8, and 8.5](../../../docs/02-architecture.md)
- [`NFR-REL-004`, `NFR-MAINT-001..006`](../../../docs/20-nonfunctional.md)
- [`docs/22-roadmap.md` S0 and S4](../../../docs/22-roadmap.md)
- [`wubilex-codec` contract tests](../../../crates/wubilex-codec/tests/model_contracts.rs)

The model, error, limit, and raw `.lex` round-trip tests are established examples. EUDP and text parser round trips, reproducible eight-scheme fixtures, and measured coverage remain obligations for later S0 tasks.
