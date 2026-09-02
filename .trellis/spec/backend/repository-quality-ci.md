# Repository Quality And CI

> Executable contracts for repository commands, generated bindings, document validation, dependency policy, and the Windows quality workflow.

---

## Scenario: Repository Commands And Windows Quality CI

### 1. Scope / Trigger

Apply this contract when changing the `xtask` command surface, Rust-to-TypeScript binding registry, requirement documents, dependency policy, toolchain version sources, or `.github/workflows/ci.yml`. These are repository-quality boundaries; they must not acquire product resource, release, signing, or Windows system-mutation behavior.

### 2. Signatures

```text
cargo xtask fixtures
cargo xtask fixtures --check
cargo xtask bindings
cargo xtask bindings --check
cargo xtask check-docs
```

Only these five argument vectors are valid. Any missing, duplicate, extra, unknown, or non-Unicode argument fails with the complete usage text.

```rust
pub fn builder<R: tauri::Runtime>() -> tauri_specta::Builder<R>;
pub fn export_mock(path: impl AsRef<Path>) -> Result<(), specta_typescript::Error>;
```

### 3. Contracts

- Repository paths derive from `xtask`'s `CARGO_MANIFEST_DIR`, so commands are independent of the caller's current directory.
- `src-tauri/src/bindings/mod.rs` is the only command/event registry. It remains generic over `tauri::Runtime`; repository export uses `MockRuntime`, exact `tauri-specta 2.0.0-rc.25` and `specta-typescript 0.0.12`, with no `wry` feature and no fake command.
- `cargo xtask bindings` exports through an ignored unique temporary file, normalizes to UTF-8 LF with no trailing line whitespace or blank EOF lines and exactly one final newline, and stages replacement of `src/types/generated/bindings.ts`. `--check` byte-compares canonical output without changing the committed target.
- `cargo xtask check-docs` scans sorted `docs/**/*.md`, accepts definition IDs only in their owning module/NFR/UX documents, requires unique counts `414/101/115/630`, rejects dangling IDs and real `TBD` or `待补充` placeholders, and delegates anchors to `.trellis/scripts/check_anchors.py`.
- `.github/workflows/ci.yml` runs on pull requests, `main` pushes, and manual dispatch on `windows-latest`, with `contents: read`, per-ref concurrency cancellation, a finite timeout, and no secrets or release steps. Every third-party action is pinned to a reviewed full commit SHA with its release tag in a comment. Frontend steps include frozen install, audit, format check, TypeScript, ESLint, Vitest, and the production Vite build.
- Node, pnpm, and Rust versions come only from `package.json.volta.node`, `package.json.volta.pnpm`, and `rust-toolchain.toml`. CI sets `VOLTA_FEATURE_PNPM=1` and uses `volta-cli/action` as the only Node/pnpm setup path. Do not add `engines.pnpm`, corepack, `packageManager`, `.nvmrc`, npm, yarn, npx, or a separate pnpm setup action.
- Cargo and pnpm caches are keyed by their lockfiles. The fixture cache is keyed by runner OS plus the fixture manifest digest. A cache hit never replaces preparation, offline verification, tests, coverage, audit, or freshness checks.
- `deny.toml` runs cargo-deny 0.20.2 advisories, licenses, bans, and sources. Advisory ignores are exact reviewed IDs with reasons; licenses are limited to the current lockfile; broad crate skips, git sources, unknown registries, and wildcard workspace dependencies remain forbidden.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Argument vector is not one of the five signatures | Exit nonzero and print all supported forms |
| Generated binding is missing or stale in `--check` | Exit nonzero, leave the target byte-identical, and print `cargo xtask bindings` |
| Binding export, normalization, staging, or replacement fails | Preserve the failing stage and path; do not report freshness success |
| Requirement definition is malformed, duplicated, wrongly owned, missing, dangling, or contains a placeholder | Aggregate every independently detectable issue and exit nonzero |
| Python or anchor checking fails | Preserve spawn/stdout/stderr evidence and fail `check-docs` |
| Codec coverage is below 90%, an audit denies, or a tool version differs | The CI step exits nonzero; no `continue-on-error` or fallback success |
| A local npm mirror has no audit endpoint | Diagnose with command-local official `--registry`; never commit mirror settings or treat the missing endpoint as a clean audit |

