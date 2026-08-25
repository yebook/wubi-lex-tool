# Research: Windows API Contracts for the S0 Live Risk Spikes

- Query: Establish the exact `windows 0.61.3` contracts, state snapshots, verification rules, restoration rules, memory ownership, and Cargo features for the TSF Profile, Task Scheduler, and temporary-file ACL live spikes.
- Scope: mixed (repository requirements, local generated Rust bindings, and Microsoft Learn API contracts)
- Date: 2026-08-24

## Findings

### Cross-cutting COM and safety contract

- Keep the three spikes independently runnable. Their default path must stop after read-only preflight; only an explicit live flag may arm a restoration guard and mutate state (`SPIKE-R01`, `SPIKE-R05`).
- Run COM work on one thread. `CoInitializeEx(None, COINIT_APARTMENTTHREADED)` is a suitable standalone-runner choice. Both `S_OK` and `S_FALSE` require one matching `CoUninitialize`; `RPC_E_CHANGED_MODE` does not. Drop every COM interface before the apartment guard calls `CoUninitialize`.
- `CoCreateInstance` in this crate is generic over the requested interface. The two required constructions are:
  - `CLSID_TF_InputProcessorProfiles` -> `ITfInputProcessorProfileMgr`
  - `TaskScheduler` -> `ITaskService`
  Both use `CLSCTX_INPROC_SERVER`.
- A standalone Task Scheduler runner that owns process initialization should establish COM security once, before obtaining Task Scheduler proxies, following Microsoft's native Task Scheduler examples. If this eventually runs inside an existing host, that host must own the process-global `CoInitializeSecurity` policy; `RPC_E_TOO_LATE` must not be silently treated as success without proving the existing policy is sufficient.
- Every mutation must have a pre-mutation snapshot and an armed restoration guard. A primary failure plus a restoration failure must preserve both errors, with the restoration failure made more visible because it means the machine may not match the baseline.

Binding evidence:

- `E:/env/rust/cargo/registry/src/rsproxy.cn-e3de039b2554c837/windows-0.61.3/src/Windows/Win32/System/Com/mod.rs:117-124` gives the generic `CoCreateInstance` signature.
- The same file at `:334-336`, `:572-574`, and `:1714` gives `CoInitializeEx`, `CoUninitialize`, and `CLSCTX_INPROC_SERVER`.

### 1. TSF Profile spike

#### Fixed identity and binding surface

- Microsoft Wubi is an input-processor profile:
  - `langid = 0x0804`
  - CLSID `{6A498709-E00B-4C45-A018-8F9E4081AE40}`
  - Profile GUID `{82590C13-F4DD-44F4-BA1D-8667246FDF8E}`
  - `dwProfileType = TF_PROFILETYPE_INPUTPROCESSOR`
  - `hkl = HKL(0)` because the Microsoft contract requires a null HKL for an input-processor profile.
- The COM coclass is `CLSID_TF_InputProcessorProfiles = {33C53A50-F456-4884-B049-85FD643ECFED}`. Request `ITfInputProcessorProfileMgr` (IID `{71C6E74C-0F28-11D8-A82A-00065B84435C}`).
- `ActivateProfile`, `DeactivateProfile`, and `GetProfile` all take the profile type, language, CLSID pointer, profile GUID pointer, HKL, and (except `GetProfile`) flags. `GetActiveProfile` takes only a category GUID plus an output structure.
- `GetActiveProfile` supports only `GUID_TFCAT_TIP_KEYBOARD`; it returns `S_FALSE` when no matching active profile exists. Use it together with `GetProfile`, not as a replacement for the target profile snapshot.
- `TF_INPUTPROCESSORPROFILE.dwFlags` contains `TF_IPP_FLAG_ACTIVE` and `TF_IPP_FLAG_ENABLED`; record both. Also record the returned type, language, CLSID, profile GUID, category, HKL values, and capabilities so verification does not depend on one Boolean.

Generated-binding evidence:

