# Design - S0 真实夹具与回归

## 1. Boundary And File Layout

The task adds repository automation and tests, not product download behavior:

```text
xtask/
  Cargo.toml
  src/main.rs                 # command dispatch and exit behavior
  src/fixtures.rs             # manifest, HTTPS fetch, hash, LZMA, cache checks
crates/wubilex-codec/
  Cargo.toml                  # dev-only property/manifest dependencies
  tests/fixtures/
    .gitignore                # keep metadata, ignore downloaded binaries/parts
    README.md                 # source, preparation and license caveat
    manifest.json             # single source of fixture truth
  tests/real_fixtures.rs      # eight real decode/detect/re-encode checks
  tests/properties.rs         # generated valid models and bounded invalid bytes
  tests/cross_codec.rs        # text/binary/EUDP projections and S0 regressions
```

`wubilex-codec/src/` changes only if coverage or real fixtures expose a verified defect. HTTP, LZMA and filesystem code stays in `xtask`; no production dependency is added to codec.

## 2. Manifest Contract

`manifest.json` has a schema version and exactly eight entries. Each entry owns:

```json
{
  "id": "wubi86",
  "scheme": "wubi86",
  "name": "Microsoft Wubi 86 minimal",
  "url": "https://wubi.aardio.com/download/lex/ChsWubi86.min.lex.lzma",
  "archive_file": "wubi86.lex.lzma",
  "lex_file": "wubi86.lex",
  "archive_size": 46580,
  "archive_sha256": "64 lowercase hex characters",
  "lex_size": 0,
  "lex_sha256": "64 lowercase hex characters",
  "source": "legacy WubiLex catalog",
  "license_note": "per-dictionary license not published; test download only"
}
```

The parser rejects unknown schema versions, duplicate ids/schemes/paths, non-HTTPS URLs, paths with directories, invalid hash syntax, zero or over-limit sizes, and any scheme set other than the required eight. Both xtask and real tests read this file; no second hard-coded fixture table is allowed.

## 3. Fixture Command And State Machine

`cargo xtask fixtures` locates the repository from `CARGO_MANIFEST_DIR`, so behavior is independent of the caller's current directory. Optional `--check` selects offline validation.

For each manifest entry:

```text
validate manifest
  -> valid archive + valid lex cache: reuse
  -> --check and either file invalid/missing: fail with command hint
  -> default mode:
       HTTPS GET to create_new temporary archive
       -> final URI remains HTTPS
       -> enforce global and manifest byte ceilings while streaming SHA-256
       -> compare archive size/hash
       -> bounded LZMA-alone decode into a temporary lex file
       -> compare lex size/hash and strict `imscwubi` decode
       -> remove invalid old targets only after both temporary files pass
       -> rename temporary files into place
```

Temporary names include the process id and use `create_new`; cleanup guards remove them on every failure. The global archive ceiling is 16 MiB and the decoded ceiling is the codec 64 MiB input limit. Redirect count is bounded by the HTTP client and the final scheme is rechecked. Error messages name the fixture and stage and include expected/actual size or digest without exposing proxy credentials.

Initial manifest bootstrapping is a two-pass review: use a one-time external fetch into an isolated temporary directory to acquire the eight selected bytes and record sizes and both hashes, then delete that directory and fetch from an empty fixture cache using the pinned manifest. The committed xtask never has an unverified/bootstrap mode; only the second pass establishes reproducibility.

## 4. Real Fixture Tests

`real_fixtures.rs` first parses and validates the manifest, then requires every decoded `.lex` file. Missing data fails with `run cargo xtask fixtures`; it is never ignored or silently skipped. Each fixture is read once and asserts:

1. exact size and SHA-256 against the manifest;
2. strict `lex::decode` succeeds under default limits;
3. document is nonempty and reports stable entry/code statistics;
4. `detect::scheme` equals the manifest scheme, including Zhengma formation shape;
5. every decoded model entry remains valid and ordered;
6. `lex::encode(document)` equals the complete original bytes.

The result table (size, entry count, distinct code count and digest) is recorded in task research after verification. Tests do not parse `.lzma`; archive integrity belongs to xtask.

