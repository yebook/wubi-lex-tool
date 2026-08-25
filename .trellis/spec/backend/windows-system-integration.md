# Windows System Integration

> Reversible TSF, security descriptor, and Task Scheduler integration contracts.

---

## Current Status

The S0 risk spikes establish the API and restoration contracts below on Windows
11. Product `SystemOps` orchestration is still pending. Keep direct Win32 and COM
calls inside `wubilex-winime`; the examples are executable evidence, not the
future product abstraction.

## Scenario: Reversible IME System Operations

### 1. Scope / Trigger

Apply this contract when code reads or changes an input processor profile, file
owner/DACL, or the `MsCtfMonitor` scheduled task. These operations can affect the
interactive Windows session, so every mutation requires an exact pre-mutation
snapshot, an armed restoration guard, explicit restoration, and final
verification.

### 2. Signatures

The S0 probes expose these exact command forms:

```text
cargo run -p wubilex-winime --example tsf_profile_spike [-- --live]
cargo run -p wubilex-winime --example acl_owner_spike [-- --live]
cargo run -p wubilex-winime --example task_scheduler_spike [-- --live]
```

No argument means read-only preflight. The only accepted mutation argument is
`--live`; missing, duplicate, or unknown arguments fail before mutation.

The validated native surfaces are:

```text
ITfInputProcessorProfileMgr::{GetProfile, GetActiveProfile,
  ActivateProfile, DeactivateProfile}
GetNamedSecurityInfoW / SetNamedSecurityInfoW
ITaskService -> ITaskFolder -> IRegisteredTask::{Stop, Run}
```

### 3. Contracts

- TSF uses `TF_IPPMF_FORSESSION` only. It never changes the ENABLED
  configuration. Input-processor identity is language + CLSID + profile GUID;
  keyboard-layout identity is language + HKL. Category, substitute HKL, and
  capability fields are evidence, not cross-type identity substitutes.
- Preserve `S_FALSE` from `GetActiveProfile`; it represents no active profile
  and must not be collapsed into the same snapshot as `S_OK`.
- ACL work accepts no caller path. Create one unique file below the probe-owned
  temp directory, then perform TrustedInstaller -> Administrators ->
  TrustedInstaller while keeping the DACL unchanged.
- ACL cleanup order is fixed: restore creation owner/DACL, restore the original
  token privilege attributes, verify both, then delete the file. If either
  restoration fails, retain the restored-or-partially-restored file as evidence
  and print its path.
- Enable only `SeTakeOwnershipPrivilege` and `SeRestorePrivilege`, and treat
  `ERROR_NOT_ALL_ASSIGNED` as failure even when `AdjustTokenPrivileges` returns
  success.
- Scheduler control uses Task Scheduler 2.0 COM only. Record enabled/state,
  running instances, and observed `ctfmon.exe` PIDs before Stop, after Stop,
  after Run, and after restoration.
- A Ready task with no running instance and an unchanged detached/singleton
  `ctfmon.exe` PID is valid. Do not require a PID change. Do not issue another
  Run while the task is queued/running or reports an existing instance because
  task policy can permit parallel instances.
- Every native failure records the stage, HRESULT or Win32 code, and readable
  system text. Forward success plus failed restoration is an overall failure.
- Do not call `takeown`, `icacls`, or `schtasks`; do not stop
  `TabletInputService`; do not terminate `ChsIME.exe` or `ctfmon.exe`.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Default invocation | Print baseline and plan; make no system change |
| Unknown or repeated argument | Exit nonzero before COM, privilege, file, or task mutation |
| TSF profile is disabled or active language is unsafe | Fail preflight; do not activate/deactivate |
| TSF transition succeeds but identity/ENABLED restore differs | Fail visibly with baseline and observed snapshots |
| Token privilege is not assigned | Fail before owner mutation and restore any changed token attributes |
| ACL owner or normalized DACL differs after round trip | Restore creation baseline, retain evidence on restore failure, and fail |
| Scheduler is disabled or baseline `ctfmon.exe` is absent | Fail preflight; do not call Stop |
| Task is queued/running or has an instance during recovery | Do not issue a parallel Run; poll and verify logical state |
| Stop/Run succeeds but final logical baseline differs | Fail with task/instance/process timeline and recovery guidance |

### 5. Good / Base / Bad Cases

- Good: a live operation changes the intended state, restores the exact logical
  baseline, and a separate read-only invocation confirms restoration.
- Base: the default command prints target identity and planned operations and
  exits without requesting elevation or changing state.
- Bad: code treats any Chinese profile as the captured TSF identity, deletes an
  ACL probe file before privilege restoration, requires `ctfmon.exe` to get a
  new PID, or starts another Scheduler instance while one is already pending.

### 6. Tests Required

- Pure tests cover exact CLI parsing, bounded polling success/timeout, operation
  selection, restoration decisions, and combined primary/restore failures.
- TSF tests cover input-processor versus keyboard-layout identity, `S_FALSE`,
  unknown profile types, ENABLED immutability, and exact restore comparison.
- ACL tests cover owner/DACL semantic comparison, create-new ownership, restore
  ordering, evidence retention, and deletion only after ACL and privilege
  restoration both succeed.
- Scheduler tests cover logical Ready/Running equivalence, detached singleton
  PIDs, Run suppression for queued/running/existing instances, and timeout.
- CI and default tests remain dry-run or pure. Live tests run only in an
  explicitly authorized isolated Windows session and require an independent
  read-only post-check.

### 7. Wrong vs Correct

```rust
// Wrong: deletes evidence before every reversible resource is restored.
restore_file_acl()?;
delete_probe_file()?;
restore_token_privileges()?;

// Correct: restore in dependency order and delete only after verification.
restore_file_acl()?;
restore_token_privileges()?;
verify_restoration()?;
delete_probe_file()?;
```

## Sources

- [`wubilex-winime` risk-spike examples](../../../crates/wubilex-winime/examples/)
- [`S0 risk-spike design`](../../tasks/08-24-s0-risk-spikes/design.md)
- [`S0 risk-spike results`](../../tasks/08-24-s0-risk-spikes/research/results/summary.md)

