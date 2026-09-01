//! Direct filesystem operations used by application-owned atomic persistence.

use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

use thiserror::Error;

/// A staged native filesystem failure with stable machine-readable evidence.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{stage} failed with code {code}: {message}")]
pub struct NativeFileError {
    /// Native operation stage.
    pub stage: &'static str,
    /// Win32 code when available, otherwise zero.
    pub code: u32,
    /// Standard-library error classification for higher layers.
    pub kind: io::ErrorKind,
    /// Readable native or standard-library detail.
    pub message: String,
}

impl NativeFileError {
    fn from_io(stage: &'static str, error: io::Error) -> Self {
        Self {
            stage,
            code: error
                .raw_os_error()
                .and_then(|code| u32::try_from(code).ok())
                .unwrap_or(0),
            kind: error.kind(),
            message: error.to_string(),
        }
    }

    #[cfg(windows)]
    fn from_windows(stage: &'static str, error: windows::core::Error) -> Self {
        let hresult = error.code().0 as u32;
        let code = if (hresult >> 16) & 0x1fff == 7 {
            hresult & 0xffff
        } else {
            hresult
        };
        Self {
            stage,
            code,
            kind: i32::try_from(code)
                .ok()
                .map(io::Error::from_raw_os_error)
                .map(|error| error.kind())
                .unwrap_or(io::ErrorKind::Other),
            message: error.to_string(),
        }
    }
}

/// Observed namespace topology after a failed replacement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplacementDisposition {
    /// The target and staging names are documented to remain unchanged.
    NamesUnchanged,
    /// The old target may have moved to the requested backup name.
    OriginalAtBackup,
    /// The native API does not provide a stronger documented topology.
    Unknown,
}

/// A replacement failure plus the namespace disposition required for recovery.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{source}")]
pub struct ReplaceFileError {
    /// Native failure evidence.
    pub source: NativeFileError,
    /// Documented path topology for the returned code.
    pub disposition: ReplacementDisposition,
}

/// Exclusively creates one staging file. Ownership begins only on success.
pub fn create_staging_exclusive(path: &Path) -> Result<File, NativeFileError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    options
        .open(path)
        .map_err(|error| NativeFileError::from_io("create_staging_exclusive", error))
}

/// Installs a staged file without replacing a concurrently-created target.
pub fn install_new_noreplace(staging: &Path, target: &Path) -> Result<(), NativeFileError> {
    move_noreplace(staging, target, "install_new_noreplace")
}

/// Restores a backup without replacing a concurrently-created target.
pub fn restore_backup_noreplace(backup: &Path, target: &Path) -> Result<(), NativeFileError> {
    move_noreplace(backup, target, "restore_backup_noreplace")
}

/// Moves malformed input aside without replacing an existing evidence file.
pub fn preserve_corrupt_noreplace(
    source: &Path,
    destination: &Path,
) -> Result<(), NativeFileError> {
    move_noreplace(source, destination, "preserve_corrupt_noreplace")
}

/// Replaces an existing file while preserving the previous target as a unique backup.
pub fn replace_file_with_backup(
    target: &Path,
    staging: &Path,
    backup: &Path,
) -> Result<(), ReplaceFileError> {
    #[cfg(windows)]
    {
        use std::{os::windows::ffi::OsStrExt, ptr};
        use windows::{
            Win32::Storage::FileSystem::{REPLACE_FILE_FLAGS, ReplaceFileW},
            core::PCWSTR,
        };

        fn wide(path: &Path) -> Vec<u16> {
            path.as_os_str().encode_wide().chain([0]).collect()
        }

        let target = wide(target);
        let staging = wide(staging);
        let backup = wide(backup);
        let result = unsafe {
            ReplaceFileW(
                PCWSTR(target.as_ptr()),
                PCWSTR(staging.as_ptr()),
                PCWSTR(backup.as_ptr()),
                REPLACE_FILE_FLAGS(0),
                Some(ptr::null()),
                Some(ptr::null()),
            )
        };
        result.map_err(|error| {
            let source = NativeFileError::from_windows("replace_file_with_backup", error);
            let disposition = match source.code {
                1175 | 1176 => ReplacementDisposition::NamesUnchanged,
                1177 => ReplacementDisposition::OriginalAtBackup,
                _ => ReplacementDisposition::Unknown,
            };
            ReplaceFileError {
                source,
                disposition,
            }
        })
    }

    #[cfg(not(windows))]
    {
        std::fs::rename(target, backup).map_err(|error| ReplaceFileError {
            source: NativeFileError::from_io("replace_file_with_backup", error),
            disposition: ReplacementDisposition::NamesUnchanged,
        })?;
        if let Err(error) = std::fs::rename(staging, target) {
            return Err(ReplaceFileError {
                source: NativeFileError::from_io("replace_file_with_backup", error),
                disposition: ReplacementDisposition::OriginalAtBackup,
            });
        }
        Ok(())
    }
}

#[cfg(windows)]
fn move_noreplace(
    source: &Path,
    destination: &Path,
    stage: &'static str,
) -> Result<(), NativeFileError> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW},
        core::PCWSTR,
    };

    let source = source
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect::<Vec<_>>();
    unsafe {
        MoveFileExW(
            PCWSTR(source.as_ptr()),
            PCWSTR(destination.as_ptr()),
            MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(|error| NativeFileError::from_windows(stage, error))
}

#[cfg(not(windows))]
fn move_noreplace(
    source: &Path,
    destination: &Path,
    stage: &'static str,
) -> Result<(), NativeFileError> {
    std::fs::hard_link(source, destination)
        .and_then(|()| std::fs::remove_file(source))
        .map_err(|error| NativeFileError::from_io(stage, error))
}

#[cfg(all(test, windows))]
mod tests {
    use super::{install_new_noreplace, replace_file_with_backup};
    use std::{fs, io::Write};

    #[test]
    fn native_install_and_replace_preserve_complete_files_and_backup() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target = directory.path().join("config.toml");
        let first = directory.path().join("first.tmp");
        let second = directory.path().join("second.tmp");
        let backup = directory.path().join("config.backup.toml");

        let mut first_file = super::create_staging_exclusive(&first).expect("first staging");
        first_file.write_all(b"first\n").expect("write first");
        first_file.sync_all().expect("sync first");
        drop(first_file);
        install_new_noreplace(&first, &target).expect("initial install");

        let mut second_file = super::create_staging_exclusive(&second).expect("second staging");
        second_file.write_all(b"second\n").expect("write second");
        second_file.sync_all().expect("sync second");
        drop(second_file);
        replace_file_with_backup(&target, &second, &backup).expect("replacement");

        assert_eq!(fs::read(&target).expect("target bytes"), b"second\n");
        assert_eq!(fs::read(&backup).expect("backup bytes"), b"first\n");
        assert!(!second.exists());
    }
}
