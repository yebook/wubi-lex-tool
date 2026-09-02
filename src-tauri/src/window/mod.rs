//! Native main-window coordination and placement policy.

pub mod bounds;
pub mod persistence;
#[cfg(feature = "desktop")]
mod tray;

use std::sync::Arc;
#[cfg(feature = "desktop")]
use std::{
    sync::{Mutex, MutexGuard, mpsc},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use specta::Type;
#[cfg(feature = "desktop")]
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent, Wry};
#[cfg(feature = "desktop")]
use tauri_specta::Event;

use crate::error::AppError;
#[cfg(feature = "desktop")]
use crate::{
    config::{AppConfigService, CloseAction, WindowBounds, WindowConfig},
    error::AppErrorCode,
    events::{ConfigChangedEvent, RuntimeNoticeEvent, WindowStateChangedEvent},
    runtime::{RuntimeNotice, RuntimeState},
};

#[cfg(feature = "desktop")]
use self::{
    bounds::{MonitorWorkArea, PhysicalRect, logical_from_physical, restore_to_work_area},
    persistence::{EXIT_FLUSH_TIMEOUT, FlushOutcome, PlacementWorker, WindowPlacement},
};

#[cfg(feature = "desktop")]
const MAIN_WINDOW_LABEL: &str = "main";
#[cfg(feature = "desktop")]
const DELAYED_TRAY_DURATION: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WindowControlIntent {
    MinimizeToTray,
    ToggleMaximize,
    Close,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WindowVisibility {
    Visible,
    Hidden,
    Exiting,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WindowStateSnapshot {
    #[specta(type = specta_typescript::Number)]
    pub revision: u64,
    pub visibility: WindowVisibility,
    pub maximized: bool,
}

#[cfg(feature = "desktop")]
#[derive(Debug)]
struct LifecycleState {
    snapshot: WindowStateSnapshot,
    delay_generation: u64,
}

#[cfg(feature = "desktop")]
impl LifecycleState {
    fn new(visibility: WindowVisibility, maximized: bool) -> Self {
        Self {
            snapshot: WindowStateSnapshot {
                revision: 0,
                visibility,
                maximized,
            },
            delay_generation: 0,
        }
    }

    fn set_visibility(&mut self, visibility: WindowVisibility) -> bool {
        if self.snapshot.visibility == visibility
            || self.snapshot.visibility == WindowVisibility::Exiting
        {
            return false;
        }
        self.snapshot.visibility = visibility;
        self.bump_revision();
        true
    }

    fn set_maximized(&mut self, maximized: bool) -> bool {
        if self.snapshot.maximized == maximized {
            return false;
        }
        self.snapshot.maximized = maximized;
        self.bump_revision();
        true
    }

    fn invalidate_delay(&mut self) -> u64 {
        self.delay_generation = self.delay_generation.wrapping_add(1);
        self.delay_generation
    }

    fn bump_revision(&mut self) {
        self.snapshot.revision = self.snapshot.revision.saturating_add(1);
    }
}

#[cfg(feature = "desktop")]
struct CoordinatorData {
    lifecycle: LifecycleState,
    tray_present: bool,
    delay_cancel: Option<mpsc::Sender<()>>,
    last_normal_bounds: Option<WindowBounds>,
}

#[cfg(feature = "desktop")]
pub struct WindowCoordinator {
    app: AppHandle<Wry>,
    data: Mutex<CoordinatorData>,
    lifecycle_operation: Mutex<()>,
    tray_operation: Mutex<()>,
    placement: PlacementWorker,
}

#[cfg(not(feature = "desktop"))]
pub struct WindowCoordinator;

#[cfg(feature = "desktop")]
impl std::fmt::Debug for WindowCoordinator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WindowCoordinator")
            .field("snapshot", &self.snapshot())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "desktop")]
