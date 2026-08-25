# S0 Risk Spike Result Summary

## Current Decision

`PASS`. All four spikes met their original acceptance thresholds using the
reviewed final code. The three controlled Windows probes restored or cleaned
their targets and passed independent read-only post-checks. The visible Edge
benchmark passed all three foreground runs. This risk-spike gate no longer
blocks creation of S1 work.

| Spike | Current evidence | Gate status |
|---|---|---|
| TSF Profile | Wubi ACTIVE/current-profile changed, ENABLED stayed true, exact profile baseline restored | PASS |
| Temporary-file ACL | TrustedInstaller -> Administrators -> TrustedInstaller verified; owner/DACL and privileges restored; file deleted | PASS |
| Task Scheduler COM | Stop/Run accepted; returned instance recorded; enabled/Ready/`ctfmon.exe` logical baseline restored | PASS |
| 300,000-row virtual scroll | 119.60 / 93.40 / 110.02 fps; maximum 45 DOM rows; all samples valid and error-free | PASS |

## Requirement Mapping

- `SPIKE-R01`: each probe is isolated and independently runnable; shared Windows
  support remains limited to probe mechanics.
- `SPIKE-R02`: session-scoped TSF ACTIVE/current-profile change and exact restore
  passed while ENABLED remained unchanged.
- `SPIKE-R03`: the task-created temporary file completed the required owner
  round trip with normalized DACL equality and complete cleanup.
- `SPIKE-R04`: Task Scheduler COM Stop/Run, instance evidence, process timeline,
  and logical-state restoration passed. The unchanged singleton PID is recorded.
- `SPIKE-R05`: defaults are non-mutating; live mode required elevation, armed
  restoration, reported native stages, and passed independent post-checks.
- `SPIKE-R06`: the isolated TanStack Virtual harness derived 300,000 rows by
  index and held the rendered DOM to 45 rows.
- `SPIKE-R07`: all three visible Edge samples exceeded 55 fps and recorded
  timing, DOM, visibility, error, scroll-range, and memory evidence.
- `SPIKE-R08`: all four reports contain environment/commands, baseline,
  observations, verdict, restoration or cleanup, and limitations; raw browser
  JSON and elevated logs/exit codes are retained in this directory.
- `SPIKE-R09`: no spike failed after final revalidation, so no blocking
  alternative or architecture review decision is required. Thresholds were not
  weakened.
- `SPIKE-R10`: the probes did not read root `resource/`, touch a real lexicon,
  stop `TabletInputService`, or terminate `ChsIME.exe`/`ctfmon.exe`.

Machine-specific paths, SIDs, PIDs, timestamps, and measurements remain only in
task research and are not product configuration.