- Text Services coclass and category: `.../Win32/UI/TextServices/mod.rs:62`, `:179`.
- Manager IID and methods: `.../Win32/UI/TextServices/mod.rs:8378-8411`.
- Profile structure and flags: `.../Win32/UI/TextServices/mod.rs:14716-14744`.
- `TF_PROFILETYPE_INPUTPROCESSOR`: `.../Win32/UI/TextServices/mod.rs:14872`.

#### Flag choice

- `TF_IPPMF_FORSESSION` is decimal `536870912`, exactly `0x20000000`; it applies the operation to all threads in the current desktop.
- Use only `TF_IPPMF_FORSESSION` for this spike. Do not combine `TF_IPPMF_ENABLEPROFILE` or `TF_IPPMF_DISABLEPROFILE`: those flags update the user's enabled-profile registry state and therefore broaden the mutation from a temporary activation test into a language-list/configuration change.
- Do not use `TF_IPPMF_FORPROCESS`, because it would not prove the desktop/session behavior required by the original `0x20000000` call.
- Do not rely on `TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE` for immediate verification. Microsoft documents that a language mismatch can defer activation until that input language is selected. The live preflight should instead require the current active keyboard category profile to have `langid == 0x0804`; otherwise report a visible precondition telling the operator to select a Simplified Chinese input method and retry.
- Preflight must also require the Wubi `GetProfile` result to contain `TF_IPP_FLAG_ENABLED`. `ActivateProfile` can return `S_FALSE` for a disabled profile. The generated `windows` wrapper returns `Result<()>` and uses `HRESULT::ok()`, so `S_FALSE` is not distinguishable from `S_OK` at the Rust return type. Post-call state verification is therefore mandatory.

Generated constants are at `.../Win32/UI/TextServices/mod.rs:14729-14744`:

- `TF_IPPMF_ENABLEPROFILE = 1`
- `TF_IPPMF_DISABLEPROFILE = 2`
- `TF_IPPMF_DONTCARECURRENTINPUTLANGUAGE = 4`
- `TF_IPPMF_FORPROCESS = 0x10000000`
- `TF_IPPMF_FORSESSION = 0x20000000`
- `TF_IPPMF_FORSYSTEMALL = 0x40000000`

#### Snapshot, exercise, verification, and restoration

Take both snapshots before arming live mutation:

1. `GetProfile` for the fixed Wubi identity. This proves the target exists and captures its enabled/active bits.
2. `GetActiveProfile(GUID_TFCAT_TIP_KEYBOARD)`. This captures the exact profile that must be active after restoration, even when it is not Wubi.

Exercise according to the initial state:

- If Wubi starts active: call `DeactivateProfile(..., TF_IPPMF_FORSESSION)`, verify it is no longer active, then restore by activating the captured Wubi profile.
- If Wubi starts inactive: call `ActivateProfile(..., TF_IPPMF_FORSESSION)`, verify Wubi becomes active, call `DeactivateProfile(..., TF_IPPMF_FORSESSION)`, verify it becomes inactive again, then explicitly reactivate the original category snapshot.

Each verification must re-query both surfaces:

- `GetProfile(Wubi).dwFlags & TF_IPP_FLAG_ACTIVE`
- `GetActiveProfile(GUID_TFCAT_TIP_KEYBOARD)` identity (`dwProfileType`, `langid`, CLSID/profile GUID or HKL)

The restoration helper must accept the captured `TF_INPUTPROCESSORPROFILE`, not hard-code Wubi. For `TF_PROFILETYPE_INPUTPROCESSOR`, pass its CLSID/profile GUID and null HKL. For `TF_PROFILETYPE_KEYBOARDLAYOUT`, pass null GUID pointers and its HKL. After restoring, assert the active identity equals the snapshot and that Wubi's `TF_IPP_FLAG_ENABLED` bit equals its preflight value. Because no enable/disable flags are used, the language-profile registry state should remain unchanged.

Critical restore rule: if the original active-profile query returned `S_FALSE`, restoring “no active profile” means deactivating the temporary Wubi activation and verifying `GetActiveProfile` again returns no active profile. Do not invent a fallback profile.

### 2. Task Scheduler 2.0 / `MsCtfMonitor` spike

#### Exact COM call chain

