# Research - Phrase And Auxiliary Text Contracts

## Sources Reviewed

- `docs/01-data-formats.md` sections 4 through 8
- `docs/modules/M1-lex-table.md` `M1-PARSE-013` and weight boundaries
- `docs/modules/M2-phrase.md` `M2-PARSE-001..008`
- `docs/modules/M3-reverse-lookup.md` `M3-SPLIT-001..007`
- `docs/02-architecture.md` codec/core/resource boundaries
- `docs/20-nonfunctional.md` compatibility, reliability, and maintainability requirements
- `.trellis/spec/backend/{directory-structure,error-handling,quality-guidelines}.md`
- archived `s0-codec-model`, `s0-eudp`, and `s0-lex-text` contracts
- `wubi-lex/lib/wubi/{phrase,lexFile,spellingTable}.aardio` as behavior evidence only

## Confirmed Phrase Text Contract

- Short phrase text uses six ordered line shapes P1 through P6. P1/P2/P4 carry an explicit one-based candidate; P3/P5/P6 use automatic assignment.
- P2/P3 with an empty right side enter multiline mode. Nonempty continuation lines join with `\n` until another record is recognized. The legacy loop trims and ignores empty lines, so empty physical continuation lines do not add blank logical lines.
- `$[...]` expands only without an explicit candidate. ASCII spaces select token mode with repeated spaces normalized; otherwise expansion is by Unicode character. Array candidates are always `1..=N`; the per-code maximum becomes `max(old, N)` for later ordinary auto-assignment. Candidate storage is the existing EUDP `u8`, so every expansion and auto-assignment must remain within `1..=255`.
- Automatic assignment is documented as the maximum used candidate for that code plus one. This is safer and more explicit than the old mutable last-value map when explicit candidates arrive out of order.
- P2 maps nine documented `$` aliases to Microsoft `%...%` placeholders. Microsoft evaluates the placeholders later; the codec performs substitution only and never reads a clock.
- Short phrase fields share the six uppercase ASCII whitespace escapes established by `s0-lex-text`.
- Canonical output sorts codes and candidates, compresses only multi-entry groups whose candidates are exactly `1..=N` and whose texts are at most two UTF-16 code units, and otherwise emits `code<TAB>text<TAB>candidate<CRLF>`.
- Emoji is two UTF-16 code units and is eligible for array compression. The threshold is not Unicode scalar count or UTF-8 byte length.

## Diagnostics Decision

The legacy parser silently ignores unknown lines. The project already chose visible warnings for compatible community text import: unknown nonempty lines remain non-fatal when another valid record exists, but callers receive bounded ordered diagnostics. Phrase text applies the same rule outside multiline state. Recognized malformed fields remain errors, and an unknown-only nonempty document is not an empty success.

Warnings retain one line of at most 160 Unicode scalar values and share the expanded-output budget with entries. This prevents compatibility diagnostics from becoming an unbounded second allocation channel.

## Word Frequency Contract

- Source format is no-BOM UTF-8, one `<word><whitespace><weight>` entry per line.
- word is a nonempty non-whitespace token; weight is decimal `1..=65535`, smaller values sort earlier.
- The default `0xF36B` and all reorder/fixed-single behavior are core transformations, not properties of the parsed file document.
- The old implementation loads into a map and duplicate keys become last-wins. Codec documents elsewhere preserve order and duplicates for round trips and diagnostics, so this layer retains every source entry. A future core map must choose duplicate resolution explicitly.
- The old writer uses TAB and LF; this becomes the canonical output.

## Spelling Table Contract

- Source format is no-BOM UTF-8, one `<term><whitespace><roots>` entry per line.
- Both fields are nonempty non-whitespace tokens. roots may contain PUA or non-BMP Unicode and must be preserved exactly.
- The old reader materializes a map, but parsing/formatting retains order and duplicates for the same reason as word frequency.
- Word/phrase root combination, lookup maps, scheme-specific fallbacks, resource download, caching, and fonts are outside codec.
- Canonical output uses TAB and LF; input tolerates Unicode whitespace and CRLF/LF.

## Scheme Detection Contract

- Detection is pure content inspection of `(code, text)` membership. Weights, source order, and duplicate entries do not change a feature hit.
- Five direct groups run in documented order: normal Zhengma, Xiaohe, Zhengma formation, 092, Biaoxingma. First complete group wins.
- Remaining 86/98/06/091 candidates use nine boolean tests and signed scores. Only strict wins select 98, 06, or 091; every tie or ambiguous result falls back to 86.
- The legacy source probes uppercase `XFXY` although model codes are lowercase. The implementation must probe `xfxy`; the regression fixture must make that one feature change the result so it cannot pass tautologically.
- The public `LexScheme::Zhengma { formation }` already models the formation variant without inventing a ninth display scheme.

## Architecture And Reliability Decisions

- All APIs remain synchronous byte/string-to-model conversions in `wubilex-codec`.
- Phrase text reuses the existing strict detector. Word frequency and spelling tables enforce their narrower no-BOM UTF-8 format instead of silently accepting system locale or replacement characters.
- Entry models validate their own nonempty/no-whitespace invariants so direct construction cannot make formatter output ambiguous.
- Input byte limits are checked before decode. Every retained phrase/aux entry and phrase warning is checked against `max_expanded_entries` before append.
- Errors preserve original byte or line/column evidence. No parser returns partial documents, silently truncates overflows, or accesses machine-local resources.

## Deferred Evidence

- Root `resource/` contains user-owned `.lex` samples and remains untouched; it is not evidence for phrase or auxiliary text formats.
- Reproducible real word-frequency, spelling-table, phrase-text, and eight-scheme fixtures belong to `s0-fixtures-regressions`.
- Default phrase content, core lookup/weighting/splitting, resource fetch/decompression, and system installation remain later layers/tasks.
