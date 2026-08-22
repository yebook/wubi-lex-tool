//! Format-neutral document models shared by binary and text codecs.

mod lexicon;
mod phrase;
mod scheme;
mod text_encoding;

pub use lexicon::{LexCode, LexiconDocument, LexiconEntry, Weight};
pub use phrase::{Candidate, PhraseCode, PhraseDocument, PhraseEntry};
pub use scheme::LexScheme;
pub use text_encoding::{DetectedTextEncoding, TextEncoding};
