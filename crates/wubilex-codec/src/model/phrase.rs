//! Ordered phrase document values.

use std::num::NonZeroU8;

use crate::{CodecError, InvalidInputReason};

use super::lexicon::{validate_code, validate_text};

/// A nonempty phrase code containing lowercase ASCII letters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhraseCode(String);

impl PhraseCode {
    /// Validates and creates a phrase code without imposing the `.lex` four-code limit.
    pub fn new(value: impl Into<String>) -> Result<Self, CodecError> {
        let value = value.into();
        validate_code("phrase code", &value, None)?;
        Ok(Self(value))
    }

    /// Returns the validated code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for PhraseCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A one-based EUDP candidate position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Candidate(NonZeroU8);

impl Candidate {
    /// Creates a candidate position in the inclusive range `1..=255`.
    pub fn new(value: u8) -> Result<Self, CodecError> {
        NonZeroU8::new(value)
            .map(Self)
            .ok_or_else(|| CodecError::invalid_input("phrase candidate", InvalidInputReason::Zero))
    }

    /// Returns the encoded one-based candidate position.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0.get()
    }
}

/// One validated phrase record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseEntry {
    code: PhraseCode,
    text: String,
    candidate: Candidate,
}

impl PhraseEntry {
    /// Creates a phrase entry with a concrete one-based candidate position.
    pub fn new(
        code: PhraseCode,
        text: impl Into<String>,
        candidate: Candidate,
    ) -> Result<Self, CodecError> {
        let text = text.into();
        validate_text("phrase text", &text)?;
        Ok(Self {
            code,
            text,
            candidate,
        })
    }

    /// Returns the code.
    #[must_use]
    pub const fn code(&self) -> &PhraseCode {
        &self.code
    }

    /// Returns the nonempty phrase text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the one-based candidate position.
    #[must_use]
    pub const fn candidate(&self) -> Candidate {
        self.candidate
    }

    /// Counts UTF-16 code units on demand, including both halves of surrogate pairs.
    #[must_use]
    pub fn utf16_len(&self) -> usize {
        self.text.encode_utf16().count()
    }
}

/// An ordered phrase record stream that preserves duplicate entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhraseDocument {
    entries: Vec<PhraseEntry>,
}

impl PhraseDocument {
    /// Creates a document without sorting or deduplicating its entries.
    #[must_use]
    pub const fn new(entries: Vec<PhraseEntry>) -> Self {
        Self { entries }
    }

    /// Returns entries in their original order.
    #[must_use]
    pub fn entries(&self) -> &[PhraseEntry] {
        &self.entries
    }

    /// Returns the number of records, including duplicates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the document has no records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Consumes the document and returns entries in their original order.
    #[must_use]
    pub fn into_entries(self) -> Vec<PhraseEntry> {
        self.entries
    }
}
