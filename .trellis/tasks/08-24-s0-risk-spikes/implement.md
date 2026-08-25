# S0 Risk Spikes Implementation Plan

## Preconditions

- [ ] Confirm the task is `in_progress`; do not implement while it remains `planning`.
- [ ] Load the curated implementation context and current backend/frontend specs.
- [ ] Confirm `git status` contains only expected task-planning changes.
- [ ] Confirm global `pnpm --version` is `11.18.0`; do not invoke Volta pnpm, Corepack, npm, yarn, or npx.
- [ ] Do not inspect or read the repository root `resource/` directory.

## 1. Establish Isolated Harnesses

- [ ] Add Windows-targeted `windows = "0.61.3"` to `crates/wubilex-winime/Cargo.toml` with the minimal feature union from `design.md`.
- [ ] Create three separate examples under `crates/wubilex-winime/examples/` and a small shared support module for COM lifetime, native error evidence, elevation, handle ownership, and polling.
- [ ] Give every example a read-only default and an exact `--live` switch; reject unknown arguments.
- [ ] Add `@tanstack/react-virtual@3.14.10` and `playwright-core@1.62.1` as exact root dev dependencies using global pnpm, then review `package.json` and `pnpm-lock.yaml` changes.
- [ ] Create the isolated `spikes/virtual-scroll/` Vite entry, runner, focused tests, and scripts without adding a product route or component.

Validation:

```powershell
cargo check -p wubilex-winime --all-targets
pnpm run typecheck
pnpm run lint
```

Rollback point: no live operation has occurred; remove only the new isolated harness files and direct dependency entries if the selected bindings cannot compile cleanly.

## 2. Implement The TSF Profile Probe

- [ ] Initialize and balance COM lifetime, including the `S_OK`/`S_FALSE` cases.
- [ ] Instantiate `ITfInputProcessorProfileMgr` with the documented Wubi identifiers.
- [ ] Snapshot Wubi with `GetProfile` and the exact current keyboard profile with `GetActiveProfile`, preserving `S_FALSE`; require Wubi ENABLED and current language `0x0804` before live mutation.
- [ ] Use only `TF_IPPMF_FORSESSION` and implement active/inactive exercise paths with bounded polling of `TF_IPP_FLAG_ACTIVE` plus current-profile identity.
- [ ] Arm restoration before mutation; explicitly restore the captured input-processor, keyboard-layout, or no-active baseline, then verify ACTIVE/current profile and unchanged ENABLED state.
- [ ] Add pure tests for operation choice, snapshot comparison, timeout, and restoration-decision logic.
- [ ] Verify dry-run output contains baseline and planned action without a state change.

Dry-run command:

```powershell
cargo run -p wubilex-winime --example tsf_profile_spike
```

## 3. Implement The Temporary-File ACL Probe

- [ ] Restrict the target to a probe-created unique file below `std::env::temp_dir()`; accept no path argument.
- [ ] Implement elevation and token privilege checks, including `ERROR_NOT_ALL_ASSIGNED`, with privilege-state restoration.
- [ ] Resolve TrustedInstaller through the two-call `LookupAccountNameW` pattern and Administrators through `CreateWellKnownSid`, keeping caller-owned SID storage alive.
- [ ] Wrap LocalAlloc security descriptor/SDDL buffers and token/snapshot handles in ownership-safe RAII types.
- [ ] Capture cleanup baseline A, establish verification baseline B as TrustedInstaller, change owner to Administrators, restore B, and compare normalized owner/DACL evidence.
- [ ] Keep DACL unchanged during both owner transitions.
- [ ] After recording the B round-trip result, restore cleanup baseline A and delete the temporary file only after cleanup verification; retain and report it on restoration failure.
- [ ] Add pure tests for stage transitions, cleanup ownership, semantic comparison inputs, and failure-path restore decisions.

Dry-run command:

```powershell
cargo run -p wubilex-winime --example acl_owner_spike
```

## 4. Implement The Task Scheduler Probe

- [ ] Initialize COM security and create/connect `ITaskService` locally with empty variants.
- [ ] Open `\Microsoft\Windows\TextServicesFramework\MsCtfMonitor` and snapshot enabled/state/running instances.
- [ ] Enumerate `ctfmon.exe` with ToolHelp32 without opening or terminating it.
- [ ] Require an enabled task and baseline-present `ctfmon.exe`, retain returned task-instance evidence, and distinguish task state from detached/singleton process presence.
- [ ] Arm the baseline-aware restore guard before `IRegisteredTask::Stop(0)`, then perform bounded observation of task/process state without requiring a PID change.
- [ ] Track whether recovery Run is required and implement `IRegisteredTask::Run` plus final logical-state verification on every exit path.
- [ ] Do not fall back to `schtasks`, direct process termination, service control, or a fabricated running instance.
- [ ] Add pure tests for logical-state equivalence, polling timeout, and restoration-decision logic.

