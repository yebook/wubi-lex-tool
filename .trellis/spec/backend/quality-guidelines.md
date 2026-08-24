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
| S0 regressions | Failure-to-pass tests for lowercase `xfxy`, asymmetric whitespace escaping, and the removed `codeWeight[0]` dead branch |
| `wubilex-core` | Input/output assertions for every implemented transform, slimming, weighting, and word-generation operation |
| `wubilex-winime` | Operation-sequence tests through recording dry-run behavior; real execution only in an isolated Windows CI environment |
| `wubilex-resource` | Mocked HTTP and hostile archive tests, including path traversal |
| `wubilex-app` | Serialization contract tests for commands and shared errors |

## Established S0 Codec Evidence

- `crates/wubilex-codec/tests/model_contracts.rs` fixes the validated model boundaries, ordered duplicate-preserving documents, nonzero weight/candidate ranges, UTF-16 code-unit behavior, scheme shape, and encoding/BOM contract.
- `crates/wubilex-codec/tests/error_and_limits.rs` fixes structured error evidence and the 64 MiB / 500,000 default limits. Assertions match error kinds and locations rather than rendered messages.
- `crates/wubilex-codec/tests/lex_binary.rs` uses hand-authored wire bytes to fix the `.lex` header, alpha-index, record, UTF-16, stable-order, implicit-weight, truncation, and resource-limit contracts independently of the production encoder. Its record-section truncation cases keep `fileSize` coherent so they exercise record bounds rather than stopping at header validation.
- `crates/wubilex-codec/tests/eudp_binary.rs` uses hand-authored wire bytes to fix the EUDP header, relative offset table, record, tombstone, candidate, strict UTF-16/NUL, stable-order, timestamp, truncation, and resource-limit contracts independently of the production encoder.
- `crates/wubilex-codec/tests/text_decode.rs`, `text_format.rs`, and `text_public_contracts.rs` fix strict encoding offsets, preprocessing, six dialects, Microsoft/Jidian branches, bounded visible warnings, stream-checked expansion limits, seven canonical outputs, UTF-16LE bytes, and the `%0B` / `%0C` escape regression.
- `crates/wubilex-codec/tests/phrase_text.rs` fixes strict shared encoding, P1-P6 priority, multiline state, comments, arrays, time aliases, bounded visible warnings, candidate and output limits, CRLF canonical formatting, UTF-16 compression boundaries, and malformed-input locations.
- `crates/wubilex-codec/tests/auxiliary_text.rs` fixes BOM-less UTF-8, exact two-column parsing, ordered duplicates, weight bounds, PUA and non-BMP preservation, LF canonical formatting, resource ceilings, and malformed-input locations for word-frequency and split-table documents.
- `crates/wubilex-codec/tests/scheme_detection.rs` fixes the five ordered direct branches, four scored branches, strict-winner fallback, duplicate and weight independence, and the lowercase `xfxy` defect regression.
- `crates/wubilex-codec/tests/real_fixtures.rs` uses the committed manifest to require all eight downloaded samples, verify decoded size and SHA-256, strictly decode, assert the expected scheme, and byte-reencode each document. Missing files fail with an actionable `cargo xtask fixtures` command; they are never skipped.
- `crates/wubilex-codec/tests/properties.rs` fixes bounded canonical `.lex` and EUDP round trips, duplicate preservation, all six ASCII whitespace escapes, canonical weighted-text reformatting, and no-panic behavior for bounded arbitrary or mutated bytes. Text-format strategies exclude unsupported Unicode whitespace and ambiguous literal percent sequences rather than assuming every model value is representable by every text grammar.
- `crates/wubilex-codec/tests/cross_codec.rs` fixes deterministic real-document projections into all seven lexicon text formats, phrase text/EUDP semantic round trips, and the named `codeWeight[0]` regression. Real and cross-codec tests share `tests/support/mod.rs` as the only test-side manifest contract.
- The crate's direct dependencies are exact-pinned. `thiserror = 2.0.20` owns errors; `chardetng = 1.0.0` and `encoding_rs = 0.8.35` own deterministic text detection and strict decoding without enabling Rayon. Platform, async, serialization, and network dependencies remain forbidden.
- Root-level user samples are not fixtures. Tests must use committed synthetic data or reproducibly fetched files under `crates/wubilex-codec/tests/fixtures/`; they must never depend on a machine-local `resource/` directory.

