# S0 Risk Spikes Design

## 1. Design Goal

Build four disposable but reproducible probes that answer the S1 entry questions with observable evidence. The implementation must remain smaller than the future product architecture: it validates APIs and performance without implementing `SystemOps`, the complete TSF shutdown window, the S2 virtual table, or real lexicon writes.

The fundamental safety invariant is: every live probe knows the original state before its first mutation, arms restoration before proceeding, explicitly restores and verifies on the normal path, and retains a best-effort `Drop` fallback for unwinding.

## 2. Placement And Dependencies

### Windows probes

Place the three executables under `crates/wubilex-winime/examples/`:

- `tsf_profile_spike.rs`
- `acl_owner_spike.rs`
- `task_scheduler_spike.rs`
- `support/` for genuinely shared COM lifetime, native error formatting, elevation checks, polling, and handle ownership

This keeps direct Win32/COM calls inside the repository's sole Windows integration crate while avoiding premature product modules under `src/`.

Add `windows = "0.61.3"` as a Windows-target dependency with only the explicit leaf features actually required by the final code. The expected union is:

- `Win32_Security_Authorization`
- `Win32_System_Com`
- `Win32_System_Diagnostics_ToolHelp`
- `Win32_System_Ole`
- `Win32_System_TaskScheduler`
- `Win32_System_Threading`
- `Win32_System_Variant`
- `Win32_UI_Input_KeyboardAndMouse`
- `Win32_UI_TextServices`

The implementation must prune any feature not referenced after compilation. Non-Windows builds retain a small unsupported-platform entry point, so workspace `--all-targets` checks remain portable.

### Virtual-scroll probe

Place an isolated Vite harness under `spikes/virtual-scroll/`, not under the reserved production `src/components/virtual-table/` directory. Add exact development dependencies `@tanstack/react-virtual@3.14.10` and `playwright-core@1.62.1`; use the existing React, React DOM, Vite, TypeScript, ESLint, and Vitest toolchain.

Add explicit root scripts for the spike runner and its focused deterministic tests. Dependency installation always uses the global `pnpm` command.

## 3. Common Probe Contract

Every Windows executable accepts no mutation by default. Its default invocation:

1. verifies platform, COM/API availability, target identity, and current state;
2. prints the exact live operations it would attempt;
3. exits without changing system state.

`--live` is the only mutation switch. Unknown flags fail. Live mode checks elevation before mutation and emits stage-labelled evidence. Native failures retain the operation stage plus HRESULT or Win32 code and readable system text.

Each live probe uses a two-layer restoration strategy:

1. `restore()` runs explicitly before returning, reports errors, and verifies the final snapshot.
2. `Drop` retries best-effort restoration when unwinding or when explicit restoration was not completed. A Drop failure is printed prominently to stderr with the target and a manual recovery instruction; it is never silently treated as success.

The probe reports failure when forward work succeeds but restoration or final verification fails. Default tests and CI invoke only pure tests, compilation, and dry-run paths.

## 4. TSF Profile Probe

### Identity

- language ID: `0x0804`
- Wubi CLSID: `{6A498709-E00B-4C45-A018-8F9E4081AE40}`
- Wubi Profile GUID: `{82590C13-F4DD-44F4-BA1D-8667246FDF8E}`
- profile type: `TF_PROFILETYPE_INPUTPROCESSOR`
- category: `GUID_TFCAT_TIP_KEYBOARD`
- live scope: `TF_IPPMF_FORSESSION`

Create `ITfInputProcessorProfileMgr` from `CLSID_TF_InputProcessorProfiles` in a COM apartment. Snapshot Wubi using `GetProfile` and the active keyboard profile using `GetActiveProfile`. Use `HKL(0)` for Wubi because it is an input-processor profile. Preserve the success HRESULT from `GetActiveProfile` so `S_FALSE` can be distinguished from `S_OK`; the generated convenience wrapper otherwise collapses both to `Ok(())`.

Live preflight requires Wubi to have `TF_IPP_FLAG_ENABLED` and the current active keyboard profile language to be `0x0804`; otherwise it asks the operator to select a Simplified Chinese input method and exits before mutation. Use only `TF_IPPMF_FORSESSION`, never the registry-writing ENABLEPROFILE/DISABLEPROFILE flags.

If Wubi starts active, call `DeactivateProfile` and poll until Wubi is no longer active. If Wubi starts inactive, call `ActivateProfile`, poll until Wubi is active, call `DeactivateProfile`, and poll until it is inactive again. Re-query both `GetProfile(Wubi)` and `GetActiveProfile(GUID_TFCAT_TIP_KEYBOARD)` at every transition; ACTIVE/current-profile change is the primary success evidence, while ENABLED must remain unchanged.

Restoration activates the exact captured original profile, handling both input-processor and keyboard-layout profile shapes, or restores "no active profile" when the baseline was `S_FALSE`. It then compares Wubi's enabled/active bits and the active profile identity with the baseline. The guard is armed before the first activation/deactivation call.

The probe never changes a default-input-method registry key and never reads or writes a lexicon file.

## 5. ACL Ownership Probe

Live mode creates one uniquely named file below a dedicated directory in `std::env::temp_dir()`. No caller-selected or repository path is accepted, which makes it impossible for this probe to target a real lexicon or user file.

