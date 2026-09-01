//! Injectable configuration filesystem boundary and its Windows implementation.

use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use thiserror::Error;
use wubilex_winime::filesystem::{NativeFileError, ReplaceFileError};

use super::codec::MAX_CONFIG_BYTES;

/// Staged filesystem failure safe to attach to an application error.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{stage} failed for {path}: {message}")]
pub struct ConfigIoError {
    pub stage: &'static str,
    pub path: PathBuf,
    pub kind: io::ErrorKind,
    pub code: u32,
    pub message: String,
}

impl ConfigIoError {
    fn from_io(stage: &'static str, path: &Path, error: io::Error) -> Self {
        Self {
            stage,
            path: path.to_path_buf(),
            kind: error.kind(),
            code: error
                .raw_os_error()
                .and_then(|code| u32::try_from(code).ok())
                .unwrap_or(0),
            message: error.to_string(),
        }
    }

    fn from_native(path: &Path, error: NativeFileError) -> Self {
        Self {
            stage: error.stage,
            path: path.to_path_buf(),
            kind: error.kind,
            code: error.code,
            message: error.message,
        }
    }

    pub(crate) fn detail(&self) -> String {
        format!(
            "stage={}; path={}; kind={:?}; code={}; error={}",
            self.stage,
            self.path.display(),
            self.kind,
            self.code,
            self.message
        )
    }
}

/// File operations exposed as separate stages for deterministic fault injection.
pub trait ConfigFileOps: Send + Sync + 'static {
    type Pending: Write + Send;

    fn create_dir_all(&self, path: &Path) -> Result<(), ConfigIoError>;
    fn path_exists(&self, path: &Path) -> Result<bool, ConfigIoError>;
    fn path_is_directory(&self, path: &Path) -> Result<bool, ConfigIoError>;
    fn canonicalize_existing(&self, path: &Path) -> Result<Option<PathBuf>, ConfigIoError>;
    fn read_bounded(&self, path: &Path) -> Result<Vec<u8>, ConfigIoError>;
    fn list_owned_backups(&self, directory: &Path) -> Result<Vec<PathBuf>, ConfigIoError>;
    fn create_temp_exclusive(&self, path: &Path) -> Result<Self::Pending, ConfigIoError>;
    fn sync(&self, path: &Path, pending: &mut Self::Pending) -> Result<(), ConfigIoError>;
    fn close(&self, path: &Path, pending: Self::Pending) -> Result<(), ConfigIoError>;
    fn replace_with_backup(
        &self,
        target: &Path,
        staging: &Path,
        backup: &Path,
    ) -> Result<(), ReplaceFileError>;
    fn install_new_noreplace(&self, staging: &Path, target: &Path) -> Result<(), ConfigIoError>;
    fn restore_backup_noreplace(&self, backup: &Path, target: &Path) -> Result<(), ConfigIoError>;
    fn preserve_corrupt_noreplace(
        &self,
        source: &Path,
        destination: &Path,
    ) -> Result<(), ConfigIoError>;
    fn remove_owned(&self, path: &Path) -> Result<(), ConfigIoError>;
}

/// Production file operations backed by Rust I/O and the typed Windows adapter.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowsConfigFileOps;

impl ConfigFileOps for WindowsConfigFileOps {
    type Pending = File;

    fn create_dir_all(&self, path: &Path) -> Result<(), ConfigIoError> {
        fs::create_dir_all(path)
            .map_err(|error| ConfigIoError::from_io("create_config_directory", path, error))
    }

