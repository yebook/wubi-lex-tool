//! Thin main-window command adapters.

use std::sync::Arc;

use tauri::State;

use crate::{
    error::AppError,
    window::{WindowControlIntent, WindowCoordinator, WindowStateSnapshot},
};

#[tauri::command]
#[specta::specta]
pub fn window_state(coordinator: State<'_, Arc<WindowCoordinator>>) -> WindowStateSnapshot {
    coordinator.snapshot()
}

#[tauri::command]
#[specta::specta]
pub fn window_control(
    intent: WindowControlIntent,
    coordinator: State<'_, Arc<WindowCoordinator>>,
) -> Result<WindowStateSnapshot, AppError> {
    Arc::clone(coordinator.inner()).control(intent)
}
