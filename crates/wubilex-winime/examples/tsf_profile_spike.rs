#[cfg(windows)]
mod support;

#[cfg(not(windows))]
fn main() {
    eprintln!("tsf_profile_spike is supported only on Windows");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_spike::run() {
        eprintln!("TSF PROFILE SPIKE FAILED: {error}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod windows_spike {
    use std::ptr;
    use std::time::Duration;

    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
    use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
    use windows::Win32::UI::TextServices::{
        CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD, ITfInputProcessorProfileMgr,
        TF_INPUTPROCESSORPROFILE, TF_IPP_FLAG_ACTIVE, TF_IPP_FLAG_ENABLED, TF_IPPMF_FORSESSION,
        TF_PROFILETYPE_INPUTPROCESSOR, TF_PROFILETYPE_KEYBOARDLAYOUT,
    };
    use windows::core::{GUID, HRESULT, Interface};

    use super::support::{ComApartment, Mode, SpikeError, poll_until, require_elevated};

    const WUBI_LANGUAGE: u16 = 0x0804;
    const WUBI_CLSID: GUID = GUID::from_u128(0x6a498709_e00b_4c45_a018_8f9e4081ae40);
    const WUBI_PROFILE: GUID = GUID::from_u128(0x82590c13_f4dd_44f4_ba1d_8667246fdf8e);

    #[derive(Clone, Copy, Debug)]
    struct Snapshot {
        wubi: TF_INPUTPROCESSORPROFILE,
        active: Option<TF_INPUTPROCESSORPROFILE>,
    }

    pub fn run() -> Result<(), SpikeError> {
        let mode = super::support::parse_mode()?;
        let _apartment = ComApartment::initialize()?;
        let manager: ITfInputProcessorProfileMgr = unsafe {
            CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)
        }
        .map_err(|error| SpikeError::windows("CoCreateInstance(TSF profiles)", error))?;

        let baseline = snapshot(&manager)?;
        print_snapshot("baseline", &baseline);
        println!("mode={mode:?}");
        println!("planned_scope=TF_IPPMF_FORSESSION only; ENABLED configuration is immutable");
        println!(
            "planned_action={:?}",
            exercise_for(is_active(&baseline.wubi))
        );

        if mode == Mode::DryRun {
            println!("verdict=DRY-RUN PASS; no ActivateProfile/DeactivateProfile call was made");
            return Ok(());
        }

        require_elevated()?;
        validate_live_preconditions(&baseline)?;
        let mut guard = RestoreGuard::new(manager.clone(), baseline);
        let operation = exercise(&manager, baseline);
        let restoration = guard.restore();
        match (operation, restoration) {
            (Ok(()), Ok(())) => {
                println!("restoration=verified");
                println!("verdict=LIVE PASS");
                Ok(())
            }
            (Err(primary), Ok(())) => Err(primary),
            (Ok(()), Err(restore)) => Err(restore),
            (Err(primary), Err(restore)) => Err(primary.with_restore(restore)),
        }
    }

    fn snapshot(manager: &ITfInputProcessorProfileMgr) -> Result<Snapshot, SpikeError> {
        let mut wubi = TF_INPUTPROCESSORPROFILE::default();
        unsafe {
            manager.GetProfile(
                TF_PROFILETYPE_INPUTPROCESSOR,
                WUBI_LANGUAGE,
                &WUBI_CLSID,
                &WUBI_PROFILE,
                HKL::default(),
                &mut wubi,
            )
        }
        .map_err(|error| {
            SpikeError::windows("ITfInputProcessorProfileMgr::GetProfile(Wubi)", error)
        })?;
        Ok(Snapshot {
            wubi,
            active: get_active_profile(manager)?,
        })
    }

    fn get_active_profile(
        manager: &ITfInputProcessorProfileMgr,
    ) -> Result<Option<TF_INPUTPROCESSORPROFILE>, SpikeError> {
        let mut profile = TF_INPUTPROCESSORPROFILE::default();
        let result = unsafe {
            (Interface::vtable(manager).GetActiveProfile)(
                Interface::as_raw(manager),
                &GUID_TFCAT_TIP_KEYBOARD,
                &mut profile,
            )
        };
        if result == HRESULT(1) {
            Ok(None)
        } else {
            result
                .ok()
                .map_err(|error| SpikeError::windows("GetActiveProfile(keyboard)", error))?;
            Ok(Some(profile))
        }
    }

    fn validate_live_preconditions(baseline: &Snapshot) -> Result<(), SpikeError> {
        if !is_enabled(&baseline.wubi) {
            return Err(SpikeError::new(
                "live-preflight",
                "Microsoft Wubi exists but is not ENABLED; profile configuration was not changed",
            ));
        }
        let Some(active) = baseline.active else {
            return Err(SpikeError::new(
                "live-preflight",
                "no active keyboard profile; select a Simplified Chinese input method and retry",
            ));
        };
        if active.langid != WUBI_LANGUAGE {
            return Err(SpikeError::new(
                "live-preflight",
                format!(
                    "active keyboard language is 0x{:04X}; select Simplified Chinese (0x0804) and retry",
                    active.langid
                ),
            ));
        }
        Ok(())
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Exercise {
        DeactivateThenRestore,
        ActivateThenDeactivateThenRestore,
    }

    fn exercise_for(active: bool) -> Exercise {
        if active {
            Exercise::DeactivateThenRestore
        } else {
            Exercise::ActivateThenDeactivateThenRestore
        }
    }

    fn exercise(
        manager: &ITfInputProcessorProfileMgr,
        baseline: Snapshot,
    ) -> Result<(), SpikeError> {
        if is_active(&baseline.wubi) {
            deactivate_wubi(manager)?;
            let observed = wait_for_wubi(manager, false)?;
            print_snapshot("after_deactivate", &observed);
            println!("transition=wubi-active -> inactive verified");
        } else {
            activate_profile(manager, &baseline.wubi)?;
            let observed = wait_for_wubi(manager, true)?;
            print_snapshot("after_activate", &observed);
            println!("transition=wubi-inactive -> active verified");
            deactivate_wubi(manager)?;
            let observed = wait_for_wubi(manager, false)?;
            print_snapshot("after_deactivate", &observed);
            println!("transition=wubi-active -> inactive verified");
        }
        Ok(())
    }

    fn wait_for_wubi(
        manager: &ITfInputProcessorProfileMgr,
        expected: bool,
    ) -> Result<Snapshot, SpikeError> {
        poll_until(
            "poll TSF ACTIVE/current profile",
            Duration::from_secs(5),
            Duration::from_millis(100),
            || snapshot(manager),
            |observed| {
                is_active(&observed.wubi) == expected
                    && observed
                        .active
                        .as_ref()
                        .map(is_wubi_identity)
                        .unwrap_or(false)
                        == expected
            },
        )
    }

    fn activate_profile(
        manager: &ITfInputProcessorProfileMgr,
        profile: &TF_INPUTPROCESSORPROFILE,
    ) -> Result<(), SpikeError> {
        let (clsid, profile_guid) = profile_guids(profile)?;
        unsafe {
            manager.ActivateProfile(
                profile.dwProfileType,
                profile.langid,
                clsid,
                profile_guid,
                profile.hkl,
                TF_IPPMF_FORSESSION,
            )
        }
        .map_err(|error| SpikeError::windows("ActivateProfile(FORSESSION)", error))
    }

    fn deactivate_wubi(manager: &ITfInputProcessorProfileMgr) -> Result<(), SpikeError> {
        unsafe {
            manager.DeactivateProfile(
                TF_PROFILETYPE_INPUTPROCESSOR,
                WUBI_LANGUAGE,
                &WUBI_CLSID,
                &WUBI_PROFILE,
                HKL::default(),
                TF_IPPMF_FORSESSION,
            )
        }
        .map_err(|error| SpikeError::windows("DeactivateProfile(Wubi, FORSESSION)", error))
    }

    fn profile_guids(
        profile: &TF_INPUTPROCESSORPROFILE,
    ) -> Result<(*const GUID, *const GUID), SpikeError> {
        if profile.dwProfileType == TF_PROFILETYPE_INPUTPROCESSOR {
            Ok((&profile.clsid, &profile.guidProfile))
        } else if profile.dwProfileType == TF_PROFILETYPE_KEYBOARDLAYOUT {
            Ok((ptr::null(), ptr::null()))
        } else {
            Err(SpikeError::new(
                "restore TSF baseline",
                format!("unsupported profile type: {}", profile.dwProfileType),
            ))
        }
    }

    fn is_active(profile: &TF_INPUTPROCESSORPROFILE) -> bool {
        profile.dwFlags & TF_IPP_FLAG_ACTIVE != 0
    }

    fn is_enabled(profile: &TF_INPUTPROCESSORPROFILE) -> bool {
        profile.dwFlags & TF_IPP_FLAG_ENABLED != 0
    }

    fn is_wubi_identity(profile: &TF_INPUTPROCESSORPROFILE) -> bool {
        profile.dwProfileType == TF_PROFILETYPE_INPUTPROCESSOR
            && profile.langid == WUBI_LANGUAGE
            && profile.clsid == WUBI_CLSID
            && profile.guidProfile == WUBI_PROFILE
    }

    fn same_identity(left: &TF_INPUTPROCESSORPROFILE, right: &TF_INPUTPROCESSORPROFILE) -> bool {
        if left.dwProfileType != right.dwProfileType || left.langid != right.langid {
            return false;
        }
        if left.dwProfileType == TF_PROFILETYPE_INPUTPROCESSOR {
            left.clsid == right.clsid && left.guidProfile == right.guidProfile
        } else if left.dwProfileType == TF_PROFILETYPE_KEYBOARDLAYOUT {
            left.hkl == right.hkl
        } else {
            false
        }
    }

    fn print_snapshot(label: &str, value: &Snapshot) {
        println!(
            "{label}.wubi=enabled:{},active:{},flags:0x{:08X}",
            is_enabled(&value.wubi),
            is_active(&value.wubi),
            value.wubi.dwFlags
        );
        match value.active {
            Some(active) => println!(
                "{label}.active=type:{},lang:0x{:04X},clsid:{:?},profile:{:?},category:{:?},hkl_substitute:{:?},caps:0x{:08X},hkl:{:?}",
                active.dwProfileType,
                active.langid,
                active.clsid,
                active.guidProfile,
                active.catid,
                active.hklSubstitute,
                active.dwCaps,
                active.hkl
            ),
            None => println!("{label}.active=none (GetActiveProfile returned S_FALSE)"),
        }
    }

    struct RestoreGuard {
        manager: ITfInputProcessorProfileMgr,
        baseline: Snapshot,
        armed: bool,
    }

    impl RestoreGuard {
        fn new(manager: ITfInputProcessorProfileMgr, baseline: Snapshot) -> Self {
            Self {
                manager,
                baseline,
                armed: true,
            }
        }

        fn restore(&mut self) -> Result<(), SpikeError> {
            restore_baseline(&self.manager, self.baseline)?;
            self.armed = false;
            Ok(())
        }
    }

    impl Drop for RestoreGuard {
        fn drop(&mut self) {
            if self.armed
                && let Err(error) = restore_baseline(&self.manager, self.baseline)
            {
                eprintln!(
                    "RESTORATION FAILURE: {error}; manually select the original keyboard profile shown in the baseline"
                );
            }
        }
    }

    fn restore_baseline(
        manager: &ITfInputProcessorProfileMgr,
        baseline: Snapshot,
    ) -> Result<(), SpikeError> {
        let current_wubi_active = if baseline.active.is_none() {
            is_active(&snapshot(manager)?.wubi)
        } else {
            false
        };
        match restore_action(baseline.active.is_some(), current_wubi_active) {
            RestoreAction::ActivateBaseline => {
                let profile = baseline.active.as_ref().ok_or_else(|| {
                    SpikeError::new("restore TSF baseline", "active baseline is unavailable")
                })?;
                activate_profile(manager, profile)?;
            }
            RestoreAction::DeactivateWubi => deactivate_wubi(manager)?,
            RestoreAction::VerifyOnly => {}
        }
        let restored = poll_until(
            "restore TSF baseline",
            Duration::from_secs(5),
            Duration::from_millis(100),
            || snapshot(manager),
            |observed| {
                let active_matches = match (baseline.active, observed.active) {
                    (Some(expected), Some(actual)) => same_identity(&expected, &actual),
                    (None, None) => true,
                    _ => false,
                };
                active_matches
                    && is_active(&observed.wubi) == is_active(&baseline.wubi)
                    && is_enabled(&observed.wubi) == is_enabled(&baseline.wubi)
            },
        )?;
        print_snapshot("restored", &restored);
        Ok(())
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum RestoreAction {
        ActivateBaseline,
        DeactivateWubi,
        VerifyOnly,
    }

    fn restore_action(baseline_has_active: bool, current_wubi_active: bool) -> RestoreAction {
        if baseline_has_active {
            RestoreAction::ActivateBaseline
        } else if current_wubi_active {
            RestoreAction::DeactivateWubi
        } else {
            RestoreAction::VerifyOnly
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{Exercise, RestoreAction, exercise_for, restore_action, same_identity};
        use windows::Win32::UI::TextServices::{
            TF_INPUTPROCESSORPROFILE, TF_PROFILETYPE_INPUTPROCESSOR, TF_PROFILETYPE_KEYBOARDLAYOUT,
        };
        use windows::core::GUID;

        #[test]
        fn active_profile_uses_deactivation_path() {
            assert_eq!(exercise_for(true), Exercise::DeactivateThenRestore);
        }

        #[test]
        fn inactive_profile_uses_activation_path() {
            assert_eq!(
                exercise_for(false),
                Exercise::ActivateThenDeactivateThenRestore
            );
        }

        #[test]
        fn identity_uses_fields_owned_by_each_profile_shape() {
            let mut input_processor = TF_INPUTPROCESSORPROFILE {
                dwProfileType: TF_PROFILETYPE_INPUTPROCESSOR,
                langid: 0x0804,
                clsid: GUID::from_u128(1),
                guidProfile: GUID::from_u128(2),
                ..Default::default()
            };
            let mut equivalent = input_processor;
            equivalent.hkl.0 = 1usize as *mut _;
            assert!(same_identity(&input_processor, &equivalent));
            equivalent.guidProfile = GUID::from_u128(3);
            assert!(!same_identity(&input_processor, &equivalent));

            input_processor.dwProfileType = TF_PROFILETYPE_KEYBOARDLAYOUT;
            input_processor.hkl.0 = 2usize as *mut _;
            equivalent = input_processor;
            equivalent.clsid = GUID::from_u128(4);
            assert!(same_identity(&input_processor, &equivalent));
            equivalent.hkl.0 = 3usize as *mut _;
            assert!(!same_identity(&input_processor, &equivalent));
        }

        #[test]
        fn restoration_decision_handles_active_and_no_active_baselines() {
            assert_eq!(restore_action(true, false), RestoreAction::ActivateBaseline);
            assert_eq!(restore_action(false, true), RestoreAction::DeactivateWubi);
            assert_eq!(restore_action(false, false), RestoreAction::VerifyOnly);
        }
    }
}
