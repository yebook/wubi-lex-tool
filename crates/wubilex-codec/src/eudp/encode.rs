use crate::{CodecError, CodecErrorKind, FieldValue, PhraseDocument};

use super::{
    COUNT_FIELD_OFFSET, HEADER_SIZE, MAGIC, PHRASE_END_FIELD_OFFSET,
    PHRASE_OFFSET_START_FIELD_OFFSET, PHRASE_START_FIELD_OFFSET, RECORD_HEADER_SIZE,
};

const CANONICAL_MAGIC2: i32 = 0x0060_0002;
const CANONICAL_VERSION: i32 = 1;

/// Encodes a phrase document as canonical Microsoft Wubi EUDP bytes.
///
/// Entries are stably sorted by code. The order within an equal-code group,
/// candidate positions, and duplicate entries are preserved. `timestamp` is
/// written verbatim so the caller owns clock access and output determinism.
pub fn encode(document: &PhraseDocument, timestamp: i32) -> Result<Vec<u8>, CodecError> {
    let mut entries = document.entries().iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.code().as_str().cmp(right.code().as_str()));

    let count = entries.len();
    let count_i32 = i32::try_from(count).map_err(|_| overflow("eudp entry count"))?;
    let table_bytes = count
        .checked_mul(4)
        .ok_or_else(|| overflow("eudp offset table length"))?;
    let phrase_start = HEADER_SIZE
        .checked_add(table_bytes)
        .ok_or_else(|| overflow("eudp phrase start"))?;
    let phrase_start_i32 =
        i32::try_from(phrase_start).map_err(|_| overflow("eudp phrase start"))?;

    let mut output = Vec::new();
    output
        .try_reserve_exact(phrase_start)
        .map_err(|_| overflow("eudp header and offset table allocation"))?;
    output.resize(phrase_start, 0);

    for (index, entry) in entries.into_iter().enumerate() {
        let code_units = entry.code().as_str().encode_utf16().collect::<Vec<_>>();
        let code_with_nul_units = code_units
            .len()
            .checked_add(1)
            .ok_or_else(|| overflow("eudp code length with terminator"))?;
        let code_bytes = code_with_nul_units
            .checked_mul(2)
            .ok_or_else(|| overflow("eudp code byte length"))?;
        let text_offset = RECORD_HEADER_SIZE
            .checked_add(code_bytes)
            .ok_or_else(|| overflow("eudp text offset"))?;
        let text_offset_u16 =
            u16::try_from(text_offset).map_err(|_| overflow("eudp text offset"))?;

        let text_units = entry.text().encode_utf16().collect::<Vec<_>>();
        if let Some(index) = text_units.iter().position(|unit| *unit == 0) {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "eudp.entry.text",
                expected: FieldValue::Text("text without U+0000".to_owned()),
                actual: FieldValue::Text(format!(
                    "embedded U+0000 at UTF-16 code unit index {index}"
                )),
            }));
        }
        let text_with_nul_units = text_units
            .len()
            .checked_add(1)
            .ok_or_else(|| overflow("eudp text length with terminator"))?;
        let text_bytes = text_with_nul_units
            .checked_mul(2)
            .ok_or_else(|| overflow("eudp text byte length"))?;
        let record_length = text_offset
            .checked_add(text_bytes)
            .ok_or_else(|| overflow("eudp record length"))?;

        let relative_offset = output
            .len()
            .checked_sub(phrase_start)
            .ok_or_else(|| overflow("eudp record relative offset"))?;
        let relative_offset_i32 =
            i32::try_from(relative_offset).map_err(|_| overflow("eudp record offset"))?;
        let next_file_size = output
            .len()
            .checked_add(record_length)
            .ok_or_else(|| overflow("eudp file size"))?;
        i32::try_from(next_file_size).map_err(|_| overflow("eudp file size"))?;
        output
            .try_reserve_exact(record_length)
            .map_err(|_| overflow("eudp record allocation"))?;

        let table_offset = HEADER_SIZE
            .checked_add(
                index
                    .checked_mul(4)
                    .ok_or_else(|| overflow("eudp offset table field"))?,
            )
            .ok_or_else(|| overflow("eudp offset table field"))?;
        write_i32(&mut output, table_offset, relative_offset_i32);

        push_u16(&mut output, RECORD_HEADER_SIZE as u16);
        push_u16(&mut output, RECORD_HEADER_SIZE as u16);
        push_u16(&mut output, text_offset_u16);
        output.push(entry.candidate().get());
        output.push(6);
        output.push(0);
        output.push(0);
        push_u16(&mut output, 0);
        push_i32(&mut output, 0);
        for unit in code_units {
            push_u16(&mut output, unit);
        }
        push_u16(&mut output, 0);
        for unit in text_units {
            push_u16(&mut output, unit);
        }
        push_u16(&mut output, 0);
    }

    output[..MAGIC.len()].copy_from_slice(MAGIC);
    write_i32(&mut output, 8, CANONICAL_MAGIC2);
    write_i32(&mut output, 12, CANONICAL_VERSION);
    write_i32(
        &mut output,
        PHRASE_OFFSET_START_FIELD_OFFSET,
        i32::try_from(HEADER_SIZE).map_err(|_| overflow("eudp canonical offset table start"))?,
    );
    write_i32(&mut output, PHRASE_START_FIELD_OFFSET, phrase_start_i32);
    let phrase_end = i32::try_from(output.len()).map_err(|_| overflow("eudp file size"))?;
    write_i32(&mut output, PHRASE_END_FIELD_OFFSET, phrase_end);
    write_i32(&mut output, COUNT_FIELD_OFFSET, count_i32);
    write_i32(&mut output, 32, timestamp);

    Ok(output)
}

fn overflow(operation: &'static str) -> CodecError {
    CodecError::new(CodecErrorKind::IntegerOverflow { operation })
}

fn push_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn push_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_i32(output: &mut [u8], offset: usize, value: i32) {
    output[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