1. Create `ITaskService` from `TaskScheduler = {0F87369F-A4E5-4CFC-BD3E-73E6154572DD}` with `CoCreateInstance(..., CLSCTX_INPROC_SERVER)`.
2. Create one `VARIANT::default()` and pass it for all four `ITaskService::Connect` arguments. A zeroed `VARIANT` has `vt = VT_EMPTY`; Microsoft documents empty server/user/domain/password as a local connection using the current token.
3. Call `Connect` before all other service methods.
4. Call `GetFolder(BSTR::from("\\Microsoft\\Windows\\TextServicesFramework"))`.
5. Call `GetTask(BSTR::from("MsCtfMonitor"))`.
6. End all running instances with `IRegisteredTask::Stop(0)`. The flags parameter is reserved and must be zero.
7. Start the task with `IRegisteredTask::Run(&empty_variant)`. This is equivalent to `RunEx` with flags zero and no user; retain the returned `IRunningTask` long enough to capture its instance GUID/state when possible.

Binding evidence:

- `ITaskService`, `GetFolder`, and `Connect`: `.../Win32/System/TaskScheduler/mod.rs:5497-5529`.
- `ITaskFolder::GetTask`: `.../Win32/System/TaskScheduler/mod.rs:4577-4626`.
- `IRegisteredTask::{State, Run, GetInstances, Stop}`: `.../Win32/System/TaskScheduler/mod.rs:2196-2301`.
- `IRunningTask::{State, InstanceGuid, EnginePID}`: `.../Win32/System/TaskScheduler/mod.rs:3109-3158`.
- Task states and coclass: `.../Win32/System/TaskScheduler/mod.rs:7290-7295`, `:7370`.
- `VARIANT::default()` is zeroed and `VT_EMPTY == 0`: `.../Win32/System/Variant/mod.rs:742-749`, `:924`.

`Connect` and `Run` are each compiled only under `all(Win32_System_Ole, Win32_System_Variant)` in addition to the interface's `Win32_System_Com` gate (`TaskScheduler/mod.rs:2235-2246`, `:5527-5529`). This is why all three feature families are required.

`windows_core::BSTR::from(&str)` owns a `SysAllocStringLen` allocation and its `Drop` calls `SysFreeString`; no manual BSTR free is needed (`windows-strings-0.4.2/src/bstr.rs:77-80`, `:160-164`). The generated Win32 `VARIANT` does not provide a Rust `Drop`; the spike uses only `VT_EMPTY`, which owns nothing. Any later nonempty VARIANT construction must be paired with `VariantClear` (`.../Win32/System/Variant/mod.rs:214-216`).

#### State and process evidence

Snapshot before `Stop(0)`:

- `IRegisteredTask::Enabled`
- `IRegisteredTask::State`
- `IRegisteredTask::GetInstances(0).Count()`
- a ToolHelp snapshot of all `ctfmon.exe` process IDs

Use `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)`, initialize `PROCESSENTRY32W.dwSize`, iterate `Process32FirstW` / `Process32NextW`, and close the snapshot handle on every path. Treat the expected final `ERROR_NO_MORE_FILES` from iteration as completion, not a failure. Do not open or terminate `ctfmon.exe`; this spike observes only name and PID.

Generated process-binding evidence:

- `CreateToolhelp32Snapshot`, `Process32FirstW`, and `Process32NextW`: `.../Win32/System/Diagnostics/ToolHelp/mod.rs:2`, `:53`, `:63`.
- `TH32CS_SNAPPROCESS`: `.../Win32/System/Diagnostics/ToolHelp/mod.rs:229`.

Poll with a fixed bounded deadline (for example, 100 ms intervals up to 5 seconds) and record every observed tuple of task state, instance count, and `ctfmon.exe` PIDs. Do not use an unbounded sleep or a single instantaneous read.

Important interpretation: a baseline of `TASK_STATE_READY` with an existing `ctfmon.exe` is valid and was observed in read-only preflight. It means the scheduled action may already have completed or detached while the monitor process remains. Therefore:

