//! Versioned application configuration and transactional persistence.

mod codec;
mod migration;
mod model;
mod storage;

use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::{Deserialize, Serialize};
use specta::Type;
use wubilex_winime::filesystem::{ReplaceFileError, ReplacementDisposition};

use crate::error::{AppError, AppErrorCode, AppErrorKind};

pub use model::{
    AppConfig, AppLocale, BindingOverride, CloseAction, Density, KeymapConfig, ThemePreference,
    UiConfig, WindowBounds, WindowConfig,
};
pub use storage::WindowsConfigFileOps;
use storage::{ConfigFileOps, ConfigIoError};

const CONFIG_FILE_NAME: &str = "config.toml";
const MAX_CONFIG_NOTICES: usize = 8;
const UNIQUE_PATH_ATTEMPTS: usize = 16;
static PATH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Whether routine configuration writes are currently permitted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConfigPersistence {
    Ready,
    ReadOnly,
}

/// Stable startup and recovery notice identifiers.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConfigNoticeCode {
    CorruptConfigPreserved,
    BackupRecovered,
    UnsupportedVersion,
    PersistenceUnavailable,
    ReplacementRecoveryFailed,
}

/// Bounded visible evidence for degraded configuration behavior.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigNotice {
    pub code: ConfigNoticeCode,
    pub message: String,
    pub detail: Option<String>,
}

impl ConfigNotice {
    fn new(code: ConfigNoticeCode, message: &str, detail: Option<String>) -> Self {
        Self {
            code,
            message: message.to_owned(),
            detail: detail.map(|value| value.chars().take(1_024).collect()),
        }
    }
}

/// Complete authoritative configuration state.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSnapshot {
    #[specta(type = specta_typescript::Number)]
    pub revision: u64,
    pub config: AppConfig,
    pub persistence: ConfigPersistence,
    pub notices: Vec<ConfigNotice>,
}

/// Complete config group accepted by update and reset commands.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConfigGroup {
    Window,
    Ui,
    Keymap,
    All,
}

/// OS path request generated for import and export commands.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigPathRequest {
    pub path: String,
}

/// Result of exporting the canonical configuration document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigExportResult {
    pub path: String,
    #[specta(type = specta_typescript::Number)]
    pub bytes_written: u64,
}

#[derive(Debug)]
struct ConfigData {
    snapshot: ConfigSnapshot,
}

/// Mutex-serialized configuration service with an injectable filesystem boundary.
#[derive(Debug)]
pub struct ConfigService<F: ConfigFileOps> {
    directory: PathBuf,
    live_path: PathBuf,
    ops: F,
    inner: Mutex<ConfigData>,
}

/// Production service managed by Tauri.
pub type AppConfigService = ConfigService<WindowsConfigFileOps>;

impl<F: ConfigFileOps> ConfigService<F> {
    /// Loads startup state without allowing a config failure to abort application startup.
    pub fn load(directory: PathBuf, ops: F) -> Self {
        let live_path = directory.join(CONFIG_FILE_NAME);
        let snapshot = load_snapshot(&ops, &directory, &live_path);
        Self {
            directory,
            live_path,
            ops,
            inner: Mutex::new(ConfigData { snapshot }),
        }
    }

    /// Creates a read-only default service when the application config path is unavailable.
    pub fn unavailable(detail: impl Into<String>, ops: F) -> Self {
        let notice = ConfigNotice::new(
            ConfigNoticeCode::PersistenceUnavailable,
            "无法访问配置目录，已使用只读默认配置。",
            Some(detail.into()),
        );
        Self {
            directory: PathBuf::new(),
            live_path: PathBuf::new(),
            ops,
            inner: Mutex::new(ConfigData {
                snapshot: snapshot(
                    AppConfig::default(),
                    ConfigPersistence::ReadOnly,
                    vec![notice],
                ),
            }),
        }
    }

    pub fn snapshot(&self) -> Result<ConfigSnapshot, AppError> {
        Ok(self.lock()?.snapshot.clone())
    }

    pub fn update_window(&self, window: WindowConfig) -> Result<ConfigSnapshot, AppError> {
        self.update(false, move |config| config.window = window)
    }

    pub fn update_ui(&self, ui: UiConfig) -> Result<ConfigSnapshot, AppError> {
        self.update(false, move |config| config.ui = ui)
    }

    pub fn update_keymap(&self, keymap: KeymapConfig) -> Result<ConfigSnapshot, AppError> {
        self.update(false, move |config| config.keymap = keymap)
    }

    pub fn restore_defaults(&self, group: ConfigGroup) -> Result<ConfigSnapshot, AppError> {
        self.update(false, move |config| {
            let defaults = AppConfig::default();
            match group {
                ConfigGroup::Window => config.window = defaults.window,
                ConfigGroup::Ui => config.ui = defaults.ui,
                ConfigGroup::Keymap => config.keymap = defaults.keymap,
                ConfigGroup::All => *config = defaults,
            }
        })
    }

    /// Imports one complete document. Missing defaultable fields never inherit current values.
    pub fn import(&self, request: ConfigPathRequest) -> Result<ConfigSnapshot, AppError> {
        if self.directory.as_os_str().is_empty() || self.live_path.as_os_str().is_empty() {
            return Err(unavailable_path_error());
        }
        let path = self.validate_external_path(&request.path, "import")?;
        let bytes = self.ops.read_bounded(&path).map_err(|error| {
            io_app_error(
                AppErrorCode::ConfigImportFailed,
                "无法读取导入配置。",
                error,
                true,
            )
        })?;
        let candidate = codec::decode(&bytes).map_err(codec_app_error)?;
        self.commit(candidate, true)
    }

    /// Exports only the canonical schema document without runtime metadata.
    pub fn export(&self, request: ConfigPathRequest) -> Result<ConfigExportResult, AppError> {
        let path = self.validate_external_path(&request.path, "export")?;
        if self.ops.path_is_directory(&path).map_err(|error| {
            io_app_error(
                AppErrorCode::ConfigExportFailed,
                "无法检查导出路径。",
                error,
                true,
            )
        })? {
            return Err(invalid_path("导出目标不能是目录。"));
        }
        let config = self.lock()?.snapshot.config.clone();
        let bytes = codec::encode(&config).map_err(codec_app_error)?;
        let parent = path
            .parent()
            .ok_or_else(|| invalid_path("导出路径缺少父目录。"))?;
        persist_bytes(&self.ops, parent, &path, &bytes, ".wubilex-export-backup")
            .map_err(|failure| failure.error)?;
        Ok(ConfigExportResult {
            path: path.to_string_lossy().into_owned(),
            bytes_written: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        })
    }

    fn update(
        &self,
        allow_read_only: bool,
        apply: impl FnOnce(&mut AppConfig),
    ) -> Result<ConfigSnapshot, AppError> {
        let mut data = self.lock()?;
        if data.snapshot.persistence == ConfigPersistence::ReadOnly && !allow_read_only {
            return Err(read_only_error());
        }
        let mut candidate = data.snapshot.config.clone();
        apply(&mut candidate);
        self.commit_locked(&mut data, candidate)
    }

    fn commit(
        &self,
        candidate: AppConfig,
        allow_read_only: bool,
    ) -> Result<ConfigSnapshot, AppError> {
        let mut data = self.lock()?;
        if data.snapshot.persistence == ConfigPersistence::ReadOnly && !allow_read_only {
            return Err(read_only_error());
        }
        self.commit_locked(&mut data, candidate)
    }