Dry-run command:

```powershell
cargo run -p wubilex-winime --example task_scheduler_spike
```

## 5. Implement And Check The Virtual-Scroll Benchmark

- [ ] Render a 300,000-row virtualizer from index-derived values with a fixed row size, 640 px viewport, and overscan 12.
- [ ] Expose ready, visibility, scroll progress, DOM-row count, and benchmark result hooks to the runner.
- [ ] Implement three foreground Edge runs with the warm-up/sample protocol in `research/virtual-scroll-contracts.md`.
- [ ] Fail on page/console errors, invalid samples, blank rows, any run below 55 fps, or more than 64 rendered rows.
- [ ] Write caller-selected JSON output and always close Edge/Vite in `finally`.
- [ ] Add deterministic Vitest coverage for row derivation, metrics calculation, validity checks, and aggregate pass/fail.

Focused validation:

```powershell
pnpm run test
pnpm run spike:virtual-scroll -- --output .trellis/tasks/08-24-s0-risk-spikes/research/results/virtual-scroll.json
```

Rollback point: the browser probe has no persistent state; on failure, close the visible Edge instance and Vite server, retain the JSON/error evidence, and do not lower the threshold.

## 6. Run Controlled Live Windows Validation

Run the three commands one at a time from an elevated Windows process. Before each command, repeat its dry-run and inspect the target snapshot. Capture stdout/stderr and exit status in its result report.

```powershell
cargo run -p wubilex-winime --example tsf_profile_spike -- --live
cargo run -p wubilex-winime --example acl_owner_spike -- --live
cargo run -p wubilex-winime --example task_scheduler_spike -- --live
```

- [ ] TSF: confirm Wubi ACTIVE/current-profile state changes, ENABLED remains unchanged, and the exact original active-profile snapshot returns.
- [ ] ACL: confirm TrustedInstaller -> Administrators -> TrustedInstaller, exact normalized owner/DACL restoration, privilege restoration, and temporary-file cleanup.
- [ ] Scheduler: confirm COM End/Run, task-instance and `ctfmon.exe` timeline evidence, and final logical baseline restoration; record that an unchanged singleton PID is valid.
- [ ] After each command, run an independent read-only check before starting the next probe.
- [ ] Never stop `TabletInputService`, terminate `ChsIME.exe`, touch a real lexicon/phrase file, or read root `resource/`.

If elevation cannot be obtained in the current session, stop at this gate and report the exact commands and missing authority. Do not report a Windows spike as passed from dry-run evidence.

## 7. Persist Results And Make The Gate Decision

- [ ] Complete the four result reports and raw virtual-scroll JSON with environment, command, baseline, observations, restoration, verdict, and limitations.
- [ ] Complete `research/results/summary.md` with a pass/fail row for all four spikes and mappings to the PRD acceptance criteria.
- [ ] For every failed item, document the blocking cause, candidate alternative, and architecture-review decision without weakening the original criterion.
- [ ] Confirm no machine-specific value was added to product configuration.

## 8. Full Quality And Safety Gates

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
$env:RUSTDOCFLAGS='-D warnings'; cargo doc --workspace --all-features --no-deps --locked
cargo llvm-cov clean --workspace
cargo llvm-cov --package wubilex-codec --all-features --summary-only --fail-under-lines 90
cargo deny check
cargo xtask fixtures --check
cargo xtask bindings --check
cargo xtask check-docs
pnpm install --frozen-lockfile
pnpm audit --audit-level high
pnpm run typecheck
pnpm run lint
pnpm run test --run
python ./.trellis/scripts/task.py validate .trellis/tasks/08-24-s0-risk-spikes
git diff --check
```

- [ ] Scan the changed implementation for `takeown`, `icacls`, `schtasks`, `TabletInputService`, `ChsIME.exe`, `unwrap()`, and `expect()`; any occurrence must be either an explicit rejection/test string or removed.
- [ ] Verify default Rust examples and default frontend tests make no system changes and launch no visible browser.
- [ ] Review `Cargo.lock` and `pnpm-lock.yaml` changes and confirm dependency features remain minimal.
- [ ] Run the full-scope Trellis check before Phase 3 spec update and commit.

## Completion Gate

The task is ready to finish only when all four criteria pass, or every failed criterion has a completed architecture-review decision that explicitly blocks S1. Live restoration failures are always task failures, even when the forward API call succeeded.
