//! Authoritative process-runtime state exposed to the initial webview.

use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use specta::Type;
use wubilex_winime::security::NativeSecurityError;

use crate::events::LaunchRequestedEvent;

const MAX_RUNTIME_NOTICES: usize = 8;

/// Actual privilege state of the running process token.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PrivilegeState {
    Elevated,
    NotElevated,
    Unavailable,
}

/// Bounded native evidence for a failed privilege probe.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegeFailure {
    pub stage: String,
    pub code: u32,
}

/// Privilege probe outcome shown by the shell without assuming manifest success.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegeStatus {
    pub state: PrivilegeState,
    pub failure: Option<PrivilegeFailure>,
}

impl PrivilegeStatus {
    pub fn from_probe(result: Result<bool, NativeSecurityError>) -> Self {
        match result {
            Ok(true) => Self {
                state: PrivilegeState::Elevated,
                failure: None,
            },
            Ok(false) => Self {
                state: PrivilegeState::NotElevated,
                failure: None,
            },
            Err(error) => Self {
                state: PrivilegeState::Unavailable,
                failure: Some(PrivilegeFailure {
                    stage: error.stage.to_owned(),
                    code: error.code,
                }),
            },
        }
    }
}

/// Stable categories for non-launch runtime warnings.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeNoticeCode {
    LoggingUnavailable,
    SessionMarkerUnavailable,
    ElevationProbeFailed,
    WindowActivationFailed,
    WindowOperationFailed,
    WindowPersistenceFailed,
    TrayUnavailable,
}

/// A bounded warning safe to render in the status view.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeNotice {
    pub code: RuntimeNoticeCode,
    pub summary: String,
    pub detail: Option<String>,
}

impl RuntimeNotice {
    pub fn logging_unavailable(stage: &str) -> Self {
        Self {
            code: RuntimeNoticeCode::LoggingUnavailable,
            summary: "日志服务未能启动，应用将继续运行。".to_owned(),
            detail: Some(format!("失败阶段：{stage}。")),
        }
    }

    pub fn session_marker_unavailable(kind: std::io::ErrorKind) -> Self {
        Self {
            code: RuntimeNoticeCode::SessionMarkerUnavailable,
            summary: "异常会话检测不可用，应用将继续运行。".to_owned(),
            detail: Some(format!("错误类型：{kind:?}。")),
        }
    }

    pub fn elevation_probe_failed(failure: &PrivilegeFailure) -> Self {
        Self {
            code: RuntimeNoticeCode::ElevationProbeFailed,
            summary: "无法确认当前进程的管理员权限。".to_owned(),
            detail: Some(format!(
                "失败阶段：{}；系统代码：{}。",
                failure.stage, failure.code
            )),
        }
    }

    pub fn window_activation_failed() -> Self {
        Self {
            code: RuntimeNoticeCode::WindowActivationFailed,
            summary: "收到新的启动请求，但窗口未能完全置前。".to_owned(),
            detail: None,
        }
    }

    pub fn window_operation_failed(stage: &'static str) -> Self {
        Self {
            code: RuntimeNoticeCode::WindowOperationFailed,
            summary: "窗口操作未能完全完成，应用仍可继续使用。".to_owned(),
            detail: Some(format!("失败阶段：{stage}。")),
        }
    }

    pub fn window_persistence_failed(stage: &'static str) -> Self {
        Self {
            code: RuntimeNoticeCode::WindowPersistenceFailed,
            summary: "窗口位置暂时无法保存，当前窗口状态不受影响。".to_owned(),
            detail: Some(format!("失败阶段：{stage}。")),
        }
    }

    pub fn tray_unavailable(stage: &'static str) -> Self {
        Self {
            code: RuntimeNoticeCode::TrayUnavailable,
            summary: "系统托盘入口不可用，主窗口已保持可见。".to_owned(),
            detail: Some(format!("失败阶段：{stage}。")),
        }
    }
}

