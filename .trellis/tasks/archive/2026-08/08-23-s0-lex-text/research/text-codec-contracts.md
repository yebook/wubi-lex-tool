# Research: Lexicon text codec implementation contracts

- Query: Fix the implementation contracts for deterministic text encoding detection, strict byte-offset errors, visible unknown-line warnings, seven canonical text outputs, effective weights, stable ordering, duplicate behavior, UTF-16LE bytes, and symmetric ASCII whitespace escaping.
- Scope: mixed (repository contracts, legacy aardio behavioral evidence, and exact upstream crate APIs)
- Date: 2026-08-23

## Findings

### Recommended public boundary

Keep all behavior synchronous and memory-to-memory under `wubilex_codec::text`; the existing crate boundary explicitly owns byte/text conversion and forbids filesystem, Tauri, Windows, network, and Tokio work (`.trellis/spec/backend/directory-structure.md`; `crates/wubilex-codec/src/lib.rs:1-13`). A concrete API shape that satisfies the approved PRD is:

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
```

`DecodedLexiconText` should own exactly:

- `LexiconDocument`, preserving decoded entry order and duplicates;
- `DetectedTextEncoding`, reusing the established four-encoding plus BOM model (`crates/wubilex-codec/src/model/text_encoding.rs:5-40`);
- an ordered `Vec<TextDecodeWarning>`.

Expose read-only accessors plus consuming accessors for the document and warnings, following the established document API (`crates/wubilex-codec/src/model/lexicon.rs:94-127`). A warning should be structurally separate from `CodecError`, for example:

```rust
pub struct TextDecodeWarning {
    kind: TextDecodeWarningKind,
    location: SourceLocation,
    actual: String,
    truncated: bool,
}

pub enum TextDecodeWarningKind {
    UnrecognizedLine,
}
```

The warning location is always `SourceLocation::Text` with the original one-based physical line and the first non-ASCII-whitespace column. `actual` should retain at most a named constant such as `MAX_WARNING_ACTUAL_CHARS = 160` Unicode scalar values, never truncate inside UTF-8, and set `truncated` when content was omitted. This makes the warning visible without retaining an unbounded second copy of each line. Warnings remain in source order.

### Encoding detection and strict decoding

Pin direct codec dependencies exactly as `chardetng = "=1.0.0"` and `encoding_rs = "=0.8.35"`. `chardetng` 1.0.0 is Apache-2.0 OR MIT and depends on `encoding_rs >=0.8.29`; `encoding_rs` 0.8.35 is `(Apache-2.0 OR MIT) AND BSD-3-Clause`. Both satisfy the pure-Rust codec boundary. The repository already locks `regex 1.13.1` transitively (`Cargo.lock:2027-2036`), but if it becomes a direct codec dependency it should also be exact-pinned per LT-R01.

Apply this deterministic sequence:

1. Call `limits.check_input_bytes(input.len())` before detection, output allocation, or decoding (`crates/wubilex-codec/src/limits.rs:41-51`).
2. Match exact BOMs before statistical detection: `EF BB BF` -> UTF-8/body offset 3; `FF FE` -> UTF-16LE/body offset 2; `FE FF` -> UTF-16BE/body offset 2. Remove exactly one selected BOM before strict decoding and report `has_bom = true`.
3. With no BOM, empty and ASCII-only inputs are UTF-8. Otherwise feed the complete bytes once to `chardetng::EncodingDetector::feed(input, true)` and call `guess(Some(b"cn"), Utf8Detection::Allow)`. The `cn` TLD hint is deterministic and biases the detector to Simplified Chinese without consulting system locale.
4. The product-supported no-BOM universe is UTF-8 or GBK. Map a `UTF_8` guess to `TextEncoding::Utf8`; map every non-UTF-8 legacy guess to the supported GBK fallback, then require strict GBK decoding. This is deliberate narrowing: `chardetng` can identify more legacy encodings than `TextEncoding` can represent.
5. Decode with `encoding_rs::{UTF_8, UTF_16LE, UTF_16BE, GBK}` and `new_decoder_without_bom_handling()`, because BOM sniffing and removal were already performed by this codec. Do not use replacement or a lossy API.

`chardetng` 1.0.0 uses `Utf8Detection::Allow`, not the older Boolean `allow_utf8` API. Its `feed(..., true)` closes the detector and must be called only once for this one-buffer boundary. The source guarantees that a surviving UTF-8 candidate is returned when UTF-8 is allowed.

For an exact malformed-byte offset, use `Decoder::decode_to_string_without_replacement`, not the convenient `Encoding::decode_without_bom_handling_and_without_replacement` helper, because the latter returns only `Option<Cow<str>>`. Reserve the capacity returned by `max_utf8_buffer_length_without_replacement(body.len())`, invoke one final call with `last = true`, and inspect `DecoderResult`:

- `InputEmpty`: successful strict decode;
- `OutputFull`: reserve/growth contract bug or checked-size failure, not malformed input;
- `Malformed(malformed_len, after_len)`: with one input call, local malformed start is `read - malformed_len - after_len`; original byte offset is `bom_len + local_start`.

Attach `CodecErrorKind::InvalidTextEncoding { encoding }` at that zero-based original offset using `at_byte_offset` (`crates/wubilex-codec/src/error.rs:151-155,220-227`). Checked capacity arithmetic maps to `IntegerOverflow`, without a fabricated source location. Tests must include an invalid sequence in the middle, a truncated multibyte sequence at EOF, an odd UTF-16 trailing byte, and an unpaired UTF-16 surrogate in each endian form.

Do not heuristically accept BOM-less UTF-16; LT-R02 and LT-R03 only promise BOM-less UTF-8/GBK. Exact BOM bytes also imply that a partial prefix such as `EF BB` is not a BOM.

### Original line mapping and preprocessing

Do not implement preprocessing as destructive whole-string replacement if errors and warnings must cite original physical lines. Split into records that retain `{ one_based_line, original_content }`, normalize only the line terminator, and perform the documented stages over those records:

1. Remove a leading Rime YAML front-matter block from `---` through `...`.
2. Remove physical lines beginning with `#`; blank and ASCII-whitespace-only lines are legal and silent.
3. Locate the first `[Text]` header. If present, exclude all preceding description records from the body; use that description only for the Jidian marker test.
4. Set Jidian only when the description contains both `~:生僻字词` and `^:用户词组`.
5. Only after `[Text]`, inspect the first nonblank body line for the Microsoft “text first, one or more codes” signature. If selected, scan the entire body with the Microsoft parser and return after its cleanup/validation path.

