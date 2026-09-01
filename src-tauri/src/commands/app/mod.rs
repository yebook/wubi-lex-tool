//! Application-shell commands.

pub(crate) mod config;
pub(crate) mod features;

use tauri::State;

use crate::runtime::{RuntimeSnapshot, RuntimeState};

/// Returns the authoritative runtime state, including events missed during bootstrap.
#[tauri::command]
#[specta::specta]
pub fn app_runtime_snapshot(state: State<'_, RuntimeState>) -> RuntimeSnapshot {
    state.snapshot()
}
