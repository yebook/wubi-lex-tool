//! Schema v1 configuration types, defaults, and bounded validation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const MAX_KEYMAP_OVERRIDES: usize = 512;

/// Complete versioned application configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub keymap: KeymapConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            window: WindowConfig::default(),
            ui: UiConfig::default(),
            keymap: KeymapConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(ConfigValidationError::new(
                "schemaVersion",
                "version does not match the current schema",
            ));
        }
        self.window.validate()?;
        self.keymap.validate()
    }
}

/// Durable window preferences. Display correction belongs to the window layer.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowConfig {
    #[serde(default)]
    pub bounds: Option<WindowBounds>,
    #[serde(default)]
    pub maximized: bool,
    #[serde(default)]
    pub close_action: CloseAction,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            bounds: None,
            maximized: false,
            close_action: CloseAction::MinimizeToTray,
        }
    }
}

impl WindowConfig {
    fn validate(&self) -> Result<(), ConfigValidationError> {
        let Some(bounds) = &self.bounds else {
            return Ok(());
        };
        if !(-1_000_000..=1_000_000).contains(&bounds.x)
            || !(-1_000_000..=1_000_000).contains(&bounds.y)
        {
            return Err(ConfigValidationError::new(
                "window.bounds.position",
                "coordinates are outside the supported range",
            ));
        }
        if !(1..=32_768).contains(&bounds.width) || !(1..=32_768).contains(&bounds.height) {
            return Err(ConfigValidationError::new(
                "window.bounds.size",
                "dimensions are outside the supported range",
            ));
        }
        if !bounds.scale_factor.is_finite() || !(0.5..=8.0).contains(&bounds.scale_factor) {
            return Err(ConfigValidationError::new(
                "window.bounds.scaleFactor",
                "scale factor is outside the supported range",
            ));
        }
        Ok(())
    }
}

/// Saved logical window rectangle and scale factor.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[specta(type = specta_typescript::Number)]
    pub scale_factor: f64,
}

/// Close-button behavior consumed by the later window/tray task.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CloseAction {
    #[default]
    MinimizeToTray,
    Exit,
}

/// Durable user-interface preferences.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiConfig {
    #[serde(default)]
    pub theme: ThemePreference,
    #[serde(default)]
    pub density: Density,
    #[serde(default)]
    pub locale: AppLocale,
    #[serde(default)]
    pub sidebar_collapsed: bool,
    #[serde(default)]
    pub onboarding_version: u32,
}

/// Theme preference; rendering belongs to the theme task.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

/// Interface density preference.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Density {
    #[default]
    Standard,
    Compact,
}

/// Supported application locale.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
pub enum AppLocale {
    #[default]
    #[serde(rename = "zh-CN")]
    ZhCn,
}

/// Durable keymap overrides. Registered actions and conflict checks are later concerns.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeymapConfig {
    #[serde(default)]
    pub bindings: BTreeMap<String, BindingOverride>,
}