    fn commit_locked(
        &self,
        data: &mut ConfigData,
        candidate: AppConfig,
    ) -> Result<ConfigSnapshot, AppError> {
        candidate.validate().map_err(|error| {
            AppError::config(
                AppErrorCode::ConfigValidationFailed,
                AppErrorKind::Validation,
                "配置内容未通过校验。",
                Some(format!("field={}; reason={}", error.field, error.reason)),
                true,
            )
        })?;
        let next_revision = data.snapshot.revision.checked_add(1).ok_or_else(|| {
            AppError::config(
                AppErrorCode::ConfigStateFailed,
                AppErrorKind::System,
                "配置版本计数已达到上限。",
                Some("stage=revision_increment".to_owned()),
                false,
            )
        })?;
        let bytes = codec::encode(&candidate).map_err(codec_app_error)?;
        match persist_bytes(
            &self.ops,
            &self.directory,
            &self.live_path,
            &bytes,
            "config.backup",
        ) {
            Ok(_) => {
                data.snapshot.config = candidate;
                data.snapshot.revision = next_revision;
                data.snapshot.persistence = ConfigPersistence::Ready;
                for notice in &mut data.snapshot.notices {
                    if notice.code == ConfigNoticeCode::ReplacementRecoveryFailed {
                        notice.message =
                            "上次配置替换恢复失败，最后有效备份仍保留在报告路径。".to_owned();
                    }
                }
                data.snapshot.notices.retain(|notice| {
                    !matches!(
                        notice.code,
                        ConfigNoticeCode::UnsupportedVersion
                            | ConfigNoticeCode::PersistenceUnavailable
                    )
                });
                Ok(data.snapshot.clone())
            }
            Err(failure) => {
                if let Some(notice) = failure.read_only_notice {
                    data.snapshot.persistence = ConfigPersistence::ReadOnly;
                    push_notice(&mut data.snapshot.notices, notice);
                }
                Err(failure.error)
            }
        }
    }

    fn validate_external_path(
        &self,
        value: &str,
        stage: &'static str,
    ) -> Result<PathBuf, AppError> {
        let path = PathBuf::from(value);
        if value.trim().is_empty() || !path.is_absolute() {
            return Err(invalid_path("配置文件路径必须是非空绝对路径。"));
        }
        if !self.directory.as_os_str().is_empty() {
            let aliases_owned = aliases_owned_config_path(&self.ops, &self.directory, &path)
                .map_err(|error| {
                    let (code, message) = if stage == "import" {
                        (AppErrorCode::ConfigImportFailed, "无法解析导入路径。")
                    } else {
                        (AppErrorCode::ConfigExportFailed, "无法解析导出路径。")
                    };
                    io_app_error(code, message, error, true)
                })?;
            if aliases_owned {
                return Err(invalid_path("该路径属于应用配置事务，不能用于导入或导出。"));
            }
        }
        if stage == "import"
            && self.ops.path_is_directory(&path).map_err(|error| {
                io_app_error(
                    AppErrorCode::ConfigImportFailed,
                    "无法检查导入路径。",
                    error,
                    true,
                )
            })?
        {
            return Err(invalid_path("导入来源不能是目录。"));
        }
        Ok(path)
    }

    fn lock(&self) -> Result<MutexGuard<'_, ConfigData>, AppError> {
        self.inner.lock().map_err(|_| {
            AppError::config(
                AppErrorCode::ConfigStateFailed,
                AppErrorKind::System,
                "配置状态暂时不可用。",
                Some("stage=config_state_lock; poisoned=true".to_owned()),
                true,
            )
        })
    }
}

fn load_snapshot<F: ConfigFileOps>(ops: &F, directory: &Path, live: &Path) -> ConfigSnapshot {
    if let Err(error) = ops.create_dir_all(directory) {
        return degraded_defaults("无法创建配置目录，已使用只读默认配置。", error.detail());
    }
    let exists = match ops.path_exists(live) {
        Ok(exists) => exists,
        Err(error) => {
            return degraded_defaults("无法检查配置文件，已使用只读默认配置。", error.detail());
        }
    };
    if !exists {
        match recover_owned_backup(ops, directory, live) {
            Ok(Some(recovered)) => return recovered,
            Ok(None) => {}
            Err(error) => {
                return degraded_defaults("无法检查配置备份，已使用只读默认配置。", error.detail());
            }
        }
        let defaults = AppConfig::default();
        let bytes = match codec::encode(&defaults) {
            Ok(bytes) => bytes,
            Err(error) => return degraded_defaults("无法生成默认配置。", error.to_string()),
        };
        return match persist_bytes(ops, directory, live, &bytes, "config.backup") {
            Ok(_) => snapshot(defaults, ConfigPersistence::Ready, Vec::new()),
            Err(failure) => degraded_defaults(
                "无法保存默认配置，已进入只读模式。",
                failure.error.detail.unwrap_or_default(),
            ),
        };
    }

    let bytes = match ops.read_bounded(live) {
        Ok(bytes) => bytes,
        Err(error) => {
            return degraded_defaults("无法读取配置文件，已使用只读默认配置。", error.detail());
        }
    };
    match codec::decode(&bytes) {
        Ok(config) => snapshot(config, ConfigPersistence::Ready, Vec::new()),
        Err(codec::ConfigCodecError::Migration(migration::MigrationError::Future(version))) => {
            snapshot(
                AppConfig::default(),
                ConfigPersistence::ReadOnly,
                vec![ConfigNotice::new(
                    ConfigNoticeCode::UnsupportedVersion,
                    "配置来自更新版本，当前程序已使用只读默认配置。",
                    Some(format!("schemaVersion={version}; path={}", live.display())),
                )],
            )
        }
        Err(error) => preserve_corrupt_and_create_defaults(ops, directory, live, error),
    }
}

fn recover_owned_backup<F: ConfigFileOps>(
    ops: &F,
    directory: &Path,
    live: &Path,
) -> Result<Option<ConfigSnapshot>, ConfigIoError> {
    let backups = ops.list_owned_backups(directory)?;
    for backup in backups {
        let bytes = ops.read_bounded(&backup)?;
        let config = match codec::decode(&bytes) {
            Ok(config) => config,
            Err(_) => continue,
        };
        return Ok(Some(match ops.restore_backup_noreplace(&backup, live) {
            Ok(()) => snapshot(
                config,
                ConfigPersistence::Ready,
                vec![ConfigNotice::new(
                    ConfigNoticeCode::BackupRecovered,
                    "已从上次保存的配置备份恢复。",
                    Some(format!("path={}", live.display())),
                )],
            ),
            Err(error) => snapshot(
                config,
                ConfigPersistence::ReadOnly,
                vec![ConfigNotice::new(
                    ConfigNoticeCode::PersistenceUnavailable,
                    "找到有效配置备份，但无法恢复到原路径。",
                    Some(error.detail()),
                )],
            ),
        }));
    }
    Ok(None)
}

fn preserve_corrupt_and_create_defaults<F: ConfigFileOps>(
    ops: &F,
    directory: &Path,
    live: &Path,
    decode_error: codec::ConfigCodecError,
) -> ConfigSnapshot {
    let mut preserved = None;
    for _ in 0..UNIQUE_PATH_ATTEMPTS {
        let corrupt = unique_path(directory, "config.corrupt", "toml");
        match ops.preserve_corrupt_noreplace(live, &corrupt) {
            Ok(()) => {
                preserved = Some(corrupt);
                break;
            }
            Err(error) if error.kind == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return degraded_defaults(
                    "配置无效且无法安全保留，原文件保持不变。",
                    format!(
                        "decode={}; preserve={}",
                        codec_error_label(&decode_error),
                        error.detail()
                    ),
                );
            }
        }
    }
    let Some(corrupt) = preserved else {
        return degraded_defaults(
            "配置无效且无法取得保留路径，原文件保持不变。",
            format!(
                "decode={}; stage=select_corrupt_path; attempts={UNIQUE_PATH_ATTEMPTS}",
                codec_error_label(&decode_error)
            ),
        );
    };
    let notice = ConfigNotice::new(
        ConfigNoticeCode::CorruptConfigPreserved,
        "配置无效，已保留副本并恢复默认值。",
        Some(format!(
            "artifact={}; decode={}",
            corrupt.display(),
            codec_error_label(&decode_error)
        )),
    );
    let defaults = AppConfig::default();
    let bytes = match codec::encode(&defaults) {
        Ok(bytes) => bytes,
        Err(error) => return degraded_defaults("无法生成默认配置。", error.to_string()),
    };
    match persist_bytes(ops, directory, live, &bytes, "config.backup") {
        Ok(_) => snapshot(defaults, ConfigPersistence::Ready, vec![notice]),
        Err(failure) => {
            let mut notices = vec![notice];
            push_notice(
                &mut notices,
                ConfigNotice::new(
                    ConfigNoticeCode::PersistenceUnavailable,
                    "已保留无效配置，但默认配置无法保存。",
                    failure.error.detail,
                ),
            );
            snapshot(defaults, ConfigPersistence::ReadOnly, notices)
        }
    }
}

