# Research - `.lex` Binary Contracts

## Sources Reviewed

- `docs/01-data-formats.md` section 1 and section 9
- `docs/modules/M1-lex-table.md` requirements `M1-PARSE-001`, `M1-PARSE-012`, and `M1-PARSE-018`
- `docs/20-nonfunctional.md` requirement `NFR-REL-008`
- `docs/02-architecture.md` codec/resource boundaries and D14
- `.trellis/spec/backend/{directory-structure,error-handling,quality-guidelines}.md`
- archived `s0-codec-model` planning and compiled public contracts
- `wubi-lex/lib/wubi/lexFile.aardio:216-286,337-417` as behavior evidence only

## Confirmed Wire Contract

- Little-endian raw file with ASCII magic `imscwubi`.
- Fixed 64-byte header followed by 26 signed 32-bit relative alpha offsets and a variable record stream.
- Canonical writer emits version 1.1, offsets 64/168, marker `0x78563412`, and zero reserved bytes.
- Each record is `u16 length`, `u16 weight`, `u16 codeLength`, four UTF-16LE code slots, UTF-16LE text, and a zero `u16` terminator.
- Records are sorted by code. Duplicate records are legal. Missing weights are assigned from a per-code running value beginning at zero.
- The parser must report byte offsets for malformed input and apply the existing 64 MiB / 500,000-entry defaults.

## Local Sample Audit

The user-owned files remain outside version control:

| File | Bytes | SHA-256 |
|---|---:|---|
| `resource/微软五笔86( 完整 ).lex` | 4,467,680 | `969BAE11DAA3C3D9A66D50C26A3EC5F47AAB629EB5076D8D7BC0A4777C3898DB` |
| `resource/微软五笔86( 完整 ).lex.lzma` | 1,539,772 | `D56DFF6234D265A1B865C14207B5DB046156AE981C991975BD6CD270F06F5F1E` |

The raw file was inspected read-only with an independent PowerShell record walk:

- header version 1.1, index offset 64, table offset 168;
- declared and actual size both 4,467,680;
- 207,055 records across 193,261 distinct codes;
- 54 consecutive duplicate code/text records;
- code order and all 26 computed alpha indexes match;
- zero invalid weights, codes, padding slots, terminators, or UTF-16 sequences;
- maximum text length is 10 UTF-16 code units.

This proves the sample is suitable for a manual canonical byte-round-trip check. It is not a reproducible repository fixture and must not be referenced by automated tests.

## Boundary Decisions

- Raw `.lex` bytes belong to `wubilex-codec::lex`.
- LZMA/zstd and filesystem concerns belong to `wubilex-resource`; the compressed sample is deferred.
- Decode preserves wire records in `LexiconDocument`; indexes and transformations remain in core.
- Magic and structural safety fields are strict. Version, marker, and reserved metadata are tolerated on read because they do not affect the documented layout, then normalized on write.
- The alpha index is semantically validated against parsed record boundaries rather than trusted or ignored.
- Standard-library slice parsing and UTF-16 conversion are sufficient; no parser or encoding dependency is justified.
