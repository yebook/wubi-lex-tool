#[cfg(windows)]
mod support;

#[cfg(not(windows))]
fn main() {
    eprintln!("acl_owner_spike is supported only on Windows");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    if let Err(error) = windows_spike::run() {
        eprintln!("ACL OWNER SPIKE FAILED: {error}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod windows_spike {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::ptr;
    use std::time::{SystemTime, UNIX_EPOCH};

    use windows::Win32::Foundation::{
        ERROR_INSUFFICIENT_BUFFER, ERROR_NOT_ALL_ASSIGNED, ERROR_SUCCESS, GetLastError, HANDLE,
        HLOCAL, LocalFree, SetLastError,
    };
    use windows::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW,
        GetNamedSecurityInfoW, SDDL_REVISION_1, SE_FILE_OBJECT, SetNamedSecurityInfoW,
    };
    use windows::Win32::Security::{
        ACL, AdjustTokenPrivileges, CreateWellKnownSid, DACL_SECURITY_INFORMATION, EqualSid,
        GetSecurityDescriptorControl, LUID_AND_ATTRIBUTES, LookupAccountNameW,
        LookupPrivilegeValueW, OBJECT_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
        PSECURITY_DESCRIPTOR, PSID, SE_PRIVILEGE_ENABLED, SE_RESTORE_NAME, SE_TAKE_OWNERSHIP_NAME,
        SID_NAME_USE, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
        WinBuiltinAdministratorsSid,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows::core::{PCWSTR, PWSTR, w};

    use super::support::{Mode, OwnedHandle, SpikeError, is_elevated, require_elevated};

    const INFO: OBJECT_SECURITY_INFORMATION =
        OBJECT_SECURITY_INFORMATION(OWNER_SECURITY_INFORMATION.0 | DACL_SECURITY_INFORMATION.0);

    pub fn run() -> Result<(), SpikeError> {
        let mode = super::support::parse_mode()?;
        println!("mode={mode:?}");
        println!("target_policy=one create_new file below %TEMP%/wubilex-risk-spikes/acl-owner");
        println!("elevated={}", is_elevated()?);
        println!("planned_owner_round_trip=TrustedInstaller -> Administrators -> TrustedInstaller");
        println!(
            "planned_cleanup=restore creation owner/DACL, restore token privileges, delete file"
        );
        if mode == Mode::DryRun {
            println!("verdict=DRY-RUN PASS; no file was created and no ACL was changed");
            return Ok(());
        }
        require_elevated()?;
        live_run()
    }

    fn live_run() -> Result<(), SpikeError> {
        let mut privileges = PrivilegeGuard::open()?;
        let (directory, path) = create_probe_file()?;
        let mut creation_cleanup = CreationCleanupGuard::new(directory.clone(), path.clone());
        println!("temporary_path={}", path.display());
        let cleanup_a = SecuritySnapshot::capture(&path, "capture cleanup baseline A")?;
        print_snapshot("baseline_a", &cleanup_a);

        let trusted_installer = lookup_trusted_installer()?;
        let administrators = create_administrators_sid()?;
        println!(
            "trusted_installer_sid={}",
            sid_to_string(trusted_installer.sid())?
        );
        println!(
            "administrators_sid={}",
            sid_to_string(administrators.sid())?
        );

        privileges.enable()?;
        let mut cleanup = FileCleanupGuard::new(directory, path.clone(), cleanup_a);
        creation_cleanup.disarm();
        let operation = owner_round_trip(&path, trusted_installer.sid(), administrators.sid());
        let cleanup_result = cleanup.restore();
        let privilege_result = privileges.restore();
        let post_restore_action =
            post_restore_action(cleanup_result.is_ok(), privilege_result.is_ok());
        let restoration = combine_restoration_results(cleanup_result, privilege_result);
        let cleanup_result = match (post_restore_action, restoration) {
            (PostRestoreAction::Delete, Ok(())) => cleanup.delete(),
            (PostRestoreAction::Retain, Err(error)) => {
                cleanup.retain_evidence();
                Err(error)
            }
            _ => Err(SpikeError::new(
                "ACL restoration decision",
                "inconsistent restore result",
            )),
        };
        match (operation, cleanup_result) {
            (Ok(()), Ok(())) => {
                println!("restoration=baseline A verified; privileges restored; file deleted");
                println!("verdict=LIVE PASS");
                Ok(())
            }
            (Err(primary), Ok(())) => Err(primary),
            (Ok(()), Err(restore)) => Err(restore),
            (Err(primary), Err(restore)) => Err(primary.with_restore(restore)),
        }
    }

    fn create_probe_file() -> Result<(PathBuf, PathBuf), SpikeError> {
        let directory = std::env::temp_dir()
            .join("wubilex-risk-spikes")
            .join("acl-owner");
        fs::create_dir_all(&directory).map_err(|error| {
            SpikeError::new(
                "create spike temp directory",
                format!("{}: {error}", directory.display()),
            )
        })?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| SpikeError::new("create unique temp name", error.to_string()))?
            .as_nanos();
        let path = directory.join(format!(
            "owner-round-trip-{}-{stamp}.tmp",
            std::process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                SpikeError::new(
                    "create_new spike file",
                    format!("{}: {error}", path.display()),
                )
            })?;
        if let Err(error) = file.write_all(b"wubilex ACL ownership risk spike\r\n") {
            drop(file);
            let cleanup = fs::remove_file(&path);
            return Err(SpikeError::new(
                "write spike marker",
                match cleanup {
                    Ok(()) => error.to_string(),
                    Err(cleanup_error) => format!(
                        "{error}; CLEANUP FAILURE for {}: {cleanup_error}",
                        path.display()
                    ),
                },
            ));
        }
        drop(file);
        let mut cleanup = CreationCleanupGuard::new(directory.clone(), path.clone());

        let canonical_dir = directory
            .canonicalize()
            .map_err(|error| SpikeError::new("canonicalize spike directory", error.to_string()))?;
        let canonical_file = path
            .canonicalize()
            .map_err(|error| SpikeError::new("canonicalize spike file", error.to_string()))?;
        if canonical_file.parent() != Some(canonical_dir.as_path()) {
            return Err(SpikeError::new(
                "validate spike target",
                format!(
                    "{} is outside {}",
                    canonical_file.display(),
                    canonical_dir.display()
                ),
            ));
        }
        cleanup.disarm();
        Ok((directory, canonical_file))
    }

    fn owner_round_trip(
        path: &Path,
        trusted_installer: PSID,
        administrators: PSID,
    ) -> Result<(), SpikeError> {
        set_owner(path, trusted_installer, "set owner TrustedInstaller")?;
        let baseline_b = SecuritySnapshot::capture(path, "capture verification baseline B")?;
        require_owner(
            &baseline_b,
            trusted_installer,
            "verify TrustedInstaller baseline B",
        )?;
        print_snapshot("baseline_b", &baseline_b);

        set_owner(path, administrators, "set owner Administrators")?;
        let admin = SecuritySnapshot::capture(path, "capture Administrators owner")?;
        require_owner(&admin, administrators, "verify Administrators owner")?;
        require_same_dacl(
            &baseline_b,
            &admin,
            "verify unchanged DACL after Administrators",
        )?;
        print_snapshot("administrators", &admin);

        set_owner(path, baseline_b.owner(), "restore owner TrustedInstaller")?;
        let restored_b = SecuritySnapshot::capture(path, "capture restored baseline B")?;
        require_owner(
            &restored_b,
            baseline_b.owner(),
            "verify restored TrustedInstaller owner",
        )?;
        require_same_semantics(&baseline_b, &restored_b, "verify baseline B semantics")?;
        print_snapshot("restored_b", &restored_b);
        println!("round_trip=verified TrustedInstaller -> Administrators -> TrustedInstaller");
        Ok(())
    }

    fn set_owner(path: &Path, owner: PSID, stage: &'static str) -> Result<(), SpikeError> {
        let wide_path = path_wide(path);
        let code = unsafe {
            SetNamedSecurityInfoW(
                PCWSTR(wide_path.as_ptr()),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION,
                Some(owner),
                None,
                None,
                None,
            )
        };
        win32_result(stage, code)
    }

    fn restore_snapshot(path: &Path, snapshot: &SecuritySnapshot) -> Result<(), SpikeError> {
        let wide_path = path_wide(path);
        let code = unsafe {
            SetNamedSecurityInfoW(
                PCWSTR(wide_path.as_ptr()),
                SE_FILE_OBJECT,
                INFO,
                Some(snapshot.owner()),
                None,
                snapshot.dacl(),
                None,
            )
        };
        win32_result("restore cleanup baseline A owner/DACL", code)?;
        let observed = SecuritySnapshot::capture(path, "verify cleanup baseline A")?;
        require_same_semantics(snapshot, &observed, "verify cleanup baseline A semantics")
    }

    fn require_owner(
        observed: &SecuritySnapshot,
        expected: PSID,
        stage: &'static str,
    ) -> Result<(), SpikeError> {
        unsafe { EqualSid(observed.owner(), expected) }
            .map_err(|error| SpikeError::windows(stage, error))
    }

    fn require_same_dacl(
        expected: &SecuritySnapshot,
        actual: &SecuritySnapshot,
        stage: &'static str,
    ) -> Result<(), SpikeError> {
        if security_evidence_equal(
            expected.dacl_sddl(),
            expected.dacl_present,
            expected.control,
            actual.dacl_sddl(),
            actual.dacl_present,
            actual.control,
        ) {
            Ok(())
        } else {
            Err(SpikeError::new(
                stage,
                format!("expected {}, observed {}", expected.sddl, actual.sddl),
            ))
        }
    }

    fn require_same_semantics(
        expected: &SecuritySnapshot,
        actual: &SecuritySnapshot,
        stage: &'static str,
    ) -> Result<(), SpikeError> {
        require_owner(actual, expected.owner(), stage)?;
        if security_evidence_equal(
            &expected.sddl,
            expected.dacl_present,
            expected.control,
            &actual.sddl,
            actual.dacl_present,
            actual.control,
        ) {
            Ok(())
        } else {
            Err(SpikeError::new(
                stage,
                format!("expected {}, observed {}", expected.sddl, actual.sddl),
            ))
        }
    }

    fn win32_result(
        stage: &'static str,
        code: windows::Win32::Foundation::WIN32_ERROR,
    ) -> Result<(), SpikeError> {
        if code == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(SpikeError::win32(stage, code))
        }
    }

    fn security_evidence_equal(
        expected_sddl: &str,
        expected_dacl_present: bool,
        expected_control: u16,
        actual_sddl: &str,
        actual_dacl_present: bool,
        actual_control: u16,
    ) -> bool {
        expected_sddl == actual_sddl
            && expected_dacl_present == actual_dacl_present
            && expected_control == actual_control
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum PostRestoreAction {
        Delete,
        Retain,
    }

    fn post_restore_action(cleanup_restored: bool, privileges_restored: bool) -> PostRestoreAction {
        if cleanup_restored && privileges_restored {
            PostRestoreAction::Delete
        } else {
            PostRestoreAction::Retain
        }
    }

    fn combine_restoration_results(
        cleanup: Result<(), SpikeError>,
        privileges: Result<(), SpikeError>,
    ) -> Result<(), SpikeError> {
        match (cleanup, privileges) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(cleanup_error), Err(privilege_error)) => Err(SpikeError::new(
                "ACL restoration",
                format!("{cleanup_error}; {privilege_error}"),
            )),
        }
    }

    struct SecuritySnapshot {
        descriptor: PSECURITY_DESCRIPTOR,
        owner: PSID,
        dacl: *mut ACL,
        sddl: String,
        dacl_present: bool,
        control: u16,
    }

    impl SecuritySnapshot {
        fn capture(path: &Path, stage: &'static str) -> Result<Self, SpikeError> {
            let wide_path = path_wide(path);
            let mut descriptor = PSECURITY_DESCRIPTOR::default();
            let mut owner = PSID::default();
            let mut dacl = ptr::null_mut();
            let code = unsafe {
                GetNamedSecurityInfoW(
                    PCWSTR(wide_path.as_ptr()),
                    SE_FILE_OBJECT,
                    INFO,
                    Some(&mut owner),
                    None,
                    Some(&mut dacl),
                    None,
                    &mut descriptor,
                )
            };
            win32_result(stage, code)?;
            let mut string = PWSTR::null();
            let conversion = unsafe {
                ConvertSecurityDescriptorToStringSecurityDescriptorW(
                    descriptor,
                    SDDL_REVISION_1,
                    INFO,
                    &mut string,
                    None,
                )
            };
            if let Err(error) = conversion {
                unsafe { LocalFree(Some(HLOCAL(descriptor.0))) };
                return Err(SpikeError::windows(
                    "convert security descriptor to SDDL",
                    error,
                ));
            }
            let sddl = unsafe { string.to_string() }.map_err(|error| {
                SpikeError::new("copy security descriptor SDDL", error.to_string())
            });
            unsafe { LocalFree(Some(HLOCAL(string.0.cast()))) };
            let sddl = match sddl {
                Ok(value) => value,
                Err(error) => {
                    unsafe { LocalFree(Some(HLOCAL(descriptor.0))) };
                    return Err(error);
                }
            };
            let mut control = 0u16;
            let mut revision = 0u32;
            if let Err(error) =
                unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) }
            {
                unsafe { LocalFree(Some(HLOCAL(descriptor.0))) };
                return Err(SpikeError::windows("GetSecurityDescriptorControl", error));
            }
            Ok(Self {
                descriptor,
                owner,
                dacl,
                sddl,
                dacl_present: !dacl.is_null(),
                control,
            })
        }

        fn owner(&self) -> PSID {
            self.owner
        }

        fn dacl(&self) -> Option<*const ACL> {
            (!self.dacl.is_null()).then_some(self.dacl.cast_const())
        }

        fn dacl_sddl(&self) -> &str {
            self.sddl
                .find("D:")
                .map(|index| &self.sddl[index..])
                .unwrap_or("")
        }
    }

    impl Drop for SecuritySnapshot {
        fn drop(&mut self) {
            unsafe { LocalFree(Some(HLOCAL(self.descriptor.0))) };
        }
    }

    fn print_snapshot(label: &str, snapshot: &SecuritySnapshot) {
        println!(
            "{label}=sddl:{},dacl_present:{},control:0x{:04X}",
            snapshot.sddl, snapshot.dacl_present, snapshot.control
        );
    }

    struct SidBuffer {
        words: Vec<usize>,
    }

    impl SidBuffer {
        fn with_bytes(bytes: u32) -> Result<Self, SpikeError> {
            if bytes == 0 {
                return Err(SpikeError::new(
                    "allocate SID",
                    "API returned zero SID bytes",
                ));
            }
            let word = std::mem::size_of::<usize>();
            let count = (bytes as usize).div_ceil(word);
            Ok(Self {
                words: vec![0; count],
            })
        }

        fn sid(&self) -> PSID {
            PSID(self.words.as_ptr().cast_mut().cast())
        }
    }

    fn lookup_trusted_installer() -> Result<SidBuffer, SpikeError> {
        let mut sid_size = 0u32;
        let mut domain_size = 0u32;
        let mut use_type = SID_NAME_USE::default();
        let first = unsafe {
            LookupAccountNameW(
                None,
                w!("NT SERVICE\\TrustedInstaller"),
                None,
                &mut sid_size,
                None,
                &mut domain_size,
                &mut use_type,
            )
        };
        match first {
            Ok(()) => {
                return Err(SpikeError::new(
                    "LookupAccountNameW(size TrustedInstaller)",
                    "unexpected success from null-buffer size probe",
                ));
            }
            Err(error)
                if error.code()
                    == windows::core::HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) => {}
            Err(error) => {
                return Err(SpikeError::windows(
                    "LookupAccountNameW(size TrustedInstaller)",
                    error,
                ));
            }
        }
        let sid = SidBuffer::with_bytes(sid_size)?;
        let mut domain = vec![0u16; domain_size as usize];
        unsafe {
            LookupAccountNameW(
                None,
                w!("NT SERVICE\\TrustedInstaller"),
                Some(sid.sid()),
                &mut sid_size,
                Some(PWSTR(domain.as_mut_ptr())),
                &mut domain_size,
                &mut use_type,
            )
        }
        .map_err(|error| SpikeError::windows("LookupAccountNameW(TrustedInstaller)", error))?;
        Ok(sid)
    }

    fn create_administrators_sid() -> Result<SidBuffer, SpikeError> {
        let mut size = 0u32;
        let first =
            unsafe { CreateWellKnownSid(WinBuiltinAdministratorsSid, None, None, &mut size) };
        match first {
            Ok(()) => {
                return Err(SpikeError::new(
                    "CreateWellKnownSid(size Administrators)",
                    "unexpected success from null-buffer size probe",
                ));
            }
            Err(error)
                if error.code()
                    == windows::core::HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) => {}
            Err(error) => {
                return Err(SpikeError::windows(
                    "CreateWellKnownSid(size Administrators)",
                    error,
                ));
            }
        }
        let sid = SidBuffer::with_bytes(size)?;
        unsafe {
            CreateWellKnownSid(
                WinBuiltinAdministratorsSid,
                None,
                Some(sid.sid()),
                &mut size,
            )
        }
        .map_err(|error| SpikeError::windows("CreateWellKnownSid(Administrators)", error))?;
        Ok(sid)
    }

    fn sid_to_string(sid: PSID) -> Result<String, SpikeError> {
        let mut string = PWSTR::null();
        unsafe { ConvertSidToStringSidW(sid, &mut string) }
            .map_err(|error| SpikeError::windows("ConvertSidToStringSidW", error))?;
        let result = unsafe { string.to_string() }
            .map_err(|error| SpikeError::new("copy SID string", error.to_string()));
        unsafe { LocalFree(Some(HLOCAL(string.0.cast()))) };
        result
    }

    fn path_wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    struct PrivilegeGuard {
        token: OwnedHandle,
        previous: Vec<(&'static str, TOKEN_PRIVILEGES)>,
        armed: bool,
    }

    impl PrivilegeGuard {
        fn open() -> Result<Self, SpikeError> {
            let mut token = HANDLE::default();
            unsafe {
                OpenProcessToken(
                    GetCurrentProcess(),
                    TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                    &mut token,
                )
            }
            .map_err(|error| SpikeError::windows("OpenProcessToken(privileges)", error))?;
            Ok(Self {
                token: OwnedHandle::new(token),
                previous: Vec::new(),
                armed: true,
            })
        }

        fn enable(&mut self) -> Result<(), SpikeError> {
            self.enable_one(
                SE_RESTORE_NAME,
                "SeRestorePrivilege",
                "enable SeRestorePrivilege",
            )?;
            self.enable_one(
                SE_TAKE_OWNERSHIP_NAME,
                "SeTakeOwnershipPrivilege",
                "enable SeTakeOwnershipPrivilege",
            )
        }

        fn enable_one(
            &mut self,
            name: windows::core::PCWSTR,
            evidence_name: &'static str,
            stage: &'static str,
        ) -> Result<(), SpikeError> {
            let mut luid = windows::Win32::Foundation::LUID::default();
            unsafe { LookupPrivilegeValueW(None, name, &mut luid) }
                .map_err(|error| SpikeError::windows(stage, error))?;
            let requested = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };
            let mut previous = TOKEN_PRIVILEGES::default();
            let mut returned = 0u32;
            unsafe { SetLastError(ERROR_SUCCESS) };
            unsafe {
                AdjustTokenPrivileges(
                    self.token.raw(),
                    false,
                    Some(&requested),
                    std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                    Some(&mut previous),
                    Some(&mut returned),
                )
            }
            .map_err(|error| SpikeError::windows(stage, error))?;
            let last = unsafe { GetLastError() };
            if last == ERROR_NOT_ALL_ASSIGNED {
                return Err(SpikeError::win32(stage, last));
            }
            if last != ERROR_SUCCESS {
                return Err(SpikeError::win32(stage, last));
            }
            self.previous.push((evidence_name, previous));
            Ok(())
        }

        fn restore(&mut self) -> Result<(), SpikeError> {
            let mut errors = Vec::new();
            for (name, previous) in self.previous.iter().rev() {
                unsafe { SetLastError(ERROR_SUCCESS) };
                let result = unsafe {
                    AdjustTokenPrivileges(self.token.raw(), false, Some(previous), 0, None, None)
                };
                if let Err(error) = result {
                    errors.push(format!(
                        "{name}: {}",
                        SpikeError::windows("AdjustTokenPrivileges(restore)", error)
                    ));
                    continue;
                }
                let last = unsafe { GetLastError() };
                if last != ERROR_SUCCESS {
                    errors.push(format!(
                        "{name}: {}",
                        SpikeError::win32("AdjustTokenPrivileges(restore)", last)
                    ));
                }
            }
            if errors.is_empty() {
                self.armed = false;
                Ok(())
            } else {
                Err(SpikeError::new(
                    "restore token privileges",
                    errors.join("; "),
                ))
            }
        }
    }

    impl Drop for PrivilegeGuard {
        fn drop(&mut self) {
            if self.armed
                && let Err(error) = self.restore()
            {
                eprintln!(
                    "RESTORATION FAILURE: {error}; close this elevated process before continuing"
                );
            }
        }
    }

    struct CreationCleanupGuard {
        directory: PathBuf,
        path: PathBuf,
        armed: bool,
    }

    impl CreationCleanupGuard {
        fn new(directory: PathBuf, path: PathBuf) -> Self {
            Self {
                directory,
                path,
                armed: true,
            }
        }

        fn disarm(&mut self) {
            self.armed = false;
        }
    }

    impl Drop for CreationCleanupGuard {
        fn drop(&mut self) {
            if self.armed && self.path.exists() {
                if let Err(error) = fs::remove_file(&self.path) {
                    eprintln!(
                        "CLEANUP FAILURE: could not delete unmodified spike file {}: {error}",
                        self.path.display()
                    );
                } else {
                    let _ = fs::remove_dir(&self.directory);
                }
            }
        }
    }

    struct FileCleanupGuard {
        directory: PathBuf,
        path: PathBuf,
        baseline: SecuritySnapshot,
        armed: bool,
        delete_after_restore: bool,
    }

    impl FileCleanupGuard {
        fn new(directory: PathBuf, path: PathBuf, baseline: SecuritySnapshot) -> Self {
            Self {
                directory,
                path,
                baseline,
                armed: true,
                delete_after_restore: true,
            }
        }

        fn restore(&mut self) -> Result<(), SpikeError> {
            restore_snapshot(&self.path, &self.baseline)
        }

        fn delete(&mut self) -> Result<(), SpikeError> {
            fs::remove_file(&self.path).map_err(|error| {
                SpikeError::new(
                    "delete restored spike file",
                    format!("{}: {error}", self.path.display()),
                )
            })?;
            let _ = fs::remove_dir(&self.directory);
            self.armed = false;
            Ok(())
        }

        fn retain_evidence(&mut self) {
            self.delete_after_restore = false;
            eprintln!(
                "RESTORATION FAILURE: retained evidence at {}",
                self.path.display()
            );
        }
    }

    impl Drop for FileCleanupGuard {
        fn drop(&mut self) {
            if self.armed && self.path.exists() {
                match restore_snapshot(&self.path, &self.baseline) {
                    Ok(()) if self.delete_after_restore => {
                        if let Err(error) = fs::remove_file(&self.path) {
                            eprintln!(
                                "RESTORATION FAILURE: restored ACL but could not delete {}: {error}",
                                self.path.display()
                            );
                        }
                    }
                    Ok(()) => eprintln!("retained evidence at {}", self.path.display()),
                    Err(error) => eprintln!(
                        "RESTORATION FAILURE: {error}; retained evidence at {}",
                        self.path.display()
                    ),
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{PostRestoreAction, post_restore_action, security_evidence_equal};

        #[test]
        fn security_evidence_requires_sddl_presence_and_control_to_match() {
            assert!(security_evidence_equal(
                "D:AI(A;;FA;;;SY)",
                true,
                0x8404,
                "D:AI(A;;FA;;;SY)",
                true,
                0x8404
            ));
            assert!(!security_evidence_equal(
                "D:AI(A;;FA;;;SY)",
                true,
                0x8404,
                "D:AI(A;;FR;;;SY)",
                true,
                0x8404
            ));
            assert!(!security_evidence_equal(
                "D:AI(A;;FA;;;SY)",
                true,
                0x8404,
                "D:AI(A;;FA;;;SY)",
                false,
                0x8404
            ));
            assert!(!security_evidence_equal(
                "D:AI(A;;FA;;;SY)",
                true,
                0x8404,
                "D:AI(A;;FA;;;SY)",
                true,
                0x9404
            ));
        }

        #[test]
        fn file_is_deleted_only_after_both_restorations_succeed() {
            assert_eq!(post_restore_action(true, true), PostRestoreAction::Delete);
            assert_eq!(post_restore_action(false, true), PostRestoreAction::Retain);
            assert_eq!(post_restore_action(true, false), PostRestoreAction::Retain);
            assert_eq!(post_restore_action(false, false), PostRestoreAction::Retain);
        }
    }
}