struct PersistFailure {
    error: AppError,
    read_only_notice: Option<ConfigNotice>,
}

fn persist_bytes<F: ConfigFileOps>(
    ops: &F,
    directory: &Path,
    target: &Path,
    bytes: &[u8],
    backup_prefix: &str,
) -> Result<Option<PathBuf>, PersistFailure> {
    let (staging, mut pending) =
        create_staging(ops, directory).map_err(|error| PersistFailure {
            error,
            read_only_notice: None,
        })?;
    if let Err(error) = pending.write_all(bytes) {
        return Err(close_then_cleanup(
            ops,
            &staging,
            pending,
            AppError::config(
                AppErrorCode::ConfigWriteFailed,
                AppErrorKind::Io,
                "无法写入配置临时文件。",
                Some(format!(
                    "stage=write_staging; path={}; kind={:?}; code={}",
                    staging.display(),
                    error.kind(),
                    error.raw_os_error().unwrap_or(0)
                )),
                true,
            ),
        ));
    }
    if let Err(error) = pending.flush() {
        return Err(close_then_cleanup(
            ops,
            &staging,
            pending,
            AppError::config(
                AppErrorCode::ConfigWriteFailed,
                AppErrorKind::Io,
                "无法刷新配置临时文件。",
                Some(format!(
                    "stage=flush_staging; path={}; kind={:?}; code={}",
                    staging.display(),
                    error.kind(),
                    error.raw_os_error().unwrap_or(0)
                )),
                true,
            ),
        ));
    }
    if let Err(error) = ops.sync(&staging, &mut pending) {
        return Err(close_then_cleanup(
            ops,
            &staging,
            pending,
            io_app_error(
                AppErrorCode::ConfigWriteFailed,
                "无法同步配置临时文件。",
                error,
                true,
            ),
        ));
    }
    if let Err(error) = ops.close(&staging, pending) {
        return Err(cleanup_failure(
            ops,
            &staging,
            io_app_error(
                AppErrorCode::ConfigWriteFailed,
                "无法关闭配置临时文件。",
                error,
                true,
            ),
        ));
    }

    let target_exists = ops.path_exists(target).map_err(|error| {
        cleanup_failure(
            ops,
            &staging,
            io_app_error(
                AppErrorCode::ConfigWriteFailed,
                "无法检查配置目标。",
                error,
                true,
            ),
        )
    })?;
    if !target_exists {
        return match ops.install_new_noreplace(&staging, target) {
            Ok(()) => Ok(None),
            Err(error) => Err(cleanup_failure(
                ops,
                &staging,
                io_app_error(
                    AppErrorCode::ConfigWriteFailed,
                    "无法安装新配置。",
                    error,
                    true,
                ),
            )),
        };
    }

    let backup = select_unused_owned_path(ops, directory, backup_prefix, "toml")
        .map_err(|error| cleanup_failure(ops, &staging, error))?;
    match ops.replace_with_backup(target, &staging, &backup) {
        Ok(()) => Ok(Some(backup)),
        Err(error) if error.disposition == ReplacementDisposition::OriginalAtBackup => {
            match ops.restore_backup_noreplace(&backup, target) {
                Ok(()) => Err(cleanup_replace_failure(
                    ops,
                    target,
                    &staging,
                    &backup,
                    error,
                    Some("restore=success"),
                )),
                Err(restore) => {
                    let detail = format!(
                        "{}; restore={}",
                        native_replace_detail(&error, target, &staging, &backup),
                        restore.detail()
                    );
                    let mut failure = cleanup_failure(
                        ops,
                        &staging,
                        AppError::config(
                            AppErrorCode::ConfigReplaceFailed,
                            AppErrorKind::Io,
                            "配置替换失败，且旧配置只能从保留的备份恢复。",
                            Some(detail.clone()),
                            true,
                        ),
                    );
                    let notice_detail = failure.error.detail.clone();
                    failure.read_only_notice = Some(ConfigNotice::new(
                        ConfigNoticeCode::ReplacementRecoveryFailed,
                        "配置替换恢复失败，已保留最后有效备份并进入只读模式。",
                        notice_detail,
                    ));
                    Err(failure)
                }
            }
        }
        Err(error) => Err(cleanup_replace_failure(
            ops, target, &staging, &backup, error, None,
        )),
    }
}

fn create_staging<F: ConfigFileOps>(
    ops: &F,
    directory: &Path,
) -> Result<(PathBuf, F::Pending), AppError> {
    for _ in 0..UNIQUE_PATH_ATTEMPTS {
        let path = unique_path(directory, "config.tmp", "toml");
        match ops.create_temp_exclusive(&path) {
            Ok(pending) => return Ok((path, pending)),
            Err(error) if error.kind == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(io_app_error(
                    AppErrorCode::ConfigWriteFailed,
                    "无法创建配置临时文件。",
                    error,
                    true,
                ));
            }
        }
    }
    Err(AppError::config(
        AppErrorCode::ConfigWriteFailed,
        AppErrorKind::Io,
        "无法取得配置临时文件名。",
        Some(format!(
            "stage=create_staging; attempts={UNIQUE_PATH_ATTEMPTS}"
        )),
        true,
    ))
}

fn select_unused_owned_path<F: ConfigFileOps>(
    ops: &F,
    directory: &Path,
    prefix: &str,
    extension: &str,
) -> Result<PathBuf, AppError> {
    for _ in 0..UNIQUE_PATH_ATTEMPTS {
        let path = unique_path(directory, prefix, extension);
        match ops.path_exists(&path) {
            Ok(false) => return Ok(path),
            Ok(true) => continue,
            Err(error) => {
                return Err(io_app_error(
                    AppErrorCode::ConfigBackupFailed,
                    "无法检查配置备份路径。",
                    error,
                    true,
                ));
            }
        }
    }
    Err(AppError::config(
        AppErrorCode::ConfigBackupFailed,
        AppErrorKind::Io,
        "无法取得唯一的配置备份路径。",
        Some(format!(
            "stage=select_backup_path; attempts={UNIQUE_PATH_ATTEMPTS}"
        )),
        true,
    ))
}

fn cleanup_replace_failure<F: ConfigFileOps>(
    ops: &F,
    target: &Path,
    staging: &Path,
    backup: &Path,
    error: ReplaceFileError,
    secondary: Option<&str>,
) -> PersistFailure {
    let mut detail = native_replace_detail(&error, target, staging, backup);
    if let Some(secondary) = secondary {
        detail.push_str("; secondary=");
        detail.push_str(secondary);
    }
    cleanup_failure(
        ops,
        staging,
        AppError::config(
            AppErrorCode::ConfigReplaceFailed,
            AppErrorKind::Io,
            "无法替换配置文件。",
            Some(detail),
            true,
        ),
    )
}

