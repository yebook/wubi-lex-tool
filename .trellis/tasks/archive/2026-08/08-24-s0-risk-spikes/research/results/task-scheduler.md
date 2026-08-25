# Task Scheduler Spike Result

## Status

`PASS`

Captured on 2026-08-25. The live command ran in a UAC-elevated PowerShell
process and returned exit code 0.

## Command

```powershell
cargo run -p wubilex-winime --example task_scheduler_spike
cargo run -p wubilex-winime --example task_scheduler_spike -- --live
cargo run -p wubilex-winime --example task_scheduler_spike
```

## Read-Only Evidence

```text
baseline=timestamp_ms:1787622872207,enabled:true,state:3,instances:[],ctfmon_pids:[5212]
mode=DryRun
planned_action=IRegisteredTask::Stop(0) -> observe -> Run(VT_EMPTY) -> restore
process_policy=observe ctfmon.exe only; never open or terminate it
verdict=DRY-RUN PASS; no Stop or Run call was made
```

`TASK_STATE` value 3 is Ready. The task was enabled, had no Scheduler running
instance, and `ctfmon.exe` PID 5212 was present. Live mode emitted:

```text
operation=Stop(0) accepted
after_stop=enabled:true,state:3,instances:[],ctfmon_pids:[5212]
stop_observation=instances:0,ctfmon_pid_changed:false
run_instance=guid:{BB6CC585-9DB0-4E68-A7FE-2ECB8764226A},state:unavailable: 0x8004130B,engine_pid:unavailable: 0x8004130B
after_run=enabled:true,state:3,instances:[],ctfmon_pids:[5212]
restored=enabled:true,state:3,instances:[],ctfmon_pids:[5212]
restoration=logical baseline verified
verdict=LIVE PASS
```

The returned instance identity proves that `Run(VT_EMPTY)` was accepted. Its
state and engine PID became unavailable with `SCHED_E_TASK_NOT_RUNNING`
(`0x8004130B`) because this system keeps `ctfmon.exe` as a detached singleton.

## Verdict And Limitations

The probe passes the Scheduler COM acceptance criterion. A subsequent dry-run
independently observed the same enabled/Ready/no-instance state and PID 5212.
Raw elevated output and exit code are retained in
`task-scheduler.live.log` and `task-scheduler.live.exitcode`.

An unchanged singleton PID is valid evidence here; this probe does not prove or
replace the future shutdown window's separate process-control behavior. The
guard still refuses to issue another Run while a task is queued, running, or
reports an existing instance.
