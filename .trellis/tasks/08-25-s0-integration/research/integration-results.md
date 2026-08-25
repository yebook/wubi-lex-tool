# S0 Integration Results

## Status

`PASS: READY FOR ORDERED COMMIT AND ARCHIVE`. The project-level Volta pin,
forced frozen reinstall, CI/workflow contract, documentation sync, complete
same-tree gate and independent Phase 2.2 review all passed. On 2026-08-25 the
user explicitly approved “跳过逐字节比对”; the seven text outputs therefore
use the existing canonical complete-string, deterministic real-fixture,
encoding, whitespace-escape and regression evidence as their S0 exit contract.
No aardio runtime or independently produced original-project golden is claimed,
and all `.lex` and EUDP byte-level requirements remain unchanged.

## Environment And Baseline

- Captured on 2026-08-25 from branch `main` at
  `771de0a5e6e7d7a0d05f5301fbee8394bfff4bcd`.
- Prior gate environment: Windows host; Node `24.18.1`, user-level pnpm `11.18.0`,
  Rust/Cargo `1.97.1`.
- The prior gate invoked the then-approved `pnpm` command directly; `Get-Command pnpm`
  resolves that host command to `E:\env\Volta\pnpm.exe`. No alternate package
  manager or project-level Volta pnpm pin was used.
- `cargo metadata --no-deps` reports exactly `wubilex-codec`, `wubilex-core`,
  `wubilex-winime`, `wubilex-resource`, `wubilex-app` (`src-tauri`) and
  `xtask` as workspace members.
- This integration pass does not read root `resource/`, rerun a Windows
  `--live` probe, or launch the visible Edge benchmark.

## Implemented pnpm Migration

- User decision: run `volta pin pnpm@11.18.0`; keep
  `package.json.volta.pnpm` as the sole pnpm version source; remove
  `engines.pnpm`; do not add `packageManager` or Corepack.
- The initial planning recheck returned `pnpm --version` `11.19.0` while
  `volta list pnpm` records `11.18.0`, confirming the project pin is required.
- Implementation preflight later returned project-directory pnpm `11.23.0`
  while the Volta inventory remained `11.18.0`. The first approved command,
  before the feature flag was available to the process, produced this exact
  non-mutating failure:

  ```text
  error: Only node and yarn can be pinned in a project

  Use `npm install` or `yarn add` to select a version of pnpm for this project.
  ```

  The command exited 1. npm/yarn, Corepack, user-level installation changes and
  a manual manifest workaround were not used. The user then persisted the
  official `VOLTA_FEATURE_PNPM=1` prerequisite and approved the resolved plan.
- With the persisted flag copied into the implementation process,
  `volta pin pnpm@11.18.0` succeeded. `package.json.volta` now contains Node
  `24.18.1` and pnpm `11.18.0`; `engines.pnpm` and `packageManager` are absent,
  and project-directory `pnpm --version` returns `11.18.0`.
- `pnpm install --frozen-lockfile --force` completed successfully and retained
  the pre-migration lockfile SHA-256
  `7CEB34F975BE75DDFCD83E0877E73ABD89ED2ECB84F34B25A4E7B4F3D8D0122D`
  unchanged.
- CI now sets the feature flag and uses only `volta-cli/action` for Node/pnpm;
  workflow-contract tests, architecture D17/section 8.5/R55, active parent
  artifacts, and Trellis toolchain/quality specs use the same source contract.
- The post-migration complete same-tree gate passed on the final reviewed tree.

## Post-Migration Complete Verification

| Check | Result |
|---|---|
| User feature flag, `package.json.volta`, Node/pnpm versions and forbidden-field assertions | PASS; flag `1`, Node `24.18.1`, pnpm `11.18.0`, no `engines.pnpm` or `packageManager` |
| `pnpm install --frozen-lockfile --force` plus SHA-256 proof | PASS; hash remained `7CEB34F975BE75DDFCD83E0877E73ABD89ED2ECB84F34B25A4E7B4F3D8D0122D` |
| Focused `xtask` workflow-contract test and actionlint 1.7.7 | PASS; Volta is the only Node/pnpm setup path and YAML has no diagnostics |
| `cargo fmt --all -- --check` | PASS after applying rustfmt's required layout to the new assertions |
| Cargo check, strict Clippy, 169 tests and warnings-denied Rustdoc | PASS; 130 codec tests, 21 Windows example pure tests and 18 xtask tests |
| Codec coverage | PASS; 90.12% lines with `--fail-under-lines 90` |
| Workspace-local cargo-deny 0.20.2 | PASS; advisories, bans, licenses and sources all `ok` |
| `cargo xtask fixtures --check` and `cargo xtask bindings --check` | PASS; all eight schemes verified offline and generated TypeScript is current |
| `cargo xtask check-docs` | PASS; `414/101/115/630`, dangling 0, placeholders 0, anchors 0 |
| Official-registry pnpm audit | PASS; no known vulnerabilities |
| TypeScript, ESLint and Vitest | PASS; Vitest 3 files / 15 tests |
| Trellis context validation, stale-contract/empty-spec scans and `git diff --check` | PASS; only normal LF-to-CRLF checkout warnings were printed |

