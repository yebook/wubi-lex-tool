//! Configurable resource limits for future codec implementations.

use crate::{CodecError, CodecErrorKind, ResourceKind};

/// Default maximum encoded input size: 64 MiB.
pub const DEFAULT_MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;

/// Default maximum number of records produced by decoding or expansion.
pub const DEFAULT_MAX_EXPANDED_ENTRIES: usize = 500_000;

/// Resource ceilings applied before parsing, allocation, or record expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeLimits {
    max_input_bytes: usize,
    max_expanded_entries: usize,
}

impl DecodeLimits {
    /// Creates limits. Zero is valid and rejects every nonempty value for that resource.
    #[must_use]
    pub const fn new(max_input_bytes: usize, max_expanded_entries: usize) -> Self {
        Self {
            max_input_bytes,
            max_expanded_entries,
        }
    }

    /// Returns the maximum accepted encoded input size in bytes.
    #[must_use]
    pub const fn max_input_bytes(&self) -> usize {
        self.max_input_bytes
    }

    /// Returns the maximum accepted expanded entry count.
    #[must_use]
    pub const fn max_expanded_entries(&self) -> usize {
        self.max_expanded_entries
    }

    /// Checks the complete encoded input size before parsing or allocation.
    pub fn check_input_bytes(&self, actual: usize) -> Result<(), CodecError> {
        self.check(ResourceKind::InputBytes, self.max_input_bytes, actual)
    }

    /// Checks the decoded or expanded record count before allocation or expansion.
    pub fn check_expanded_entries(&self, actual: usize) -> Result<(), CodecError> {
        self.check(
            ResourceKind::ExpandedEntries,
            self.max_expanded_entries,
            actual,
        )
    }

    fn check(&self, resource: ResourceKind, limit: usize, actual: usize) -> Result<(), CodecError> {
        if actual > limit {
            return Err(CodecError::new(CodecErrorKind::ResourceLimitExceeded {
                resource,
                limit,
                actual,
            }));
        }

        Ok(())
    }
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_INPUT_BYTES, DEFAULT_MAX_EXPANDED_ENTRIES)
    }
}