impl WindowCoordinator {
    pub fn new(
        app: AppHandle<Wry>,
        service: Arc<AppConfigService>,
        initial_visibility: WindowVisibility,
        initial_maximized: bool,
        last_normal_bounds: Option<WindowBounds>,
    ) -> Arc<Self> {
        let saved_app = app.clone();
        let placement = PlacementWorker::start(
            move |placement| {
                let snapshot =
                    service.update_window_placement(placement.bounds, placement.maximized)?;
                if let Err(error) = (ConfigChangedEvent { snapshot }).emit(&saved_app) {
                    tracing::warn!(
                        event = "config_event_emit_failed",
                        stage = "window_placement",
                        error = %error,
                        pid = std::process::id(),
                        app_version = env!("CARGO_PKG_VERSION")
                    );
                }
                Ok(())
            },
            {
                let notice_app = app.clone();
                move |error| {
                    tracing::warn!(
                        event = "window_placement_save_failed",
                        stage = "window_placement",
                        error_code = ?error.code,
                        pid = std::process::id(),
                        app_version = env!("CARGO_PKG_VERSION")
                    );
                    report_runtime_notice(
                        &notice_app,
                        RuntimeNotice::window_persistence_failed("window_placement"),
                    );
                }
            },
        );
        let placement = match placement {
            Ok(worker) => worker,
            Err(error) => {
                tracing::warn!(
                    event = "window_placement_worker_unavailable",
                    stage = "window_placement",
                    error_kind = ?error.kind(),
                    pid = std::process::id(),
                    app_version = env!("CARGO_PKG_VERSION")
                );
                report_runtime_notice(
                    &app,
                    RuntimeNotice::window_persistence_failed("placement_worker_start"),
                );
                PlacementWorker::disabled()
            }
        };

        Arc::new(Self {
            app,
            data: Mutex::new(CoordinatorData {
                lifecycle: LifecycleState::new(initial_visibility, initial_maximized),
                tray_present: false,
                delay_cancel: None,
                last_normal_bounds,
            }),
            lifecycle_operation: Mutex::new(()),
            tray_operation: Mutex::new(()),
            placement,
        })
    }

    pub fn snapshot(&self) -> WindowStateSnapshot {
        self.lock().lifecycle.snapshot.clone()
    }

    pub fn control(
        self: &Arc<Self>,
        intent: WindowControlIntent,
    ) -> Result<WindowStateSnapshot, AppError> {
        match intent {
            WindowControlIntent::MinimizeToTray => self.hide_to_tray(),
            WindowControlIntent::ToggleMaximize => self.toggle_maximize(),
            WindowControlIntent::Close => self.request_policy_close(),
        }
    }

    pub fn hide_to_tray(&self) -> Result<WindowStateSnapshot, AppError> {
        let _operation = self.lock_lifecycle_operation();
        let current = self.snapshot();
        if matches!(
            current.visibility,
            WindowVisibility::Exiting | WindowVisibility::Hidden
        ) {
            return Ok(current);
        }
        if let Err(error) = self.ensure_tray() {
            self.restore_after_tray_failure();
            return Err(error);
        }
        let window = self.main_window()?;

        let taskbar_error = window.set_skip_taskbar(true).err().map(|error| {
            self.log_native_failure("hide_set_skip_taskbar", &error);
            self.report_notice(RuntimeNotice::window_operation_failed(
                "hide_set_skip_taskbar",
            ));
            self.window_error("hide_set_skip_taskbar")
        });
        if let Err(error) = window.hide() {
            self.log_native_failure("hide_window", &error);
            let _ = window.set_skip_taskbar(false);
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
            self.report_notice(RuntimeNotice::window_operation_failed("hide_window"));
            return Err(self.window_error("hide_window"));
        }

        let snapshot = {
            let mut data = self.lock();
            data.lifecycle.set_visibility(WindowVisibility::Hidden);
            data.lifecycle.snapshot.clone()
        };
        self.publish_state(snapshot.clone());
        tracing::info!(
            event = "window_hidden",
            stage = "window_lifecycle",
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION")
        );
        if let Some(error) = taskbar_error {
            Err(error)
        } else {
            Ok(snapshot)
        }
    }

