use crate::{CodecError, CodecErrorKind, LexiconDocument};

use super::{
    ALPHA_COUNT, CANONICAL_INDEX_OFFSET, CANONICAL_TABLE_OFFSET, FILE_SIZE_FIELD_OFFSET,
    INDEX_OFFSET_FIELD_OFFSET, MAGIC, TABLE_OFFSET_FIELD_OFFSET,
};

const CANONICAL_MAJOR_VERSION: u16 = 1;
const CANONICAL_MINOR_VERSION: u16 = 1;
const CANONICAL_MARKER: i32 = 0x7856_3412;
const RECORD_FIXED_BYTES: usize = 16;

/// Encodes a lexicon document as canonical Microsoft Wubi `.lex` 1.1 bytes.
///
/// Entries are stably sorted by code. Duplicate entries and the original
/// order within each equal-code group are preserved.
pub fn encode(document: &LexiconDocument) -> Result<Vec<u8>, CodecError> {
    let mut entries = document.entries().iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.code().as_str().cmp(right.code().as_str()));

    let mut output = vec![0; CANONICAL_TABLE_OFFSET];
    let mut alpha_indexes = [0_i32; ALPHA_COUNT];
    let mut next_alpha = 0;
    let mut previous_code: Option<&str> = None;
    let mut current_weight = 0_u16;

    for entry in entries {
        let code = entry.code().as_str();
        if previous_code != Some(code) {
            current_weight = 0;
        }

        let relative_offset = output
            .len()
            .checked_sub(CANONICAL_TABLE_OFFSET)
            .ok_or_else(|| overflow("lex record relative offset"))?;
        let relative_offset_i32 =
            i32::try_from(relative_offset).map_err(|_| overflow("lex alpha index offset"))?;
        let first_alpha = usize::from(code.as_bytes()[0] - b'a');
        while next_alpha <= first_alpha {
            alpha_indexes[next_alpha] = relative_offset_i32;
            next_alpha += 1;
        }

        current_weight = match entry.weight() {
            Some(weight) => weight.get(),
            None => current_weight
                .checked_add(1)
                .ok_or_else(|| overflow("lex implicit weight"))?,
        };

        let text_units = entry.text().encode_utf16().collect::<Vec<_>>();
        let text_bytes = text_units
            .len()
            .checked_mul(2)
            .ok_or_else(|| overflow("lex record length"))?;
        let record_length = RECORD_FIXED_BYTES
            .checked_add(text_bytes)
            .ok_or_else(|| overflow("lex record length"))?;
        let record_length_u16 =
            u16::try_from(record_length).map_err(|_| overflow("lex record length"))?;
        let next_file_size = output
            .len()
            .checked_add(record_length)
            .ok_or_else(|| overflow("lex file size"))?;
        i32::try_from(next_file_size).map_err(|_| overflow("lex file size"))?;

        push_u16(&mut output, record_length_u16);
        push_u16(&mut output, current_weight);
        push_u16(
            &mut output,
            u16::try_from(code.len()).map_err(|_| overflow("lex code length"))?,
        );
        for byte in code.bytes() {
            push_u16(&mut output, u16::from(byte));
        }
        for _ in code.len()..4 {
            push_u16(&mut output, 0);
        }
        for unit in text_units {
            push_u16(&mut output, unit);
        }
        push_u16(&mut output, 0);

        previous_code = Some(code);
    }

    let record_bytes = output
        .len()
        .checked_sub(CANONICAL_TABLE_OFFSET)
        .ok_or_else(|| overflow("lex record section length"))?;
    let record_bytes_i32 =
        i32::try_from(record_bytes).map_err(|_| overflow("lex alpha index offset"))?;
    while next_alpha < ALPHA_COUNT {
        alpha_indexes[next_alpha] = record_bytes_i32;
        next_alpha += 1;
    }

    output[..MAGIC.len()].copy_from_slice(MAGIC);
    write_u16(&mut output, 8, CANONICAL_MAJOR_VERSION);
    write_u16(&mut output, 10, CANONICAL_MINOR_VERSION);
    write_i32(
        &mut output,
        INDEX_OFFSET_FIELD_OFFSET,
        i32::try_from(CANONICAL_INDEX_OFFSET)
            .map_err(|_| overflow("lex canonical index offset"))?,
    );
    write_i32(
        &mut output,
        TABLE_OFFSET_FIELD_OFFSET,
        i32::try_from(CANONICAL_TABLE_OFFSET)
            .map_err(|_| overflow("lex canonical table offset"))?,
    );
    let file_size = i32::try_from(output.len()).map_err(|_| overflow("lex file size"))?;
    write_i32(&mut output, FILE_SIZE_FIELD_OFFSET, file_size);
    write_i32(&mut output, 24, CANONICAL_MARKER);
    for (alpha, relative_offset) in alpha_indexes.into_iter().enumerate() {
        write_i32(
            &mut output,
            CANONICAL_INDEX_OFFSET + alpha * 4,
            relative_offset,
        );
    }

    Ok(output)
}

fn overflow(operation: &'static str) -> CodecError {
    CodecError::new(CodecErrorKind::IntegerOverflow { operation })
}

fn push_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u16(output: &mut [u8], offset: usize, value: u16) {
    output[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(output: &mut [u8], offset: usize, value: i32) {
    output[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