Three earlier cargo-deny attempts stopped before evaluation because GitHub reset
the RustSec advisory DB fetch. The independent review retried the exact online
command successfully; advisories, bans, licenses and sources all passed.

## Archived Child Evidence

Every child below has `status: completed` in its archived `task.json` below
`.trellis/tasks/archive/2026-08/`. Blank historical `commit` metadata is not
used as completion evidence; the archived task record, current deliverable and
current executable checks are the evidence chain.

| Order | Archived child | Current deliverable and key evidence | Parent mapping | Result |
|---:|---|---|---|---|
| 0 | `08-22-s0-docs-spec-alignment` | Corrected roadmap/architecture contracts and initial backend/frontend spec baselines; current `cargo xtask check-docs` owns the `414/101/115/630` and anchor checks. | S0-R09; AC 9-10 | PASS |
| 1 | `08-22-s0-workspace-toolchain` | Root workspace/toolchain manifests, minimal Tauri/frontend shell and frozen lockfiles; current metadata reports exactly six active Rust members. | S0-R01; AC 1 and 7 | PASS |
| 2 | `08-22-s0-codec-model` | `crates/wubilex-codec/src/{model,error,limits}` plus `model_contracts.rs` and `error_and_limits.rs` establish ordered duplicate-preserving models, structured errors and bounds. | S0-R02; supports AC 2-5 | PASS |
| 3 | `08-22-s0-lex-binary` | `crates/wubilex-codec/src/lex/` and `tests/lex_binary.rs` implement explicit bounded `.lex` parsing/encoding without ABI byte mapping; real eight-scheme proof is supplied by child 7. | S0-R02, S0-R03; AC 2 | PASS |
| 4 | `08-23-s0-eudp` | `crates/wubilex-codec/src/eudp/` and `tests/eudp_binary.rs` establish strict raw EUDP round trips, UTF-16, ordering, tombstones, explicit timestamps and malformed-input errors. | S0-R03; AC 4 | PASS |
| 5 | `08-23-s0-lex-text` | `src/text/`, `src/escape/` and text tests establish strict encoding detection, six input dialects plus Microsoft handling, seven canonical outputs and visible warnings. | S0-R04, S0-R05; AC 3 and 5 | PASS |
| 6 | `08-24-s0-phrase-aux` | Phrase, word-frequency, split-table and scheme-detection modules/tests cover P1-P6, multiline/arrays/time aliases, two auxiliary formats and all eight scheme branches. | S0-R04, S0-R05; AC 4-5 | PASS |
| 7 | `08-24-s0-fixtures-regressions` | Pinned eight-entry fixture manifest, `cargo xtask fixtures [--check]`, shared manifest loader, real-fixture/property/cross-codec tests and the measured 90.12% codec line baseline. | S0-R06, S0-R07; AC 2-6 | PASS |
| 8 | `08-24-s0-xtask-ci` | `fixtures`, `bindings` and `check-docs` xtask commands, `deny.toml`, generated TypeScript baseline, and SHA-pinned Windows CI workflow. | S0-R01, S0-R07; AC 7 and 9 | PASS |
| 9 | `08-24-s0-risk-spikes` | Three isolated Windows examples, virtual-scroll harness, four result reports, three raw live logs/exit codes and Edge JSON. | S0-R08; AC 8 | PASS |

All archived children completed their approved scopes. The fixture-regression
and xtask-CI records historically deferred the independent aardio runtime/golden
comparison to this integration task. The 2026-08-25 explicit user decision
removed that comparison from the S0 exit criteria without rewriting archived
history or claiming the evidence exists. S1/S2 product implementation and
per-dictionary redistribution licensing remain separate future work.

## Parent Requirement Evidence

