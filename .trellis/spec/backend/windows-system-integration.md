# Windows System Integration

> Reversible TSF, security descriptor, Task Scheduler, and forbidden companion-tool integration contracts.

---

## Current Status

The S0 risk spikes establish the API and restoration contracts below on Windows
11. S1 adds current-process token elevation detection and the application config
file adapter. Product `SystemOps` orchestration is still pending. Keep direct
Win32 and COM calls inside `wubilex-winime`; the examples are executable
evidence, not the future product abstraction.

## Scenario: Current Process Elevation Detection

### 1. Scope / Trigger

Apply this contract when startup, feature availability, or diagnostics need to
know whether the current process token is actually elevated. An executable
manifest requests elevation but is not runtime proof that elevation succeeded.

### 2. Signatures

```rust
pub trait ElevationProbe {
    fn is_elevated(&self) -> Result<bool, NativeSecurityError>;
}

pub fn current_process_is_elevated() -> Result<bool, NativeSecurityError>;
```

### 3. Contracts

- `crates/wubilex-winime/src/security.rs` exclusively owns
  `OpenProcessToken` and `GetTokenInformation(TokenElevation)`. Tauri consumes
  the typed result and does not duplicate token inspection.
- The probe opens only the current process token with `TOKEN_QUERY`, validates
  the returned `TOKEN_ELEVATION` byte count, and closes the token through an
  ownership guard.
- A native failure preserves a stable operation stage, unsigned HRESULT/Win32
  code, and readable system message. The runtime projection exposes bounded
  stage/code evidence without assuming elevation.
- `Ok(false)` is a real non-elevated state, not an unavailable probe. The UI
  presents an administrator restart action and does not perform system writes.
- Non-Windows builds return a typed unavailable failure so mock bindings and
  cross-platform library checks stay deterministic.

### 4. Tests Required

- Adapter tests cover elevated, non-elevated, and typed native failures.
- Runtime projection tests preserve the three privilege states and serialized
  stage/code contract.
- The checked-in application manifest independently contains both Common
  Controls v6 and `requireAdministrator` with `uiAccess="false"`.

### 5. Wrong vs Correct

```rust
// Wrong: assumes the manifest proves the current token state.
let elevated = true;

// Correct: consume the Windows-owned typed probe at the application boundary.
let privilege = PrivilegeStatus::from_probe(current_process_is_elevated());
```

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

## Scenario: Transactional Application Configuration Files

### 1. Scope / Trigger

Apply this contract to versioned application TOML creation, replacement,
backup recovery, corrupt preservation, import, and export. It is a file
transaction contract, not permission for S2 data writes or S3 system changes.

### 2. Signatures

```rust
pub fn create_staging_exclusive(path: &Path) -> Result<File, NativeFileError>;
pub fn install_new_noreplace(staging: &Path, target: &Path) -> Result<(), NativeFileError>;
pub fn replace_file_with_backup(
    target: &Path,
    staging: &Path,
    backup: &Path,
) -> Result<(), ReplaceFileError>;
pub fn restore_backup_noreplace(backup: &Path, target: &Path) -> Result<(), NativeFileError>;
```

The app-layer `ConfigFileOps` port exposes directory creation, bounded reads,
backup listing, exclusive staging, sync, close, replace/install/restore,
corrupt preservation, canonical path identity, and owned cleanup as separate
injectable stages.

### 3. Contracts

- `crates/wubilex-winime/src/filesystem.rs` exclusively owns direct Win32 file APIs; `src-tauri/src/config/` owns schema and recovery policy.
- Staging is same-directory and `create_new`; on Windows the open handle uses share mode zero. Ownership starts only after creation succeeds.
- Write all bytes, flush, `sync_all`, and close before namespace installation.
- Initial installation uses `MoveFileExW(MOVEFILE_WRITE_THROUGH)` without replace/copy flags. Existing replacement uses `ReplaceFileW` with flags zero and a unique nonexistent backup.
- `REPLACEFILE_WRITE_THROUGH`, delete-live-then-rename, and adoption or cleanup of a failed create are forbidden.
- Native 1177 means the old target may be at backup. Attempt no-clobber restore; if it fails, retain backup, leave memory/revision uncommitted, enter read-only mode, and expose combined evidence.
- When live is missing, backup enumeration or read failure is a read-only startup failure. Never install defaults after losing visibility into a possible last-valid owned backup.
- Canonicalize existing external paths and their parents before alias checks so dot segments and symlinks cannot target config-owned live/temp/backup/corrupt artifacts.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Staging name collision | Retry finitely; never delete collision bytes |
| Initial target appears concurrently | No-clobber install fails; external target wins |
| Write/flush/sync/close failure | Close then remove only owned staging; keep live and revision |
| Replace error 1175/1176 | Names unchanged; clean owned staging |
| Replace error 1177, restore succeeds | Old target restored; clean staging; return primary replace failure |
| Replace error 1177, restore fails | Keep backup and combined evidence; enter read-only mode |
| Backup listing/read fails while live is missing | Do not create live defaults; use read-only defaults with notice |
| Future schema | Preserve live bytes in place; routine writes fail read-only |