fn close_then_cleanup<F: ConfigFileOps>(
    ops: &F,
    staging: &Path,
    pending: F::Pending,
    mut primary: AppError,
) -> PersistFailure {
    if let Err(close) = ops.close(staging, pending) {
        let detail = format!(
            "{}; close={}",
            primary.detail.as_deref().unwrap_or("stage=unknown"),
            close.detail()
        );
        primary.detail = Some(detail.chars().take(1_024).collect());
    }
    cleanup_failure(ops, staging, primary)
}

fn cleanup_failure<F: ConfigFileOps>(
    ops: &F,
    staging: &Path,
    mut primary: AppError,
) -> PersistFailure {
    if let Err(cleanup) = ops.remove_owned(staging) {
        let detail = format!(
            "{}; cleanup={}",
            primary.detail.as_deref().unwrap_or("stage=unknown"),
            cleanup.detail()
        );
        primary.detail = Some(detail.chars().take(1_024).collect());
    }
    PersistFailure {
        error: primary,
        read_only_notice: None,
    }
}

fn native_replace_detail(
    error: &ReplaceFileError,
    target: &Path,
    staging: &Path,
    backup: &Path,
) -> String {
    format!(
        "stage={}; code={}; disposition={:?}; target={}; staging={}; backup={}; error={}",
        error.source.stage,
        error.source.code,
        error.disposition,
        target.display(),
        staging.display(),
        backup.display(),
        error.source.message
    )
}

fn unique_path(directory: &Path, prefix: &str, extension: &str) -> PathBuf {
    let sequence = PATH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    directory.join(format!(
        "{prefix}-{}-{sequence}.{extension}",
        std::process::id()
    ))
}

fn aliases_owned_config_path<F: ConfigFileOps>(
    ops: &F,
    directory: &Path,
    candidate: &Path,
) -> Result<bool, ConfigIoError> {
    let resolved_directory = ops
        .canonicalize_existing(directory)?
        .unwrap_or_else(|| directory.to_path_buf());
    let resolved_candidate = match ops.canonicalize_existing(candidate)? {
        Some(path) => path,
        None => {
            let parent = candidate.parent().unwrap_or(candidate);
            let resolved_parent = ops
                .canonicalize_existing(parent)?
                .unwrap_or_else(|| parent.to_path_buf());
            candidate
                .file_name()
                .map_or(resolved_parent.clone(), |name| resolved_parent.join(name))
        }
    };
    let same_parent = resolved_candidate
        .parent()
        .is_some_and(|parent| paths_equal_for_platform(parent, &resolved_directory));
    if !same_parent {
        return Ok(false);
    }
    let name = resolved_candidate
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    Ok(name == CONFIG_FILE_NAME
        || name.starts_with("config.tmp-")
        || name.starts_with("config.backup-")
        || name.starts_with("config.corrupt-"))
}

fn paths_equal_for_platform(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn snapshot(
    config: AppConfig,
    persistence: ConfigPersistence,
    notices: Vec<ConfigNotice>,
) -> ConfigSnapshot {
    ConfigSnapshot {
        revision: 1,
        config,
        persistence,
        notices: notices.into_iter().take(MAX_CONFIG_NOTICES).collect(),
    }
}

fn degraded_defaults(message: &str, detail: String) -> ConfigSnapshot {
    snapshot(
        AppConfig::default(),
        ConfigPersistence::ReadOnly,
        vec![ConfigNotice::new(
            ConfigNoticeCode::PersistenceUnavailable,
            message,
            Some(detail),
        )],
    )
}

fn push_notice(notices: &mut Vec<ConfigNotice>, notice: ConfigNotice) {
    if notices.len() < MAX_CONFIG_NOTICES {
        notices.push(notice);
    }
}

fn codec_error_label(error: &codec::ConfigCodecError) -> &'static str {
    match error {
        codec::ConfigCodecError::SizeLimit => "size_limit",
        codec::ConfigCodecError::Utf8 => "utf8",
        codec::ConfigCodecError::Parse(_) => "toml_parse",
        codec::ConfigCodecError::Migration(migration::MigrationError::Missing) => "missing_version",
        codec::ConfigCodecError::Migration(migration::MigrationError::Invalid) => "invalid_version",
        codec::ConfigCodecError::Migration(migration::MigrationError::Future(_)) => {
            "future_version"
        }
        codec::ConfigCodecError::Migration(migration::MigrationError::Unsupported(_)) => {
            "unsupported_version"
        }
        codec::ConfigCodecError::Validation(_) => "validation",
        codec::ConfigCodecError::Encode(_) => "encode",
    }
}

fn codec_app_error(error: codec::ConfigCodecError) -> AppError {
    match error {
        codec::ConfigCodecError::Migration(migration::MigrationError::Future(version)) => {
            AppError::config(
                AppErrorCode::ConfigUnsupportedVersion,
                AppErrorKind::Validation,
                "配置版本高于当前程序支持范围。",
                Some(format!("schemaVersion={version}")),
                false,
            )
        }
        codec::ConfigCodecError::Migration(error) => AppError::config(
            AppErrorCode::ConfigUnsupportedVersion,
            AppErrorKind::Validation,
            "配置版本不受支持。",
            Some(format!(
                "kind={}",
                codec_error_label(&codec::ConfigCodecError::Migration(error))
            )),
            false,
        ),
        codec::ConfigCodecError::Validation(error) => AppError::config(
            AppErrorCode::ConfigValidationFailed,
            AppErrorKind::Validation,
            "配置内容未通过校验。",
            Some(format!("field={}; reason={}", error.field, error.reason)),
            true,
        ),
        error @ (codec::ConfigCodecError::SizeLimit
        | codec::ConfigCodecError::Utf8
        | codec::ConfigCodecError::Parse(_)) => AppError::config(
            AppErrorCode::ConfigParseFailed,
            AppErrorKind::Parse,
            "配置文件无法解析。",
            Some(format!("kind={}", codec_error_label(&error))),
            true,
        ),
        codec::ConfigCodecError::Encode(_) => AppError::config(
            AppErrorCode::ConfigWriteFailed,
            AppErrorKind::System,
            "配置无法编码。",
            Some("stage=encode_config".to_owned()),
            false,
        ),
    }
}

fn io_app_error(
    code: AppErrorCode,
    message: &str,
    error: ConfigIoError,
    recoverable: bool,
) -> AppError {
    let kind = if error.kind == std::io::ErrorKind::PermissionDenied {
        AppErrorKind::Permission
    } else {
        AppErrorKind::Io
    };
    AppError::config(code, kind, message, Some(error.detail()), recoverable)
}

fn invalid_path(message: &str) -> AppError {
    AppError::config(
        AppErrorCode::ConfigInvalidPath,
        AppErrorKind::Validation,
        message,
        Some("stage=validate_config_path".to_owned()),
        true,
    )
}

fn read_only_error() -> AppError {
    AppError::config(
        AppErrorCode::ConfigUnavailable,
        AppErrorKind::Io,
        "配置当前为只读状态。请先导入一份有效配置。",
        Some("stage=config_commit; persistence=readOnly".to_owned()),
        true,
    )
}