The remaining three known defect regressions belong to S4: EUDP drag-and-drop dispatch, duplicate Zhengma word generation, and the non-incrementing `unique()` loop.

## Scenario: Reproducible Real Codec Fixtures

### 1. Scope / Trigger

Apply this contract when adding or changing real codec regression data, the `cargo xtask fixtures` command, fixture cache behavior, or codec coverage measurement. This is repository test infrastructure only. Product download, archive, cache, and redistribution behavior remains owned by `wubilex-resource` and later product tasks.

### 2. Signatures

```text
cargo xtask fixtures
cargo xtask fixtures --check
```

The first command verifies and reuses valid cache entries or downloads and repairs invalid entries. The second command is strictly offline and reports every missing or invalid entry without repair.

### 3. Contracts

- `crates/wubilex-codec/tests/fixtures/manifest.json` is the single committed source of fixture entries. Each entry includes a unique `id` and `scheme`, an HTTPS `url`, portable archive/decoded file names, exact compressed and decoded byte sizes, lowercase 64-character SHA-256 values, source attribution, and a license note.
- The manifest contains exactly 86, 98, 06, 091, 092, Zhengma, Xiaohe, and Biaoxingma. Tests locate files through the shared manifest loader instead of hard-coded paths or a second scheme list.
- Downloads accept at most five redirects, require both the initial and final URL to be HTTPS, cap compressed input at 16 MiB, and decode LZMA-alone output through a 64 MiB bounded writer.
- Size and SHA-256 are verified for compressed and decoded content. Strict `.lex` decode and expected magic are required before either final path is installed.
- Temporary files use `create_new` in the destination directory. A cleanup guard takes ownership only after creation succeeds, so a collision cannot delete another run's partial file. Both temporary payloads must validate before final replacement; cache validity always requires both final files.
- Standard proxy environment variables may be honored by the HTTP client. Repository code and committed configuration must not contain proxy hosts, credentials, or machine-specific paths.
- Downloaded payloads and partial files are ignored. Root `resource/` is user data and is never a manifest source, fallback cache, or integrity oracle.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Manifest schema, scheme set, uniqueness, path, URL, size, or digest is invalid | Fail before cache or network work with entry-specific context |
| `--check` sees a missing, mismatched, corrupt, or partial cache | Fail offline and print the actionable `cargo xtask fixtures` preparation command |
| Initial/final URL is not HTTPS or redirect count exceeds five | Reject the response and leave no newly owned partial file |
| Compressed body exceeds 16 MiB or declared size/digest differs | Abort, remove only the partial created by this run, and preserve prior final files |
| LZMA output exceeds 64 MiB or decoded size/digest/magic/strict decode fails | Abort before final placement and remove newly owned temporary files |
| One final replacement is interrupted | The next cache verification rejects the pair unless both final files pass complete validation |
| A real fixture is absent during tests | Hard failure with `cargo xtask fixtures`; never ignore or return early success |
| Codec line coverage is below 90% | The workspace-local `cargo-llvm-cov` command exits nonzero via `--fail-under-lines 90` |

### 5. Good / Base / Bad Cases

- Good: an empty cache is populated from the eight pinned HTTPS URLs, every size/digest and strict decode passes, and a second run reuses all files without network access.
- Base: `cargo xtask fixtures --check` on a complete valid cache performs no download and succeeds; a valid `create_new` collision remains untouched.
- Bad: a stale partial, one corrupt half of a pair, upstream drift, oversized expansion, or missing real file produces a visible failure and cannot be consumed by codec tests.

### 6. Tests Required