/// Complete process state returned on every frontend bootstrap.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub privilege: PrivilegeStatus,
    pub previous_abnormal_session_count: u32,
    pub primary_launch: LaunchRequestedEvent,
    pub latest_secondary_launch: Option<LaunchRequestedEvent>,
    pub notices: Vec<RuntimeNotice>,
}

impl RuntimeSnapshot {
    pub fn new(
        privilege: PrivilegeStatus,
        previous_abnormal_session_count: u32,
        primary_launch: LaunchRequestedEvent,
        notices: Vec<RuntimeNotice>,
    ) -> Self {
        Self {
            privilege,
            previous_abnormal_session_count,
            primary_launch,
            latest_secondary_launch: None,
            notices: notices.into_iter().take(MAX_RUNTIME_NOTICES).collect(),
        }
    }
}

/// Mutex-backed managed state. Poisoning preserves the last available snapshot.
#[derive(Debug)]
pub struct RuntimeState {
    inner: Mutex<RuntimeData>,
}

#[derive(Debug)]
struct RuntimeData {
    snapshot: RuntimeSnapshot,
    window_ready: bool,
    activation_requested: bool,
}

impl RuntimeState {
    pub fn new(snapshot: RuntimeSnapshot) -> Self {
        Self {
            inner: Mutex::new(RuntimeData {
                snapshot,
                window_ready: false,
                activation_requested: false,
            }),
        }
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        self.lock().snapshot.clone()
    }

    /// Replaces startup-only fields without losing a secondary request received during setup.
    pub fn initialize(&self, mut snapshot: RuntimeSnapshot) {
        let mut data = self.lock();
        snapshot.latest_secondary_launch = data.snapshot.latest_secondary_launch.clone();
        data.snapshot = snapshot;
    }

    pub fn record_secondary_launch(&self, launch: LaunchRequestedEvent) {
        let mut data = self.lock();
        data.snapshot.latest_secondary_launch = Some(launch);
        data.activation_requested = true;
    }

    pub fn push_notice(&self, notice: RuntimeNotice) -> bool {
        let mut data = self.lock();
        let duplicate = data
            .snapshot
            .notices
            .iter()
            .any(|current| current.code == notice.code && current.detail == notice.detail);
        if !duplicate && data.snapshot.notices.len() < MAX_RUNTIME_NOTICES {
            data.snapshot.notices.push(notice);
            true
        } else {
            false
        }
    }

    /// Marks the main window available and claims any activation queued during setup.
    pub fn mark_window_ready(&self) -> bool {
        let mut data = self.lock();
        data.window_ready = true;
        take_activation_request(&mut data)
    }

    /// Claims a queued activation only after the main window exists.
    pub fn take_activation_request(&self) -> bool {
        take_activation_request(&mut self.lock())
    }

    /// Restores an activation request after a failed native window operation.
    pub fn restore_activation_request(&self) {
        self.lock().activation_requested = true;
    }

    fn lock(&self) -> MutexGuard<'_, RuntimeData> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn take_activation_request(data: &mut RuntimeData) -> bool {
    if data.window_ready && data.activation_requested {
        data.activation_requested = false;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_RUNTIME_NOTICES, PrivilegeState, PrivilegeStatus, RuntimeNotice, RuntimeNoticeCode,
        RuntimeSnapshot, RuntimeState,
    };
    use crate::{events::LaunchRequestedEvent, launch::LaunchRequest};
    use std::sync::Arc;
    use wubilex_winime::security::NativeSecurityError;

    fn launch(path: Option<&str>) -> LaunchRequestedEvent {
        LaunchRequestedEvent {
            request: LaunchRequest {
                start_hidden: false,
                navigation_path: path.map(str::to_owned),
            },
            notices: Vec::new(),
        }
    }

