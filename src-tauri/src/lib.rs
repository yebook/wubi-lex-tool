//! Tauri application shell and process-lifecycle contracts for WubiLex.

pub mod bindings;
pub mod commands;
pub mod config;
pub mod error;
pub mod events;
pub mod features;
pub mod launch;
pub mod logging;
pub mod recovery;
pub mod runtime;

#[cfg(feature = "desktop")]
use std::{
    ffi::OsString,
    io,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "desktop")]
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, Wry};
#[cfg(feature = "desktop")]
use tauri_specta::Event;

/// Builds and runs the desktop application, returning its process exit code.
#[cfg(feature = "desktop")]
pub fn run() -> Result<i32, tauri::Error> {
    let primary_launch = launch::parse_launch_args(std::env::args_os().skip(1));
    let start_hidden = primary_launch.request.start_hidden;
    let registry = bindings::builder::<Wry>();
    let invoke_handler = registry.invoke_handler();
    let initial_runtime = runtime::RuntimeState::new(runtime::RuntimeSnapshot::new(
        runtime::PrivilegeStatus {
            state: runtime::PrivilegeState::Unavailable,
            failure: None,
        },
        0,
        events::LaunchRequestedEvent::from(primary_launch.clone()),
        Vec::new(),
    ));

    let app = tauri::Builder::default()
        .manage(initial_runtime)
        .plugin(tauri_plugin_single_instance::init(handle_secondary_launch))
        .invoke_handler(invoke_handler)
        .setup(move |app| {
            registry.mount_events(app);
            let config_service = match app.path().app_config_dir() {
                Ok(directory) => {
                    config::ConfigService::load(directory, config::WindowsConfigFileOps)
                }
                Err(error) => config::ConfigService::unavailable(
                    format!("stage=resolve_app_config_directory; error={error}"),
                    config::WindowsConfigFileOps,
                ),
            };
            app.manage(Arc::new(config_service));
            let config_event_handle = app.handle().clone();
            app.manage(events::ConfigEventEmitter::new(move |event| {
                event
                    .emit(&config_event_handle)
                    .map_err(|error| error.to_string())
            }));
            let mut notices = Vec::new();
            let app_data_directory = app.path().app_data_dir();

            let logging_guard = match &app_data_directory {
                Ok(directory) => match logging::initialize(&directory.join("logs")) {
                    Ok(guard) => Some(guard),
                    Err(error) => {
                        notices.push(runtime::RuntimeNotice::logging_unavailable(error.stage()));
                        None
                    }
                },
                Err(_) => {
                    notices.push(runtime::RuntimeNotice::logging_unavailable(
                        "resolve_app_data_directory",
                    ));
                    None
                }
            };

            let (marker, previous_abnormal_session_count) = match &app_data_directory {
                Ok(directory) => match create_session_marker(&directory.join("sessions")) {
                    Ok(marker) => {
                        let count = u32::try_from(marker.previous_abnormal_session_count())
                            .unwrap_or(u32::MAX);
                        (Some(marker), count)
                    }
                    Err(error) => {
                        notices.push(runtime::RuntimeNotice::session_marker_unavailable(
                            error.kind(),
                        ));
                        (None, 0)
                    }
                },
                Err(_) => {
                    notices.push(runtime::RuntimeNotice::session_marker_unavailable(
                        io::ErrorKind::NotFound,
                    ));
                    (None, 0)
                }
            };

            let privilege = runtime::PrivilegeStatus::from_probe(
                wubilex_winime::security::current_process_is_elevated(),
            );
            if let Some(failure) = privilege.failure.as_ref() {
                notices.push(runtime::RuntimeNotice::elevation_probe_failed(failure));
            }
            log_launch_notices("primary_launch", &primary_launch.notices);

            let runtime_state = app.state::<runtime::RuntimeState>();
            runtime_state.initialize(runtime::RuntimeSnapshot::new(
                privilege,
                previous_abnormal_session_count,
                events::LaunchRequestedEvent::from(primary_launch.clone()),
                notices,
            ));
            app.manage(recovery::SessionLifecycle::new(marker));
            if let Some(guard) = logging_guard {
                app.manage(guard);
            }

            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("WubiLex")
                .inner_size(960.0, 680.0)
                .min_inner_size(720.0, 520.0)
                .center()
                .visible(!start_hidden)
                .focused(!start_hidden)
                .build()?;

            if runtime_state.mark_window_ready() {
                activate_main_window(app.handle(), &runtime_state);
            }

            tracing::info!(
                event = "application_started",
                stage = "setup",
                pid = std::process::id(),
                app_version = env!("CARGO_PKG_VERSION"),
                start_hidden,
                previous_abnormal_session_count
            );
            Ok(())
        })
        .build(tauri::generate_context!())?;

    Ok(app.run_return(|app_handle, event| {
        if matches!(event, RunEvent::Exit)
            && let Some(lifecycle) = app_handle.try_state::<recovery::SessionLifecycle>()
        {
            if lifecycle.clean_exit().is_ok() {
                tracing::info!(
                    event = "application_exit",
                    stage = "session_cleanup",
                    pid = std::process::id(),
                    app_version = env!("CARGO_PKG_VERSION")
                );
            } else {
                tracing::error!(
                    event = "session_marker_cleanup_failed",
                    stage = "session_cleanup",
                    pid = std::process::id(),
                    app_version = env!("CARGO_PKG_VERSION")
                );
            }
        }
    }))
}

