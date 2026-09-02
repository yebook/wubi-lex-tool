//! Stable command-boundary errors generated for frontend consumers.

use serde::{Deserialize, Serialize};
use specta::Type;

const MAX_ERROR_DETAIL_CHARS: usize = 1_024;

/// Stable error identifiers for application commands.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AppErrorCode {
    ConfigUnavailable,
    ConfigInvalidPath,
    ConfigReadFailed,
    ConfigParseFailed,
    ConfigUnsupportedVersion,
    ConfigValidationFailed,
    ConfigPreservationFailed,
    ConfigBackupFailed,
    ConfigWriteFailed,
    ConfigReplaceFailed,
    ConfigImportFailed,
    ConfigExportFailed,
    ConfigStateFailed,
    WindowUnavailable,
    WindowOperationFailed,
}

/// Cross-command error categories.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AppErrorKind {
    Io,
    Parse,
    Network,
    Permission,
    System,
    Validation,
    Cancelled,
}

/// Requirement module responsible for an operation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum RequirementModule {
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    M7,
    M8,
}

/// Serializable error returned by every fallible application command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: AppErrorCode,
    pub kind: AppErrorKind,
    pub module: RequirementModule,
    pub message: String,
    pub detail: Option<String>,
    pub recoverable: bool,
}

impl AppError {
    pub(crate) fn config(
        code: AppErrorCode,
        kind: AppErrorKind,
        message: impl Into<String>,
        detail: Option<String>,
        recoverable: bool,
    ) -> Self {
        Self {
            code,
            kind,
            module: RequirementModule::M7,
            message: message.into(),
            detail: detail.map(|value| value.chars().take(MAX_ERROR_DETAIL_CHARS).collect()),
            recoverable,
        }
    }

    pub(crate) fn task_join(stage: &'static str, detail: impl std::fmt::Display) -> Self {
        Self::config(
            AppErrorCode::ConfigStateFailed,
            AppErrorKind::System,
            "配置任务未能完成。",
            Some(format!("stage={stage}; error={detail}")),
            true,
        )
    }

    #[cfg(feature = "desktop")]
    pub(crate) fn window(
        code: AppErrorCode,
        message: impl Into<String>,
        stage: &'static str,
        recoverable: bool,
    ) -> Self {
        Self {
            code,
            kind: AppErrorKind::System,
            module: RequirementModule::M7,
            message: message.into(),
            detail: Some(format!("stage={stage}")),
            recoverable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppError, AppErrorCode, AppErrorKind};

    #[test]
    fn serialized_error_has_stable_codes_and_bounded_detail() {
        let error = AppError::config(
            AppErrorCode::ConfigWriteFailed,
            AppErrorKind::Io,
            "配置写入失败。",
            Some("x".repeat(2_000)),
            true,
        );
        let value = serde_json::to_value(&error).expect("error must serialize");
        assert_eq!(value["code"], "configWriteFailed");
        assert_eq!(value["kind"], "io");
        assert_eq!(value["module"], "m7");
        assert_eq!(error.detail.as_deref().map(str::len), Some(1_024));
    }
}