    pub fn restore(&self) -> Result<WindowStateSnapshot, AppError> {
        let _operation = self.lock_lifecycle_operation();
        self.cancel_delayed_tray();
        if self.snapshot().visibility == WindowVisibility::Exiting {
            return Ok(self.snapshot());
        }
        let window = self.main_window()?;
        let mut first_failure = None;
        run_native_stage(
            self,
            "restore_set_skip_taskbar",
            window.set_skip_taskbar(false),
            &mut first_failure,
        );
        run_native_stage(
            self,
            "restore_unminimize",
            window.unminimize(),
            &mut first_failure,
        );
        let shown = window.show();
        let show_succeeded = shown.is_ok();
        run_native_stage(self, "restore_show", shown, &mut first_failure);
        run_native_stage(
            self,
            "restore_focus",
            window.set_focus(),
            &mut first_failure,
        );
        let maximized = window
            .is_maximized()
            .unwrap_or_else(|_| self.snapshot().maximized);

        let snapshot = {
            let mut data = self.lock();
            if show_succeeded {
                data.lifecycle.set_visibility(WindowVisibility::Visible);
            }
            data.lifecycle.set_maximized(maximized);
            data.lifecycle.snapshot.clone()
        };
        self.publish_state(snapshot.clone());
        if show_succeeded {
            tracing::info!(
                event = "window_restored",
                stage = "window_lifecycle",
                pid = std::process::id(),
                app_version = env!("CARGO_PKG_VERSION")
            );
            self.observe_native_placement();
        }
        if let Some(stage) = first_failure {
            self.report_notice(RuntimeNotice::window_operation_failed(stage));
            Err(self.window_error(stage))
        } else {
            Ok(snapshot)
        }
    }

    pub fn request_exit(self: &Arc<Self>) -> WindowStateSnapshot {
        let _operation = self.lock_lifecycle_operation();
        let (snapshot, should_prepare) = {
            let mut data = self.lock();
            let should_prepare = data.lifecycle.snapshot.visibility != WindowVisibility::Exiting;
            data.lifecycle.set_visibility(WindowVisibility::Exiting);
            (data.lifecycle.snapshot.clone(), should_prepare)
        };
        if !should_prepare {
            return snapshot;
        }
        self.cancel_delayed_tray();
        self.publish_state(snapshot.clone());
        tracing::info!(
            event = "window_exit_requested",
            stage = "window_lifecycle",
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION")
        );

        let coordinator = Arc::clone(self);
        let fallback = Arc::clone(self);
        if thread::Builder::new()
            .name("window-exit".to_owned())
            .spawn(move || {
                coordinator.flush_placement();
                tray::remove_owned_tray(&coordinator.app);
                coordinator.app.exit(0);
            })
            .is_err()
        {
            fallback.report_notice(RuntimeNotice::window_operation_failed("exit_thread_start"));
            fallback.flush_placement();
            tray::remove_owned_tray(&fallback.app);
            fallback.app.exit(0);
        }
        snapshot
    }

    pub fn start_delayed_tray(self: &Arc<Self>) {
        let (generation, cancelled) = {
            let mut data = self.lock();
            if data.lifecycle.snapshot.visibility != WindowVisibility::Hidden || data.tray_present {
                return;
            }
            let generation = data.lifecycle.invalidate_delay();
            let (cancel, receiver) = mpsc::channel();
            data.delay_cancel = Some(cancel);
            (generation, receiver)
        };

        let coordinator = Arc::clone(self);
        let fallback = Arc::clone(self);
        tracing::info!(
            event = "tray_delay_scheduled",
            stage = "window_lifecycle",
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION")
        );
        if thread::Builder::new()
            .name("delayed-tray".to_owned())
            .spawn(move || {
                if cancelled.recv_timeout(DELAYED_TRAY_DURATION).is_ok() {
                    return;
                }
                if coordinator.complete_delayed_tray(generation).is_err() {
                    let _ = coordinator.restore();
                }
            })
            .is_err()
        {
            fallback.report_notice(RuntimeNotice::tray_unavailable("delay_thread_start"));
            if fallback.complete_delayed_tray(generation).is_err() {
                let _ = fallback.restore();
            }
        }
    }

