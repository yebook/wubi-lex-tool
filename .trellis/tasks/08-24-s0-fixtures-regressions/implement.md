# Implementation Plan - S0 真实夹具与回归

## 1. Baseline And Dependency Review

- [x] Record current 117 codec tests, workspace gate results, lockfile hashes, dependency trees, and the missing `cargo-llvm-cov` baseline.
- [x] Select exact compatible versions and minimal features for synchronous HTTPS, Serde JSON, SHA-256, pure Rust LZMA-alone and Proptest; record licenses and prove no codec production dependency changes.
- [x] Add failing xtask/unit contracts for command dispatch, manifest validation, URL/path/hash/size rejection, bounded decompression and offline check behavior.
- [x] Add failing public test harnesses for a missing real fixture, one property round trip, one cross-codec round trip and the named `codeWeight[0]` regression.

## 2. Manifest And Fixture Acquisition

- [x] Add `tests/fixtures/{manifest.json,README.md,.gitignore}` with exactly eight schemes and no binary payloads.
- [x] Implement `cargo xtask fixtures` argument parsing, repository-root resolution and actionable exit errors.
- [x] Implement strict manifest parsing and uniqueness/scheme/path/HTTPS/hash/size validation.
- [x] Implement streamed HTTPS download to create-new temporary files, bounded redirects/body, SHA-256 and exact archive-size checks.
- [x] Implement bounded LZMA-alone decode, decoded hash/size/magic validation, cleanup guards and validated final placement.
- [x] Implement cache reuse and network-free `--check`; test missing, corrupt, stale partial and valid-cache behavior.
- [x] Bootstrap compressed/decoded hashes, delete all generated files, refetch from empty cache and confirm Git ignores every binary/partial artifact.

## 3. Eight-Scheme Real Regressions

- [x] Parse the shared manifest in `real_fixtures.rs` and fail with an explicit preparation command when any fixture is absent.
- [x] For all eight fixtures assert size/hash, strict decode, nonempty document, expected scheme and byte-identical re-encode.
- [x] Record stable size, entry, distinct-code and digest results in task research; verify Zhengma formation and direct/scored scheme paths.
- [x] Investigate every noncanonical or misdetected fixture as a compatibility finding before changing codec behavior; add a minimal synthetic regression for any fix.

## 4. Property And Damage Tests

- [x] Add bounded strategies for validated codes, Unicode text, weights, candidates, lexicon documents and phrase documents.
- [x] Add canonical `.lex` encode/decode/re-encode properties with explicit weights, duplicate entries and non-BMP text.
- [x] Add fixed-timestamp EUDP canonical projection and byte-reencode properties, including candidate order and duplicates.
- [x] Add six-whitespace escape/unescape and literal-percent properties.
- [x] Add bounded arbitrary/mutated byte decode no-panic properties; convert every discovered minimal counterexample into a named deterministic regression.
- [x] Pin practical case/size limits and verify failures print a replayable Proptest seed.

## 5. Cross-Codec And S0 Defects

- [x] Add real-document deterministic projections for all seven lexicon text formats without generating expected values from the formatter under test.
- [x] Add phrase text -> EUDP -> canonical phrase text semantic round trip with arrays, multiline text, aliases, escapes, emoji, gaps and duplicates.
- [x] Add an explicitly named `codeWeight[0]` regression around D/E/F parsing; retain and reference the existing `xfxy` and `%0B/%0C` failure-to-pass tests.
- [x] Keep independent aardio runtime/golden comparison deferred in task evidence and parent integration acceptance; do not overstate this task's result.

## 6. Coverage Closure

- [x] Install an exact `cargo-llvm-cov` version under ignored `target/tools` if absent and record the executable/version without changing global Cargo state.
- [x] Run codec-only coverage with all synthetic, property, cross-codec and real tests; save the report under `target/`.
- [x] Review uncovered business branches, add behavior-driven tests for reachable gaps and document unreachable defensive paths.
- [x] Re-run with a hard 90% line threshold and capture the final percentage and command for `s0-xtask-ci`.

## 7. Full Validation

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo xtask fixtures
cargo xtask fixtures --check
cargo test -p wubilex-codec --all-features
cargo test --workspace --all-features
cargo doc -p wubilex-codec --all-features --no-deps
cargo tree -p wubilex-codec
cargo tree -p xtask
pnpm lint
pnpm typecheck
pnpm test
python ./.trellis/scripts/task.py validate 08-24-s0-fixtures-regressions
python ./.trellis/scripts/check_anchors.py
git check-ignore crates/wubilex-codec/tests/fixtures/wubi86.lex
git diff --check
git status --short
```

- [x] Run the exact workspace-local coverage command with `--fail-under-lines 90` after the full test set.
- [x] Verify fresh and warm fixture paths, lockfile review, production forbidden-pattern scan and no root `resource/` access.
- [x] Run independent Trellis check against FR-R01..R13 and every acceptance criterion; fix findings and repeat affected gates.

## 8. Finish

- [x] Update backend directory/quality/error specs with the established fixture manifest, xtask and property/coverage contracts.
- [x] Update parent progress while leaving independent aardio golden, CI wiring, risk spikes and S0 integration open.
- [ ] Commit implementation, specs and task records in coherent batches; archive this child and record the journal only after all gates pass.

## Rollback Points

- Manifest/downloader changes are separable from codec tests; a network client problem must not trigger production codec changes.
- Real fixture incompatibility is isolated per scheme and digest. Do not replace it with a machine-local sample or weaken exact byte assertions.
- Property-discovered production fixes require a minimized deterministic regression before the fix and their own reviewable commit chunk.
- Coverage shortfall keeps the task open; exclusions require a documented non-business-source justification, never a percentage-only workaround.