| Requirement | Evidence | Current result |
|---|---|---|
| S0-R01 | Root `Cargo.toml`, `rust-toolchain.toml`, `package.json`, `pnpm-workspace.yaml`, both lockfiles, current six-member metadata, and repository-quality toolchain scans. | PASS |
| S0-R02 | `wubilex-codec` is synchronous and its direct production dependencies remain limited to codec concerns; `.lex`/EUDP readers use explicit bounded field parsing. `quality-guidelines.md` forbids ABI mapping and cross-layer dependencies. | PASS |
| S0-R03 | `tests/lex_binary.rs`, `tests/eudp_binary.rs`, real-fixture byte re-encoding, and codec wire contracts cover byte layout, stable ordering, UTF-16 and damaged input. | PASS |
| S0-R04 | Text and phrase test suites cover six lexicon dialects, the Microsoft branch, seven outputs, P1-P6, `$[...]`, multiline text and time aliases. | PASS |
| S0-R05 | `text_decode.rs` covers BOM-first UTF-8/UTF-16LE/BE and GBK; `scheme_detection.rs` plus the eight real fixtures cover all schemes and lowercase `xfxy`. | PASS |
| S0-R06 | The committed manifest pins HTTPS URLs, sizes and two SHA-256 values for eight schemes. Offline verification is non-repairing and missing fixtures hard-fail. | PASS |
| S0-R07 | `.github/workflows/ci.yml` and repository specs require fmt/check/Clippy/tests/Rustdoc, 90% codec coverage, cargo-deny, bindings/docs and frozen frontend gates with shared version sources. | PASS |
| S0-R08 | The archived risk summary and raw evidence report PASS for TSF, ACL, Task Scheduler COM and 300,000-row virtualization with restoration/cleanup. | PASS |
| S0-R09 | Existing codec, repository automation, bindings, Windows and virtualization specs cite real S0 source/tests. Five unimplemented product areas are marked `Pending implementation evidence` with boundaries and update triggers, not examples. | PASS |

## Parent Acceptance Evidence

| # | Parent acceptance criterion | Evidence | Current result |
|---:|---|---|---|
| 1 | Six-member workspace and minimal frontend shell build under fixed tools | Current metadata plus passing workspace/frontend gates; `wubilex-learn` is absent from members. | PASS |
| 2 | Eight real `.lex` files byte-round-trip | `real_fixtures.rs` and the archived fixture result table cover 86, 98, 06, 091, 092, Zhengma, Xiaohe and Biaoxingma with exact sizes/digests; both final test and offline check passed. | PASS |
| 3 | Seven lexicon text outputs satisfy the approved canonical and real-fixture contract | `text_format.rs` asserts complete canonical strings for all seven formats; `cross_codec.rs` projects real documents deterministically; strict encoding, whitespace escaping and named regressions pass. The 2026-08-25 user decision removes the independent legacy comparison, and no aardio golden is claimed. | PASS |
| 4 | EUDP covers emoji, multiline phrases, `$[...]` and damaged input | Raw EUDP tests cover emoji/newlines/corruption; the passing phrase-to-EUDP cross-codec test covers arrays, multiline text, variables, gaps and duplicates. | PASS |
| 5 | Three S0 defects have regressions and S4 defects remain assigned | Passing named tests cover lowercase `xfxy`, `%0B/%0C` symmetry and the removed `codeWeight[0]` branch. Backend quality guidance assigns the remaining three defects to S4. | PASS |
| 6 | Codec line coverage is at least 90% | Final measured line coverage is 90.12% with `--fail-under-lines 90`. | PASS |
| 7 | CI and local commands share version sources and pass | CI and local commands share `package.json.volta` and `rust-toolchain.toml`; the migrated complete same-tree gate passed. | PASS |
| 8 | Four technical spikes pass or trigger architecture review | All four archived reports are PASS, so no alternative architecture review is required. | PASS |
| 9 | Requirement counts stay `414/101/115/630` and anchors stay valid | Final `cargo xtask check-docs` reported `414/101/115/630`, zero dangling IDs, zero placeholders and zero invalid anchors. | PASS |
| 10 | Related Trellis specs contain real S0 examples and bootstrap can close | Established S0 guides cite codec/automation/binding/Windows/virtualization paths. The five unimplemented areas now have honest pending-state contracts and zero empty templates remain. | PASS |

## Risk-Spike Evidence Audit

| Probe | Raw evidence checked without rerun | Restoration or cleanup | Result |
|---|---|---|---|
| TSF | `tsf-profile.live.exitcode` is `0`; log records Wubi ACTIVE changing true -> false -> true while ENABLED stays true. | `restoration=verified`, `verdict=LIVE PASS`; the report records an independent matching dry-run snapshot. | PASS |
| ACL | `acl-owner.live.exitcode` is `0`; log records `TrustedInstaller -> Administrators -> TrustedInstaller`. | `restoration=baseline A verified; privileges restored; file deleted`; report records zero residual probe files. | PASS |
| Scheduler | `task-scheduler.live.exitcode` is `0`; log records accepted `Stop(0)` and returned Run instance evidence. | `restoration=logical baseline verified`, `verdict=LIVE PASS`; unchanged detached `ctfmon.exe` PID is an approved logical baseline. | PASS |
| Virtual scroll | `virtual-scroll.json` has `passed=true`, exactly three valid visible runs at 119.60, 93.40 and 110.02 fps, no errors/blanks, and maximum 45 DOM rows. | Runner report states Edge and Vite closed in `finally`; no product state was created. | PASS |