- Unit-test manifest rejection, compressed and decoded bounds, valid cache, corrupt cache, stale partial files, and `create_new` collision ownership.
- Run one fresh fetch from an empty cache after hashes are pinned, then run the warm default command and offline `--check` against all eight entries.
- For every real `.lex`, assert exact decoded size/digest, nonempty strict decode, expected scheme, and byte-identical re-encode.
- Keep bounded property tests for `.lex`, EUDP, six whitespace escapes, representable text models, and arbitrary/mutated invalid bytes. Convert any stable counterexample into a named regression.
- Measure all `wubilex-codec` tests with an exact workspace-local `cargo-llvm-cov` executable and `--fail-under-lines 90`. Coverage additions must assert behavior and must not exclude business source merely to pass the threshold.

### 7. Wrong vs Correct

```rust
// Wrong: silently skips a developer-only file and lets CI pass without evidence.
if !Path::new("../../resource/sample.lex").exists() {
    return;
}

// Correct: resolve the pinned manifest entry and fail with its preparation step.
let fixture = manifest.fixture("wubi86")?;
let bytes = read_required_fixture(fixture, "cargo xtask fixtures")?;
```

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

## Scenario: Raw Microsoft Wubi EUDP Codec

### 1. Scope / Trigger

Apply this contract whenever raw `mschxudp` bytes are decoded or a `PhraseDocument` is encoded. It covers the synchronous in-memory wire codec only. The v1/v2 names identify system paths containing the same bytes; path selection, dual writes, clock access, Windows checks, TSF orchestration, and recovery remain outside `wubilex-codec`.

### 2. Signatures

```rust
pub fn decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<PhraseDocument, CodecError>;

pub fn encode(
    document: &PhraseDocument,
    timestamp: i32,
) -> Result<Vec<u8>, CodecError>;
```

### 3. Contracts

- Decode checks the complete input limit before parsing and the declared wire count before allocating the offset table. Deleted records count toward this ceiling.
- Magic and structural fields are strict: signed offsets must describe a complete contiguous table and record section, `phraseEnd` equals the input length, the first relative offset is zero, and later offsets strictly increase.
- Non-structural header/record metadata is tolerated on decode and normalized on encode. `cbSize` is the explicit record-layout discriminator and must equal 16.
- Every record has a nonzero candidate, a nonempty lowercase ASCII code, and nonempty strictly valid UTF-16LE text. Code and text contain no embedded `U+0000` and each ends with its own NUL code unit.
- Decode validates every tombstone before omitting it, preserves active wire order and duplicates, and requires codes to be lexicographically nondecreasing. Candidate values do not impose a second physical sort order.
- Encode stably sorts by code only, preserves equal-code order/candidates/duplicates, emits canonical constants, and writes the caller-supplied `i32` timestamp verbatim. Equal document and timestamp inputs produce equal bytes.

### 4. Validation & Error Matrix

| Condition | Required error |
|---|---|
| Input or declared wire-count ceiling exceeded | `ResourceLimitExceeded` with exact `limit` and `actual`; count errors are located at byte 28 |
| Magic differs from `mschxudp` | `MagicMismatch` at byte 0 with expected and actual bytes |
| Header bytes or a complete bounded record are missing | `UnexpectedEof` at the field or record start |
| Signed header/table offset is negative or outside its valid range | `InvalidOffset` preserving the signed value and bounds |
| Table end, first/order constraints, record length, text offset parity, candidate, code/order, text, or terminator is invalid | `MalformedField` at the nearest wire field |
| `cbSize` is not 16 | `UnsupportedFormat { format: "eudp", ... }` at the record start |
| Code or text contains an unpaired surrogate | `InvalidUtf16` at the exact surrogate byte |
| Count, table size, text offset, relative offset, record size, or file size cannot be represented | `IntegerOverflow` with a stable operation and no invented model-source location |
| Encode receives phrase text containing `U+0000` | Structured field failure with no byte location because no wire input exists |

### 5. Good / Base / Bad Cases

- Good: canonical bytes containing duplicate phrases, candidate 255, an emoji surrogate pair, a newline, and a `%yyyy%` variable decode and re-encode byte-for-byte when the same timestamp is supplied.
- Base: an empty document encodes to a 64-byte canonical header with `count = 0`, then decodes to an empty document.
- Bad: a deleted record with an invalid candidate, empty text, malformed UTF-16, or bad terminator is rejected rather than silently omitted.