Resolve TrustedInstaller with the two-call `LookupAccountNameW` buffer pattern and Administrators with `CreateWellKnownSid(WinBuiltinAdministratorsSid)`, avoiding localized account-name assumptions. Enable only `SeTakeOwnershipPrivilege` and `SeRestorePrivilege`, preserving each token privilege's original attributes for restoration. `AdjustTokenPrivileges` success must also be checked for `ERROR_NOT_ALL_ASSIGNED`.

The ownership sequence uses two baselines:

1. create the temporary file and capture cleanup baseline A (original owner and DACL), arming cleanup before any ownership mutation;
2. set its owner to TrustedInstaller using `SetNamedSecurityInfoW`;
3. snapshot verification baseline B (TrustedInstaller owner and unchanged DACL) using `GetNamedSecurityInfoW`;
4. set only the owner to Administrators;
5. query and verify the Administrators owner;
6. restore only the owner to the TrustedInstaller SID saved in baseline B;
7. query owner and DACL again and compare them with baseline B, recording the required round-trip result;
8. restore cleanup baseline A's owner and DACL, restore token privileges, verify cleanup state, and only then delete the temporary file.

Buffers returned by `GetNamedSecurityInfoW` and SDDL conversion are owned by RAII wrappers and released with `LocalFree`; caller-owned lookup buffers remain alive while their `PSID` is borrowed. SID comparison uses `EqualSid`. Owner/DACL semantic evidence is normalized through `ConvertSecurityDescriptorToStringSecurityDescriptorW` with owner and DACL information; transitions never rewrite the DACL. Exact normalized owner/DACL equality is therefore required after the round trip.

If restoration fails, retain the temporary path for diagnosis and print its path, observed owner/DACL, and recovery failure. Do not delete evidence and claim cleanup.

## 6. Task Scheduler Probe

The standalone runner owns its COM setup: initialize the apartment, call `CoInitializeSecurity` once before creating proxies, create `ITaskService` from the Task Scheduler 2.0 `TaskScheduler` coclass, connect locally with empty `VARIANT` credentials, open `\Microsoft\Windows\TextServicesFramework`, and obtain `MsCtfMonitor` through `ITaskFolder::GetTask`. An unexpected `RPC_E_TOO_LATE` is a visible precondition failure in this standalone process, not silent success.

The baseline contains:

- task enabled flag and `TASK_STATE`;
- running task instance identities;
- the set of `ctfmon.exe` PIDs from ToolHelp32;
- the observation timestamp.

Live mode requires an enabled task and baseline-present `ctfmon.exe`; an absent process would not be safely restorable without a forbidden termination fallback. It invokes `IRegisteredTask::Stop(0)` as the End operation, polls task instances/state and `ctfmon.exe`, then invokes `IRegisteredTask::Run` with an empty `VARIANT` as the Run operation. It retains the returned `IRunningTask` long enough to record instance identity/state. It does not call `schtasks`, terminate `ctfmon.exe`, terminate `ChsIME.exe`, or touch `TabletInputService`.

The pass condition requires the COM End and Run calls to succeed, task instance/state evidence from the returned interfaces, and the final logical state to match the baseline: enabled remains unchanged, task returns to the equivalent Ready/Running condition, and baseline-present `ctfmon.exe` is present again. A Ready task can own no running instance while a detached or singleton `ctfmon.exe` remains resident, so a process exit or PID change is recorded when present but is not required.

Before the End attempt, the restoration guard is armed with the complete baseline. It tracks whether Stop was attempted and whether subsequent observations diverged; on every exit path it calls Run when recovery is required, then verifies the logical baseline. If COM End/Run fails or final task/process presence cannot be restored, the spike fails and records the evidence; it must not manufacture a passing instance or use process termination as a fallback. The report explicitly states that Scheduler COM does not prove or replace the future shutdown window's separate process-control step.

## 7. Virtual-Scroll Probe

The harness exposes a fixed 300,000-row count to TanStack Virtual and computes visible row labels directly from each virtual index. It never constructs a full array of row objects.

The visible Edge runner owns the benchmark lifecycle and follows `research/virtual-scroll-contracts.md`: fixed viewport/row/overscan dimensions, one-second warm-up, five-second sample, three runs, foreground visibility checks, and a maximum of 64 rendered row nodes. Every valid run must reach at least 55 fps.

The runner writes structured JSON only to the path supplied by `--output`, closes Edge and the Vite server in `finally`, and exits nonzero on an invalid or failing run. The live performance command is not added to CI.

## 8. Evidence And Decision Output

Persist one result report per spike:

- `research/results/tsf-profile.md`
- `research/results/acl-owner.md`
- `research/results/task-scheduler.md`
- `research/results/virtual-scroll.md` plus raw `virtual-scroll.json`

Each report records environment, exact command, preconditions, baseline, intermediate observations, final state, restoration/cleanup, pass/fail, and limitations. `research/results/summary.md` maps the four outcomes back to `SPIKE-R01..R10` and the roadmap gate.

Any failed spike keeps its original threshold and adds candidate alternatives plus an explicit architecture-review outcome before S1. Machine-specific PIDs, SIDs, paths, timestamps, and measurements stay in task research and never become product configuration.

## 9. Compatibility And Rollback

- Windows code is guarded by `cfg(windows)` and validated on the repository's pinned MSVC toolchain.
- The Edge measurement is a local foreground benchmark, not a deterministic CI gate.
- Product source directories, Tauri commands, bindings, real resource files, services, and release configuration remain untouched.
- Dependency or harness rollback consists of removing the isolated examples/harness, their scripts, and their direct dependencies; no migration or persistent product state is introduced.
- A live probe's operational rollback is its state-specific restoration guard and independent post-run read-only verification.