#[cfg(feature = "desktop")]
fn handle_secondary_launch(
    app_handle: &tauri::AppHandle<Wry>,
    arguments: Vec<String>,
    _working_directory: String,
) {
    let parsed = launch::parse_launch_args(arguments.into_iter().skip(1).map(OsString::from));
    let event = events::LaunchRequestedEvent::from(parsed);
    log_launch_notices("secondary_launch", &event.notices);
    if let Some(state) = app_handle.try_state::<runtime::RuntimeState>() {
        state.record_secondary_launch(event.clone());
        if state.take_activation_request() {
            activate_main_window(app_handle, &state);
        }
    }

    if event.emit(app_handle).is_err() {
        tracing::warn!(
            event = "launch_event_emit_failed",
            stage = "secondary_launch",
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION")
        );
    }

    tracing::info!(
        event = "secondary_launch_received",
        stage = "secondary_launch",
        pid = std::process::id(),
        app_version = env!("CARGO_PKG_VERSION"),
        notice_count = event.notices.len()
    );
}

#[cfg(feature = "desktop")]
fn activate_main_window(app_handle: &tauri::AppHandle<Wry>, state: &runtime::RuntimeState) {
    let activation_succeeded = app_handle.get_webview_window("main").is_some_and(|window| {
        let unminimized = window.unminimize().is_ok();
        let shown = window.show().is_ok();
        let focused = window.set_focus().is_ok();
        unminimized && shown && focused
    });
    if activation_succeeded {
        return;
    }

    state.restore_activation_request();
    state.push_notice(runtime::RuntimeNotice::window_activation_failed());
    tracing::warn!(
        event = "window_activation_failed",
        stage = "secondary_launch",
        pid = std::process::id(),
        app_version = env!("CARGO_PKG_VERSION")
    );
}

#[cfg(feature = "desktop")]
fn log_launch_notices(stage: &'static str, notices: &[launch::LaunchNotice]) {
    for evidence in launch_notice_evidence(notices) {
        tracing::warn!(
            event = "launch_argument_notice",
            stage,
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION"),
            notice_code = ?evidence.code,
            argument_position = evidence.argument_position
        );
    }
}

#[cfg(feature = "desktop")]
#[derive(Debug, Eq, PartialEq)]
struct LaunchNoticeEvidence {
    code: launch::LaunchNoticeCode,
    argument_position: Option<u16>,
}

#[cfg(feature = "desktop")]
fn launch_notice_evidence(notices: &[launch::LaunchNotice]) -> Vec<LaunchNoticeEvidence> {
    notices
        .iter()
        .map(|notice| LaunchNoticeEvidence {
            code: notice.code,
            argument_position: notice.argument_position,
        })
        .collect()
}

#[cfg(feature = "desktop")]
fn create_session_marker(directory: &std::path::Path) -> io::Result<recovery::SessionMarker> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?;
    let session_id = format!("{:x}-{:x}", std::process::id(), elapsed.as_nanos());
    let started_at_unix_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
    recovery::SessionMarker::create(
        directory,
        &session_id,
        std::process::id(),
        env!("CARGO_PKG_VERSION"),
        started_at_unix_ms,
    )
}

#[cfg(all(test, feature = "desktop"))]
mod desktop_tests {
    use super::{LaunchNoticeEvidence, launch_notice_evidence};
    use crate::launch::{LaunchNotice, LaunchNoticeCode};

    #[test]
    fn launch_log_projection_excludes_summary_detail_and_argument_value() {
        let evidence = launch_notice_evidence(&[LaunchNotice {
            code: LaunchNoticeCode::UnknownArgument,
            summary: "secret-summary".to_owned(),
            detail: Some("secret-argument-value".to_owned()),
            argument_position: Some(2),
        }]);

        assert_eq!(
            evidence,
            vec![LaunchNoticeEvidence {
                code: LaunchNoticeCode::UnknownArgument,
                argument_position: Some(2),
            }]
        );
        let rendered = format!("{evidence:?}");
        assert!(!rendered.contains("secret-summary"));
        assert!(!rendered.contains("secret-argument-value"));
    }
}
