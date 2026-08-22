//! Ordered lexicon document values.

use std::num::NonZeroU16;

use crate::{CodecError, InvalidInputReason};

const MAX_LEX_CODE_LEN: usize = 4;

/// A `.lex` code containing one through four lowercase ASCII letters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LexCode(String);

impl LexCode {
    /// Validates and creates a `.lex` code.
    pub fn new(value: impl Into<String>) -> Result<Self, CodecError> {
        let value = value.into();
        validate_code("lexicon code", &value, Some(MAX_LEX_CODE_LEN))?;
        Ok(Self(value))
    }

    /// Returns the validated code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for LexCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A nonzero `.lex` candidate weight. Smaller values sort first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Weight(NonZeroU16);

impl Weight {
    /// Creates a weight in the inclusive range `1..=65535`.
    pub fn new(value: u16) -> Result<Self, CodecError> {
        NonZeroU16::new(value)
            .map(Self)
            .ok_or_else(|| CodecError::invalid_input("lexicon weight", InvalidInputReason::Zero))
    }

    /// Returns the encoded weight value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

/// One validated lexicon record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconEntry {
    code: LexCode,
    text: String,
    weight: Option<Weight>,
}

impl LexiconEntry {
    /// Creates an entry while preserving whether a source supplied an explicit weight.
    pub fn new(
        code: LexCode,
        text: impl Into<String>,
        weight: Option<Weight>,
    ) -> Result<Self, CodecError> {
        let text = text.into();
        validate_text("lexicon text", &text)?;
        Ok(Self { code, text, weight })
    }

    /// Returns the code.
    #[must_use]
    pub const fn code(&self) -> &LexCode {
        &self.code
    }

    /// Returns the nonempty text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the explicit weight, or `None` when the source omitted it.
    #[must_use]
    pub const fn weight(&self) -> Option<Weight> {
        self.weight
    }
}

/// An ordered lexicon record stream that preserves duplicate entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LexiconDocument {
    entries: Vec<LexiconEntry>,
}

impl LexiconDocument {
    /// Creates a document without sorting or deduplicating its entries.
    #[must_use]
    pub const fn new(entries: Vec<LexiconEntry>) -> Self {
        Self { entries }
    }

    /// Returns entries in their original order.
    #[must_use]
    pub fn entries(&self) -> &[LexiconEntry] {
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
    pub fn into_entries(self) -> Vec<LexiconEntry> {
        self.entries
    }
}

pub(super) fn validate_code(
    field: &'static str,
    value: &str,
    max_len: Option<usize>,
) -> Result<(), CodecError> {
    if value.is_empty() {
        return Err(CodecError::invalid_input(field, InvalidInputReason::Empty));
    }

    if let Some((index, character)) = value
        .chars()
        .enumerate()
        .find(|(_, character)| !character.is_ascii_lowercase())
    {
        return Err(CodecError::invalid_input(
            field,
            InvalidInputReason::NotLowercaseAscii { index, character },
        ));
    }

    if let Some(max) = max_len
        && value.len() > max
    {
        return Err(CodecError::invalid_input(
            field,
            InvalidInputReason::TooLong {
                max,
                actual: value.len(),
            },
        ));
    }

    Ok(())
}

pub(super) fn validate_text(field: &'static str, value: &str) -> Result<(), CodecError> {
    if value.is_empty() {
        return Err(CodecError::invalid_input(field, InvalidInputReason::Empty));
    }

    Ok(())
}
