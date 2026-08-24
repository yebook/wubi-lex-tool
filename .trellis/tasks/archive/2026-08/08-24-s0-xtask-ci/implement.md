# Implementation Plan - S0 xtask And CI Gates

## 1. Baseline And Failing Contracts

- [x] Record current tool versions, Cargo/pnpm lock hashes, 130 codec + 6 xtask tests, 90.12% line coverage, 414/101/115/630 document counts, missing workflow/deny/binding baselines, and the local npm-mirror audit limitation.
- [x] Resolve exact compatible dependency versions/features and immutable action SHAs; record upstream release, SHA, license, MSRV and why each action is required.
- [x] Add failing xtask parser tests for the five accepted command forms and rejection of all other argument vectors.
- [x] Add failing bindings freshness tests and document-validator tests for good, duplicate, count-drift, dangling, placeholder and anchor-failure cases.
- [x] Add failing workflow static assertions for triggers, permissions, concurrency, full SHA pins, dynamic versions, cache keys, order, and absence of forbidden/bypass patterns.

## 2. Binding Baseline

- [x] Add exact `tauri-specta`/`specta-typescript` dependencies with no Wry runtime and expose one generic registry under `src-tauri/src/bindings/`.
- [x] Implement `xtask bindings` generation through `MockRuntime`, repository-root resolution, unique ignored temporary output, LF stability and cleanup.
- [x] Implement non-mutating `bindings --check` with actionable missing/stale diagnostics and no repair path.
- [x] Commit the generated empty TypeScript baseline, remove obsolete `.gitkeep`, and add a narrow LF `.gitattributes` rule.
- [x] Prove repeat generation is byte-identical and check mode catches mutation while leaving the target untouched.

## 3. Document Gate

- [x] Implement deterministic Markdown discovery and requirement definition/reference extraction using the fixed M1..M8/NFR/UX grammar.
- [x] Enforce definition uniqueness and exact 414/101/115/630 counts with per-file/line diagnostics.
- [x] Enforce no dangling references and no real `TBD`/`待补充` placeholders without adding broad exclusions.
- [x] Invoke the verified Python anchor checker, preserve stage output/failure context, and fail clearly when Python or the script is unavailable.
- [x] Run focused unit cases plus `cargo xtask check-docs` against the real repository.

## 4. Dependency Audit

- [x] Add a strict `deny.toml` for advisories, licenses, bans and sources based on the actual full lockfile; document every exact exception.
- [x] Install `cargo-deny 0.20.2` only under ignored `target/tools` and run all four checks locally without changing global Cargo state.
- [x] Run global `pnpm audit --audit-level high`; when the developer mirror lacks the endpoint, verify through a command-level official-registry override without creating `.npmrc`.
- [x] Review Cargo.lock changes for feature leakage, duplicate stacks, licenses and MSRV; prove pnpm-lock remains byte-identical.

## 5. GitHub Actions Workflow

- [x] Add one least-privilege Windows quality workflow for pull requests, `main` pushes and manual dispatch with per-ref concurrency cancellation.
- [x] Pin every action to a reviewed full commit SHA and retain release comments; do not pin Node/pnpm/Rust versions in workflow YAML.
- [x] Read Node through the Volta action, pnpm from `engines.pnpm`, and Rust from `rust-toolchain.toml`; assert actual versions before gates.
- [x] Add Cargo/pnpm caches keyed by lockfiles and fixture cache keyed by OS + manifest; verification remains mandatory after every restore.
- [x] Install exact ephemeral cargo-llvm-cov/cargo-deny tools and implement the complete fail-fast gate order from `design.md` without tolerated failures.
- [x] Add static workflow contract tests and run a local YAML/actionlint-equivalent validation; manually inspect expression and PowerShell quoting.

## 6. Full Local Validation

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask fixtures
cargo xtask fixtures --check
cargo test --workspace --all-features
$env:RUSTDOCFLAGS = '-D warnings'; cargo doc --workspace --all-features --no-deps; Remove-Item Env:RUSTDOCFLAGS
cargo xtask bindings
cargo xtask bindings --check
cargo xtask check-docs
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov clean --workspace
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov --package wubilex-codec --all-features --summary-only --fail-under-lines 90
& .\target\tools\cargo-deny-0.20.2\bin\cargo-deny.exe check
pnpm install --frozen-lockfile
pnpm audit --audit-level high
pnpm typecheck
pnpm lint
pnpm test --run
python ./.trellis/scripts/task.py validate 08-24-s0-xtask-ci
python ./.trellis/scripts/check_anchors.py
git diff --check
git status --short
```

- [x] Verify lockfile hashes, exact tool versions, dependency trees, action SHA provenance, no root `resource/` access and all forbidden toolchain/package-manager scans.
- [x] Run independent Trellis check against CI-R01..R14 and every acceptance criterion; fix findings and repeat affected gates.

## 7. Finish

- [x] Update backend/frontend directory, type-safety, quality and toolchain specs with established xtask/CI contracts.
- [x] Update parent progress while leaving resources/licenses, risk spikes, independent aardio golden and S0 integration open.
- [x] Commit implementation, specs and task records in coherent batches; archive and journal only after all gates pass.

## Rollback Points

- Binding exporter and generated output are one rollback unit. A proven tauri-specta incompatibility returns to planning before selecting `ts-rs`.
- Document validation is independent from CI YAML; keep a useful local `check-docs` even while debugging workflow setup.
- Audit policy failures require dependency or exact justified policy changes, never disabling the whole check.
- Cache/network failures do not weaken fixture count, hashes, coverage, audits or missing-file hard failures.
