//! Ordered word-frequency table values.

use crate::{CodecError, Weight};

use super::validate_token;

/// One word and its nonzero frequency-derived weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordFrequencyEntry {
    word: String,
    weight: Weight,
}

impl WordFrequencyEntry {
    /// Validates and creates a word-frequency entry.
    pub fn new(word: impl Into<String>, weight: Weight) -> Result<Self, CodecError> {
        let word = word.into();
        validate_token("word frequency word", &word)?;
        Ok(Self { word, weight })
    }

    /// Returns the nonempty word token.
    #[must_use]
    pub fn word(&self) -> &str {
        &self.word
    }

    /// Returns the nonzero weight. Smaller values sort first.
    #[must_use]
    pub const fn weight(&self) -> Weight {
        self.weight
    }
}

/// An ordered word-frequency stream that preserves duplicate words.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WordFrequencyDocument {
    entries: Vec<WordFrequencyEntry>,
}

impl WordFrequencyDocument {
    /// Creates a document without sorting, deduplication, or map projection.
    #[must_use]
    pub const fn new(entries: Vec<WordFrequencyEntry>) -> Self {
        Self { entries }
    }

    /// Returns entries in source order.
    #[must_use]
    pub fn entries(&self) -> &[WordFrequencyEntry] {
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
    pub fn into_entries(self) -> Vec<WordFrequencyEntry> {
        self.entries
    }
}