    pub fn handle_window_event(self: &Arc<Self>, event: &WindowEvent) {
        match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = self.request_policy_close();
            }
            WindowEvent::Moved(_)
            | WindowEvent::Resized(_)
            | WindowEvent::ScaleFactorChanged { .. } => self.observe_native_placement(),
            _ => {}
        }
    }

    pub fn cleanup_on_run_exit(&self) {
        self.cancel_delayed_tray();
        self.flush_placement();
        tray::remove_owned_tray(&self.app);
    }

    pub fn report_notice(&self, notice: RuntimeNotice) {
        report_runtime_notice(&self.app, notice);
    }

    fn request_policy_close(self: &Arc<Self>) -> Result<WindowStateSnapshot, AppError> {
        let close_action = self
            .app
            .try_state::<Arc<AppConfigService>>()
            .and_then(|service| service.snapshot().ok())
            .map(|snapshot| snapshot.config.window.close_action);
        match close_action {
            Some(CloseAction::Exit) => Ok(self.request_exit()),
            Some(CloseAction::MinimizeToTray) => self.hide_to_tray(),
            None => {
                self.report_notice(RuntimeNotice::window_persistence_failed(
                    "read_close_action",
                ));
                self.hide_to_tray()
            }
        }
    }

    fn toggle_maximize(&self) -> Result<WindowStateSnapshot, AppError> {
        let _operation = self.lock_lifecycle_operation();
        if self.snapshot().visibility == WindowVisibility::Exiting {
            return Ok(self.snapshot());
        }
        let window = self.main_window()?;
        let maximized = window.is_maximized().map_err(|error| {
            self.log_native_failure("query_maximized", &error);
            self.report_notice(RuntimeNotice::window_operation_failed("query_maximized"));
            self.window_error("query_maximized")
        })?;
        let result = if maximized {
            window.unmaximize()
        } else {
            window.maximize()
        };
        result.map_err(|error| {
            self.log_native_failure("toggle_maximize", &error);
            self.report_notice(RuntimeNotice::window_operation_failed("toggle_maximize"));
            self.window_error("toggle_maximize")
        })?;
        let new_maximized = !maximized;
        let snapshot = {
            let mut data = self.lock();
            data.lifecycle.set_maximized(new_maximized);
            data.lifecycle.snapshot.clone()
        };
        self.schedule_current_placement(new_maximized);
        self.publish_state(snapshot.clone());
        Ok(snapshot)
    }

    fn observe_native_placement(&self) {
        if self.snapshot().visibility != WindowVisibility::Visible {
            return;
        }
        let Ok(window) = self.main_window() else {
            return;
        };
        if window.is_minimized().unwrap_or(false) {
            let _ = self.hide_to_tray();
            return;
        }
        let maximized = match window.is_maximized() {
            Ok(value) => value,
            Err(error) => {
                self.log_native_failure("observe_maximized", &error);
                self.report_notice(RuntimeNotice::window_operation_failed("observe_maximized"));
                return;
            }
        };
        let snapshot_changed = {
            let mut data = self.lock();
            data.lifecycle.set_maximized(maximized)
        };
        if !maximized {
            match sample_normal_bounds(&window) {
                Some(bounds) => self.lock().last_normal_bounds = Some(bounds),
                None => self.report_notice(RuntimeNotice::window_persistence_failed(
                    "sample_normal_bounds",
                )),
            }
        }
        self.schedule_current_placement(maximized);
        if snapshot_changed {
            self.publish_state(self.snapshot());
        }
    }

    fn schedule_current_placement(&self, maximized: bool) {
        let bounds = self.lock().last_normal_bounds.clone();
        if !self
            .placement
            .schedule(WindowPlacement { bounds, maximized })
        {
            self.report_notice(RuntimeNotice::window_persistence_failed(
                "placement_worker_unavailable",
            ));
        }
    }

    fn ensure_tray(&self) -> Result<(), AppError> {
        let _operation = self
            .tray_operation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.app.tray_by_id(tray::TRAY_ID).is_some() {
            self.lock().tray_present = true;
            return Ok(());
        }
        if let Err(error) = tray::create_owned_tray(&self.app) {
            self.log_native_failure("create_tray", &error);
            self.report_notice(RuntimeNotice::tray_unavailable("create_tray"));
            return Err(self.window_error("create_tray"));
        }
        self.lock().tray_present = true;
        tracing::info!(
            event = "tray_created",
            stage = "window_lifecycle",
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION")
        );
        Ok(())
    }

    fn complete_delayed_tray(&self, generation: u64) -> Result<bool, AppError> {
        let _operation = self.lock_lifecycle_operation();
        let should_create = {
            let mut data = self.lock();
            let current = data.lifecycle.delay_generation == generation
                && data.lifecycle.snapshot.visibility == WindowVisibility::Hidden
                && !data.tray_present;
            if current {
                data.delay_cancel = None;
            }
            current
        };
        if !should_create {
            return Ok(false);
        }
        self.ensure_tray()?;
        Ok(true)
    }

    fn restore_after_tray_failure(&self) {
        let Ok(window) = self.main_window() else {
            return;
        };
        let mut first_failure = None;
        run_native_stage(
            self,
            "tray_failure_set_skip_taskbar",
            window.set_skip_taskbar(false),
            &mut first_failure,
        );
        run_native_stage(
            self,
            "tray_failure_unminimize",
            window.unminimize(),
            &mut first_failure,
        );
        run_native_stage(self, "tray_failure_show", window.show(), &mut first_failure);
        run_native_stage(
            self,
            "tray_failure_focus",
            window.set_focus(),
            &mut first_failure,
        );
        if let Some(stage) = first_failure {
            self.report_notice(RuntimeNotice::window_operation_failed(stage));
        }
    }

    fn cancel_delayed_tray(&self) {
        let cancel = {
            let mut data = self.lock();
            data.lifecycle.invalidate_delay();
            data.delay_cancel.take()
        };
        if let Some(cancel) = cancel {
            let _ = cancel.send(());
            tracing::info!(
                event = "tray_delay_cancelled",
                stage = "window_lifecycle",
                pid = std::process::id(),
                app_version = env!("CARGO_PKG_VERSION")
            );
        }
    }

    fn flush_placement(&self) {
        match self.placement.flush_and_stop(EXIT_FLUSH_TIMEOUT) {
            FlushOutcome::Flushed | FlushOutcome::Disconnected => {}
            FlushOutcome::TimedOut => tracing::warn!(
                event = "window_placement_flush_timeout",
                stage = "application_exit",
                pid = std::process::id(),
                app_version = env!("CARGO_PKG_VERSION")
            ),
        }
    }

    fn main_window(&self) -> Result<WebviewWindow<Wry>, AppError> {
        self.app
            .get_webview_window(MAIN_WINDOW_LABEL)
            .ok_or_else(|| {
                self.report_notice(RuntimeNotice::window_operation_failed(
                    "main_window_unavailable",
                ));
                AppError::window(
                    AppErrorCode::WindowUnavailable,
                    "主窗口暂时不可用。",
                    "main_window_unavailable",
                    true,
                )
            })
    }

    fn publish_state(&self, snapshot: WindowStateSnapshot) {
        if let Err(error) = (WindowStateChangedEvent { snapshot }).emit(&self.app) {
            tracing::warn!(
                event = "window_state_emit_failed",
                stage = "window_lifecycle",
                error = %error,
                pid = std::process::id(),
                app_version = env!("CARGO_PKG_VERSION")
            );
        }
    }

    fn window_error(&self, stage: &'static str) -> AppError {
        AppError::window(
            AppErrorCode::WindowOperationFailed,
            "窗口操作未能完全完成。",
            stage,
            true,
        )
    }

    fn log_native_failure(&self, stage: &'static str, error: &impl std::fmt::Display) {
        tracing::warn!(
            event = "window_operation_failed",
            stage,
            error = %error,
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION")
        );
    }

    fn lock(&self) -> MutexGuard<'_, CoordinatorData> {
        self.data
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_lifecycle_operation(&self) -> MutexGuard<'_, ()> {
        self.lifecycle_operation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(not(feature = "desktop"))]