### 6. Tests Required

- Hand-authored canonical bytes must assert decoded entries, relative offsets, timestamp, metadata normalization, and exact encoded bytes independently of the production writer.
- Header, offset-table, record-header, candidate, code/text, embedded NUL, strict UTF-16, tombstone, ordering, maximum text offset, and resource-limit boundaries must assert both `kind()` and `location()` where a wire source exists.
- Every truncated prefix must return an error without panic. Record-section prefixes must update `phraseEnd` so they exercise table/record validation instead of only header-size mismatch.
- Tests must prove equal-code records are stable even when candidates are not monotonic, and that tombstones still count toward the declared wire-entry limit.
- Machine-local files under the root `resource/` directory are never automated fixtures. Real Windows EUDP compatibility remains pending until a reproducible sample source is established.

### 7. Wrong vs Correct

```rust
// Wrong: reads the clock inside the codec and skips corrupt tombstones.
let bytes = encode_with_system_time(document)?;

// Correct: keep output deterministic and validate every wire record first.
let document = wubilex_codec::eudp::decode(bytes, DecodeLimits::default())?;
let encoded = wubilex_codec::eudp::encode(&document, timestamp)?;
```

## Scenario: Community Lexicon Text Codec

### 1. Scope / Trigger

Apply this contract when encoded community lexicon text is decoded into a `LexiconDocument`, when a document is formatted into one of the seven text layouts, or when lexicon/phrase text needs the shared ASCII whitespace escapes. The codec remains synchronous and memory-to-memory; file paths, filesystem I/O, containers, scheme detection, and domain indexes are separate concerns.

### 2. Signatures

```rust
pub fn decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<DecodedLexiconText, CodecError>;

pub fn format(
    document: &LexiconDocument,
    format: LexiconTextFormat,
) -> Result<String, CodecError>;

pub fn encode_utf16le(
    document: &LexiconDocument,
    format: LexiconTextFormat,
) -> Result<Vec<u8>, CodecError>;

pub fn escape_whitespace(input: &str) -> String;
pub fn unescape_whitespace(input: &str) -> String;
```

### 3. Contracts

- Decode checks the complete byte limit before BOM selection or allocation. BOMs select UTF-8 or UTF-16LE/BE; valid BOM-less UTF-8 wins, while other detector guesses narrow to strict GBK. Decoding never inserts replacement characters.
- Preprocessing keeps original line provenance and runs YAML, `#` comments, `[Text]`, Jidian, then Microsoft detection in that order. The main parser tries A through F with `NoMatch`, `Matched`, and `Invalid` outcomes, so recognized corruption cannot fall through into a warning.
- Each expanded entry and each unknown-line warning is charged to `max_expanded_entries` before it is retained. Multi-entry lines are tokenized and checked incrementally instead of collecting unbounded tokens before the ceiling is known.
- Unknown nonempty lines produce ordered `UnrecognizedLine` warnings with their original one-based location, at most 160 Unicode scalar values of preview, and an explicit truncation flag. A nonempty body with no surviving entry is still an error.
- D weights use `65535 - source` and only D entries participate in the minimum-to-5000 shift. Jidian cleanup drops `^`, `$`, and `!`, strips one `~`, clears weights, and rejects a stripped-empty text.
- All seven formats use one stable code/effective-weight projection without mutating the document. Non-aggregate forms preserve duplicates; aggregate forms fold only adjacent duplicates. Every nonempty output line ends in CRLF, and UTF-16LE output begins with `FF FE`.
- Escaping is symmetric only for `%20`, `%09`, `%0A`, `%0D`, `%0B`, and `%0C`. Unknown, lowercase, incomplete, and literal percent sequences stay literal; do not add `%25`.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Input or retained entry/warning ceiling exceeded | `ResourceLimitExceeded` with exact `limit` / `actual`; expanded output also has the current source line |
| UTF-8, UTF-16, or GBK bytes are malformed | `InvalidTextEncoding` at the original zero-based byte offset, including any removed BOM prefix |
| YAML is unclosed or a nonempty body has zero surviving entries | `MalformedField` at the original one-based text location |
| A recognized code, text, or nonzero weight is invalid | Preserve `InvalidInput` and attach the owning token's original line/column |
| A recognized numeric field is outside its accepted representation | `MalformedField` at the numeric token |
| A nonempty line matches no supported layout | Ordered `UnrecognizedLine` warning; parsing continues within the shared output budget |
| Effective weight, phrase candidate, output size, or checked count overflows | `IntegerOverflow`; do not invent a source location for an in-memory format failure |