fn unavailable_path_error() -> AppError {
    AppError::config(
        AppErrorCode::ConfigUnavailable,
        AppErrorKind::Io,
        "应用配置目录不可用，当前无法导入配置。",
        Some("stage=config_import; app_config_path=unavailable".to_owned()),
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AppConfig, BindingOverride, ConfigGroup, ConfigPathRequest, ConfigPersistence,
        ConfigService, UiConfig, WindowBounds, WindowsConfigFileOps,
    };
    use crate::config::storage::{ConfigFileOps, ConfigIoError};
    use std::{
        fs,
        io::{self, Write},
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };
    use wubilex_winime::filesystem::{NativeFileError, ReplaceFileError, ReplacementDisposition};

    #[test]
    fn missing_file_creates_defaults_and_updates_commit_one_revision() {
        let directory = tempfile::tempdir().expect("temporary config directory");
        let service = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let initial = service.snapshot().expect("initial snapshot");
        assert_eq!(initial.revision, 1);
        assert_eq!(initial.persistence, ConfigPersistence::Ready);
        assert_eq!(
            super::codec::decode(
                &fs::read(directory.path().join("config.toml")).expect("live config")
            )
            .expect("canonical config"),
            AppConfig::default()
        );

        let ui = UiConfig {
            sidebar_collapsed: true,
            ..UiConfig::default()
        };
        let updated = service.update_ui(ui).expect("update UI");
        assert_eq!(updated.revision, 2);
        assert!(updated.config.ui.sidebar_collapsed);
        let restored = service
            .restore_defaults(ConfigGroup::Ui)
            .expect("restore UI");
        assert_eq!(restored.revision, 3);
        assert_eq!(restored.config.ui, UiConfig::default());
    }

    #[test]
    fn corrupt_input_is_preserved_and_future_input_remains_untouched_until_valid_import() {
        let corrupt_directory = tempfile::tempdir().expect("corrupt directory");
        fs::write(corrupt_directory.path().join("config.toml"), b"not = [toml")
            .expect("write corrupt");
        let corrupt =
            ConfigService::load(corrupt_directory.path().to_path_buf(), WindowsConfigFileOps)
                .snapshot()
                .expect("corrupt snapshot");
        assert_eq!(corrupt.persistence, ConfigPersistence::Ready);
        assert!(
            fs::read_dir(corrupt_directory.path())
                .expect("directory entries")
                .flatten()
                .any(|entry| entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.corrupt-"))
        );

        let future_directory = tempfile::tempdir().expect("future directory");
        let future_bytes = b"schemaVersion = 2\nfutureValue = 'keep'\n";
        fs::write(future_directory.path().join("config.toml"), future_bytes).expect("write future");
        let service =
            ConfigService::load(future_directory.path().to_path_buf(), WindowsConfigFileOps);
        let future = service.snapshot().expect("future snapshot");
        assert_eq!(future.persistence, ConfigPersistence::ReadOnly);
        assert_eq!(
            fs::read(future_directory.path().join("config.toml")).expect("future bytes"),
            future_bytes
        );
        assert!(service.update_ui(UiConfig::default()).is_err());

        let import_path = future_directory.path().join("supported-import.toml");
        fs::write(&import_path, b"schemaVersion = 1\n").expect("write supported import");
        let imported = service
            .import(ConfigPathRequest {
                path: import_path.to_string_lossy().into_owned(),
            })
            .expect("explicit supported import");
        assert_eq!(imported.persistence, ConfigPersistence::Ready);
        assert_eq!(imported.revision, 2);
        assert_eq!(imported.config, AppConfig::default());
        assert!(
            imported
                .notices
                .iter()
                .all(|notice| notice.code != super::ConfigNoticeCode::UnsupportedVersion)
        );
        let backups = WindowsConfigFileOps
            .list_owned_backups(future_directory.path())
            .expect("future backup");
        assert_eq!(backups.len(), 1);
        assert_eq!(
            fs::read(&backups[0]).expect("preserved future bytes"),
            future_bytes
        );
    }

    #[test]
    fn whole_document_import_defaults_missing_fields_instead_of_merging() {
        let directory = tempfile::tempdir().expect("config directory");
        let service = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let mut keymap = super::KeymapConfig::default();
        keymap
            .bindings
            .insert("shell.search".to_owned(), BindingOverride::Unbound);
        service.update_keymap(keymap).expect("seed current keymap");

        let import_path = directory.path().join("external-import.toml");
        fs::write(
            &import_path,
            b"schemaVersion = 1\n[ui]\nsidebarCollapsed = true\n",
        )
        .expect("write import");
        let imported = service
            .import(ConfigPathRequest {
                path: import_path.to_string_lossy().into_owned(),
            })
            .expect("import complete document");
        assert!(imported.config.ui.sidebar_collapsed);
        assert!(imported.config.keymap.bindings.is_empty());
        assert_eq!(imported.revision, 3);
    }

    #[test]
    fn import_rejects_resolved_owned_aliases_and_unavailable_service_never_stages() {
        let directory = tempfile::tempdir().expect("config directory");
        let service = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let aliased_live = directory.path().join(".").join("config.toml");
        let error = service
            .import(ConfigPathRequest {
                path: aliased_live.to_string_lossy().into_owned(),
            })
            .expect_err("resolved live alias must be rejected");
        assert_eq!(error.code, crate::error::AppErrorCode::ConfigInvalidPath);
        assert_eq!(service.snapshot().expect("snapshot").revision, 1);

        let import_directory = tempfile::tempdir().expect("import directory");
        let import_path = import_directory.path().join("import.toml");
        fs::write(&import_path, b"schemaVersion = 1\n").expect("write import");
        let ops = FaultOps::new(FaultStage::Create);
        let trace = Arc::clone(&ops.trace);
        let unavailable = ConfigService::unavailable("path unavailable", ops);
        let error = unavailable
            .import(ConfigPathRequest {
                path: import_path.to_string_lossy().into_owned(),
            })
            .expect_err("unavailable service cannot import");
        assert_eq!(error.code, crate::error::AppErrorCode::ConfigUnavailable);
        assert!(
            error
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("app_config_path=unavailable"))
        );
        assert!(trace.lock().expect("trace").is_empty());
    }

    #[test]
    fn failed_import_keeps_snapshot_revision_and_live_bytes_exact() {
        let directory = tempfile::tempdir().expect("config directory");
        let service = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let before_snapshot = service.snapshot().expect("before snapshot");
        let before_bytes = fs::read(directory.path().join("config.toml")).expect("before bytes");
        let import_path = directory.path().join("invalid-import.toml");
        fs::write(
            &import_path,
            b"schemaVersion = 1\n[ui]\ntheme = 'invalid'\n",
        )
        .expect("write invalid import");

        let error = service
            .import(ConfigPathRequest {
                path: import_path.to_string_lossy().into_owned(),
            })
            .expect_err("invalid import must fail");
        assert!(
            !error
                .detail
                .as_deref()
                .unwrap_or_default()
                .contains("invalid")
        );
        assert_eq!(service.snapshot().expect("after snapshot"), before_snapshot);
        assert_eq!(
            fs::read(directory.path().join("config.toml")).expect("after bytes"),
            before_bytes
        );
    }

    #[test]
    fn export_contains_only_canonical_config() {
        let directory = tempfile::tempdir().expect("config directory");
        let export_directory = tempfile::tempdir().expect("export directory");
        let service = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let path = export_directory.path().join("settings.toml");
        let result = service
            .export(ConfigPathRequest {
                path: path.to_string_lossy().into_owned(),
            })
            .expect("export config");
        let text = fs::read_to_string(path).expect("exported TOML");
        assert_eq!(result.bytes_written as usize, text.len());
        assert!(text.contains("schemaVersion = 1"));
        assert!(!text.contains("revision"));
        assert!(!text.contains("notices"));
    }

    #[test]
    fn invalid_update_rolls_back_snapshot_revision_and_live_bytes() {
        let directory = tempfile::tempdir().expect("config directory");
        let service = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let before_snapshot = service.snapshot().expect("before snapshot");
        let before_bytes = fs::read(directory.path().join("config.toml")).expect("before bytes");
        let mut window = before_snapshot.config.window.clone();
        window.bounds = Some(WindowBounds {
            x: 0,
            y: 0,
            width: 0,
            height: 600,
            scale_factor: 1.0,
        });

        assert!(service.update_window(window).is_err());
        assert_eq!(service.snapshot().expect("after snapshot"), before_snapshot);
        assert_eq!(
            fs::read(directory.path().join("config.toml")).expect("after bytes"),
            before_bytes
        );
    }

    #[test]
    fn concurrent_group_updates_serialize_without_lost_fields() {
        let directory = tempfile::tempdir().expect("config directory");
        let service = Arc::new(ConfigService::load(
            directory.path().to_path_buf(),
            WindowsConfigFileOps,
        ));
        let ui_service = Arc::clone(&service);
        let ui = std::thread::spawn(move || {
            let config = UiConfig {
                sidebar_collapsed: true,
                ..UiConfig::default()
            };
            ui_service.update_ui(config)
        });
        let window_service = Arc::clone(&service);
        let window = std::thread::spawn(move || {
            let config = super::WindowConfig {
                maximized: true,
                ..super::WindowConfig::default()
            };
            window_service.update_window(config)
        });

        ui.join().expect("UI thread").expect("UI update");
        window
            .join()
            .expect("window thread")
            .expect("window update");
        let snapshot = service.snapshot().expect("final snapshot");
        assert_eq!(snapshot.revision, 3);
        assert!(snapshot.config.ui.sidebar_collapsed);
        assert!(snapshot.config.window.maximized);
    }

    #[test]
    fn native_1177_topology_restores_old_target_and_keeps_snapshot_exact() {
        let directory = tempfile::tempdir().expect("config directory");
        let initial = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let before_snapshot = initial.snapshot().expect("initial snapshot");
        let before_bytes = fs::read(directory.path().join("config.toml")).expect("initial bytes");
        drop(initial);

        let service = ConfigService::load(
            directory.path().to_path_buf(),
            Simulated1177Ops {
                fail_restore: false,
                fail_sync: false,
            },
        );
        let ui = UiConfig {
            sidebar_collapsed: true,
            ..UiConfig::default()
        };
        assert!(service.update_ui(ui).is_err());
        assert_eq!(
            service.snapshot().expect("rolled back snapshot"),
            before_snapshot
        );
        assert_eq!(
            fs::read(directory.path().join("config.toml")).expect("restored bytes"),
            before_bytes
        );
    }

    #[test]
    fn failed_1177_restore_retains_last_valid_backup_and_enters_read_only() {
        let directory = tempfile::tempdir().expect("config directory");
        let initial = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let before = initial.snapshot().expect("initial snapshot");
        let before_bytes = fs::read(directory.path().join("config.toml")).expect("initial bytes");
        drop(initial);

        let service = ConfigService::load(
            directory.path().to_path_buf(),
            Simulated1177Ops {
                fail_restore: true,
                fail_sync: false,
            },
        );
        let ui = UiConfig {
            sidebar_collapsed: true,
            ..UiConfig::default()
        };
        let error = service
            .update_ui(ui)
            .expect_err("1177 restore failure must fail the commit");

        let after = service.snapshot().expect("degraded snapshot");
        assert_eq!(after.revision, before.revision);
        assert_eq!(after.config, before.config);
        assert_eq!(after.persistence, ConfigPersistence::ReadOnly);
        assert!(!directory.path().join("config.toml").exists());
        let backups = WindowsConfigFileOps
            .list_owned_backups(directory.path())
            .expect("owned backups");
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).expect("backup bytes"), before_bytes);
        let detail = error.detail.expect("combined error evidence");
        assert!(detail.contains("code=1177"));
        assert!(detail.contains("target="));
        assert!(detail.contains("staging="));
        assert!(detail.contains("backup="));
        assert!(detail.contains("restore=stage=restore_backup_noreplace"));
        assert_eq!(
            after
                .notices
                .iter()
                .find(|notice| {
                    notice.code == super::ConfigNoticeCode::ReplacementRecoveryFailed
                })
                .and_then(|notice| notice.detail.as_deref()),
            Some(detail.as_str())
        );
    }

    #[test]
    fn sync_failure_closes_and_cleans_owned_staging_before_rollback() {
        let directory = tempfile::tempdir().expect("config directory");
        let initial = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let before_snapshot = initial.snapshot().expect("initial snapshot");
        let before_bytes = fs::read(directory.path().join("config.toml")).expect("initial bytes");
        drop(initial);

        let service = ConfigService::load(
            directory.path().to_path_buf(),
            Simulated1177Ops {
                fail_restore: false,
                fail_sync: true,
            },
        );
        let ui = UiConfig {
            sidebar_collapsed: true,
            ..UiConfig::default()
        };
        assert!(service.update_ui(ui).is_err());
        assert_eq!(
            service.snapshot().expect("rolled back snapshot"),
            before_snapshot
        );
        assert_eq!(
            fs::read(directory.path().join("config.toml")).expect("live bytes"),
            before_bytes
        );
        assert!(
            fs::read_dir(directory.path())
                .expect("directory entries")
                .flatten()
                .all(|entry| !entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.tmp-"))
        );
    }

    #[test]
    fn staged_failures_preserve_snapshot_live_bytes_and_cleanup_after_close() {
        for stage in [
            FaultStage::Create,
            FaultStage::Write,
            FaultStage::Flush,
            FaultStage::Sync,
            FaultStage::Close,
            FaultStage::InspectTarget,
            FaultStage::SelectBackup,
            FaultStage::Replace,
        ] {
            let directory = tempfile::tempdir().expect("config directory");
            let initial = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
            let before_snapshot = initial.snapshot().expect("initial snapshot");
            let before_bytes =
                fs::read(directory.path().join("config.toml")).expect("initial bytes");
            drop(initial);

            let ops = FaultOps::new(stage);
            let trace = Arc::clone(&ops.trace);
            let service = ConfigService::load(directory.path().to_path_buf(), ops);
            let result = service.update_ui(UiConfig {
                sidebar_collapsed: true,
                ..UiConfig::default()
            });

            assert!(result.is_err(), "{stage:?} must fail");
            assert_eq!(
                service.snapshot().expect("rolled back snapshot"),
                before_snapshot,
                "{stage:?} changed memory state"
            );
            assert_eq!(
                fs::read(directory.path().join("config.toml")).expect("live bytes"),
                before_bytes,
                "{stage:?} changed the last-valid file"
            );
            assert!(
                fs::read_dir(directory.path())
                    .expect("directory entries")
                    .flatten()
                    .all(|entry| !entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("config.tmp-")),
                "{stage:?} left an owned staging file"
            );

            if matches!(
                stage,
                FaultStage::Write | FaultStage::Flush | FaultStage::Sync | FaultStage::Close
            ) {
                let trace = trace.lock().expect("fault trace");
                let close = trace
                    .iter()
                    .position(|entry| *entry == "close")
                    .expect("close stage");
                let cleanup = trace
                    .iter()
                    .position(|entry| *entry == "cleanup")
                    .expect("cleanup stage");
                assert!(close < cleanup, "{stage:?} cleaned before closing");
            }
        }
    }

    #[test]
    fn cleanup_failure_retains_primary_and_secondary_evidence() {
        let directory = tempfile::tempdir().expect("config directory");
        let initial = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let before_snapshot = initial.snapshot().expect("initial snapshot");
        let before_bytes = fs::read(directory.path().join("config.toml")).expect("initial bytes");
        drop(initial);

        let service = ConfigService::load(
            directory.path().to_path_buf(),
            FaultOps::new(FaultStage::ReplaceAndCleanup),
        );
        let error = service
            .update_ui(UiConfig {
                sidebar_collapsed: true,
                ..UiConfig::default()
            })
            .expect_err("replace and cleanup must fail");

        let detail = error.detail.expect("combined error detail");
        assert!(detail.contains("stage=replace_file_with_backup"));
        assert!(detail.contains("cleanup=stage=cleanup_owned_file"));
        assert_eq!(service.snapshot().expect("snapshot"), before_snapshot);
        assert_eq!(
            fs::read(directory.path().join("config.toml")).expect("live bytes"),
            before_bytes
        );
        assert!(
            fs::read_dir(directory.path())
                .expect("directory entries")
                .flatten()
                .any(|entry| entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.tmp-")),
            "failed owned cleanup must retain the staging evidence"
        );
    }

    #[test]
    fn create_collisions_are_never_adopted_or_cleaned() {
        let directory = tempfile::tempdir().expect("config directory");
        let initial = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let before_snapshot = initial.snapshot().expect("initial snapshot");
        drop(initial);

        let service = ConfigService::load(
            directory.path().to_path_buf(),
            FaultOps::new(FaultStage::CreateCollision),
        );
        let error = service
            .update_ui(UiConfig {
                sidebar_collapsed: true,
                ..UiConfig::default()
            })
            .expect_err("bounded collisions must fail");

        assert!(
            error
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("attempts=16"))
        );
        assert_eq!(service.snapshot().expect("snapshot"), before_snapshot);
        let collisions = fs::read_dir(directory.path())
            .expect("directory entries")
            .flatten()
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.tmp-")
            })
            .collect::<Vec<_>>();
        assert_eq!(collisions.len(), super::UNIQUE_PATH_ATTEMPTS);
        for collision in collisions {
            assert_eq!(
                fs::read(collision.path()).expect("foreign collision bytes"),
                b"foreign owner"
            );
        }
    }

    #[test]
    fn initial_install_never_clobbers_a_concurrently_created_target() {
        let directory = tempfile::tempdir().expect("config directory");
        let target = directory.path().join("new-config.toml");
        let bytes = super::codec::encode(&AppConfig::default()).expect("canonical config");

        let failure = super::persist_bytes(
            &FaultOps::new(FaultStage::InstallRace),
            directory.path(),
            &target,
            &bytes,
            "config.backup",
        )
        .expect_err("concurrent target must win");

        assert_eq!(
            fs::read(&target).expect("concurrent target"),
            b"external owner"
        );
        assert!(
            failure
                .error
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("install_new_noreplace"))
        );
        assert!(
            fs::read_dir(directory.path())
                .expect("directory entries")
                .flatten()
                .all(|entry| !entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.tmp-"))
        );
    }

    #[test]
    fn corrupt_preservation_failure_keeps_source_untouched_and_read_only() {
        let directory = tempfile::tempdir().expect("config directory");
        let live = directory.path().join("config.toml");
        let corrupt = b"not = [toml";
        fs::write(&live, corrupt).expect("write corrupt config");

        let service = ConfigService::load(
            directory.path().to_path_buf(),
            FaultOps::new(FaultStage::PreserveCorrupt),
        );
        let snapshot = service.snapshot().expect("degraded snapshot");

        assert_eq!(snapshot.persistence, ConfigPersistence::ReadOnly);
        assert_eq!(fs::read(live).expect("source bytes"), corrupt);
        assert!(
            fs::read_dir(directory.path())
                .expect("directory entries")
                .flatten()
                .all(|entry| !entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.corrupt-"))
        );
    }

    #[test]
    fn startup_restores_the_newest_valid_owned_backup_without_clobber() {
        let directory = tempfile::tempdir().expect("config directory");
        let backup = directory.path().join("config.backup-test.toml");
        let mut config = AppConfig::default();
        config.ui.sidebar_collapsed = true;
        let bytes = super::codec::encode(&config).expect("canonical config");
        fs::write(&backup, &bytes).expect("write owned backup");

        let service = ConfigService::load(directory.path().to_path_buf(), WindowsConfigFileOps);
        let snapshot = service.snapshot().expect("recovered snapshot");

        assert_eq!(snapshot.config, config);
        assert_eq!(snapshot.persistence, ConfigPersistence::Ready);
        assert_eq!(
            fs::read(directory.path().join("config.toml")).expect("live"),
            bytes
        );
        assert!(!backup.exists());
        assert!(
            snapshot
                .notices
                .iter()
                .any(|notice| { notice.code == super::ConfigNoticeCode::BackupRecovered })
        );
    }

    #[test]
    fn backup_discovery_or_read_failure_never_installs_defaults_over_recovery_state() {
        for stage in [FaultStage::ListBackups, FaultStage::ReadBackup] {
            let directory = tempfile::tempdir().expect("config directory");
            if stage == FaultStage::ReadBackup {
                let backup = directory.path().join("config.backup-test.toml");
                let bytes = super::codec::encode(&AppConfig::default()).expect("backup bytes");
                fs::write(backup, bytes).expect("write backup");
            }

            let snapshot =
                ConfigService::load(directory.path().to_path_buf(), FaultOps::new(stage))
                    .snapshot()
                    .expect("degraded snapshot");

            assert_eq!(snapshot.persistence, ConfigPersistence::ReadOnly);
            assert!(!directory.path().join("config.toml").exists());
            assert!(snapshot.notices.iter().any(|notice| {
                notice.code == super::ConfigNoticeCode::PersistenceUnavailable
                    && notice.detail.as_deref().is_some_and(|detail| {
                        detail.contains(if stage == FaultStage::ListBackups {
                            "list_config_backups"
                        } else {
                            "read_owned_backup"
                        })
                    })
            }));
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FaultStage {
        Create,
        CreateCollision,
        Write,
        Flush,
        Sync,
        Close,
        InspectTarget,
        SelectBackup,
        Replace,
        ReplaceAndCleanup,
        InstallRace,
        PreserveCorrupt,
        ListBackups,
        ReadBackup,
    }

    #[derive(Clone)]
    struct FaultOps {
        stage: FaultStage,
        trace: Arc<Mutex<Vec<&'static str>>>,
    }

    impl FaultOps {
        fn new(stage: FaultStage) -> Self {
            Self {
                stage,
                trace: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn record(&self, stage: &'static str) {
            self.trace.lock().expect("fault trace").push(stage);
        }

        fn error(stage: &'static str, path: &Path, kind: io::ErrorKind) -> ConfigIoError {
            ConfigIoError {
                stage,
                path: path.to_path_buf(),
                kind,
                code: if kind == io::ErrorKind::PermissionDenied {
                    5
                } else {
                    0
                },
                message: format!("simulated {stage} failure"),
            }
        }

        fn replace_error(disposition: ReplacementDisposition) -> ReplaceFileError {
            ReplaceFileError {
                source: NativeFileError {
                    stage: "replace_file_with_backup",
                    code: 5,
                    kind: io::ErrorKind::PermissionDenied,
                    message: "simulated replacement failure".to_owned(),
                },
                disposition,
            }
        }
    }

    struct FaultPending {
        file: fs::File,
        stage: FaultStage,
        trace: Arc<Mutex<Vec<&'static str>>>,
    }

    impl FaultPending {
        fn record(&self, stage: &'static str) {
            self.trace.lock().expect("fault trace").push(stage);
        }
    }

    impl Write for FaultPending {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.record("write");
            if self.stage == FaultStage::Write {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "simulated write failure",
                ));
            }
            self.file.write(bytes)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.record("flush");
            if self.stage == FaultStage::Flush {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "simulated flush failure",
                ));
            }
            self.file.flush()
        }
    }

    impl ConfigFileOps for FaultOps {
        type Pending = FaultPending;

        fn create_dir_all(&self, path: &Path) -> Result<(), ConfigIoError> {
            WindowsConfigFileOps.create_dir_all(path)
        }

        fn path_exists(&self, path: &Path) -> Result<bool, ConfigIoError> {
            if self.stage == FaultStage::InspectTarget
                && path.file_name().is_some_and(|name| name == "config.toml")
                && self.trace.lock().expect("fault trace").contains(&"create")
            {
                return Err(Self::error(
                    "inspect_config_target",
                    path,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            if self.stage == FaultStage::SelectBackup
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("config.backup-"))
            {
                return Err(Self::error(
                    "inspect_backup_path",
                    path,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            WindowsConfigFileOps.path_exists(path)
        }

        fn path_is_directory(&self, path: &Path) -> Result<bool, ConfigIoError> {
            WindowsConfigFileOps.path_is_directory(path)
        }

        fn canonicalize_existing(&self, path: &Path) -> Result<Option<PathBuf>, ConfigIoError> {
            WindowsConfigFileOps.canonicalize_existing(path)
        }

        fn read_bounded(&self, path: &Path) -> Result<Vec<u8>, ConfigIoError> {
            if self.stage == FaultStage::ReadBackup
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("config.backup-"))
            {
                return Err(Self::error(
                    "read_owned_backup",
                    path,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            WindowsConfigFileOps.read_bounded(path)
        }

        fn list_owned_backups(&self, directory: &Path) -> Result<Vec<PathBuf>, ConfigIoError> {
            if self.stage == FaultStage::ListBackups {
                return Err(Self::error(
                    "list_config_backups",
                    directory,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            WindowsConfigFileOps.list_owned_backups(directory)
        }

        fn create_temp_exclusive(&self, path: &Path) -> Result<Self::Pending, ConfigIoError> {
            self.record("create");
            if self.stage == FaultStage::Create {
                return Err(Self::error(
                    "create_staging_exclusive",
                    path,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            if self.stage == FaultStage::CreateCollision {
                fs::write(path, b"foreign owner").expect("create foreign collision");
                return Err(Self::error(
                    "create_staging_exclusive",
                    path,
                    io::ErrorKind::AlreadyExists,
                ));
            }
            let file = WindowsConfigFileOps.create_temp_exclusive(path)?;
            Ok(FaultPending {
                file,
                stage: self.stage,
                trace: Arc::clone(&self.trace),
            })
        }

        fn sync(&self, path: &Path, pending: &mut Self::Pending) -> Result<(), ConfigIoError> {
            self.record("sync");
            if self.stage == FaultStage::Sync {
                return Err(Self::error(
                    "sync_staging",
                    path,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            pending.file.sync_all().map_err(|error| ConfigIoError {
                stage: "sync_staging",
                path: path.to_path_buf(),
                kind: error.kind(),
                code: error
                    .raw_os_error()
                    .and_then(|code| u32::try_from(code).ok())
                    .unwrap_or(0),
                message: error.to_string(),
            })
        }

        fn close(&self, path: &Path, pending: Self::Pending) -> Result<(), ConfigIoError> {
            self.record("close");
            drop(pending);
            if self.stage == FaultStage::Close {
                return Err(Self::error(
                    "close_staging",
                    path,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            Ok(())
        }

        fn replace_with_backup(
            &self,
            target: &Path,
            staging: &Path,
            backup: &Path,
        ) -> Result<(), ReplaceFileError> {
            self.record("replace");
            if matches!(
                self.stage,
                FaultStage::Replace | FaultStage::ReplaceAndCleanup
            ) {
                return Err(Self::replace_error(ReplacementDisposition::NamesUnchanged));
            }
            WindowsConfigFileOps.replace_with_backup(target, staging, backup)
        }

        fn install_new_noreplace(
            &self,
            staging: &Path,
            target: &Path,
        ) -> Result<(), ConfigIoError> {
            self.record("install");
            if self.stage == FaultStage::InstallRace {
                fs::write(target, b"external owner").expect("create concurrent target");
            }
            WindowsConfigFileOps.install_new_noreplace(staging, target)
        }

        fn restore_backup_noreplace(
            &self,
            backup: &Path,
            target: &Path,
        ) -> Result<(), ConfigIoError> {
            WindowsConfigFileOps.restore_backup_noreplace(backup, target)
        }

        fn preserve_corrupt_noreplace(
            &self,
            source: &Path,
            destination: &Path,
        ) -> Result<(), ConfigIoError> {
            if self.stage == FaultStage::PreserveCorrupt {
                return Err(Self::error(
                    "preserve_corrupt_noreplace",
                    destination,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            WindowsConfigFileOps.preserve_corrupt_noreplace(source, destination)
        }

        fn remove_owned(&self, path: &Path) -> Result<(), ConfigIoError> {
            self.record("cleanup");
            if self.stage == FaultStage::ReplaceAndCleanup {
                return Err(Self::error(
                    "cleanup_owned_file",
                    path,
                    io::ErrorKind::PermissionDenied,
                ));
            }
            WindowsConfigFileOps.remove_owned(path)
        }
    }

    #[derive(Clone, Copy)]
    struct Simulated1177Ops {
        fail_restore: bool,
        fail_sync: bool,
    }

    impl ConfigFileOps for Simulated1177Ops {
        type Pending = fs::File;

        fn create_dir_all(&self, path: &Path) -> Result<(), ConfigIoError> {
            WindowsConfigFileOps.create_dir_all(path)
        }

        fn path_exists(&self, path: &Path) -> Result<bool, ConfigIoError> {
            WindowsConfigFileOps.path_exists(path)
        }

        fn path_is_directory(&self, path: &Path) -> Result<bool, ConfigIoError> {
            WindowsConfigFileOps.path_is_directory(path)
        }

        fn canonicalize_existing(&self, path: &Path) -> Result<Option<PathBuf>, ConfigIoError> {
            WindowsConfigFileOps.canonicalize_existing(path)
        }

        fn read_bounded(&self, path: &Path) -> Result<Vec<u8>, ConfigIoError> {
            WindowsConfigFileOps.read_bounded(path)
        }

        fn list_owned_backups(&self, directory: &Path) -> Result<Vec<PathBuf>, ConfigIoError> {
            WindowsConfigFileOps.list_owned_backups(directory)
        }

        fn create_temp_exclusive(&self, path: &Path) -> Result<Self::Pending, ConfigIoError> {
            WindowsConfigFileOps.create_temp_exclusive(path)
        }

        fn sync(&self, path: &Path, pending: &mut Self::Pending) -> Result<(), ConfigIoError> {
            if self.fail_sync {
                return Err(ConfigIoError {
                    stage: "sync_staging",
                    path: path.to_path_buf(),
                    kind: io::ErrorKind::PermissionDenied,
                    code: 5,
                    message: "simulated sync failure".to_owned(),
                });
            }
            WindowsConfigFileOps.sync(path, pending)
        }

        fn close(&self, path: &Path, pending: Self::Pending) -> Result<(), ConfigIoError> {
            WindowsConfigFileOps.close(path, pending)
        }

        fn replace_with_backup(
            &self,
            target: &Path,
            _staging: &Path,
            backup: &Path,
        ) -> Result<(), ReplaceFileError> {
            fs::rename(target, backup).expect("simulate native target-to-backup move");
            Err(ReplaceFileError {
                source: NativeFileError {
                    stage: "replace_file_with_backup",
                    code: 1177,
                    kind: io::ErrorKind::Other,
                    message: "simulated ERROR_UNABLE_TO_MOVE_REPLACEMENT_2".to_owned(),
                },
                disposition: ReplacementDisposition::OriginalAtBackup,
            })
        }

        fn install_new_noreplace(
            &self,
            staging: &Path,
            target: &Path,
        ) -> Result<(), ConfigIoError> {
            WindowsConfigFileOps.install_new_noreplace(staging, target)
        }

        fn restore_backup_noreplace(
            &self,
            backup: &Path,
            target: &Path,
        ) -> Result<(), ConfigIoError> {
            if self.fail_restore {
                return Err(ConfigIoError {
                    stage: "restore_backup_noreplace",
                    path: target.to_path_buf(),
                    kind: io::ErrorKind::PermissionDenied,
                    code: 5,
                    message: "simulated restore failure".to_owned(),
                });
            }
            WindowsConfigFileOps.restore_backup_noreplace(backup, target)
        }

        fn preserve_corrupt_noreplace(
            &self,
            source: &Path,
            destination: &Path,
        ) -> Result<(), ConfigIoError> {
            WindowsConfigFileOps.preserve_corrupt_noreplace(source, destination)
        }

        fn remove_owned(&self, path: &Path) -> Result<(), ConfigIoError> {
            WindowsConfigFileOps.remove_owned(path)
        }
    }
}