impl WindowCoordinator {
    pub fn snapshot(&self) -> WindowStateSnapshot {
        unreachable!("window coordinator is only available in desktop builds")
    }

    pub fn control(
        self: &Arc<Self>,
        _intent: WindowControlIntent,
    ) -> Result<WindowStateSnapshot, AppError> {
        unreachable!("window coordinator is only available in desktop builds")
    }
}

#[cfg(feature = "desktop")]
pub fn apply_initial_placement(
    window: &WebviewWindow<Wry>,
    config: &WindowConfig,
) -> Vec<&'static str> {
    let mut failures = Vec::new();
    if let Some(saved) = config.bounds.as_ref() {
        match monitor_work_areas(window) {
            Ok(work_areas) => {
                if let Some(placement) = restore_to_work_area(saved, &work_areas) {
                    if window
                        .set_position(PhysicalPosition::new(placement.rect.x, placement.rect.y))
                        .is_err()
                    {
                        failures.push("restore_position");
                    }
                    if window
                        .set_size(PhysicalSize::new(
                            placement.rect.width,
                            placement.rect.height,
                        ))
                        .is_err()
                    {
                        failures.push("restore_size");
                    }
                } else {
                    failures.push("restore_monitor_selection");
                    if window.center().is_err() {
                        failures.push("restore_center");
                    }
                }
            }
            Err(stage) => {
                failures.push(stage);
                if window.center().is_err() {
                    failures.push("restore_center");
                }
            }
        }
    } else if window.center().is_err() {
        failures.push("initial_center");
    }
    if config.maximized && window.maximize().is_err() {
        failures.push("restore_maximized");
    }
    failures
}

