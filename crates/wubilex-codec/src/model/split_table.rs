//! Ordered spelling split-table values.

use crate::CodecError;

use super::validate_token;

/// One term and its nonempty root sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitTableEntry {
    term: String,
    roots: String,
}

impl SplitTableEntry {
    /// Validates and creates a split-table entry.
    pub fn new(term: impl Into<String>, roots: impl Into<String>) -> Result<Self, CodecError> {
        let term = term.into();
        let roots = roots.into();
        validate_token("split table term", &term)?;
        validate_token("split table roots", &roots)?;
        Ok(Self { term, roots })
    }

    /// Returns the indexed character or phrase token.
    #[must_use]
    pub fn term(&self) -> &str {
        &self.term
    }

    /// Returns the root sequence, including any PUA or non-BMP scalars.
    #[must_use]
    pub fn roots(&self) -> &str {
        &self.roots
    }
}

/// An ordered split-table stream that preserves duplicate terms.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SplitTableDocument {
    entries: Vec<SplitTableEntry>,
}

impl SplitTableDocument {
    /// Creates a document without sorting, deduplication, or map projection.
    #[must_use]
    pub const fn new(entries: Vec<SplitTableEntry>) -> Self {
        Self { entries }
    }

    /// Returns entries in source order.
    #[must_use]
    pub fn entries(&self) -> &[SplitTableEntry] {
        &self.entries
    }

    /// Returns the number of entries, including duplicates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the document has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Consumes the document and returns entries in source order.
    #[must_use]
    pub fn into_entries(self) -> Vec<SplitTableEntry> {
        self.entries
    }
}
