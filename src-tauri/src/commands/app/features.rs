//! Application feature-catalog command.

use crate::features::{self, AppFeatureCatalog};

#[tauri::command]
#[specta::specta]
pub fn app_features() -> AppFeatureCatalog {
    features::catalog()
}