### 5. Good / Base / Bad Cases

- Good: a Rust IPC change updates the single registry, regenerates committed LF TypeScript, and passes Rust tests, `bindings --check`, TypeScript, and workflow checks.
- Base: the empty S0 command/event registry exports a deterministic non-handwritten TypeScript baseline and passes freshness checks without enabling a window runtime.
- Bad: a fake command is added to avoid empty output, a generated file is hand-edited, a requirement row is ignored because its ID is malformed, or CI skips verification after a cache hit.

### 6. Tests Required

- Parser tests accept exactly the five forms and reject invalid Unicode and every extra/unknown argument with full usage.
- Binding tests prove repeat generation is byte-identical, LF-only, free of trailing whitespace, and has exactly one final newline; mutation causes `--check` to fail without repair.
- Document tests cover valid trees, malformed/wrong-owner and duplicate definitions, count drift, dangling references, placeholders, anchor nonzero output, spawn failure, and the live `414/101/115/630` baseline.
- Workflow contract tests parse the checked-in YAML and root manifest, then assert triggers, permissions, concurrency, exact action SHAs, Volta Node/pnpm pins, the pnpm feature flag, cache keys, step order, frontend format/build steps, and forbidden bypass/package-manager patterns. Run actionlint as an independent syntax check.
- The final local gate mirrors CI: fmt, check, strict Clippy, fixtures, tests, Rustdoc, coverage, cargo-deny, bindings/docs, frozen pnpm install, audit, frontend format check, typecheck, lint, Vitest, and production build.

### 7. Wrong vs Correct

```yaml
# Wrong: a floating action and a tolerated security gate.
- uses: actions/checkout@v4
- run: cargo deny check
  continue-on-error: true

# Correct: reviewed immutable action code and fail-closed audit.
- uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0
- run: cargo deny check
```

## Scenario: Windows Runtime Smoke Harness

### 1. Scope / Trigger

Apply this contract when changing application startup, single-instance handoff,
launch warnings, session markers, runtime logging, or the checked-in Windows
smoke harness. The smoke is a destructive-process test, but it may delete only
markers and processes that the current invocation created and recorded.

### 2. Signatures

```text
pnpm run smoke:runtime
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/smoke-runtime.ps1
```

The package command builds the debug application without a bundle before the
PowerShell script runs. A non-administrator invocation self-elevates through a
visible UAC request and waits for the elevated result.

### 3. Contracts

- Resolve the executable below the repository `target/` directory and reject a
  path that escapes that root. Refuse to run while that exact debug executable
  already has a live process.
- Capture pre-existing marker and log baselines before starting product
  processes. Track every process and marker created by the current invocation;
  cleanup may stop or remove only those tracked resources.
- Verify four stages: hidden `/tray` startup; second-instance restore with one
  redacted invalid-argument notice and no late tray; close-to-tray plus a second
  restore without duplicate tray creation; forced termination followed by
  abnormal-session detection and `closeAction=exit` cleanup.
- Windows Known Folder resolution ignores a process-local `APPDATA` override.
  The debug application therefore accepts `WUBILEX_SMOKE_DATA_ROOT` only when
  its canonical path is the current debug executable's sibling
  `target/smoke-runtime-appdata`. Release builds ignore the variable. The smoke
  creates this root, uses its app-identifier child for config/log/session data,
  and removes it after owned process cleanup.
- A second instance can arrive before window readiness, in which case the
  activation queue is consumed without scheduling a tray delay. If a delay was
  scheduled, cancellation must be logged. Both paths must remain tray-free for
  more than three seconds after restore.
- PowerShell 5.1 unwraps a function's single `PSCustomObject` output. Every
  count or indexed read from `Get-LogEvents` must first force an array with
  `@(Get-LogEvents ...)`; a direct `.Count` is invalid for the one-record case.
- `Start-Process` rejects a null or empty `-ArgumentList`. The no-argument
  application path must omit that parameter entirely; non-empty vectors use
  the explicit argument branch.
- Transcript files are allowed only at the fixed repository-owned target path,
  are replayed to the parent process, and are removed after the elevated run.