### 5. Good / Base / Bad Cases

- Good: replacement succeeds with exact new live bytes and exact previous bytes in a unique backup.
- Base: first run installs canonical defaults without a backup and starts at revision one.
- Bad: swallowing backup enumeration failure, deleting live before rename, passing an unsupported replace flag, or cleaning a path whose create failed.

### 6. Tests Required

- Windows integration tests assert real no-clobber initial install and `ReplaceFileW` backup bytes.
- Fault injection covers create, write, flush, sync, close, target inspection, backup selection, install/replace, 1177 restore, and cleanup ordering.
- A stateful 1177 fake must move old target to backup before returning the failure; assertions inspect live/backup/staging bytes, snapshot, revision, persistence, error, and notice.
- Startup tests cover missing, valid, corrupt, future, newest valid backup, backup listing failure, backup read failure, and corrupt-preservation failure.
- Import/export tests reject owned aliases and prove failed operations do not mutate the authoritative snapshot.

### 7. Wrong vs Correct

```rust
// Wrong: destroys the live-name recovery boundary.
std::fs::remove_file(target)?;
std::fs::rename(staging, target)?;

// Correct: one supported replacement call with an owned backup.
replace_file_with_backup(target, staging, unique_backup)?;
```

## Scenario: ImTip Integration Is Forbidden

### 1. Scope / Trigger

Apply this contract whenever shell navigation, actions, tray menus, settings,
deep links, external-process discovery, URL opening, Tauri commands,
capabilities, dependencies, or migration work could introduce an ImTip entry or
integration. The user permanently removed this product surface on 2026-08-25;
it is not a deferred feature.

### 2. Signatures

There is deliberately no command, event, route, action, configuration key,
capability, dependency, process probe, executable launch, or URL contract for
ImTip. `M7-WIN-005` remains only as a deprecated P3 requirement ID and must not
be registered in generated bindings.

### 3. Contracts

- Production roots `src/`, `src-tauri/`, `crates/`, manifests, capabilities,
  route tables, action catalogs and tray projections contain no ImTip surface.
- No feature flag may make the integration available; permanently excluded
  behavior is not represented as a disabled product feature.
- The legacy `wubi-lex/` snapshot and archived Trellis tasks may retain the
  original name solely as immutable historical evidence. They are never copied
  into product code, current planning, UI text, telemetry, links, or metadata.
- No generic “related tools” abstraction may be introduced solely to preserve
  the removed behavior under another label.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| A task proposes an ImTip route, action, button, tray item or settings entry | Reject as out of scope and cite deprecated `M7-WIN-005` |
| Code proposes process/atom lookup, executable launch or website fallback | Reject; remove the integration code and any supporting capability/dependency |
| A feature catalog contains the removed integration as unavailable | Remove the feature ID entirely; do not expose a permanent placeholder |
| Historical source or archived task mentions the original integration | Treat as evidence only; do not use it as implementation input |
| Requirement tooling encounters `M7-WIN-005` | Accept only its P3 deprecated definition and references that state it is forbidden |

### 5. Good / Base / Bad Cases

- Good: the seven-domain shell, action catalog, settings and tray contain no
  entry, and focused searches over production roots return no integration
  identifier, URL or executable name.
- Base: the legacy snapshot retains its original source for traceability while
  current requirements classify the ID as P3 and S1 explicitly skips it.
- Bad: hiding the entry behind a feature flag, renaming it “related tools”, or
  keeping an unused launcher command “for later”.

### 6. Tests Required

- Search production roots and manifests for case-insensitive `imtip`; expect no
  matches.
- Assert route, action, tray and feature snapshots contain no removed entry.
- Run document count, dangling-reference and anchor checks after changing the
  deprecated requirement.
- Review new external-process or URL-opening capabilities against this scenario
  so generic infrastructure cannot reintroduce the integration indirectly.

### 7. Wrong vs Correct

```rust
// Wrong: keeps a hidden integration path for a future release.
registry.register("related_tool.open", open_imtip);

// Correct: no action, command, capability, dependency, URL, or placeholder exists.
let registry = shell_actions_without_companion_integrations();
```

## Sources

- [Current-process elevation probe](../../../crates/wubilex-winime/src/security.rs)
- [Runtime privilege projection](../../../src-tauri/src/runtime/mod.rs)
- [`wubilex-winime` risk-spike examples](../../../crates/wubilex-winime/examples/)
- [Atomic file adapter](../../../crates/wubilex-winime/src/filesystem.rs)
- [Application config transaction service](../../../src-tauri/src/config/mod.rs)
- [`S0 risk-spike design`](../../tasks/archive/2026-08/08-24-s0-risk-spikes/design.md)
- [`S0 risk-spike results`](../../tasks/archive/2026-08/08-24-s0-risk-spikes/research/results/summary.md)