    fn path_exists(&self, path: &Path) -> Result<bool, ConfigIoError> {
        match fs::metadata(path) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(ConfigIoError::from_io("inspect_path", path, error)),
        }
    }

    fn path_is_directory(&self, path: &Path) -> Result<bool, ConfigIoError> {
        match fs::metadata(path) {
            Ok(metadata) => Ok(metadata.is_dir()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(ConfigIoError::from_io("inspect_path_type", path, error)),
        }
    }

    fn canonicalize_existing(&self, path: &Path) -> Result<Option<PathBuf>, ConfigIoError> {
        match fs::canonicalize(path) {
            Ok(path) => Ok(Some(path)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(ConfigIoError::from_io(
                "canonicalize_config_path",
                path,
                error,
            )),
        }
    }

    fn read_bounded(&self, path: &Path) -> Result<Vec<u8>, ConfigIoError> {
        let file =
            File::open(path).map_err(|error| ConfigIoError::from_io("open_config", path, error))?;
        let mut bytes = Vec::new();
        file.take((MAX_CONFIG_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| ConfigIoError::from_io("read_config", path, error))?;
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(ConfigIoError {
                stage: "read_config_limit",
                path: path.to_path_buf(),
                kind: io::ErrorKind::InvalidData,
                code: 0,
                message: format!("configuration exceeds {MAX_CONFIG_BYTES} bytes"),
            });
        }
        Ok(bytes)
    }

    fn list_owned_backups(&self, directory: &Path) -> Result<Vec<PathBuf>, ConfigIoError> {
        let entries = fs::read_dir(directory)
            .map_err(|error| ConfigIoError::from_io("list_config_backups", directory, error))?;
        let mut backups = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| {
                ConfigIoError::from_io("read_config_backup_entry", directory, error)
            })?;
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("config.backup-") && name.ends_with(".toml") {
                let modified = entry
                    .metadata()
                    .map_err(|error| {
                        ConfigIoError::from_io("read_config_backup_metadata", &entry_path, error)
                    })?
                    .modified()
                    .map_err(|error| {
                        ConfigIoError::from_io(
                            "read_config_backup_modified_time",
                            &entry_path,
                            error,
                        )
                    })?;
                backups.push((modified, entry_path));
            }
        }
        backups.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));
        Ok(backups.into_iter().map(|(_, path)| path).collect())
    }

    fn create_temp_exclusive(&self, path: &Path) -> Result<Self::Pending, ConfigIoError> {
        wubilex_winime::filesystem::create_staging_exclusive(path)
            .map_err(|error| ConfigIoError::from_native(path, error))
    }

    fn sync(&self, path: &Path, pending: &mut Self::Pending) -> Result<(), ConfigIoError> {
        pending
            .sync_all()
            .map_err(|error| ConfigIoError::from_io("sync_staging", path, error))
    }

    fn close(&self, _path: &Path, pending: Self::Pending) -> Result<(), ConfigIoError> {
        drop(pending);
        Ok(())
    }

    fn replace_with_backup(
        &self,
        target: &Path,
        staging: &Path,
        backup: &Path,
    ) -> Result<(), ReplaceFileError> {
        wubilex_winime::filesystem::replace_file_with_backup(target, staging, backup)
    }

    fn install_new_noreplace(&self, staging: &Path, target: &Path) -> Result<(), ConfigIoError> {
        wubilex_winime::filesystem::install_new_noreplace(staging, target)
            .map_err(|error| ConfigIoError::from_native(target, error))
    }

    fn restore_backup_noreplace(&self, backup: &Path, target: &Path) -> Result<(), ConfigIoError> {
        wubilex_winime::filesystem::restore_backup_noreplace(backup, target)
            .map_err(|error| ConfigIoError::from_native(target, error))
    }

    fn preserve_corrupt_noreplace(
        &self,
        source: &Path,
        destination: &Path,
    ) -> Result<(), ConfigIoError> {
        wubilex_winime::filesystem::preserve_corrupt_noreplace(source, destination)
            .map_err(|error| ConfigIoError::from_native(destination, error))
    }

    fn remove_owned(&self, path: &Path) -> Result<(), ConfigIoError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(ConfigIoError::from_io("cleanup_owned_file", path, error)),
        }
    }
}
