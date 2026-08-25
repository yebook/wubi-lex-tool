# S0 Integration Implementation Plan

## Preconditions

- [x] Return the changed pnpm scope to Phase 1 planning even though the active task metadata remains `in_progress`; do not implement the migration before fresh approval.
- [x] Confirm the worktree baseline and all ten prerequisite children are archived.
- [x] Load backend/frontend spec indexes, repository quality contracts, parent task artifacts, and this task's curated context.
- [x] Confirm the revised decision: use `package.json.volta.pnpm = 11.18.0`, remove `engines.pnpm`, and do not add `packageManager` or Corepack.
- [x] Record the pre-migration mismatch: project `pnpm --version` is `11.19.0` while `volta list pnpm` records `11.18.0`.
- [x] Obtain fresh implementation approval for this revised plan before pinning pnpm or reinstalling dependencies.
- [x] Confirm the user enabled `VOLTA_FEATURE_PNPM=1` in the Windows user environment and the official Volta pnpm prerequisite is now satisfied.
- [x] Obtain fresh implementation approval after presenting the resolved feature-flag plan summary.
- [x] Record the 2026-08-25 user decision to skip the independent legacy-output byte comparison while retaining all existing canonical, fixture, encoding, escaping, and regression evidence.
- [x] Do not inspect the repository root `resource/` directory.

## 1. Build The Integration Evidence Matrix

- [x] Create `research/integration-results.md` with environment and commit baseline.
- [x] Record all ten archived children, their deliverables, final status, and parent requirement mapping.
- [x] Map S0-R01..R09 and every parent acceptance criterion to executable or committed evidence.
- [x] Verify the four risk-spike reports, three exit codes, restoration markers, and Edge JSON aggregate without rerunning live operations.

## 2. Close Bootstrap Specifications Honestly

- [x] Replace every remaining `(To be filled by the team)` backend/frontend template.
- [x] For database, logging, component, Hook, and state files, document only approved boundaries, unselected decisions, forbidden assumptions, and future update triggers.
- [x] Preserve established S0 code references and status labels in both spec indexes.
- [x] Update `00-bootstrap-guidelines/prd.md` checkboxes only after the complete spec tree is non-placeholder and contains real S0 examples.

Rollback point: revert only integration documentation; do not invent a convention to make the placeholder scan pass.

## 3. Reconcile Parent Task State

- [x] Update the parent PRD acceptance checkboxes from the evidence matrix.
- [x] Update ordered child steps 5-7 and global review gates in the parent implementation plan.
- [x] Add the `s0-integration` child row if task metadata and the documented child map differ.
- [x] Keep any unsupported criterion unchecked and record its blocker instead of weakening it.

## 4. Migrate The pnpm Contract

- [x] Capture the current `pnpm-lock.yaml` SHA-256, copy the persisted user value into the current process, set it in CI, then run `volta pin pnpm@11.18.0`.
- [x] Verify `package.json.volta` preserves Node `24.18.1`, adds pnpm `11.18.0`, removes `engines.pnpm`, and has no `packageManager` field.
- [x] Verify project-directory `pnpm --version` is exactly `11.18.0` and no user-level pnpm installation is introduced.
- [x] Reinstall dependencies with `pnpm install --frozen-lockfile --force`; require the lockfile hash to remain unchanged.
- [x] Make `volta-cli/action` the sole CI Node/pnpm setup path; remove the pnpm version reader and `pnpm/action-setup`, then validate both versions against `package.json.volta`.
- [x] Update `xtask` workflow contracts and tests to require `volta.pnpm`, reject `engines.pnpm`, `packageManager`, and Corepack, and preserve direct `pnpm` commands.
- [x] Synchronize D17, architecture toolchain/risk/tree text, parent S0-R01 and plans, Trellis toolchain/quality specs, and this task's baseline/results.

Rollback point: restore the pre-migration `package.json` if Volta cannot resolve the project pin; do not add a second package-manager version source as a workaround.