    fn state() -> RuntimeState {
        RuntimeState::new(RuntimeSnapshot::new(
            PrivilegeStatus {
                state: PrivilegeState::Elevated,
                failure: None,
            },
            0,
            launch(None),
            Vec::new(),
        ))
    }

    #[test]
    fn secondary_launch_updates_the_authoritative_snapshot() {
        let state = state();
        state.record_secondary_launch(launch(Some("/settings/runtime")));
        assert_eq!(
            state
                .snapshot()
                .latest_secondary_launch
                .and_then(|launch| launch.request.navigation_path),
            Some("/settings/runtime".to_owned())
        );
    }

    #[test]
    fn startup_initialization_preserves_an_early_secondary_launch() {
        let state = state();
        state.record_secondary_launch(launch(Some("/settings/runtime")));
        state.initialize(RuntimeSnapshot::new(
            PrivilegeStatus {
                state: PrivilegeState::NotElevated,
                failure: None,
            },
            2,
            launch(None),
            Vec::new(),
        ));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.privilege.state, PrivilegeState::NotElevated);
        assert_eq!(snapshot.previous_abnormal_session_count, 2);
        assert_eq!(
            snapshot
                .latest_secondary_launch
                .and_then(|launch| launch.request.navigation_path),
            Some("/settings/runtime".to_owned())
        );
    }

    #[test]
    fn activation_waits_for_the_window_and_can_be_retried() {
        let state = state();
        state.record_secondary_launch(launch(None));
        assert!(!state.take_activation_request());
        assert!(state.mark_window_ready());
        assert!(!state.take_activation_request());

        state.restore_activation_request();
        assert!(state.take_activation_request());
    }

    #[test]
    fn privilege_projection_distinguishes_actual_state_and_native_failure() {
        assert_eq!(
            PrivilegeStatus::from_probe(Ok(true)).state,
            PrivilegeState::Elevated
        );
        assert_eq!(
            PrivilegeStatus::from_probe(Ok(false)).state,
            PrivilegeState::NotElevated
        );

        let unavailable = PrivilegeStatus::from_probe(Err(NativeSecurityError {
            stage: "OpenProcessToken",
            code: 5,
            message: "access denied".to_owned(),
        }));
        assert_eq!(unavailable.state, PrivilegeState::Unavailable);
        assert_eq!(
            unavailable.failure,
            Some(super::PrivilegeFailure {
                stage: "OpenProcessToken".to_owned(),
                code: 5,
            })
        );
    }

    #[test]
    fn snapshot_serializes_with_the_generated_ipc_field_contract() {
        let value = serde_json::to_value(state().snapshot())
            .expect("runtime snapshot must serialize for the command boundary");
        assert_eq!(value["privilege"]["state"], "elevated");
        assert_eq!(value["previousAbnormalSessionCount"], 0);
        assert_eq!(value["primaryLaunch"]["request"]["startHidden"], false);
        assert!(value.get("latestSecondaryLaunch").is_some());
    }

    #[test]
    fn poisoned_state_remains_readable_and_notice_count_is_bounded() {
        let state = Arc::new(state());
        let poison_target = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            let _guard = poison_target
                .inner
                .lock()
                .expect("test must acquire runtime lock");
            panic!("poison runtime state for recovery coverage");
        })
        .join();

        for index in 0..16 {
            state.push_notice(RuntimeNotice {
                code: RuntimeNoticeCode::WindowOperationFailed,
                summary: "窗口操作失败。".to_owned(),
                detail: Some(format!("stage={index}")),
            });
        }
        assert_eq!(state.snapshot().notices.len(), MAX_RUNTIME_NOTICES);
    }

    #[test]
    fn identical_runtime_notices_are_deduplicated() {
        let state = state();
        assert!(state.push_notice(RuntimeNotice::window_activation_failed()));
        assert!(!state.push_notice(RuntimeNotice::window_activation_failed()));
        assert_eq!(state.snapshot().notices.len(), 1);
    }
}
