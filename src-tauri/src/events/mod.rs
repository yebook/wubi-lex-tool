//! Typed application events shared by the runtime and generated bindings.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::config::ConfigSnapshot;
use crate::launch::{LaunchNotice, LaunchRequest, ParsedLaunch};
use crate::runtime::RuntimeNotice;
use crate::window::WindowStateSnapshot;

/// A validated launch request submitted by a secondary process.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestedEvent {
    /// Safe launch options after parser fallback.
    pub request: LaunchRequest,
    /// Visible diagnostics that caused normal-start fallback.
    pub notices: Vec<LaunchNotice>,
}

impl From<ParsedLaunch> for LaunchRequestedEvent {
    fn from(parsed: ParsedLaunch) -> Self {
        Self {
            request: parsed.request,
            notices: parsed.notices,
        }
    }
}

impl tauri_specta::Event for LaunchRequestedEvent {
    const NAME: &'static str = "app://launch-requested";
}

/// Full configuration snapshot published after a successful commit.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigChangedEvent {
    pub snapshot: ConfigSnapshot,
}

impl tauri_specta::Event for ConfigChangedEvent {
    const NAME: &'static str = "config://changed";
}

/// Authoritative main-window state published after a native transition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WindowStateChangedEvent {
    pub snapshot: WindowStateSnapshot,
}

impl tauri_specta::Event for WindowStateChangedEvent {
    const NAME: &'static str = "window://state-changed";
}

/// A bounded native warning emitted after frontend bootstrap.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeNoticeEvent {
    pub notice: RuntimeNotice,
}

impl tauri_specta::Event for RuntimeNoticeEvent {
    const NAME: &'static str = "app://runtime-notice";
}

/// Runtime-independent event adapter consumed by generic command signatures.
pub struct ConfigEventEmitter {
    emit: Box<dyn Fn(ConfigChangedEvent) -> Result<(), String> + Send + Sync>,
}

impl ConfigEventEmitter {
    pub fn new(
        emit: impl Fn(ConfigChangedEvent) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            emit: Box::new(emit),
        }
    }

    pub fn emit_changed(&self, snapshot: ConfigSnapshot) {
        if let Err(error) = (self.emit)(ConfigChangedEvent { snapshot }) {
            tracing::warn!(
                event = "config_event_emit_failed",
                stage = "config_commit",
                error = %error,
                pid = std::process::id(),
                app_version = env!("CARGO_PKG_VERSION")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigChangedEvent, RuntimeNoticeEvent, WindowStateChangedEvent};
    use crate::config::{AppConfig, ConfigPersistence, ConfigSnapshot};
    use crate::runtime::RuntimeNotice;
    use crate::window::{WindowStateSnapshot, WindowVisibility};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tauri_specta::Event;

    #[test]
    fn config_event_name_and_snapshot_fields_are_stable() {
        let event = ConfigChangedEvent {
            snapshot: ConfigSnapshot {
                revision: 4,
                config: AppConfig::default(),
                persistence: ConfigPersistence::Ready,
                notices: Vec::new(),
            },
        };
        let value = serde_json::to_value(event).expect("event must serialize");
        assert_eq!(ConfigChangedEvent::NAME, "config://changed");
        assert_eq!(value["snapshot"]["revision"], 4);
        assert_eq!(value["snapshot"]["config"]["schemaVersion"], 1);
    }

    #[test]
    fn config_emit_failure_is_observed_once_without_becoming_a_command_error() {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let emitter = super::ConfigEventEmitter::new(move |_| {
            observed.fetch_add(1, Ordering::Relaxed);
            Err("simulated listener failure".to_owned())
        });
        emitter.emit_changed(ConfigSnapshot {
            revision: 2,
            config: AppConfig::default(),
            persistence: ConfigPersistence::Ready,
            notices: Vec::new(),
        });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn window_and_runtime_event_names_and_fields_are_stable() {
        let state = WindowStateChangedEvent {
            snapshot: WindowStateSnapshot {
                revision: 7,
                visibility: WindowVisibility::Hidden,
                maximized: false,
            },
        };
        let state_value = serde_json::to_value(state).expect("window state event");
        assert_eq!(WindowStateChangedEvent::NAME, "window://state-changed");
        assert_eq!(state_value["snapshot"]["revision"], 7);
        assert_eq!(state_value["snapshot"]["visibility"], "hidden");

        let notice = RuntimeNoticeEvent {
            notice: RuntimeNotice::tray_unavailable("create_tray"),
        };
        let notice_value = serde_json::to_value(notice).expect("runtime notice event");
        assert_eq!(RuntimeNoticeEvent::NAME, "app://runtime-notice");
        assert_eq!(notice_value["notice"]["code"], "trayUnavailable");
    }
}
