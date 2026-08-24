//! Binary and text codec primitives for WubiLex.

pub mod detect;
pub mod error;
pub mod escape;
pub mod eudp;
pub mod lex;
pub mod limits;
pub mod model;
pub mod split_table;
pub mod text;
pub mod weight;

pub use error::{
    CodecError, CodecErrorKind, FieldValue, InvalidInputReason, ResourceKind, SourceLocation,
};
pub use limits::DecodeLimits;
pub use model::{
    Candidate, DetectedTextEncoding, LexCode, LexScheme, LexiconDocument, LexiconEntry, PhraseCode,
    PhraseDocument, PhraseEntry, SplitTableDocument, SplitTableEntry, TextEncoding, Weight,
    WordFrequencyDocument, WordFrequencyEntry,
};
pub use text::phrase as phrase_text;
pub use text::phrase::{DecodedPhraseText, PhraseTextWarning, PhraseTextWarningKind};
pub use text::{DecodedLexiconText, LexiconTextFormat, LexiconTextWarning, LexiconTextWarningKind};