- `pnpm tauri dev` uses Cargo's direct process launch and does not request UAC.
  With the required `requireAdministrator` manifest, run it from an already
  elevated terminal. Do not weaken or replace the manifest for dev ergonomics.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| UAC is rejected or times out | Exit nonzero; do not report smoke success |
| Non-elevated `tauri dev` returns Windows error 740 | Relaunch the terminal elevated; do not mark the dev launch as passed |
| The exact debug executable is already running | Fail before creating a marker or process |
| A hidden primary has a visible window | Fail and stop only the tracked primary |
| A secondary does not exit or activate the primary | Fail with the named wait stage |
| Exactly one log event is returned | Treat it as an array of count one |
| A launch has no arguments | Omit `-ArgumentList`; do not pass an empty collection |
| Smoke data root is absent, renamed, outside `target/`, or resolves through a junction outside `target/` | Debug startup rejects the override; never fall back to real user config |
| Early secondary activation occurs before delay scheduling | Restore directly; do not require a synthetic cancellation event and do not create a late tray |
| A scheduled hidden-start delay survives secondary restore | Fail after the three-second boundary if cancellation is absent or a tray appears |
| Clean exit leaves its owned marker | Fail; preserve unrelated baseline markers |
| Forced termination removes its marker | Fail because abnormal evidence was lost |
| Recovery does not observe the abnormal baseline | Fail without performing system recovery |

### 5. Good / Base / Bad Cases

- Good: all four stages pass inside the canonical isolated target directory;
  no debug process, transcript, isolated root, or owned marker remains.
- Base: the second instance is consumed before delayed-tray scheduling, so no
  cancellation event exists; waiting past the deadline still proves no tray
  was created after restore.
- Bad: changing `APPDATA` and assuming Tauri follows it, requiring cancellation
  when no delay was scheduled, counting a scalar log object through `.Count`,
  deleting every marker, or treating a rejected UAC request as success.

### 6. Tests Required

- Parse `scripts/smoke-runtime.ps1` with the Windows PowerShell 5.1 parser.
- Assert zero, one, and multiple log records produce counts `0`, `1`, and `N`
  through the array-forcing call form.
- Assert null/empty launch arguments select the no-`ArgumentList` branch while
  one or more arguments select the explicit branch.
- Run `pnpm run smoke:runtime` in an interactive Windows session and require
  all four named stages plus `runtime smoke: passed`.
- Unit-test canonical smoke-root acceptance and rejection of an outside path;
  run the real smoke to prove all app data appears only below the isolated root.
- Exercise or tolerate both hidden-start races: early activation without delay,
  and a scheduled delay that is cancelled. In either case assert no tray exists
  after waiting longer than three seconds from restore.
- From an elevated terminal, run Tauri dev and assert the real main window
  loads the local Vite React entry; a compile followed by error 740 is not this
  assertion.
- After success and failure, assert no owned process or transcript remains and
  only pre-existing baseline markers survive.

### 7. Wrong vs Correct

```powershell
# Wrong: APPDATA does not redirect the Windows Known Folder used by Tauri.
$env:APPDATA = $smokeRoot

# Wrong: scalar output has no reliable Count, and an empty ArgumentList fails.
(Get-LogEvents "launch_argument_notice").Count
Start-Process -FilePath $executable -ArgumentList @() -PassThru

# Correct: use the debug-only, executable-owned root; force arrays; omit args.
$env:WUBILEX_SMOKE_DATA_ROOT = $smokeRoot
@(Get-LogEvents "launch_argument_notice").Count
Start-Process -FilePath $executable -PassThru
```

## Sources

- [`xtask` command parser](../../../xtask/src/main.rs)
- [`xtask` binding generator](../../../xtask/src/bindings.rs)
- [`xtask` document validator](../../../xtask/src/check_docs.rs)
- [Canonical Tauri binding registry](../../../src-tauri/src/bindings/mod.rs)
- [Windows quality workflow](../../../.github/workflows/ci.yml)
- [Windows runtime smoke harness](../../../scripts/smoke-runtime.ps1)
- [Cargo dependency policy](../../../deny.toml)
- [Frontend toolchain version sources](../../../package.json)