## 5. Property And Corruption Tests

Proptest strategies generate bounded validated `LexCode`, `PhraseCode`, text, nonzero weights/candidates and ordered duplicate-preserving documents. Case sizes stay small enough for Clippy/tests/coverage to remain practical.

- `.lex`: explicit-weight documents are canonicalized by stable code order; encode/decode must preserve that projection and re-encoding must be byte-identical.
- EUDP: fixed timestamp, stable code projection, duplicate/candidate/non-BMP preservation and byte-identical second encoding.
- Whitespace: strings composed from ordinary Unicode plus the six supported ASCII whitespace characters round-trip through escape/unescape; unknown percent sequences remain literal.
- Text/phrase: only format-specific lossless or documented canonical projections are compared. Aggregate/renumbering layouts never claim full model equality.
- Invalid bytes: bounded arbitrary buffers and systematic mutations execute decoders inside `catch_unwind`; success must return a valid document and failure a structured error. Cases and collection sizes have explicit ceilings.

Any discovered minimal counterexample becomes a named deterministic regression before a production fix.

## 6. Cross-Codec And Defect Regressions

`cross_codec.rs` keeps independently authored inputs and expectations:

- all seven `LexiconTextFormat` variants retain complete hand-written string assertions, then run against a representative real document for deterministic repeat output and documented semantic projection;
- phrase source containing multiline content, `$[...]`, all aliases/escapes, emoji, duplicates and candidate gaps is decoded, EUDP encoded with a fixed timestamp, decoded, canonically formatted and decoded again before comparing canonical documents;
- the `codeWeight[0]` regression places valid D/E/F-style records around the former dead-branch position and proves all expected entries survive. Existing `xfxy` and `%0B/%0C` tests remain named S0 regressions.

Because no aardio runtime or legacy golden exists, this task does not claim an independent real aardio byte comparison. That evidence remains an explicit `s0-integration` item.

## 7. Coverage Contract

Install an exact compatible `cargo-llvm-cov` into an ignored workspace-local `target/tools` prefix when absent; do not modify the user's global Cargo tools. Record the installed version in research. The stable measurement command must:

- clean stale coverage data;
- prepare/check fixtures before tests;
- run codec unit, property, cross-codec and real fixture tests;
- measure only `wubilex-codec` business source;
- fail below 90% line coverage;
- keep HTML/LCOV artifacts under ignored `target/`.

Coverage gaps are reviewed branch by branch. Add behavior assertions for reachable gaps; document genuinely unreachable defensive allocation branches instead of fabricating tests or excluding ordinary source files.

## 8. Dependencies And Reuse

- `xtask`: exact-pinned synchronous HTTPS client, Serde/JSON, SHA-256 and pure Rust LZMA-alone crates, plus a workspace path dependency on `wubilex-codec` for strict post-decompression validation.
- `wubilex-codec` dev-only: exact-pinned Proptest plus the smallest manifest/hash support needed by integration tests.
- Production codec dependencies and public APIs remain unchanged unless a verified fixture/property bug requires a reviewed fix.
- Manifest validation, digest formatting and file naming each have one owner in `xtask::fixtures`; tests consume the manifest contract rather than copy its eight entries.

The HTTP client may honor standard proxy environment configuration but never stores or logs proxy URLs or credentials. Lockfile changes are expected and must be reviewed for Tokio/Tauri/Windows leakage, duplicate crypto stacks, license compatibility and unnecessary default features.

## 9. Verification And Rollback

Verification runs fixture acquisition/check first, then focused real/property/cross tests, full codec/workspace gates, coverage, Rustdoc, dependency tree, global pnpm gates, Trellis/anchor checks and Git ignore/status checks.

Rollback units are: manifest/downloader, real fixtures, property/cross regressions, verified production fixes, and coverage/spec updates. If one upstream fixture is unavailable, keep the manifest requirement and fail visibly; do not substitute root `resource/` or silently reduce the eight-scheme set. If a real file is noncanonical, isolate the exact byte difference and review the codec contract before changing either side.