### 5. Good / Base / Bad Cases

- Good: UTF-8, BOM-prefixed UTF-16LE/BE, and GBK versions of the same mixed-dialect input decode to equal documents; formatting and decoding all six escaped whitespace characters round-trip.
- Base: empty or preprocessing-only text decodes to an empty document, and an empty document formats to an empty string or BOM-only UTF-16LE bytes.
- Bad: an unknown line followed by a valid entry returns a document plus a visible warning, but an unknown-only body, an A line with empty text, or a third expansion past a two-entry limit returns a structured error without a partial document.

### 6. Tests Required

- Hand-author bytes for every supported encoding and malformed middle/trailing sequences; assert `DetectedTextEncoding`, `CodecErrorKind`, and exact byte offsets.
- Test A through F independently plus Microsoft/Jidian, preprocessing provenance, D endpoints/normalization, recognized invalid fields, warning order/preview/truncation, warning-plus-entry budgets, and truncated prefixes without panic.
- Assert the complete string for each `LexiconTextFormat`, including CRLF, effective weights, equal-weight stability, duplicate behavior, aggregate adjacency, phrase renumbering, and format-time overflow.
- Assert exact BOM-prefixed UTF-16LE bytes for empty, BMP, and non-BMP output, plus decode/encode symmetry for all six whitespace escapes and literal preservation for unknown percent sequences.
- Automated tests must remain synthetic or reproducibly fetched and must never read the root `resource/` directory.

### 7. Wrong vs Correct

```rust
// Wrong: hides detector failures and silently drops unsupported lines.
let text = String::from_utf8_lossy(bytes);
let document = parse_known_lines_only(&text);

// Correct: keep strict byte and text evidence plus visible compatibility diagnostics.
let decoded = wubilex_codec::text::decode(bytes, DecodeLimits::default())?;
for warning in decoded.warnings() {
    report_warning(warning.location(), warning.preview());
}
```

## Scenario: Phrase, Auxiliary Text, And Scheme Detection

### 1. Scope / Trigger

Apply this contract when community phrase text is decoded or formatted, when
word-frequency or split-table text is decoded or formatted, or when a
`LexiconDocument` is inspected to select one of the eight supported schemes.
All operations are synchronous and memory-to-memory; paths, resource loading,
domain indexes, and UI reporting remain outside `wubilex-codec`.

### 2. Signatures

```rust
pub fn phrase_text::decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<DecodedPhraseText, CodecError>;

pub fn phrase_text::format(
    document: &PhraseDocument,
) -> Result<String, CodecError>;

pub fn weight::decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<WordFrequencyDocument, CodecError>;

pub fn weight::format(
    document: &WordFrequencyDocument,
) -> Result<String, CodecError>;

pub fn split_table::decode(
    input: &[u8],
    limits: DecodeLimits,
) -> Result<SplitTableDocument, CodecError>;

pub fn split_table::format(
    document: &SplitTableDocument,
) -> Result<String, CodecError>;

pub fn detect::scheme(document: &LexiconDocument) -> LexScheme;
```

### 3. Contracts

- Phrase decode reuses the strict BOM-first UTF-8, UTF-16LE/BE, and GBK
  selector. It removes non-greedy cross-line `/* ... */` comments, then tries
  P1 through P6 in order with recognized invalid fields failing in place.
- P2/P3 empty text enters multiline state. Automatic candidates use the
  current per-code maximum plus one. A candidate-less `$[...]` expands to
  `1..=N` and updates that maximum to `max(old, N)`; every candidate remains
  in `1..=255`.