## Complete Same-Tree Gate

The independent review reran or verified the following results on the migrated
tree. Earlier equivalent results remain supporting history only.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets --all-features --locked` | PASS |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS |
| `cargo test --workspace --all-targets --all-features --locked` | PASS; 130 codec tests, 21 Windows example pure tests and 18 xtask tests passed with no failures or ignored tests |
| `$env:RUSTDOCFLAGS='-D warnings'; cargo doc --workspace --all-features --no-deps --locked` | PASS |
| workspace-local `cargo-llvm-cov 0.9.0 llvm-cov clean --workspace` and `--package wubilex-codec --all-features --summary-only --fail-under-lines 90` | PASS; line coverage 90.12% |
| workspace-local `cargo-deny 0.20.2 check` | PASS; advisories, bans, licenses and sources all `ok`; the independent review retried the unchanged command after one transient GitHub TLS EOF, and approved duplicate-version warnings remain informational |
| `cargo xtask fixtures --check` | PASS; all eight named schemes reported verified cached files without network repair |
| `cargo xtask bindings --check` | PASS; generated TypeScript is current |
| `cargo xtask check-docs` | PASS; modules 414, NFR 101, UX 115, total 630, dangling 0, placeholders 0, anchors 0 |
| workspace-local `actionlint 1.7.7 .github/workflows/ci.yml` | PASS with no diagnostics |
| `pnpm install --frozen-lockfile --force` | PASS with project-pinned pnpm 11.18.0; lockfile hash unchanged |
| `pnpm audit --audit-level high --registry https://registry.npmjs.org/` | PASS; one transient `ECONNRESET` retried automatically, final result `No known vulnerabilities found` |
| `pnpm run typecheck` | PASS, including the isolated spike TypeScript project |
| `pnpm run lint` | PASS with zero warnings |
| `pnpm run test --run` | PASS; 3 files and 15 tests |
| `python ./.trellis/scripts/task.py validate` for parent and integration tasks | PASS; parent has 0/0 historical context entries and integration has 8/8 implementation/check entries |
| Spec placeholder, version-source, forbidden-package-manager, machine-local-resource and trailing-whitespace scans | PASS with zero disallowed matches |
| `git diff --check` | PASS; Git reports only the repository's normal LF-to-CRLF checkout warnings |

`Cargo.lock` and `pnpm-lock.yaml` retained their pre-gate SHA-256 values
`172B6FCC5096E0F852642949C7737068486375E4B89C4386DC456E3A67A0FD15`
and `7CEB34F975BE75DDFCD83E0877E73ABD89ED2ECB84F34B25A4E7B4F3D8D0122D`.
The machine does not expose `cargo llvm-cov`, `cargo deny` or `actionlint` on
`PATH`; the gate used the already established ignored `target/tools`
executables at the exact approved versions and did not install or alter global
tools.

## Independent Phase 2.2 Review

The full-scope review rechecked all ten archived child records, S0-R01..R09,
all parent and bootstrap acceptance items, the five pending-evidence guides,
the four raw risk results and the complete same-tree gate. It strengthened the
workflow contract to require exactly the Node/pnpm Volta pins and reject npm,
yarn, npx and Corepack package scripts. Under the then-current roadmap wording,
it correctly identified the absent external golden as the only blocker.

After the later 2026-08-25 user decision, an independent follow-up review
verified that the change removes only that external comparison: complete
canonical strings, deterministic real-fixture projections, strict encoding,
whitespace escapes, named regressions, and every `.lex`/EUDP byte-level
requirement remain in force. Both task validations, document checks, anchors,
focused consistency scans and the diff check pass; no legacy golden is claimed.

## Phase 3.3 Specification Review

The five pending-evidence guides remain honest about unimplemented product
areas. Toolchain, repository-quality, frontend-quality, virtualization and
index specs consistently use the project Volta pnpm pin and feature flag, with
no competing version source or package-manager command. Backend quality guidance
now records the approved text-output evidence boundary and explicitly avoids
claiming an aardio/original-project golden exists.

## S1 Entry Verdict

`PASS: S1 ENTRY ALLOWED AFTER THE ORDERED S0 COMMIT/ARCHIVE SEQUENCE`. The
archived S0 product and risk evidence, project pin, reinstall, synchronized
contracts and complete same-tree gate are green. On 2026-08-25 the user
explicitly approved skipping the independent aardio/original-project golden
comparison; the retained canonical and real-fixture evidence satisfies the
revised roadmap criterion without implying that legacy golden evidence exists.

Remaining limitations do not block closure: Windows and Edge conclusions are
machine-specific archived evidence and were intentionally not rerun;
per-dictionary redistribution licensing remains an S5 decision; and database,
logging, component, Hook and store conventions remain pending until real
product implementation and tests exist.