Implementation discovery (2026-08-25): Volta `2.0.2` initially rejected the
`volta pin pnpm@11.18.0` command with exit code 1 and
`Only node and yarn can be pinned in a project`. Its suggested npm/yarn
workaround and Corepack remain forbidden. Official documentation identifies
`VOLTA_FEATURE_PNPM=1` as the required experimental pnpm prerequisite. After
the user persisted it and approved execution, the command succeeded natively:
the project now resolves pnpm `11.18.0`, Node remains `24.18.1`, and the forced
frozen reinstall retained lockfile SHA-256
`7CEB34F975BE75DDFCD83E0877E73ABD89ED2ECB84F34B25A4E7B4F3D8D0122D`.

## 5. Run The Complete Final Gate

Phase 2.1 migration verification and the independent Phase 2.2 same-tree gate
passed for the project toolchain, forced frozen reinstall/hash proof, complete
Rust suite, coverage, cargo-deny, workflow contract, actionlint, fixtures,
bindings, docs, official-registry pnpm audit, TypeScript, ESLint, Vitest,
Trellis validation, stale-contract scans and diff checks. The technical gate is
green. On 2026-08-25 the user explicitly approved removing the independent
legacy-output byte comparison from the S0 exit criteria; all retained canonical
and real-fixture evidence remains green.

```powershell
$env:VOLTA_FEATURE_PNPM = [Environment]::GetEnvironmentVariable('VOLTA_FEATURE_PNPM', 'User')
if ($env:VOLTA_FEATURE_PNPM -ne '1') { throw 'VOLTA_FEATURE_PNPM must be 1' }
$package = Get-Content package.json -Raw | ConvertFrom-Json
if ($package.volta.pnpm -ne '11.18.0') { throw 'unexpected volta.pnpm' }
if ($package.engines -and $package.engines.PSObject.Properties['pnpm']) { throw 'engines.pnpm is forbidden' }
if ($package.PSObject.Properties['package' + 'Manager']) { throw 'packageManager is forbidden' }
if ((pnpm --version).Trim() -ne $package.volta.pnpm) { throw 'pnpm version mismatch' }
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
$env:RUSTDOCFLAGS='-D warnings'; cargo doc --workspace --all-features --no-deps --locked
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov clean --workspace
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov --package wubilex-codec --all-features --summary-only --fail-under-lines 90
& .\target\tools\cargo-deny-0.20.2\bin\cargo-deny.exe check
cargo xtask fixtures --check
cargo xtask bindings --check
cargo xtask check-docs
& .\target\tools\actionlint-1.7.7\bin\actionlint.exe .github/workflows/ci.yml
pnpm install --frozen-lockfile --force
pnpm audit --audit-level high --registry https://registry.npmjs.org/
pnpm run typecheck
pnpm run lint
pnpm run test --run
python ./.trellis/scripts/task.py validate .trellis/tasks/08-25-s0-integration
python ./.trellis/scripts/task.py validate .trellis/tasks/08-22-s0-foundation
git diff --check
```

- [x] Confirm Node/pnpm/Rust versions come only from approved repository sources.
- [x] Confirm codec line coverage remains at least 90% and all eight real fixture checks execute rather than skip.
- [x] Scan specs for template placeholders and the repository for forbidden package-manager/version-source drift.
- [x] Run a full-scope Trellis check after all migration, evidence, and checklist updates.

## 6. Finish S0

- [x] Replace the reopened integration verdict with PASS after the migrated same-tree gate and the explicit user-approved exit-criterion change.
- [x] Repeat Phase 3.3 spec review after implementation and independent review.
- [x] Synchronize roadmap, backend quality guidance, parent acceptance evidence, and integration results without claiming aardio/original-project golden evidence exists.
- [ ] Commit integration work before any archive operation.
- [ ] On PASS, archive `s0-integration`, `00-bootstrap-guidelines`, and `s0-foundation` in that order, then record the journal.
- [ ] Verify the worktree is clean and the next task is not an S1 implementation task unless separately created and approved.

## Completion Gate

S0 is complete only when every acceptance criterion in this task and its parent
has evidence, the complete final gate passes on the same tree, the spec tree has
no empty templates, and the integration, bootstrap, and parent tasks are all
archived. Any failed criterion keeps S1 blocked.