- `Stop(0)` success proves the Task Scheduler End call was accepted and stops registered running task instances; it does not prove that a detached/already-running `ctfmon.exe` must exit.
- `Run` success must produce an `IRunningTask`; the task can transition through queued/running too quickly to catch and return to Ready. Record the returned instance GUID and the polling timeline.
- After `Run`, require at least one `ctfmon.exe` within the deadline when the baseline contained one. A singleton can retain the same PID, so a new PID is not required.
- `IRunningTask::EnginePID` is the task engine's PID, not a guaranteed `ctfmon.exe` child PID. Never equate the two.

This distinction is part of the spike result: Task Scheduler COM replaces `schtasks /End` and `/Run`, but it does not remove the separate process-control step from the future full shutdown-window design.

#### Restoration

- Never change the registered task's `Enabled` property.
- The restoration guard uses semantic baseline state, not exact process identity. If the baseline had a running task instance or `ctfmon.exe` present and the exercise removed it, call `Run(VT_EMPTY)` and poll until the equivalent presence/state returns.
- If the baseline was Ready with `ctfmon.exe` present, restoration succeeds when the task returns to Ready, no unexpected running instances remain, and at least one `ctfmon.exe` is present. The PID is allowed to differ.
- If preflight finds the task disabled, absent, or `ctfmon.exe` absent, abort live execution. Restoring a previously absent process would require termination if `Run` leaves it present, which is outside this spike's approved mutation boundary.
- Access-denied HRESULTs from `Connect`, `Stop`, or `Run` are evidence of an elevation/ACL precondition failure, not a reason to fall back to `schtasks.exe`.

### 3. Temporary-file owner round trip

#### Two baselines are required

A newly created temporary file is normally owned by the creating user, so “final owner/DACL equals baseline” cannot use the state immediately after creation while also ending the measured round trip at TrustedInstaller. Use two snapshots:

1. **Cleanup baseline A:** capture the task-created file's initial owner and DACL immediately after creation. This is the state restored before deleting the file.
2. Enable `SeRestorePrivilege`, set owner to TrustedInstaller, then capture **verification baseline B** (owner = TrustedInstaller plus unchanged DACL).
3. Enable `SeTakeOwnershipPrivilege`, set owner to BUILTIN\\Administrators, and verify it.
4. Enable/use `SeRestorePrivilege`, set owner back to the owner SID captured in baseline B, and verify final owner/DACL semantics equal baseline B.
5. After recording the pass/fail evidence, restore baseline A and delete the temporary file.

Only the task-created path may enter this flow. Resolve it to an absolute path, prove it is inside the spike-owned temporary directory, and refuse any real lexicon, phrase, or user path.

#### SID resolution

- Resolve TrustedInstaller with the two-call `LookupAccountNameW(None, "NT SERVICE\\TrustedInstaller", ...)` pattern. The first call uses null buffers and zero sizes and must fail with `ERROR_INSUFFICIENT_BUFFER` while returning SID bytes and referenced-domain UTF-16 character count (including the terminator). Allocate an aligned SID buffer, allocate the domain buffer, retry, require a valid SID, and record the returned `SID_NAME_USE`.
- Resolve BUILTIN\\Administrators without a localized account name. Preferred choices are `CreateWellKnownSid(WinBuiltinAdministratorsSid, ...)` or `ConvertStringSidToSidW("S-1-5-32-544", ...)`. If `ConvertStringSidToSidW` is used, its output is LocalAlloc-owned and must be freed with `LocalFree`.
- Convert resolved SIDs back to SDDL strings for evidence. `ConvertSidToStringSidW` also returns a LocalAlloc-owned `PWSTR`; copy it to a Rust string before `LocalFree`.
- `LookupAccountNameW` writes into caller-owned buffers and those buffers must outlive every `PSID` borrowed from them. They are not passed to `LocalFree`.

Binding evidence:

- `LookupAccountNameW`: `.../Win32/Security/mod.rs:573-579`.
- `CreateWellKnownSid`, `EqualSid`, and `WinBuiltinAdministratorsSid`: `.../Win32/Security/mod.rs:290`, `:333-335`, `:2310`.
- String/SID conversion functions: `.../Win32/Security/Authorization/mod.rs:329-373`.
- `PSID` and `PSECURITY_DESCRIPTOR` are pointer wrappers: `.../Win32/Security/mod.rs:1394-1412`.

