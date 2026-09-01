//! Thin asynchronous configuration command adapters.

use std::sync::Arc;

use tauri::State;

use crate::{
    config::{
        AppConfigService, ConfigExportResult, ConfigGroup, ConfigPathRequest, ConfigSnapshot,
        KeymapConfig, UiConfig, WindowConfig,
    },
    error::AppError,
    events::ConfigEventEmitter,
};

#[tauri::command]
#[specta::specta]
pub async fn config_snapshot(
    service: State<'_, Arc<AppConfigService>>,
) -> Result<ConfigSnapshot, AppError> {
    let service = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || service.snapshot())
        .await
        .map_err(|error| AppError::task_join("config_snapshot", error))?
}

#[tauri::command]
#[specta::specta]
pub async fn config_update_window(
    window: WindowConfig,
    service: State<'_, Arc<AppConfigService>>,
    events: State<'_, ConfigEventEmitter>,
) -> Result<ConfigSnapshot, AppError> {
    let service = Arc::clone(service.inner());
    let snapshot = tauri::async_runtime::spawn_blocking(move || service.update_window(window))
        .await
        .map_err(|error| AppError::task_join("config_update_window", error))??;
    events.emit_changed(snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn config_update_ui(
    ui: UiConfig,
    service: State<'_, Arc<AppConfigService>>,
    events: State<'_, ConfigEventEmitter>,
) -> Result<ConfigSnapshot, AppError> {
    let service = Arc::clone(service.inner());
    let snapshot = tauri::async_runtime::spawn_blocking(move || service.update_ui(ui))
        .await
        .map_err(|error| AppError::task_join("config_update_ui", error))??;
    events.emit_changed(snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn config_update_keymap(
    keymap: KeymapConfig,
    service: State<'_, Arc<AppConfigService>>,
    events: State<'_, ConfigEventEmitter>,
) -> Result<ConfigSnapshot, AppError> {
    let service = Arc::clone(service.inner());
    let snapshot = tauri::async_runtime::spawn_blocking(move || service.update_keymap(keymap))
        .await
        .map_err(|error| AppError::task_join("config_update_keymap", error))??;
    events.emit_changed(snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn config_restore_defaults(
    group: ConfigGroup,
    service: State<'_, Arc<AppConfigService>>,
    events: State<'_, ConfigEventEmitter>,
) -> Result<ConfigSnapshot, AppError> {
    let service = Arc::clone(service.inner());
    let snapshot = tauri::async_runtime::spawn_blocking(move || service.restore_defaults(group))
        .await
        .map_err(|error| AppError::task_join("config_restore_defaults", error))??;
    events.emit_changed(snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn config_import(
    request: ConfigPathRequest,
    service: State<'_, Arc<AppConfigService>>,
    events: State<'_, ConfigEventEmitter>,
) -> Result<ConfigSnapshot, AppError> {
    let service = Arc::clone(service.inner());
    let snapshot = tauri::async_runtime::spawn_blocking(move || service.import(request))
        .await
        .map_err(|error| AppError::task_join("config_import", error))??;
    events.emit_changed(snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
#[specta::specta]
pub async fn config_export(
    request: ConfigPathRequest,
    service: State<'_, Arc<AppConfigService>>,
) -> Result<ConfigExportResult, AppError> {
    let service = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || service.export(request))
        .await
        .map_err(|error| AppError::task_join("config_export", error))?
}