impl KeymapConfig {
    fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.bindings.len() > MAX_KEYMAP_OVERRIDES {
            return Err(ConfigValidationError::new(
                "keymap.bindings",
                "too many keymap overrides",
            ));
        }
        for (action_id, binding) in &self.bindings {
            let valid_id = (1..=96).contains(&action_id.len())
                && action_id.bytes().all(|value| {
                    value.is_ascii_lowercase()
                        || value.is_ascii_digit()
                        || matches!(value, b'.' | b'_' | b'-')
                });
            if !valid_id {
                return Err(ConfigValidationError::new(
                    "keymap.bindings.actionId",
                    "action id has an invalid shape",
                ));
            }
            if let BindingOverride::Custom { accelerator } = binding {
                let count = accelerator.chars().count();
                if !(1..=128).contains(&count) || accelerator.chars().any(char::is_control) {
                    return Err(ConfigValidationError::new(
                        "keymap.bindings.accelerator",
                        "accelerator has an invalid shape",
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Distinguishes a custom binding from an explicitly cleared action binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum BindingOverride {
    Custom { accelerator: String },
    Unbound,
}

/// A bounded schema validation failure that does not expose configuration values.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{field}: {reason}")]
pub struct ConfigValidationError {
    pub field: &'static str,
    pub reason: &'static str,
}

impl ConfigValidationError {
    fn new(field: &'static str, reason: &'static str) -> Self {
        Self { field, reason }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, BindingOverride, KeymapConfig, MAX_KEYMAP_OVERRIDES, WindowBounds};

    #[test]
    fn defaults_are_valid_and_unbound_is_distinct_from_absence() {
        let mut config = AppConfig::default();
        config
            .keymap
            .bindings
            .insert("shell.search".to_owned(), BindingOverride::Unbound);
        assert!(config.validate().is_ok());
        assert_ne!(config.keymap, KeymapConfig::default());
    }

    #[test]
    fn validation_rejects_non_finite_bounds_and_unbounded_keymap_values() {
        let mut config = AppConfig::default();
        config.window.bounds = Some(WindowBounds {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            scale_factor: f64::NAN,
        });
        assert_eq!(
            config.validate().expect_err("NaN must fail").field,
            "window.bounds.scaleFactor"
        );

        let mut config = AppConfig::default();
        config.keymap.bindings.insert(
            "INVALID".to_owned(),
            BindingOverride::Custom {
                accelerator: "Ctrl+K".to_owned(),
            },
        );
        assert_eq!(
            config.validate().expect_err("invalid id must fail").field,
            "keymap.bindings.actionId"
        );
    }

    #[test]
    fn every_bounded_field_accepts_edges_and_rejects_values_beyond_them() {
        let mut config = AppConfig::default();
        config.window.bounds = Some(WindowBounds {
            x: -1_000_000,
            y: 1_000_000,
            width: 1,
            height: 32_768,
            scale_factor: 0.5,
        });
        config.keymap.bindings.insert(
            "a".repeat(96),
            BindingOverride::Custom {
                accelerator: "x".repeat(128),
            },
        );
        assert!(config.validate().is_ok());

        for bounds in [
            WindowBounds {
                x: -1_000_001,
                y: 0,
                width: 1,
                height: 1,
                scale_factor: 1.0,
            },
            WindowBounds {
                x: 0,
                y: 1_000_001,
                width: 1,
                height: 1,
                scale_factor: 1.0,
            },
            WindowBounds {
                x: 0,
                y: 0,
                width: 32_769,
                height: 1,
                scale_factor: 1.0,
            },
            WindowBounds {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                scale_factor: 8.01,
            },
        ] {
            let mut invalid = AppConfig::default();
            invalid.window.bounds = Some(bounds);
            assert!(invalid.validate().is_err());
        }

        for (action_id, accelerator) in [
            (String::new(), "Ctrl+A".to_owned()),
            ("a".repeat(97), "Ctrl+A".to_owned()),
            ("Uppercase".to_owned(), "Ctrl+A".to_owned()),
            ("valid.action".to_owned(), String::new()),
            ("valid.action".to_owned(), "x".repeat(129)),
            ("valid.action".to_owned(), "Ctrl+\nA".to_owned()),
        ] {
            let mut invalid = AppConfig::default();
            invalid
                .keymap
                .bindings
                .insert(action_id, BindingOverride::Custom { accelerator });
            assert!(invalid.validate().is_err());
        }

        let mut too_many = AppConfig::default();
        for index in 0..=MAX_KEYMAP_OVERRIDES {
            too_many
                .keymap
                .bindings
                .insert(format!("action.{index}"), BindingOverride::Unbound);
        }
        assert_eq!(
            too_many
                .validate()
                .expect_err("too many overrides must fail")
                .field,
            "keymap.bindings"
        );
    }
}