#### Privilege acquisition and restoration

1. Open the current process token using `OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, ...)`.
2. Resolve each LUID with `LookupPrivilegeValueW(None, SE_TAKE_OWNERSHIP_NAME, ...)` and `LookupPrivilegeValueW(None, SE_RESTORE_NAME, ...)`.
3. Enable one privilege at a time using a one-entry `TOKEN_PRIVILEGES`, request `PreviousState`, and save that exact previous state for restoration.
4. Immediately after a successful Boolean return from `AdjustTokenPrivileges`, check `GetLastError`. `ERROR_NOT_ALL_ASSIGNED` means the process token does not hold the requested privilege even though the API call itself returned success. Abort before any ownership mutation.
5. Restore each exact `PreviousState` and close the token handle on every path. Do not merely “disable” a privilege that was already enabled before the spike.

Relevant generated bindings:

- `AdjustTokenPrivileges`: `.../Win32/Security/mod.rs:207-209`.
- `LookupPrivilegeValueW`: `.../Win32/Security/mod.rs:641-647`.
- `TOKEN_PRIVILEGES` and `LUID_AND_ATTRIBUTES`: `.../Win32/Security/mod.rs:1286-1289`, `:2124-2131`.
- `SE_RESTORE_NAME`, `SE_TAKE_OWNERSHIP_NAME`, `SE_PRIVILEGE_ENABLED`, `TOKEN_ADJUST_PRIVILEGES`, and `TOKEN_QUERY`: `.../Win32/Security/mod.rs:1680-1687`, `:1719`, `:1993`, `:2169`.
- `GetLastError`, `ERROR_NOT_ALL_ASSIGNED`, and `CloseHandle`: `.../Win32/Foundation/mod.rs:2`, `:27`, `:3454`.

The live mode should still perform an explicit elevated-token preflight, but successful acquisition of both privileges without `ERROR_NOT_ALL_ASSIGNED` is the decisive capability check for this operation.

#### Security descriptor ownership and mutation

- Query owner and DACL together with `GetNamedSecurityInfoW(path, SE_FILE_OBJECT, OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION, ...)`.
- This API returns a `WIN32_ERROR`, not `windows_core::Result`. Require `ERROR_SUCCESS` and preserve the returned code on failure.
- The owner SID and DACL outputs point inside the returned security descriptor. They remain valid only while that descriptor is alive. Free the descriptor exactly once with `LocalFree(HLOCAL(descriptor.0))`; never free its owner or DACL pointers separately.
- `SetNamedSecurityInfoW` also returns a `WIN32_ERROR`. For this spike, pass only `OWNER_SECURITY_INFORMATION`, the requested owner SID, and null group/DACL/SACL pointers. Do not mutate the DACL merely to prove an owner round trip.
- Setting TrustedInstaller as owner requires `SeRestorePrivilege`, because that SID is not an owner-enabled SID in the caller's token. Taking ownership as BUILTIN\\Administrators uses `SeTakeOwnershipPrivilege`; the target SID must be present as an owner-eligible token SID/group.

Binding evidence:

- `GetNamedSecurityInfoW`: `.../Win32/Security/Authorization/mod.rs:455-460`.
- `SetNamedSecurityInfoW`: `.../Win32/Security/Authorization/mod.rs:526-531`.
- `SE_FILE_OBJECT`: `.../Win32/Security/Authorization/mod.rs:9932`.
- `OWNER_SECURITY_INFORMATION` and `DACL_SECURITY_INFORMATION`: `.../Win32/Security/mod.rs:1378`, `:1218`.
- `LocalFree`: `.../Win32/Foundation/mod.rs:38-40`; `HLOCAL`: `:5488`.

#### Semantic owner/DACL comparison

