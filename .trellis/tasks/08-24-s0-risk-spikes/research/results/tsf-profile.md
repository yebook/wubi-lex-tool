# TSF Profile Spike Result

## Status

`PASS`

Captured on 2026-08-25. The live command ran in a UAC-elevated PowerShell
process and returned exit code 0.

## Command

```powershell
cargo run -p wubilex-winime --example tsf_profile_spike
cargo run -p wubilex-winime --example tsf_profile_spike -- --live
cargo run -p wubilex-winime --example tsf_profile_spike
```

## Read-Only Evidence

```text
baseline.wubi=enabled:true,active:true,flags:0x00000003
baseline.active=type:1,lang:0x0804,clsid:6A498709-E00B-4C45-A018-8F9E4081AE40,profile:82590C13-F4DD-44F4-BA1D-8667246FDF8E,category:34745C63-B2F0-4784-8B67-5E12C8701A31,hkl_substitute:HKL(0x0),caps:0x5003002E,hkl:HKL(0x0)
mode=DryRun
planned_scope=TF_IPPMF_FORSESSION only; ENABLED configuration is immutable
planned_action=DeactivateThenRestore
verdict=DRY-RUN PASS; no ActivateProfile/DeactivateProfile call was made
```

The preflight located the Microsoft Wubi profile and the current active keyboard
profile through TSF. The live evidence was:

```text
after_deactivate.wubi=enabled:true,active:false,flags:0x00000002
after_deactivate.active=type:1,lang:0x0804,clsid:81D4E9C9-1D3B-41BC-9E6C-4B40BF79E35E,profile:FA550B04-5AD7-411F-A5AC-CA038EC515D7,category:34745C63-B2F0-4784-8B67-5E12C8701A31,hkl_substitute:HKL(0x0),caps:0x5003002E,hkl:HKL(0x0)
transition=wubi-active -> inactive verified
restored.wubi=enabled:true,active:true,flags:0x00000003
restored.active=type:1,lang:0x0804,clsid:6A498709-E00B-4C45-A018-8F9E4081AE40,profile:82590C13-F4DD-44F4-BA1D-8667246FDF8E,category:34745C63-B2F0-4784-8B67-5E12C8701A31,hkl_substitute:HKL(0x0),caps:0x5003002E,hkl:HKL(0x0)
restoration=verified
verdict=LIVE PASS
```

The second dry-run independently returned the same Wubi and active-profile
snapshot shown above. The Wubi ENABLED bit remained set throughout, while its
ACTIVE bit and the current-profile identity changed and were then restored.

## Verdict And Limitations

The probe passes the TSF acceptance criterion. The raw elevated output and exit
code are retained in `tsf-profile.live.log` and
`tsf-profile.live.exitcode`. This validates session-scoped TSF profile control;
it does not validate the future product's complete process/service shutdown
window or persistent default-input-method configuration.
