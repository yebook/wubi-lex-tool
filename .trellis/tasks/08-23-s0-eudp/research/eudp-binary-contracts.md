# Research - EUDP Binary Contracts

## Sources Reviewed

- `docs/01-data-formats.md` section 2 and its suggested EUDP boundary tests
- `docs/modules/M2-phrase.md` requirements `M2-IO-002/009` and `M2-PARSE-009/010/011`
- `docs/20-nonfunctional.md` requirements `NFR-COMPAT-008/011` and `NFR-REL-008`
- `docs/02-architecture.md` contracts C2, codec boundary, and UTF-16 guidance
- `.trellis/spec/backend/{directory-structure,error-handling,quality-guidelines}.md`
- archived `s0-codec-model` and `s0-lex-binary` planning/implementation evidence
- `wubi-lex/lib/wubi/phrase.aardio:11-104,186-391` as behavior evidence only

## Confirmed Wire Contract

- Raw little-endian file with ASCII magic `mschxudp` and a 64-byte header.
- The offset table contains signed 32-bit offsets relative to `phraseStart`; appending the data-section length as a sentinel gives each record range.
- Canonical header values are `magic2 = 0x00600002`, version 1, offset-table start 64, `phraseStart = 64 + count * 4`, `phraseEnd = file size`, caller-supplied Unix timestamp, and zero reserved bytes.
- Each record has a 16-byte fixed header followed by NUL-terminated UTF-16LE code and text. Canonical record constants are `cbSize/cbSize2 = 16`, unknown bytes 6/0, deleted 0, and zero reserved fields.
- Candidate is one-based `u8`. Active code/text are nonempty NUL-terminated strings without embedded `U+0000`. Code is lowercase ASCII; text may contain non-BMP characters, embedded newlines, and Microsoft `%...%` variables.
- Writer ordering is by code. The legacy writer comparator only examines code; candidate remains an explicit field, so equal-code physical order is preserved rather than independently sorted.
- Deleted records are tombstones: structurally valid deleted records do not produce PhraseEntry values.

## Compatibility Decisions

- `ChsWubiEUDPv1.lex` and `ChsWubiEUDPv2.lex` are path-level compatibility names. The legacy writer creates one byte stream and copies it unchanged; no second wire layout is evidenced.
- Decode is strict about magic and every field needed to derive safe boundaries. Non-structural metadata (`magic2`, version, timestamp, `cbSize2`, unknown, reserved) is tolerated because the legacy reader does not reject it and it does not change the documented layout. Canonical encode normalizes it.
- `cbSize` is the explicit record-layout discriminator in `M2-PARSE-010`; values other than 16 return `UnsupportedFormat` rather than being guessed.
- A coherent `count == 0` file is accepted as the empty format-neutral PhraseDocument, matching the frozen model contract. A missing or truncated 64-byte header is still an error.
- Decode validates code order but not candidate monotonicity within equal codes. This preserves old files created from explicitly numbered lines in arbitrary order while retaining the candidate value that determines display position.

## Reliability Decisions

- The legacy parser skips an entry when the derived text range is negative. That would be a silent partial success in the current API and conflicts with `NFR-REL-008` plus the backend error contract. The new raw decoder rejects the malformed offset at its byte location.
- Deleted records are validated before being skipped. Otherwise malformed UTF-16 or offsets could evade validation merely by setting one byte.
- The declared wire count is checked against `DecodeLimits::max_expanded_entries` before allocating the offset vector. Counting tombstones prevents a malicious file from bypassing the allocation/iteration ceiling.
- Offset table entries are signed on the wire and remain signed until range validation. First offset 0, strict monotonicity, checked record ranges, exact `phraseEnd`, and NUL positions prevent underflow, overlap, and cross-record reads.
- Strict UTF-16 scanning identifies the first unpaired surrogate and its source byte. Lossy replacement is prohibited.

## Timestamp Decision

The timestamp is required in canonical output but is not phrase-domain data and is ignored by the reader. A pure codec must not call the wall clock. The planned API therefore accepts an explicit `i32` timestamp:

```rust
encode(&document, timestamp)
```

This keeps output deterministic, allows tests to reproduce exact bytes, and leaves “current Unix time” ownership with the future installation/application layer. Decode returns `PhraseDocument`, so byte-exact tests reuse the known fixture timestamp; arbitrary accepted metadata only promises semantic normalization.

## Boundaries And Deferred Evidence

- Raw EUDP conversion belongs to `wubilex-codec::eudp`.
- File discovery, v1/v2 modification-time selection, dual write, backup/rollback, Windows gating, registry and TSF belong outside codec.
- P1-P6 text dialects, `$[...]`, whitespace escapes, multiline parsing and time aliases are deferred to `s0-phrase-aux`; raw newline/emoji/variable strings are exercised here.
- The user-owned `resource/` directory contains only `.lex` and `.lex.lzma`; it is not modified, committed, or referenced by tests.
- No real EUDP fixture is currently available. Hand-authored bytes establish the wire contract now; reproducible Windows samples and system-level compatibility remain explicit follow-up work.
