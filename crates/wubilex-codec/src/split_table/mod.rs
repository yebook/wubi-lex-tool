//! Strict spelling split-table decoding and canonical formatting.

use crate::{
    CodecError, DecodeLimits, SplitTableDocument, SplitTableEntry,
    text::auxiliary::{append_tab_line, at_token, decode_two_columns},
};

const FORMAT: &str = "split-table";

/// Decodes a BOM-less UTF-8 split table while preserving order and duplicates.
pub fn decode(input: &[u8], limits: DecodeLimits) -> Result<SplitTableDocument, CodecError> {
    let entries = decode_two_columns(
        input,
        limits,
        FORMAT,
        "split_table.line",
        "split table entry count",
        |line, term, roots| {
            SplitTableEntry::new(term.value, roots.value)
                .map_err(|error| at_token(error, line, term))
        },
    )?;
    Ok(SplitTableDocument::new(entries))
}

/// Formats a split table as canonical BOM-less UTF-8 text with LF endings.
pub fn format(document: &SplitTableDocument) -> Result<String, CodecError> {
    let mut output = String::new();
    for entry in document.entries() {
        append_tab_line(
            &mut output,
            entry.term(),
            entry.roots(),
            "split table output size",
        )?;
    }
    Ok(output)
}