These stages are documented at `docs/01-data-formats.md:243-263` and implemented in legacy form at `wubi-lex/lib/wubi/lexFile.aardio:40-79`. Comments and description lines do not generate warnings. Unknown nonempty body lines do.

### Dialect matching and visible-warning boundary

Implement each dialect attempt as a tri-state parser:

```text
NoMatch             -> try the next dialect
Matched(entries)    -> append after checking the expansion limit
Invalid(error)      -> return immediately; never fall through to a looser dialect
```

This is required to distinguish compatibility warnings from corruption. A regex that already constrains a code to `[a-z]{1,4}` is not sufficient as the recognition layer: `abcde word`, `Abcd word`, `abcd word 0`, and `abcd word 65536` resemble known layouts and must reach model/weight validation rather than become warnings. Recognize delimiter/layout shape broadly, then validate captured fields through `LexCode`, `Weight`, decimal conversion, and escape handling. Attach the exact original line and the first column of the bad field. Only a nonempty line for which every A..F recognizer returns `NoMatch` produces `UnrecognizedLine`.

The attempt order and semantic actions are fixed by `docs/01-data-formats.md:265-290` and legacy `wubi-lex/lib/wubi/lexFile.aardio:85-177`:

| Priority | Layout | Result |
|---|---|---|
| A | code, text, ascending decimal weight | one entry with explicit nonzero `u16` weight |
| B | `code=weight,text` | one entry with explicit nonzero `u16` weight |
| C | code then one or more whitespace-separated texts | one unweighted entry per decoded text |
| D | text, code, descending decimal weight, optional trailing letters | one entry with transformed explicit weight |
| E | text then one code | one unweighted entry |
| F | text then one or more codes | one unweighted entry per code |

Apply whitespace escape decoding to every captured text, including Microsoft and F records; legacy omitted decoding in those branches, but the approved symmetric round-trip contract requires it. Before appending `n` expanded entries, checked-add `n` to the current entry count, call `check_expanded_entries(actual)`, and attach the current line to a limit error. Never append some entries from a line before discovering that the same line crosses the limit.

Unknown-line warnings do not stop valid later lines. After parsing and Jidian cleanup:

- an actually empty body (including a body containing only YAML, comments, description, and blank lines) may return an empty document;
- a nonempty body with at least one surviving valid entry returns the document plus its ordered warnings;
- a nonempty body with zero surviving valid entries returns a structured error at the first contributing body line, not an empty successful document and not a partial warning-only result.

The approved visible-warning requirement is now recorded in `.trellis/tasks/08-23-s0-lex-text/prd.md:12,25,54,65`.

### Descending weights, Jidian cleanup, and per-entry semantics

A/B source weights must parse into an integer wide enough to distinguish zero from overflow, then validate `1..=65535`. D source weight transforms as:

```text
effective = 65535 - source
```

Therefore D accepts source `0..=65534`; source `65535` yields forbidden internal zero, and larger values are invalid. Track which entries came from D. After the full parse, let `minimum` be the minimum transformed D weight; if `minimum > 5000`, subtract `minimum - 5000` from D entries only. This preserves relative order and leaves A/B weights unchanged. The documentation says “all D weights” (`docs/01-data-formats.md:292-302`); legacy's map-wide adjustment at `wubi-lex/lib/wubi/lexFile.aardio:179-186` would also touch A/B weights in a mixed input and should not be copied.

For Jidian, remove `^`, `$`, and `!` entries; strip one leading `~`; reject/remove any result that becomes empty consistently with the nonempty model invariant; clear every surviving entry's weight; and remove empty code groups. The legacy evidence is `wubi-lex/lib/wubi/lexFile.aardio:189-212`, while the approved requirement is LT-R09.

The new `LexiconEntry` stores weight per entry (`crates/wubilex-codec/src/model/lexicon.rs:55-90`), unlike the legacy text-keyed weight map. Do not recreate the old collision where duplicate texts shared one weight.

### Effective weights and stable canonical ordering

Formatting must not mutate the input document. Build an indexed projection containing `(original_index, code, text, explicit_weight, effective_weight)`. For each code, walk entries in original order with `previous = 0`:

- explicit weight -> use it and assign it to `previous`;
- missing weight -> `previous.checked_add(1)`, use the result, and assign it to `previous`.

An omitted weight after 65535 is a format-time `IntegerOverflow` with no text location. Then stable-sort by code lexicographically and, within one code, by effective weight ascending. Equal code/effective-weight entries retain original order. This fixes `M1-PARSE-018` (`docs/modules/M1-lex-table.md:88`) without modifying `LexiconDocument`; legacy filling/sorting evidence is `wubi-lex/lib/wubi/lexFile.aardio:491-512`.

All seven formats consume the same sorted projection, even formats that do not print a weight. Non-aggregate formats preserve every duplicate. Aggregate formats perform only the documented adjacent folding after sorting; they do not globally deduplicate the document.

### Seven output formats

Use a public enum with seven explicit variants rather than four interacting Booleans. Exact layouts, separators, and duplicate behavior should be:

| Variant | Per-line bytes before CRLF | Grouping / duplicate contract |
|---|---|---|
| CodeThenText | `code<TAB>escaped_text` | one line per entry; preserve all duplicates |
| CodeThenTexts | `code<TAB>text1<TAB>text2...` | one line per code; fold adjacent equal texts only |
| CodeThenTextWeight | `code<TAB>escaped_text<TAB>effective_weight` | one line per entry; preserve all duplicates |
| TextThenCode | `escaped_text<TAB>code` | one line per entry; preserve all duplicates |
| TextThenCodes | `escaped_text<TAB>code1<SPACE>code2...` | text groups ordered by first encounter in the canonical projection; code order is canonical; fold adjacent equal codes only |
| TextThenCodeDescendingWeight | `escaped_text<TAB>code<TAB>(65535 - effective_weight)` | one line per entry; preserve all duplicates; zero output is valid when effective weight is 65535 |
| PhraseAscendingCandidate | `code=one_based_index,escaped_text` | per code, already stable-sorted by effective weight, then renumber from 1; preserve all entries |

Append `\r\n` after every emitted line, including the last. An empty document formats to an empty `String`. Reject a phrase candidate index that cannot be represented by the parser's accepted numeric range rather than emitting text that cannot round-trip. The authoritative matrix is `docs/01-data-formats.md:320-344`; the detailed legacy branches are `wubi-lex/lib/wubi/lexFile.aardio:889-1051`; requirements are `docs/modules/M1-lex-table.md:137-143`.

