//! Structured failures shared by codec implementations.

use std::{fmt, num::NonZeroUsize};

use thiserror::Error;

use crate::model::TextEncoding;

/// The position in an external byte stream or text document where a failure occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLocation {
    /// A zero-based offset in the original byte stream.
    ByteOffset(u64),
    /// A one-based line and optional one-based column in the original text.
    Text {
        /// The one-based line number.
        line: NonZeroUsize,
        /// The one-based column number, when known.
        column: Option<NonZeroUsize>,
    },
}

/// The reason a validated public model value was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidInputReason {
    /// A required value was empty.
    #[error("value is empty")]
    Empty,
    /// A code contained a character outside `a` through `z`.
    #[error("character {character:?} at character index {index} is not a lowercase ASCII letter")]
    NotLowercaseAscii {
        /// Zero-based character index of the rejected value.
        index: usize,
        /// Rejected Unicode scalar value.
        character: char,
    },
    /// A value exceeded its format-specific maximum length.
    #[error("length {actual} exceeds maximum {max}")]
    TooLong {
        /// Maximum accepted length.
        max: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A value whose format starts at one was zero.
    #[error("value must be nonzero")]
    Zero,
}

/// The resource governed by a [`crate::DecodeLimits`] check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ResourceKind {
    /// Bytes in the complete encoded input.
    #[error("input bytes")]
    InputBytes,
    /// Records produced after expansion or decoding.
    #[error("expanded entries")]
    ExpandedEntries,
}

/// An expected or observed value from an encoded header or record field.
///
/// Numeric variants keep signed and unsigned wire values distinct. Bytes and
/// text remain owned so an error can outlive the caller's input buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    /// An unsigned integer field value.
    Unsigned(u64),
    /// A signed integer field value.
    Signed(i64),
    /// An uninterpreted byte sequence.
    Bytes(Vec<u8>),
    /// A textual value or a concise textual constraint.
    Text(String),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsigned(value) => value.fmt(formatter),
            Self::Signed(value) => value.fmt(formatter),
            Self::Bytes(value) => write!(formatter, "{value:02X?}"),
            Self::Text(value) => value.fmt(formatter),
        }
    }
}

/// Stable codec failure categories for parser and caller pattern matching.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CodecErrorKind {
    /// A public model value violates its invariant.
    #[error("invalid {field}: {reason}")]
    InvalidInput {
        /// Name of the invalid field.
        field: &'static str,
        /// Validation rule that failed.
        reason: InvalidInputReason,
    },
    /// The input magic bytes do not identify the expected format.
    #[error("magic value does not match: expected {expected:02X?}, actual {actual:02X?}")]
    MagicMismatch {
        /// Magic bytes required by the selected format.
        expected: Vec<u8>,
        /// Magic bytes observed in the input.
        actual: Vec<u8>,
    },
    /// The input ended before a complete field or record could be read.
    #[error(
        "unexpected end of input while reading {field}: needed {needed} bytes, remaining {remaining}"
    )]
    UnexpectedEof {
        /// Header or record field being read.
        field: &'static str,
        /// Complete number of bytes required for the field.
        needed: usize,
        /// Number of unread bytes still available.
        remaining: usize,
    },
    /// A decoded header or record field does not satisfy its fixed contract.
    #[error("malformed {field}: expected {expected}, actual {actual}")]
    MalformedField {
        /// Stable name of the malformed header or record field.
        field: &'static str,
        /// Required value or constraint.
        expected: FieldValue,
        /// Value observed in the input.
        actual: FieldValue,
    },
    /// An encoded offset falls outside the valid inclusive range.
    #[error("invalid {field} offset {offset}: expected an offset in {minimum}..={maximum}")]
    InvalidOffset {
        /// Stable name of the offset field.
        field: &'static str,
        /// Signed value observed on the wire, preserving negative offsets.
        offset: i64,
        /// Smallest valid byte offset.
        minimum: u64,
        /// Largest valid byte offset.
        maximum: u64,
    },
    /// A UTF-16 sequence is not well formed.
    #[error("invalid UTF-16 in {field}: unpaired surrogate {unpaired_surrogate:#06X}")]
    InvalidUtf16 {
        /// Stable name of the text field being decoded.
        field: &'static str,
        /// Unpaired UTF-16 surrogate reported by the decoder.
        unpaired_surrogate: u16,
    },
    /// Text bytes cannot be decoded using the selected or detected encoding.
    #[error("input is not valid {encoding:?} text")]
    InvalidTextEncoding {
        /// Encoding selected by BOM or detection before decoding failed.
        encoding: TextEncoding,
    },
    /// The input is recognized but uses an unsupported format variant.
    #[error("unsupported {format} format variant: {variant}")]
    UnsupportedFormat {
        /// Stable family name, such as `lex` or `eudp`.
        format: &'static str,
        /// Observed version or variant identifier.
        variant: String,
    },
    /// Checked arithmetic or an integer conversion overflowed.
    #[error("integer overflow while computing {operation}")]
    IntegerOverflow {
        /// Stable description of the failed calculation or conversion.
        operation: &'static str,
    },
    /// A configured resource limit was exceeded.
    #[error("{resource} limit exceeded: actual {actual}, limit {limit}")]
    ResourceLimitExceeded {
        /// Resource whose limit was exceeded.
        resource: ResourceKind,
        /// Configured maximum.
        limit: usize,
        /// Observed or requested amount.
        actual: usize,
    },
}

/// A codec failure with an optional structured source position.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{kind}")]
pub struct CodecError {
    kind: CodecErrorKind,
    location: Option<SourceLocation>,
}

impl CodecError {
    /// Creates an error without a source position.
    #[must_use]
    pub const fn new(kind: CodecErrorKind) -> Self {
        Self {
            kind,
            location: None,
        }
    }

    /// Returns the stable failure category and its structured fields.
    #[must_use]
    pub const fn kind(&self) -> &CodecErrorKind {
        &self.kind
    }

    /// Returns the external source position, when one is known.
    #[must_use]
    pub const fn location(&self) -> Option<SourceLocation> {
        self.location
    }

    /// Adds or replaces the external source position.
    #[must_use]
    pub const fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Adds a zero-based byte offset.
    #[must_use]
    pub const fn at_byte_offset(self, offset: u64) -> Self {
        self.with_location(SourceLocation::ByteOffset(offset))
    }

    /// Adds a one-based text line and optional one-based column.
    #[must_use]
    pub const fn at_text(self, line: NonZeroUsize, column: Option<NonZeroUsize>) -> Self {
        self.with_location(SourceLocation::Text { line, column })
    }

    pub(crate) const fn invalid_input(field: &'static str, reason: InvalidInputReason) -> Self {
        Self::new(CodecErrorKind::InvalidInput { field, reason })
    }
}
