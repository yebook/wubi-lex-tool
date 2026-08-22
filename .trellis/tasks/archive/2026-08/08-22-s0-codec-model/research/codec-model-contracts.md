# Codec Model Contract Research

## Sources Read

- `docs/01-data-formats.md` sections 1-9
- `docs/02-architecture.md` sections 0, 1, 2, 5.1, D1, and 8
- `docs/20-nonfunctional.md` PERF, COMPAT, REL, and MAINT requirements
- `docs/modules/M1-lex-table.md` LIST/PARSE model and behavior requirements
- `docs/modules/M2-phrase.md` IO/PARSE behavior requirements
- `.trellis/spec/backend/{directory-structure,error-handling,quality-guidelines}.md`

## Architecture Contract Extract

1. `wubilex-codec` owns pure, synchronous, platform-neutral conversion between external bytes/text and typed format documents. It may use parsing/encoding crates but not Tauri, Windows, network, or Tokio.
2. `wubilex-core` owns the mutable domain model, reverse indexes, transformations, sorting, statistics, weighting, and word generation. Codec models must not absorb those responsibilities.
3. C1/C2/C5/C6/C9/C10/C12 are compatibility contracts. Data structures may improve, but binary layout and documented text behavior cannot drift.
4. The model must preserve order and duplicates because candidate order and byte/text round trips depend on them. A map or set is not the authoritative codec representation.
5. D1 expects a full domain lexicon to use about 50-150 MB and `NFR-PERF-010` caps one open lexicon at 300 MB. The documented source size is several MB with hundreds of thousands of records.
6. Library failures are typed errors. Malformed data preserves line number or byte offset; Tauri `AppError` and Chinese presentation belong to the app boundary.
7. Parsing must validate offsets, lengths, UTF-16 units, integer conversions, and resource limits before slicing or allocation. Production code cannot use `unwrap()` or `expect()`.
8. The approved error dependency baseline is `thiserror 2.0.20`; application-only `anyhow` is not needed here.

## Model Consequences

- Use ordered entry documents in codec and defer indexed structures to core.
- Keep lex weights optional so text sources without explicit weights retain their semantics until normalization/serialization.
- Require a concrete nonzero EUDP candidate after text parsing has performed automatic assignment.
- Derive UTF-16 unit counts from text to avoid stale redundant state, especially for emoji surrogate pairs.
- Represent Zhengma formation in the Zhengma scheme branch so impossible cross-scheme combinations are unrepresentable.
- Represent encoding and BOM separately; detection implementation remains deferred.

## Limit Decision

Use configurable defaults of 64 MiB input and 500,000 expanded entries. Both must be checked: input size alone does not protect `$[...]` expansion, while entry count alone does not protect one hostile large field. Format-specific numeric widths and checked arithmetic remain mandatory in each later parser/encoder.

This limit is intentionally revisitable after reproducible real fixtures exist. Any adjustment requires measured evidence and must retain configurable limits rather than removing the guard.

## User-Provided Sample Evidence

The root `resource/` directory is user-owned and remains untracked. A read-only header probe on `微软五笔86( 完整 ).lex` found:

- actual size: 4,467,680 bytes
- magic: `imscwubi`
- version: 1.1
- index offset: 64
- table offset: 168
- declared file size: 4,467,680 bytes, equal to the actual size

The accompanying `.lex.lzma` is 1,539,772 bytes. This evidence supports ample headroom under the 64 MiB default, but the sample is not a task fixture and must not be moved, edited, staged, or coupled to model tests.

## Historical Search

`trellis mem search` for `s0-codec-model`, `codec 公共模型`, and `输入限制` found only the current session. No earlier unrecorded model or limit decision was found. The OpenCode adapter reported its documented temporary SQLite limitation; the relevant project history in Codex was available.
