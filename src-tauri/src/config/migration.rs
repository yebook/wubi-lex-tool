//! Explicit adjacent schema-version routing.

use thiserror::Error;

use super::model::CURRENT_SCHEMA_VERSION;

/// Version-routing failures detected before current-model deserialization.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MigrationError {
    #[error("schemaVersion is missing")]
    Missing,
    #[error("schemaVersion must be a positive integer")]
    Invalid,
    #[error("schema version {0} is newer than this application")]
    Future(u32),
    #[error("schema version {0} has no adjacent migration path")]
    Unsupported(u32),
}

/// Validates the discriminator and applies every registered adjacent transition.
pub fn migrate_to_current(value: toml::Value) -> Result<toml::Value, MigrationError> {
    let version = value
        .as_table()
        .and_then(|table| table.get("schemaVersion"))
        .and_then(toml::Value::as_integer)
        .ok_or(MigrationError::Missing)?;
    let version = u32::try_from(version).map_err(|_| MigrationError::Invalid)?;
    if version == 0 {
        return Err(MigrationError::Invalid);
    }
    if version > CURRENT_SCHEMA_VERSION {
        return Err(MigrationError::Future(version));
    }
    if version < CURRENT_SCHEMA_VERSION {
        return Err(MigrationError::Unsupported(version));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{MigrationError, migrate_to_current};

    #[test]
    fn v1_has_no_fictional_predecessor_or_transition() {
        let current = toml::from_str("schemaVersion = 1").expect("current TOML");
        assert!(migrate_to_current(current).is_ok());

        for (source, expected) in [
            ("window = {}", MigrationError::Missing),
            ("schemaVersion = 0", MigrationError::Invalid),
            ("schemaVersion = 2", MigrationError::Future(2)),
        ] {
            let value = toml::from_str(source).expect("probe TOML");
            assert_eq!(migrate_to_current(value), Err(expected));
        }
    }
}
