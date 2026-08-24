//! Community phrase text decoding and canonical formatting.

mod decode;
mod encode;

use crate::{DetectedTextEncoding, PhraseDocument, SourceLocation};

pub use decode::decode;
pub use encode::format;

/// A decoded phrase document with encoding metadata and ordered diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPhraseText {
    document: PhraseDocument,
    detected_encoding: DetectedTextEncoding,
    warnings: Vec<PhraseTextWarning>,
}

impl DecodedPhraseText {
    pub(crate) const fn new(
        document: PhraseDocument,
        detected_encoding: DetectedTextEncoding,
        warnings: Vec<PhraseTextWarning>,
    ) -> Self {
        Self {
            document,
            detected_encoding,
            warnings,
        }
    }

    /// Returns the ordered, duplicate-preserving phrase document.
    #[must_use]
    pub const fn document(&self) -> &PhraseDocument {
        &self.document
    }

    /// Returns the selected text encoding and BOM metadata.
    #[must_use]
    pub const fn detected_encoding(&self) -> DetectedTextEncoding {
        self.detected_encoding
    }

    /// Returns non-fatal diagnostics in original source order.
    #[must_use]
    pub fn warnings(&self) -> &[PhraseTextWarning] {
        &self.warnings
    }

    /// Consumes the result into its document, encoding metadata, and warnings.
    #[must_use]
    pub fn into_parts(self) -> (PhraseDocument, DetectedTextEncoding, Vec<PhraseTextWarning>) {
        (self.document, self.detected_encoding, self.warnings)
    }
}

/// A non-fatal phrase text diagnostic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseTextWarningKind {
    /// A nonempty line outside multiline state did not match a supported layout.
    UnrecognizedLine,
}

/// A bounded visible diagnostic for one source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseTextWarning {
    kind: PhraseTextWarningKind,
    location: SourceLocation,
    preview: String,
    truncated: bool,
}

impl PhraseTextWarning {
    pub(crate) const fn new(
        kind: PhraseTextWarningKind,
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
    pub const fn kind(&self) -> PhraseTextWarningKind {
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
