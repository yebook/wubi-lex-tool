//! Format-neutral document models shared by binary and text codecs.

mod lexicon;
mod phrase;
mod scheme;
mod split_table;
mod text_encoding;
mod word_frequency;

pub use lexicon::{LexCode, LexiconDocument, LexiconEntry, Weight};
pub use phrase::{Candidate, PhraseCode, PhraseDocument, PhraseEntry};
pub use scheme::LexScheme;
pub use split_table::{SplitTableDocument, SplitTableEntry};
pub use text_encoding::{DetectedTextEncoding, TextEncoding};
pub use word_frequency::{WordFrequencyDocument, WordFrequencyEntry};

use crate::{CodecError, InvalidInputReason};

fn validate_token(field: &'static str, value: &str) -> Result<(), CodecError> {
    if value.is_empty() {
        return Err(CodecError::invalid_input(field, InvalidInputReason::Empty));
    }

    if let Some((index, character)) = value
        .chars()
        .enumerate()
        .find(|(_, character)| character.is_whitespace())
    {
        return Err(CodecError::invalid_input(
            field,
            InvalidInputReason::ContainsWhitespace { index, character },
        ));
    }

    Ok(())
}