#[cfg(feature = "desktop")]
pub fn report_runtime_notice(app: &AppHandle<Wry>, notice: RuntimeNotice) {
    let should_emit = app
        .try_state::<RuntimeState>()
        .is_none_or(|runtime| runtime.push_notice(notice.clone()));
    if !should_emit {
        return;
    }
    if let Err(error) = (RuntimeNoticeEvent { notice }).emit(app) {
        tracing::warn!(
            event = "runtime_notice_emit_failed",
            stage = "window_lifecycle",
            error = %error,
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION")
        );
    }
}

#[cfg(feature = "desktop")]
fn run_native_stage(
    coordinator: &WindowCoordinator,
    stage: &'static str,
    result: tauri::Result<()>,
    first_failure: &mut Option<&'static str>,
) {
    if let Err(error) = result {
        coordinator.log_native_failure(stage, &error);
        first_failure.get_or_insert(stage);
    }
}

#[cfg(feature = "desktop")]
fn sample_normal_bounds(window: &WebviewWindow<Wry>) -> Option<WindowBounds> {
    let position = window.outer_position().ok()?;
    let size = window.inner_size().ok()?;
    let scale_factor = window.scale_factor().ok()?;
    logical_from_physical(
        PhysicalRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        },
        scale_factor,
    )
}

#[cfg(feature = "desktop")]
fn monitor_work_areas(window: &WebviewWindow<Wry>) -> Result<Vec<MonitorWorkArea>, &'static str> {
    let monitors = window
        .available_monitors()
        .map_err(|_| "query_available_monitors")?;
    let primary = window
        .primary_monitor()
        .map_err(|_| "query_primary_monitor")?;
    Ok(monitors
        .into_iter()
        .map(|monitor| {
            let area = monitor.work_area();
            let is_primary = primary
                .as_ref()
                .is_some_and(|primary| same_monitor(primary, &monitor));
            MonitorWorkArea {
                rect: PhysicalRect {
                    x: area.position.x,
                    y: area.position.y,
                    width: area.size.width,
                    height: area.size.height,
                },
                scale_factor: monitor.scale_factor(),
                primary: is_primary,
            }
        })
        .collect())
}

#[cfg(feature = "desktop")]
fn same_monitor(left: &tauri::Monitor, right: &tauri::Monitor) -> bool {
    left.work_area().position == right.work_area().position
        && left.work_area().size == right.work_area().size
        && left.scale_factor() == right.scale_factor()
}

#[cfg(all(test, feature = "desktop"))]
mod tests {
    use super::{LifecycleState, WindowVisibility};

    #[test]
    fn lifecycle_transitions_are_idempotent_and_revisioned() {
        let mut state = LifecycleState::new(WindowVisibility::Visible, false);
        assert!(!state.set_visibility(WindowVisibility::Visible));
        assert_eq!(state.snapshot.revision, 0);
        assert!(state.set_visibility(WindowVisibility::Hidden));
        assert_eq!(state.snapshot.revision, 1);
        assert!(!state.set_visibility(WindowVisibility::Hidden));
        assert!(state.set_maximized(true));
        assert_eq!(state.snapshot.revision, 2);
        assert!(!state.set_maximized(true));
        assert!(state.set_visibility(WindowVisibility::Exiting));
        assert_eq!(state.snapshot.revision, 3);
        assert!(!state.set_visibility(WindowVisibility::Visible));
        assert!(!state.set_visibility(WindowVisibility::Hidden));
        assert_eq!(state.snapshot.visibility, WindowVisibility::Exiting);
        assert_eq!(state.snapshot.revision, 3);
    }

    #[test]
    fn delayed_tray_generation_invalidates_stale_timeouts() {
        let mut state = LifecycleState::new(WindowVisibility::Hidden, false);
        let first = state.invalidate_delay();
        let second = state.invalidate_delay();
        assert_ne!(first, second);
        assert_eq!(state.delay_generation, second);
    }
}