For deterministic UTF-16LE bytes, first call the String formatter, prefix the byte vector with `FF FE`, then append each `str::encode_utf16()` unit in little-endian order. `encoding_rs` intentionally has no UTF-16 encoder (`Encoding::new_encoder()` maps UTF-16 output encoding to UTF-8), so manual `u16::to_le_bytes` is the correct boundary. A formatted empty document becomes BOM-only bytes. Including the BOM is necessary because this task deliberately does not detect BOM-less UTF-16.

### ASCII whitespace escape symmetry

The shared escape helper must use an explicit six-character ASCII table:

| Character | Escape |
|---|---|
| U+0020 space | `%20` |
| U+0009 tab | `%09` |
| U+000A LF | `%0A` |
| U+000D CR | `%0D` |
| U+000B VT | `%0B` |
| U+000C FF | `%0C` |

Encoding scans Unicode scalar values and replaces exactly those six, using uppercase hex. Decoding scans left-to-right and replaces exactly those six uppercase three-byte sequences. Lowercase forms and every other `%xx` remain literal; do not add `%25`, URL decoding, or general hex decoding. This is the narrow interpretation of LT-R10 and preserves the documented literal-`%20` ambiguity instead of introducing an incompatible protocol.

Use explicit ASCII whitespace sets in dialect delimiters rather than Rust regex's default Unicode `\s`, otherwise a Unicode space embedded in a word can be split unexpectedly. All output text fields pass through the same escape helper. The legacy encoder matched more whitespace than its decoder (`wubi-lex/lib/wubi/text.aardio:8-13`); the defect and required VT/FF repair are documented at `docs/01-data-formats.md:640-672`.

### Test obligations

Hand-author synthetic bytes independently of the production UTF-16 formatter for at least:

- UTF-8 with/without BOM, UTF-16LE/BE with BOM, GBK, ASCII ambiguity -> UTF-8;
- malformed UTF-8, malformed GBK, odd UTF-16 byte length, unpaired UTF-16 surrogate, with exact original byte offsets including BOM;
- original physical line locations after YAML/comments/description removal;
- one isolated test for A through F and Microsoft, plus mixed-priority lines;
- unknown-before-valid, valid-before-unknown, multiple unknowns in source order, recognized malformed fields, warning excerpt truncation, and nonempty zero-valid-entry failure;
- D endpoints 0/65534/65535/overflow, mixed A/B/D normalization, and minimum at/below/above 5000;
- Jidian removal, tilde stripping, weight clearing, empty group, and all-removed body;
- every output as an exact full string with CRLF and final CRLF;
- missing-after-65535 effective-weight overflow, equal-weight stability, duplicates in nonaggregate forms, adjacent folding in both aggregate forms, and phrase renumbering;
- round trips containing all six ASCII whitespace characters, lowercase/unknown `%xx` literals, and the known literal `%20` ambiguity;
- UTF-16LE output BOM and exact little-endian bytes, including BOM-only empty output;
- resource-limit checks before input decode and before whole-line expansion.

Tests should inspect `CodecErrorKind` and `SourceLocation`, never `Display`, following `.trellis/spec/backend/error-handling.md:37-55` and existing tests in `crates/wubilex-codec/tests/error_and_limits.rs`.

### Files found

- `.trellis/tasks/08-23-s0-lex-text/prd.md` - approved task scope, visible-warning decision, acceptance criteria, and deferred fixture boundary.
- `.trellis/workflow.md` - Trellis planning/research persistence and later implementation/check flow.
- `.trellis/spec/backend/index.md` - backend spec index and evidence status.
- `.trellis/spec/backend/directory-structure.md` - codec ownership, synchronous boundary, and target `text`/`escape` module locations.
- `.trellis/spec/backend/error-handling.md` - structured error/location and no-partial-success contract.
- `.trellis/spec/backend/quality-guidelines.md` - strict parser gates and test obligations.
- `docs/01-data-formats.md` - authoritative text dialect, output matrix, encoding, and escape requirements.
- `docs/modules/M1-lex-table.md` - `M1-PARSE-002..008`, `M1-PARSE-018`, and `M1-XFORM-001..007` requirements.
- `docs/02-architecture.md` - selected Rust replacements: chardetng, encoding_rs, regex, and static precompilation.
- `docs/20-nonfunctional.md` - 300 MB open-document memory target and exact parse-error location requirement.
- `crates/wubilex-codec/src/model/text_encoding.rs` - established supported encoding/BOM value types.
- `crates/wubilex-codec/src/model/lexicon.rs` - validated code/text/weight types and ordered duplicate-preserving document.
- `crates/wubilex-codec/src/error.rs` - reusable error kinds and byte/text locations.
- `crates/wubilex-codec/src/limits.rs` - 64 MiB input and 500,000 expanded-entry ceilings.
- `crates/wubilex-codec/tests/model_contracts.rs` - public model and encoding metadata contract tests.
- `crates/wubilex-codec/tests/error_and_limits.rs` - error/location/limit matching pattern.
- `wubi-lex/lib/wubi/lexFile.aardio` - legacy preprocessing, dialect control flow, weight fill/sort, outputs, and UTF-16LE file evidence.
- `wubi-lex/lib/wubi/text.aardio` - legacy whitespace escape asymmetry evidence.