- Compare owner identities with `EqualSid` and also record their SDDL SID strings. Do not compare pointer values.
- Convert each complete snapshot descriptor with `ConvertSecurityDescriptorToStringSecurityDescriptorW(SDDL_REVISION_1, OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION, ...)`, copy the returned UTF-16 text, then `LocalFree` it. SDDL compares owner and DACL semantics using SID values rather than localized account names.
- Also record whether a DACL is present/null and the DACL control flags (especially protected/auto-inherited state) from `GetSecurityDescriptorControl`. Because this spike never writes the DACL, exact DACL SDDL equality before and after is expected; an ACE-order or flag change is evidence of an unexpected mutation, not something to normalize away.
- Do not compare raw self-relative security-descriptor pointer addresses or whole backing-buffer bytes. Their layout is not the ownership/ACL semantic contract.

Binding evidence:

- SDDL conversion: `.../Win32/Security/Authorization/mod.rs:329-331`; `SDDL_REVISION_1`: `:9907`.
- `EqualSid`, `GetSecurityDescriptorControl`, and ACL inspection helpers: `.../Win32/Security/mod.rs:333-355`, `:399`.

### 4. Minimal `windows 0.61.3` feature set

The minimal explicit leaf-feature set for all three Windows spikes, including `ctfmon.exe` observation, is:

```toml
windows = { version = "0.61.3", features = [
    "Win32_Security_Authorization",
    "Win32_System_Com",
    "Win32_System_Diagnostics_ToolHelp",
    "Win32_System_Ole",
    "Win32_System_TaskScheduler",
    "Win32_System_Threading",
    "Win32_System_Variant",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_TextServices",
] }
```

Why each leaf is needed:

| Feature | Required surface |
|---|---|
| `Win32_Security_Authorization` | named security information, SID/SDDL conversion; transitively enables `Win32_Security` |
| `Win32_System_Com` | COM apartment, security, `CoCreateInstance`, and `IDispatch`-based Task Scheduler interfaces |
| `Win32_System_Diagnostics_ToolHelp` | read-only `ctfmon.exe` process enumeration |
| `Win32_System_Ole` | generated `VARIANT` type and Task Scheduler `Connect`/`Run` cfg gate |
| `Win32_System_TaskScheduler` | Task Scheduler 2.0 interfaces and coclass |
| `Win32_System_Threading` | `GetCurrentProcess` / `OpenProcessToken` |
| `Win32_System_Variant` | `VARIANT::default()` and the Task Scheduler method cfg gate |
| `Win32_UI_Input_KeyboardAndMouse` | `HKL` and the generated TSF manager methods/profile structure, which are separately cfg-gated |
| `Win32_UI_TextServices` | TSF coclass, manager, profiles, category, and flags |

The feature graph in the crate manifest transitively supplies parent modules and `Win32_Foundation`; see `windows-0.61.3/Cargo.toml:397`, `:437`, `:530-535`, `:582-613`, `:640`, `:670-678`, and `:702-727`. Do not add `Win32_System_Services`, `Win32_System_Registry`, or process-termination features for this task: they belong to later product work or explicitly excluded behavior.

## Design Reconciliation

The current `.trellis/tasks/08-24-s0-risk-spikes/design.md` was compared after this API review. The following items need reconciliation before implementation:

1. **TSF enabled-versus-active conflict (critical).** Design section 4 branches on `TF_IPP_FLAG_ENABLED` and polls until that bit changes after `ActivateProfile` / `DeactivateProfile`. With the approved `TF_IPPMF_FORSESSION` scope, the state under test is `TF_IPP_FLAG_ACTIVE` plus `GetActiveProfile`; changing `ENABLED` requires `TF_IPPMF_ENABLEPROFILE` / `TF_IPPMF_DISABLEPROFILE`, which updates the user's registry-backed enabled-profile state. The design should require Wubi to be enabled in preflight, branch on its active bit, and poll the active bit/current category identity. The enabled bit should be an unchanged restoration invariant.
2. **ACL cleanup restore timing (critical).** Design section 5 captures “cleanup ownership” before seeding TrustedInstaller but its numbered step 8 restores token privileges and deletes the file without explicitly restoring that initial owner/security state. Baseline A must be restored while `SeRestorePrivilege` is still available, then privilege attributes can be restored, then deletion can occur. If restoring baseline A fails, retain the file and path as evidence as the design already requires.
3. **Task Scheduler Ready-state/process-transition conflict (critical).** Design section 6 requires a task/process transition attributable to the cycle and says no observable `ctfmon.exe` transition is failure. The observed and valid baseline is `TASK_STATE_READY` plus a resident `ctfmon.exe`; `Stop(0)` can have no process effect and `Run(VT_EMPTY)` can contact an existing singleton with the same PID. The pass evidence should accept the returned `IRunningTask`/instance GUID and bounded task timeline as proof of Run, while treating same-PID `ctfmon.exe` presence as expected behavior. Require a PID transition only if the baseline/runtime actually makes one observable.
4. **COM security ownership (design gap).** Design section 6 says only “Initialize COM.” For a standalone example that owns the process, state whether it calls `CoInitializeSecurity` once before creating Task Scheduler proxies, as Microsoft native examples do. If the implementation intentionally relies on default local COM security instead, record that decision and its access-denied behavior. Do not call process-global COM security repeatedly from a helper or silently ignore `RPC_E_TOO_LATE`.
5. **Feature-list redundancy (minor).** Design section 2 lists explicit `Win32_Foundation` and `Win32_Security`; the exact Cargo graph enables them transitively from the leaf features above. Keeping them is harmless, but the stated “features actually required” / pruning rule means the final minimal manifest should use the leaf set unless compilation demonstrates an explicit parent feature is necessary.

## Files Found

- `.trellis/tasks/08-24-s0-risk-spikes/prd.md` - approved spike scope, live safety gate, restoration, and acceptance criteria.
- `docs/00-overview.md:218-226` - TSF terminology and the fixed Microsoft Wubi CLSID/profile/language identity.
- `docs/22-roadmap.md:541-570` - four S0 risk gates and their fixed success thresholds.
- `docs/modules/M4-ime-control.md:190-200` - scheduler, ACL, profile, and recovery requirements.
- `docs/modules/M4-ime-control.md:243-280` - stage-preserving errors, unconditional restore, and Win32/COM preference.
- `docs/02-architecture.md:424-435` - authoritative aardio-to-Win32 API mapping.
- `docs/02-architecture.md:593-599` - RAII restoration decision.
- `docs/02-architecture.md:724-735` - future `SystemOps`/recording boundary; this spike must not prematurely implement it.
- `crates/wubilex-winime/README.md:19-44` - required-feature-only dependency rule and prohibition on localized command-line tools.
- `crates/wubilex-winime/src/lib.rs:1` - the crate currently contains only its scaffold marker; no implementation pattern exists yet.
- `.trellis/spec/backend/directory-structure.md:30-48` - all direct Win32/COM work belongs in `wubilex-winime`.
- `.trellis/spec/backend/error-handling.md:20-32` - typed errors, native codes/text, checked shutdown stages, and reachable restoration.
- `Cargo.lock:3824-3834` - repository-resolved `windows` version is `0.61.3` with `windows-core 0.61.2`.
- `E:/env/rust/cargo/registry/src/rsproxy.cn-e3de039b2554c837/windows-0.61.3/src/Windows/Win32/...` - local generated binding source used for every signature and cfg assertion above.

## Code Patterns

- `crates/wubilex-winime/README.md:33-44` fixes the long-term single-orchestration/recording design, but `SPIKE-R01` limits this task to small independently runnable harnesses and truly shared helpers.
- `.trellis/spec/backend/error-handling.md:27-32` requires native error evidence and continued restoration after a forward failure. Each spike therefore needs stage-tagged HRESULT/Win32 errors plus a restoration report.
- `docs/02-architecture.md:593-599` and `crates/wubilex-winime/README.md:58-60` require reverse-order RAII restoration, including panic paths. The spike guards should be armed only after their snapshot exists and updated after each successful mutation.
- `docs/02-architecture.md:1022-1023` identifies lingering Administrators ownership and localized CLI parsing as explicit high risks. The ACL evidence must use SID/SDDL values, and the scheduler evidence must use COM state, never command output.

## External References

