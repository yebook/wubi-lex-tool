//! Bounded UTF-8 TOML decoding and canonical encoding.

use thiserror::Error;

use super::{
    migration::{MigrationError, migrate_to_current},
    model::{AppConfig, ConfigValidationError},
};

pub const MAX_CONFIG_BYTES: usize = 1024 * 1024;

/// Config decoding or encoding failure without raw document contents.
#[derive(Debug, Error)]
pub enum ConfigCodecError {
    #[error("configuration exceeds the {MAX_CONFIG_BYTES}-byte limit")]
    SizeLimit,
    #[error("configuration is not valid UTF-8")]
    Utf8,
    #[error("configuration TOML could not be parsed: {0}")]
    Parse(String),
    #[error(transparent)]
    Migration(#[from] MigrationError),
    #[error(transparent)]
    Validation(#[from] ConfigValidationError),
    #[error("configuration could not be encoded: {0}")]
    Encode(String),
}

/// Decodes, version-routes, defaults, and validates one complete document.
pub fn decode(bytes: &[u8]) -> Result<AppConfig, ConfigCodecError> {
    if bytes.len() > MAX_CONFIG_BYTES {
        return Err(ConfigCodecError::SizeLimit);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| ConfigCodecError::Utf8)?;
    let value = toml::from_str::<toml::Value>(text)
        .map_err(|error| ConfigCodecError::Parse(error.to_string()))?;
    let migrated = migrate_to_current(value)?;
    let config = migrated
        .try_into::<AppConfig>()
        .map_err(|error| ConfigCodecError::Parse(error.to_string()))?;
    config.validate()?;
    Ok(config)
}

/// Encodes canonical UTF-8 TOML with LF endings and exactly one final newline.
pub fn encode(config: &AppConfig) -> Result<Vec<u8>, ConfigCodecError> {
    config.validate()?;
    let text = toml::to_string_pretty(config)
        .map_err(|error| ConfigCodecError::Encode(error.to_string()))?;
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized
        .split('\n')
        .map(str::trim_end)
        .collect::<Vec<_>>();
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    let mut output = lines.join("\n");
    output.push('\n');
    Ok(output.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::{ConfigCodecError, MAX_CONFIG_BYTES, decode, encode};
    use crate::config::{AppConfig, BindingOverride};

    #[test]
    fn canonical_roundtrip_is_deterministic_and_lf_stable() {
        let mut config = AppConfig::default();
        config
            .keymap
            .bindings
            .insert("z.action".to_owned(), BindingOverride::Unbound);
        config.keymap.bindings.insert(
            "a.action".to_owned(),
            BindingOverride::Custom {
                accelerator: "Ctrl+A".to_owned(),
            },
        );
        let first = encode(&config).expect("encode config");
        let second = encode(&decode(&first).expect("decode config")).expect("re-encode config");
        assert_eq!(first, second);
        assert!(!first.contains(&b'\r'));
        assert_eq!(first.last(), Some(&b'\n'));
        assert!(!first.ends_with(b"\n\n"));
        let text = std::str::from_utf8(&first).expect("UTF-8 output");
        assert!(text.find("a.action").expect("a key") < text.find("z.action").expect("z key"));
    }

    #[test]
    fn missing_groups_default_but_unknown_fields_and_versions_fail() {
        let minimal = decode(b"schemaVersion = 1\n").expect("defaultable fields");
        assert_eq!(minimal, AppConfig::default());

        assert!(matches!(
            decode(b"schemaVersion = 1\nunknown = true\n"),
            Err(ConfigCodecError::Parse(_))
        ));
        assert!(matches!(
            decode(b"schemaVersion = 2\n"),
            Err(ConfigCodecError::Migration(_))
        ));
        assert!(matches!(
            decode(&vec![b'x'; MAX_CONFIG_BYTES + 1]),
            Err(ConfigCodecError::SizeLimit)
        ));
    }

    #[test]
    fn invalid_enum_and_numeric_ranges_fail_without_partial_defaults() {
        assert!(decode(b"schemaVersion = 1\n[ui]\ntheme = 'blue'\n").is_err());
        assert!(decode(
            b"schemaVersion = 1\n[window]\n[window.bounds]\nx=0\ny=0\nwidth=0\nheight=600\nscaleFactor=1.0\n"
        )
        .is_err());
    }
}
