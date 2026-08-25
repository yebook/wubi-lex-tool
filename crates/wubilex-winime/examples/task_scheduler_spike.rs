#[cfg(windows)]
mod support;

#[cfg(not(windows))]
fn main() {
    eprintln!("task_scheduler_spike is supported only on Windows");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_spike::run() {
        eprintln!("TASK SCHEDULER SPIKE FAILED: {error}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod windows_spike {
    use std::mem::ManuallyDrop;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use windows::Win32::Foundation::ERROR_NO_MORE_FILES;
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitializeSecurity, EOAC_NONE,
        RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE,
    };
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::TaskScheduler::{
        IRegisteredTask, IRunningTask, ITaskService, TASK_STATE, TASK_STATE_QUEUED,
        TASK_STATE_READY, TASK_STATE_RUNNING, TaskScheduler,
    };
    use windows::Win32::System::Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4};
    use windows::core::BSTR;

    use super::support::{
        ComApartment, Mode, OwnedHandle, SpikeError, poll_until, require_elevated,
    };

    const TASK_FOLDER: &str = "\\Microsoft\\Windows\\TextServicesFramework";
    const TASK_NAME: &str = "MsCtfMonitor";

    #[derive(Clone, Debug)]
    struct Snapshot {
        enabled: bool,
        state: TASK_STATE,
        instances: Vec<String>,
        ctfmon_pids: Vec<u32>,
        observed_ms: u128,
    }

    pub fn run() -> Result<(), SpikeError> {
        let mode = super::support::parse_mode()?;
        let _apartment = ComApartment::initialize()?;
        initialize_com_security()?;
        let task = connect_task()?;
        let baseline = snapshot(&task)?;
        print_snapshot("baseline", &baseline);
        println!("mode={mode:?}");
        println!("planned_action=IRegisteredTask::Stop(0) -> observe -> Run(VT_EMPTY) -> restore");
        println!("process_policy=observe ctfmon.exe only; never open or terminate it");
        if mode == Mode::DryRun {
            println!("verdict=DRY-RUN PASS; no Stop or Run call was made");
            return Ok(());
        }

        require_elevated()?;
        validate_live_preconditions(&baseline)?;
        let mut guard = RestoreGuard::new(task.clone(), baseline.clone());
        let operation = exercise(&task, &baseline);
        let restoration = guard.restore();
        match (operation, restoration) {
            (Ok(()), Ok(())) => {
                println!("restoration=logical baseline verified");
                println!("verdict=LIVE PASS");
                Ok(())
            }
            (Err(primary), Ok(())) => Err(primary),
            (Ok(()), Err(restore)) => Err(restore),
            (Err(primary), Err(restore)) => Err(primary.with_restore(restore)),
        }
    }

    fn initialize_com_security() -> Result<(), SpikeError> {
        unsafe {
            CoInitializeSecurity(
                None,
                -1,
                None,
                None,
                RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            )
        }
        .map_err(|error| SpikeError::windows("CoInitializeSecurity", error))
    }

    fn connect_task() -> Result<IRegisteredTask, SpikeError> {
        let service: ITaskService =
            unsafe { CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER) }
                .map_err(|error| SpikeError::windows("CoCreateInstance(TaskScheduler)", error))?;
        let empty = VARIANT::default();
        unsafe { service.Connect(&empty, &empty, &empty, &empty) }
            .map_err(|error| SpikeError::windows("ITaskService::Connect(local)", error))?;
        let folder = unsafe { service.GetFolder(&BSTR::from(TASK_FOLDER)) }.map_err(|error| {
            SpikeError::windows("ITaskService::GetFolder(TextServicesFramework)", error)
        })?;
        unsafe { folder.GetTask(&BSTR::from(TASK_NAME)) }
            .map_err(|error| SpikeError::windows("ITaskFolder::GetTask(MsCtfMonitor)", error))
    }

    fn snapshot(task: &IRegisteredTask) -> Result<Snapshot, SpikeError> {
        let enabled = unsafe { task.Enabled() }
            .map_err(|error| SpikeError::windows("IRegisteredTask::Enabled", error))?
            .0
            != 0;
        let state = unsafe { task.State() }
            .map_err(|error| SpikeError::windows("IRegisteredTask::State", error))?;
        let instances = running_instances(task)?;
        let ctfmon_pids = ctfmon_pids()?;
        let observed_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| SpikeError::new("capture observation timestamp", error.to_string()))?
            .as_millis();
        Ok(Snapshot {
            enabled,
            state,
            instances,
            ctfmon_pids,
            observed_ms,
        })
    }

    fn running_instances(task: &IRegisteredTask) -> Result<Vec<String>, SpikeError> {
        let collection = unsafe { task.GetInstances(0) }
            .map_err(|error| SpikeError::windows("IRegisteredTask::GetInstances", error))?;
        let count = unsafe { collection.Count() }
            .map_err(|error| SpikeError::windows("IRunningTaskCollection::Count", error))?;
        if count < 0 {
            return Err(SpikeError::new(
                "IRunningTaskCollection::Count",
                format!("negative count: {count}"),
            ));
        }
        let mut identities = Vec::with_capacity(count as usize);
        for index in 1..=count {
            let item = unsafe { collection.get_Item(&variant_i32(index)) }
                .map_err(|error| SpikeError::windows("IRunningTaskCollection::get_Item", error))?;
            identities.push(running_identity(&item)?);
        }
        Ok(identities)
    }

    fn running_identity(task: &IRunningTask) -> Result<String, SpikeError> {
        let guid = unsafe { task.InstanceGuid() }
            .map_err(|error| SpikeError::windows("IRunningTask::InstanceGuid", error))?;
        Ok(guid.to_string())
    }

    fn variant_i32(value: i32) -> VARIANT {
        VARIANT {
            Anonymous: VARIANT_0 {
                Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                    vt: VT_I4,
                    wReserved1: 0,
                    wReserved2: 0,
                    wReserved3: 0,
                    Anonymous: VARIANT_0_0_0 { lVal: value },
                }),
            },
        }
    }

    fn ctfmon_pids() -> Result<Vec<u32>, SpikeError> {
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
            .map_err(|error| SpikeError::windows("CreateToolhelp32Snapshot(processes)", error))?;
        let handle = OwnedHandle::new(handle);
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        unsafe { Process32FirstW(handle.raw(), &mut entry) }
            .map_err(|error| SpikeError::windows("Process32FirstW", error))?;
        let mut pids = Vec::new();
        loop {
            let end = entry
                .szExeFile
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
            if name.eq_ignore_ascii_case("ctfmon.exe") {
                pids.push(entry.th32ProcessID);
            }
            match unsafe { Process32NextW(handle.raw(), &mut entry) } {
                Ok(()) => {}
                Err(error)
                    if error.code()
                        == windows::core::HRESULT::from_win32(ERROR_NO_MORE_FILES.0) =>
                {
                    break;
                }
                Err(error) => return Err(SpikeError::windows("Process32NextW", error)),
            }
        }
        pids.sort_unstable();
        Ok(pids)
    }

    fn validate_live_preconditions(baseline: &Snapshot) -> Result<(), SpikeError> {
        if !baseline.enabled {
            return Err(SpikeError::new(
                "live-preflight",
                "MsCtfMonitor is disabled; no mutation was attempted",
            ));
        }
        if baseline.ctfmon_pids.is_empty() {
            return Err(SpikeError::new(
                "live-preflight",
                "ctfmon.exe is absent; safe restoration cannot be guaranteed",
            ));
        }
        Ok(())
    }

    fn exercise(task: &IRegisteredTask, baseline: &Snapshot) -> Result<(), SpikeError> {
        unsafe { task.Stop(0) }
            .map_err(|error| SpikeError::windows("IRegisteredTask::Stop(0)", error))?;
        println!("operation=Stop(0) accepted");
        let after_stop = poll_until(
            "observe task after Stop",
            Duration::from_secs(5),
            Duration::from_millis(100),
            || {
                let observed = snapshot(task)?;
                print_snapshot("after_stop", &observed);
                Ok(observed)
            },
            can_start_task,
        )?;
        println!(
            "stop_observation=instances:{},ctfmon_pid_changed:{}",
            after_stop.instances.len(),
            after_stop.ctfmon_pids != baseline.ctfmon_pids
        );

        let running = unsafe { task.Run(&VARIANT::default()) }
            .map_err(|error| SpikeError::windows("IRegisteredTask::Run(VT_EMPTY)", error))?;
        let identity = running_identity(&running)?;
        let state = unsafe { running.State() }
            .map(|value| format!("{}", value.0))
            .unwrap_or_else(|error| format!("unavailable: {error}"));
        let engine_pid = unsafe { running.EnginePID() }
            .map(|value| value.to_string())
            .unwrap_or_else(|error| format!("unavailable: {error}"));
        println!("run_instance=guid:{identity},state:{state},engine_pid:{engine_pid}");
        let restored = wait_for_logical_baseline(task, baseline)?;
        print_snapshot("after_run", &restored);
        Ok(())
    }

    fn logical_equivalent(baseline: &Snapshot, observed: &Snapshot) -> bool {
        if baseline.enabled != observed.enabled {
            return false;
        }
        if !baseline.ctfmon_pids.is_empty() && observed.ctfmon_pids.is_empty() {
            return false;
        }
        if baseline.state == TASK_STATE_READY {
            observed.state == TASK_STATE_READY && observed.instances.is_empty()
        } else if baseline.state == TASK_STATE_RUNNING || !baseline.instances.is_empty() {
            observed.state == TASK_STATE_RUNNING || !observed.instances.is_empty()
        } else {
            observed.state == baseline.state
        }
    }

    fn can_start_task(snapshot: &Snapshot) -> bool {
        snapshot.state != TASK_STATE_QUEUED
            && snapshot.state != TASK_STATE_RUNNING
            && snapshot.instances.is_empty()
    }

    fn wait_for_logical_baseline(
        task: &IRegisteredTask,
        baseline: &Snapshot,
    ) -> Result<Snapshot, SpikeError> {
        poll_until(
            "restore MsCtfMonitor logical baseline",
            Duration::from_secs(5),
            Duration::from_millis(100),
            || snapshot(task),
            |observed| logical_equivalent(baseline, observed),
        )
    }

    fn print_snapshot(label: &str, snapshot: &Snapshot) {
        println!(
            "{label}=timestamp_ms:{},enabled:{},state:{},instances:{:?},ctfmon_pids:{:?}",
            snapshot.observed_ms,
            snapshot.enabled,
            snapshot.state.0,
            snapshot.instances,
            snapshot.ctfmon_pids
        );
    }

    struct RestoreGuard {
        task: IRegisteredTask,
        baseline: Snapshot,
        armed: bool,
    }

    impl RestoreGuard {
        fn new(task: IRegisteredTask, baseline: Snapshot) -> Self {
            Self {
                task,
                baseline,
                armed: true,
            }
        }

        fn restore(&mut self) -> Result<(), SpikeError> {
            restore_task(&self.task, &self.baseline)?;
            self.armed = false;
            Ok(())
        }
    }

    impl Drop for RestoreGuard {
        fn drop(&mut self) {
            if self.armed
                && let Err(error) = restore_task(&self.task, &self.baseline)
            {
                eprintln!(
                    "RESTORATION FAILURE: {error}; manually run {TASK_FOLDER}\\{TASK_NAME} and verify ctfmon.exe"
                );
            }
        }
    }

    fn restore_task(task: &IRegisteredTask, baseline: &Snapshot) -> Result<(), SpikeError> {
        let mut current = snapshot(task)?;
        if !can_start_task(&current) {
            current = poll_until(
                "wait for existing MsCtfMonitor instance before restoration",
                Duration::from_secs(5),
                Duration::from_millis(100),
                || snapshot(task),
                |observed| logical_equivalent(baseline, observed) || can_start_task(observed),
            )?;
        }
        if logical_equivalent(baseline, &current) {
            print_snapshot("restored", &current);
            return Ok(());
        }

        let baseline_needs_presence = !baseline.ctfmon_pids.is_empty()
            || baseline.state == TASK_STATE_RUNNING
            || !baseline.instances.is_empty();
        let recovery_run_is_safe =
            baseline_needs_presence && current.ctfmon_pids.is_empty() && can_start_task(&current);
        if recovery_run_is_safe {
            let running = unsafe { task.Run(&VARIANT::default()) }
                .map_err(|error| SpikeError::windows("restoration Run(VT_EMPTY)", error))?;
            let identity = running_identity(&running)?;
            println!("restoration_run_instance={identity}");
        } else {
            return Err(SpikeError::new(
                "restore MsCtfMonitor logical baseline",
                format!(
                    "state mismatch but an existing task/process presence makes another Run unsafe: state={}, instances={:?}, ctfmon_pids={:?}",
                    current.state.0, current.instances, current.ctfmon_pids
                ),
            ));
        }
        let restored = wait_for_logical_baseline(task, baseline)?;
        print_snapshot("restored", &restored);
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::{Snapshot, can_start_task, logical_equivalent};
        use windows::Win32::System::TaskScheduler::{
            TASK_STATE_QUEUED, TASK_STATE_READY, TASK_STATE_RUNNING,
        };

        fn state(task_state: windows::Win32::System::TaskScheduler::TASK_STATE) -> Snapshot {
            Snapshot {
                enabled: true,
                state: task_state,
                instances: Vec::new(),
                ctfmon_pids: vec![10],
                observed_ms: 0,
            }
        }

        #[test]
        fn ready_baseline_accepts_same_process_presence() {
            assert!(logical_equivalent(
                &state(TASK_STATE_READY),
                &state(TASK_STATE_READY)
            ));
        }

        #[test]
        fn running_baseline_rejects_ready_without_instance() {
            assert!(!logical_equivalent(
                &state(TASK_STATE_RUNNING),
                &state(TASK_STATE_READY)
            ));
        }

        #[test]
        fn queued_or_running_task_cannot_be_started_again() {
            assert!(!can_start_task(&state(TASK_STATE_QUEUED)));
            assert!(!can_start_task(&state(TASK_STATE_RUNNING)));

            let mut ready_with_instance = state(TASK_STATE_READY);
            ready_with_instance.instances.push("existing".to_owned());
            assert!(!can_start_task(&ready_with_instance));
            assert!(can_start_task(&state(TASK_STATE_READY)));
        }
    }
}
