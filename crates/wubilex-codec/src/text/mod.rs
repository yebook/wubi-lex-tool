//! Community lexicon text decoding and canonical formatting.

pub(crate) mod auxiliary;
mod decode;
mod encode;
pub(crate) mod encoding;
pub mod phrase;

use crate::{DetectedTextEncoding, LexiconDocument, SourceLocation};

pub use decode::decode;
pub use encode::{encode_utf16le, format};

/// A successfully decoded lexicon together with encoding metadata and ordered diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedLexiconText {
    document: LexiconDocument,
    detected_encoding: DetectedTextEncoding,
    warnings: Vec<LexiconTextWarning>,
}

impl DecodedLexiconText {
    pub(crate) const fn new(
        document: LexiconDocument,
        detected_encoding: DetectedTextEncoding,
        warnings: Vec<LexiconTextWarning>,
    ) -> Self {
        Self {
            document,
            detected_encoding,
            warnings,
        }
    }

    /// Returns the ordered, duplicate-preserving lexicon document.
    #[must_use]
    pub const fn document(&self) -> &LexiconDocument {
        &self.document
    }

    /// Returns the selected text encoding and BOM metadata.
    #[must_use]
    pub const fn detected_encoding(&self) -> DetectedTextEncoding {
        self.detected_encoding
    }

    /// Returns non-fatal diagnostics in original source order.
    #[must_use]
    pub fn warnings(&self) -> &[LexiconTextWarning] {
        &self.warnings
    }

    /// Consumes the result into its document, encoding metadata, and warnings.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        LexiconDocument,
        DetectedTextEncoding,
        Vec<LexiconTextWarning>,
    ) {
        (self.document, self.detected_encoding, self.warnings)
    }
}

/// A non-fatal lexicon text diagnostic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexiconTextWarningKind {
    /// A nonempty body line did not match a supported layout.
    UnrecognizedLine,
}

/// A bounded visible diagnostic for one source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconTextWarning {
    kind: LexiconTextWarningKind,
    location: SourceLocation,
    preview: String,
    truncated: bool,
}

impl LexiconTextWarning {
    pub(crate) const fn new(
        kind: LexiconTextWarningKind,
        location: SourceLocation,
        preview: String,
        truncated: bool,
    ) -> Self {
        Self {
            kind,
            location,
            preview,
            truncated,
        }
    }

    /// Returns the stable warning category.
    #[must_use]
    pub const fn kind(&self) -> LexiconTextWarningKind {
        self.kind
    }

    /// Returns the original one-based text position.
    #[must_use]
    pub const fn location(&self) -> SourceLocation {
        self.location
    }

    /// Returns at most 160 Unicode scalar values from the original line.
    #[must_use]
    pub fn preview(&self) -> &str {
        &self.preview
    }

    /// Returns whether the preview omitted trailing source characters.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }
}

/// One of the seven deterministic community text layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexiconTextFormat {
    /// One `code<TAB>text` line per entry.
    CodeThenText,
    /// One code line containing adjacent-deduplicated texts.
    CodeThenTexts,
    /// One `code<TAB>text<TAB>ascending-weight` line per entry.
    CodeThenTextWeight,
    /// One `text<TAB>code` line per entry.
    TextThenCode,
    /// One text line containing adjacent-deduplicated codes.
    TextThenCodes,
    /// One `text<TAB>code<TAB>descending-weight` line per entry.
    TextThenCodeDescendingWeight,
    /// One `code=candidate,text` line per entry, renumbered per code.
    PhraseAscendingCandidate,
}
