//! Typed application events shared by the runtime and generated bindings.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::launch::{LaunchNotice, LaunchRequest, ParsedLaunch};

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