- Phrase warnings retain source order, at most 160 Unicode scalar values of
  preview, and a truncation flag. Warnings and expanded entries share one
  output ceiling, and a nonempty body with no entry is an error.
- Phrase format stably orders by code and candidate, uses CRLF, and compresses
  only groups with more than one entry, candidates exactly `1..=N`, and text
  no longer than two UTF-16 code units. It uses the shared six-value ASCII
  whitespace escape contract.
- Word-frequency and split-table inputs are BOM-less strict UTF-8. Each
  retained line has exactly two nonempty Unicode-whitespace-delimited tokens;
  frequency weights are ASCII decimal `1..=65535`. Both document types retain
  source order and duplicates and format with TAB plus LF.
- Scheme detection ignores weights and duplicate observations. It checks the
  five direct feature groups in documented order, then scores 86/98/06/091;
  only a strict 98, 06, or 091 winner overrides the 86 fallback. The 06
  feature code is lowercase `xfxy`.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Phrase input bytes are malformed | `InvalidTextEncoding` at the original zero-based byte offset |
| Phrase comment is unclosed | Structured text error at the opening delimiter |
| A recognized phrase field, array, or candidate is invalid | Field-specific error at the original one-based line and Unicode-scalar column |
| Phrase retained output exceeds its ceiling | `ResourceLimitExceeded` at the producing source line; no partial document |
| Auxiliary input has a BOM or malformed UTF-8 | `UnsupportedFormat` at byte zero or `InvalidTextEncoding` at the malformed byte |
| Auxiliary line has other than two tokens, or a weight is signed/out of range | Structured text error at the owning token |
| Model or formatter count/capacity cannot be represented or allocated | `IntegerOverflow` without an invented input location |
| Scheme features tie or do not produce a strict non-86 winner | `LexScheme::Wubi86` |

### 5. Good / Base / Bad Cases

- Good: all six phrase dialects, multiline text, arrays, time aliases, PUA
  roots, non-BMP text, duplicate auxiliary entries, and every scheme branch
  produce their documented typed values and canonical output.
- Base: empty phrase and auxiliary inputs produce empty documents; empty or
  tied lexicons detect as Wubi86.
- Bad: an unclosed phrase comment, a 256th candidate, a signed frequency, an
  auxiliary third column, or malformed encoded bytes returns a located error
  and never a partial document.

### 6. Tests Required

- Test P1 through P6 independently and together, including parser priority,
  multiline termination, comments, arrays, aliases, escapes, warning bounds,
  resource ceilings, candidate overflow, and complete canonical strings.
- Test auxiliary empty documents, ordered duplicates, exact weight endpoints,
  PUA/non-BMP preservation, BOM and UTF-8 failures, exact two-column failures,
  signed numbers, source positions, resource ceilings, and complete LF output.
- Test all five direct scheme branches and all four scored branches, direct
  priority, strict-winner ties, duplicate/weight independence, and lowercase
  `xfxy` as a failure-to-pass regression.
- Use synthetic bytes or reproducibly fetched fixtures only. Automated tests
  must never inspect the root `resource/` directory.

### 7. Wrong vs Correct

```rust
// Wrong: silently drops phrase lines and collapses duplicate auxiliary keys.
let phrases = parse_known_phrase_lines_only(text);
let frequencies: HashMap<_, _> = parse_frequency_lines(text).collect();

// Correct: preserve strict source evidence, warnings, order, and duplicates.
let decoded = wubilex_codec::phrase_text::decode(bytes, limits)?;
for warning in decoded.warnings() {
    report_warning(warning.location(), warning.preview());
}
let frequencies = wubilex_codec::weight::decode(frequency_bytes, limits)?;
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

The model, error, limit, raw `.lex`, raw EUDP, community lexicon text, phrase
text, auxiliary text, synthetic eight-scheme, reproducible real-fixture, and
measured-coverage tests are established examples. Independent aardio golden
comparison and CI installation/caching of the coverage tool remain later S0
obligations.