- Microsoft Learn, `ITfInputProcessorProfileMgr::ActivateProfile`: <https://learn.microsoft.com/windows/win32/api/msctf/nf-msctf-itfinputprocessorprofilemgr-activateprofile>
- Microsoft Learn, `DeactivateProfile`: <https://learn.microsoft.com/windows/win32/api/msctf/nf-msctf-itfinputprocessorprofilemgr-deactivateprofile>
- Microsoft Learn, `GetProfile`: <https://learn.microsoft.com/windows/win32/api/msctf/nf-msctf-itfinputprocessorprofilemgr-getprofile>
- Microsoft Learn, `GetActiveProfile`: <https://learn.microsoft.com/windows/win32/api/msctf/nf-msctf-itfinputprocessorprofilemgr-getactiveprofile>
- Microsoft Learn, `TaskService.Connect`: <https://learn.microsoft.com/windows/win32/taskschd/taskservice-connect>
- Microsoft Learn, `RegisteredTask.Run`: <https://learn.microsoft.com/windows/win32/taskschd/registeredtask-run>
- Microsoft Learn, `RegisteredTask.Stop`: <https://learn.microsoft.com/windows/win32/taskschd/registeredtask-stop>
- Microsoft Learn, `GetNamedSecurityInfoW`: <https://learn.microsoft.com/windows/win32/api/aclapi/nf-aclapi-getnamedsecurityinfow>
- Microsoft Learn, `SetNamedSecurityInfoW`: <https://learn.microsoft.com/windows/win32/api/aclapi/nf-aclapi-setnamedsecurityinfow>
- Microsoft Learn, `AdjustTokenPrivileges`: <https://learn.microsoft.com/windows/win32/api/securitybaseapi/nf-securitybaseapi-adjusttokenprivileges>
- Microsoft Learn, `LookupAccountNameW`: <https://learn.microsoft.com/windows/win32/api/winbase/nf-winbase-lookupaccountnamew>
- Microsoft Learn, `Taking Object Ownership in C++`: <https://learn.microsoft.com/windows/win32/secauthz/taking-object-ownership-in-c-->
- `windows` crate 0.61.3 docs/source baseline: <https://docs.rs/windows/0.61.3/windows/>

## Related Specs

- `SPIKE-R01..R05`, `SPIKE-R08..R10` in `.trellis/tasks/08-24-s0-risk-spikes/prd.md`.
- `M4-TIP-008`, `M4-TSF-003`, `M4-TSF-007`, `M4-TSF-010`, and `M4-TSF-011` in `docs/modules/M4-ime-control.md`.
- `R1`, `R9`, `R10`, and `R24` in `docs/02-architecture.md`.
- Backend directory, error, and quality rules in `.trellis/spec/backend/{directory-structure,error-handling,quality-guidelines}.md`.

## Caveats / Not Found

- No Windows implementation exists in `wubilex-winime` yet, so there is no local production wrapper pattern to reuse; all concrete signatures above come from the repository-resolved generated bindings.
- The feature set is derived from the exact `windows 0.61.3` cfg gates and feature graph. It still needs a compile check when implementation is authorized because no manifest or product code may be changed during this research pass.
- `ActivateProfile`'s generated wrapper collapses `S_FALSE` into `Ok(())`; state re-query is the only reliable success criterion unless the implementation deliberately calls the raw vtable to preserve the success HRESULT.
- `MsCtfMonitor` being Ready while `ctfmon.exe` is present means Stop and process exit are not equivalent. The full future shutdown window still needs its separately specified process step; this spike must neither terminate `ChsIME.exe` nor add that product behavior.
- Exact Task Scheduler instance identity and exact `ctfmon.exe` PID are not restorable contracts. Restore task readiness/running semantics and process presence, and report PID changes as evidence.
- A live ACL run requires an elevated token containing both `SeTakeOwnershipPrivilege` and `SeRestorePrivilege`. `AdjustTokenPrivileges` cannot add absent privileges.
- `GetNamedSecurityInfoW` is documented as race-prone against concurrent descriptor modification. Restricting the spike to a uniquely created temporary file removes normal external writers but does not change the API contract.