### External references

- `chardetng 1.0.0` `EncodingDetector`: https://docs.rs/chardetng/1.0.0/chardetng/struct.EncodingDetector.html
- `chardetng 1.0.0` exact source/API: https://github.com/hsivonen/chardetng/blob/v1.0.0/src/lib.rs
- `chardetng 1.0.0` manifest/license: https://github.com/hsivonen/chardetng/blob/v1.0.0/Cargo.toml
- `encoding_rs 0.8.35` `Decoder`: https://docs.rs/encoding_rs/0.8.35/encoding_rs/struct.Decoder.html
- `encoding_rs 0.8.35` `DecoderResult`: https://docs.rs/encoding_rs/0.8.35/encoding_rs/enum.DecoderResult.html
- `encoding_rs 0.8.35` exact source/API: https://github.com/hsivonen/encoding_rs/blob/v0.8.35/src/lib.rs
- `encoding_rs 0.8.35` manifest/license: https://github.com/hsivonen/encoding_rs/blob/v0.8.35/Cargo.toml

### Related specs

- LT-R01..LT-R15 and all acceptance criteria in `.trellis/tasks/08-23-s0-lex-text/prd.md`.
- `M1-PARSE-002..008`, `M1-PARSE-018`, and `M1-XFORM-001..007` in `docs/modules/M1-lex-table.md`.
- Text dialect sections 3.1..3.6 and whitespace section 8 in `docs/01-data-formats.md`.
- `NFR-PERF-010` and `NFR-REL-008` in `docs/20-nonfunctional.md:38,139`.
- Backend directory, error, and quality contracts under `.trellis/spec/backend/`.

## Caveats / Not Found

- **Warning-count resource ceiling is not specified.** LT-R15 requires one warning per unknown nonempty line, but a 64 MiB input can contain millions of tiny unknown lines and exceed the 300 MB memory target even with 160-character excerpts. Before implementation, the design should either add a diagnostics limit/`ResourceKind` (erroring instead of silently dropping warnings) or explicitly charge warnings to an existing aggregate-output budget. Capping the vector while returning success would violate the visible-warning requirement.
- **Escape-error wording conflicts with LT-R10.** LT-R15 mentions invalid escapes as errors, while LT-R10 says only the six uppercase escapes are decoded and other `%xx` must not be guessed. This research interprets unknown/lowercase `%xx` as literal for compatibility. Rejecting them would also reject plausible literal percent content. The design should record this interpretation explicitly.
- **BOM on UTF-16LE output is inferred, not stated verbatim in legacy docs.** The legacy opens `ccs=UTF-16LE`, while the new decoder only promises BOM-bearing UTF-16. Emitting `FF FE` is the only deterministic round-trip contract; fixture comparison remains deferred.
- **`chardetng` recognizes encodings outside the public model.** Mapping every non-UTF-8 no-BOM guess to strict GBK is a supported-universe policy, not a claim that the detector identified GBK. Add synthetic GBK and unsupported-legacy tests and document this narrowing.
- **Legacy B matching was unanchored and legacy D normalization used a shared weight map.** Full-line anchored recognition and D-only normalization are safer interpretations of the approved requirements but may differ on malformed mixed legacy files.
- No reproducible real text fixture or seven-format golden output was found. Root `resource/` contains only user-owned binary `.lex`/`.lex.lzma` and must remain untouched; real compatibility validation belongs to `s0-fixtures-regressions`.
