# ACL Owner Spike Result

## Status

`PASS`

Captured on 2026-08-25. The live command ran in a UAC-elevated PowerShell
process and returned exit code 0.

## Command

```powershell
cargo run -p wubilex-winime --example acl_owner_spike
cargo run -p wubilex-winime --example acl_owner_spike -- --live
cargo run -p wubilex-winime --example acl_owner_spike
```

## Read-Only Evidence

```text
mode=DryRun
target_policy=one create_new file below %TEMP%/wubilex-risk-spikes/acl-owner
elevated=false
planned_owner_round_trip=TrustedInstaller -> Administrators -> TrustedInstaller
planned_cleanup=restore creation owner/DACL, restore token privileges, delete file
verdict=DRY-RUN PASS; no file was created and no ACL was changed
```

The dry-run accepted no target path and created no file. Live mode created one
unique file under `%TEMP%/wubilex-risk-spikes/acl-owner` and emitted these
stage results:

```text
trusted_installer_sid=S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464
administrators_sid=S-1-5-32-544
round_trip=verified TrustedInstaller -> Administrators -> TrustedInstaller
restoration=baseline A verified; privileges restored; file deleted
verdict=LIVE PASS
```

The normalized baseline-B security descriptor had TrustedInstaller as owner.
The intermediate descriptor had Administrators (`O:BA`) as owner, and the
restored descriptor exactly matched baseline B. Its DACL and control value
`0x8404` remained unchanged across both owner transitions.

## Verdict And Cleanup

The probe passes the ACL acceptance criterion. It restored creation baseline A,
restored the token privileges, and deleted the task-created file. An independent
scan found `residual_temp_file_count=0` and the dedicated probe directory no
longer existed. Raw elevated output and exit code are retained in
`acl-owner.live.log` and `acl-owner.live.exitcode`.

The result proves the owner-only round trip for a controlled temporary file;
it does not authorize or validate changes to a real lexicon or user file.
