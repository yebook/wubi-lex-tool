# Environment Preflight

## Captured Baseline

Read-only inspection on 2026-08-24 established the environment in which the live spikes will be run:

| Item | Observed value |
|---|---|
| OS | Windows 11 Pro, 64-bit, build `26200` |
| Current identity | `YE-PC\yekm` |
| Elevated | No |
| Rust | `1.97.1` |
| Node | `24.18.1` |
| Global pnpm | `11.18.0` |
| Microsoft Edge | `151.0.4129.101` at `C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe` |
| `MsCtfMonitor` | Enabled, demand start allowed, state `Ready`, multiple instances `Parallel` |
| `MsCtfMonitor` action | COM handler `{01575CFE-9A55-4003-A5E1-F38D1EBDCBE1}` |
| `ctfmon.exe` | Present during preflight; observed PID `6968` |

The current Simplified Chinese user language entry contains three TIPs:

- Microsoft Pinyin: `0804:{81D4E9C9-1D3B-41BC-9E6C-4B40BF79E35E}{FA550B04-5AD7-411F-A5AC-CA038EC515D7}`
- an additional installed Chinese TIP: `0804:{E7EA138E-69F8-11D7-A6EA-00065B844310}{E7EA138F-69F8-11D7-A6EA-00065B844311}`
- Microsoft Wubi: `0804:{6A498709-E00B-4C45-A018-8F9E4081AE40}{82590C13-F4DD-44F4-BA1D-8667246FDF8E}`

## Planning Consequences

- Live Windows commands cannot pass from the current non-elevated process. They must be run one at a time from an elevated process after another dry-run snapshot.
- Wubi is present in the user's language/TIP list, so the TSF spike can target the documented profile instead of inventing or registering one.
- `MsCtfMonitor` uses a COM handler rather than a direct executable action. A successful Task Scheduler `Run` call is therefore insufficient evidence by itself; the probe must separately observe task instances/state and `ctfmon.exe` PIDs.
- The task is currently `Ready` while `ctfmon.exe` is present. The live scheduler spike is expected to determine whether COM End can produce an attributable transition from this real baseline. It must report failure rather than terminate the process directly if it cannot.
- Installed Edge can be driven with `playwright-core` through the `msedge` channel, avoiding a downloaded test browser.

Machine-specific paths, versions, PIDs, and timestamps in this file are evidence only and must not be copied into product configuration.

