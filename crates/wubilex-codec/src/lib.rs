//! Binary and text codec primitives for WubiLex.

pub mod error;
pub mod eudp;
pub mod lex;
pub mod limits;
pub mod model;

pub use error::{
    CodecError, CodecErrorKind, FieldValue, InvalidInputReason, ResourceKind, SourceLocation,
};
pub use limits::DecodeLimits;
pub use model::{
    Candidate, DetectedTextEncoding, LexCode, LexScheme, LexiconDocument, LexiconEntry, PhraseCode,
    PhraseDocument, PhraseEntry, TextEncoding, Weight,
};
