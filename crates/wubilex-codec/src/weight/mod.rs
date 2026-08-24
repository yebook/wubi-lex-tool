//! Strict word-frequency text decoding and canonical formatting.

use crate::{
    CodecError, DecodeLimits, Weight, WordFrequencyDocument, WordFrequencyEntry,
    text::auxiliary::{append_tab_u16_line, at_token, decode_two_columns, malformed},
};

const FORMAT: &str = "word-frequency";

/// Decodes a BOM-less UTF-8 word-frequency file while preserving order and duplicates.
pub fn decode(input: &[u8], limits: DecodeLimits) -> Result<WordFrequencyDocument, CodecError> {
    let entries = decode_two_columns(
        input,
        limits,
        FORMAT,
        "word_frequency.line",
        "word frequency entry count",
        |line, word, weight| {
            if weight.value.is_empty() || !weight.value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(at_token(
                    malformed(
                        "word frequency weight",
                        "an integer in 1..=65535",
                        weight.value,
                    ),
                    line,
                    weight,
                ));
            }
            let source = weight.value.parse::<u64>().map_err(|_| {
                at_token(
                    malformed(
                        "word frequency weight",
                        "an integer in 1..=65535",
                        weight.value,
                    ),
                    line,
                    weight,
                )
            })?;
            let value = u16::try_from(source)
                .ok()
                .filter(|value| *value != 0)
                .ok_or_else(|| {
                    at_token(
                        malformed(
                            "word frequency weight",
                            "an integer in 1..=65535",
                            weight.value,
                        ),
                        line,
                        weight,
                    )
                })?;
            let weight_value = Weight::new(value).map_err(|error| at_token(error, line, weight))?;
            WordFrequencyEntry::new(word.value, weight_value)
                .map_err(|error| at_token(error, line, word))
        },
    )?;
    Ok(WordFrequencyDocument::new(entries))
}

/// Formats a word-frequency document as canonical BOM-less UTF-8 text with LF endings.
pub fn format(document: &WordFrequencyDocument) -> Result<String, CodecError> {
    let mut output = String::new();
    for entry in document.entries() {
        append_tab_u16_line(
            &mut output,
            entry.word(),
            entry.weight().get(),
            "word frequency output size",
        )?;
    }
    Ok(output)
}
