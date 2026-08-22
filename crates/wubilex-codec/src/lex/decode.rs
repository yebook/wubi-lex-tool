use crate::{
    CodecError, CodecErrorKind, DecodeLimits, FieldValue, LexCode, LexiconDocument, LexiconEntry,
    Weight,
};

use super::{
    ALPHA_COUNT, FILE_SIZE_FIELD_OFFSET, HEADER_SIZE, INDEX_OFFSET_FIELD_OFFSET, INDEX_SIZE, MAGIC,
    TABLE_OFFSET_FIELD_OFFSET, offset_u64,
};

const RECORD_FIXED_BYTES: usize = 16;
const RECORD_TEXT_OFFSET: usize = 14;

#[derive(Debug, Clone, Copy)]
struct RecordBoundary {
    relative_offset: usize,
    first_alpha: usize,
}

/// Decodes one complete Microsoft Wubi `.lex` byte stream.
///
/// The decoder preserves wire order and duplicate records. Every returned
/// entry has an explicit nonzero weight.
pub fn decode(input: &[u8], limits: DecodeLimits) -> Result<LexiconDocument, CodecError> {
    limits.check_input_bytes(input.len())?;

    let magic = read_bytes(input, 0, MAGIC.len(), "lex.header.magic")?;
    if magic != MAGIC {
        return Err(CodecError::new(CodecErrorKind::MagicMismatch {
            expected: MAGIC.to_vec(),
            actual: magic.to_vec(),
        })
        .at_byte_offset(0));
    }

    // These metadata fields are deliberately read but tolerated for legacy compatibility.
    let _major_version = read_u16(input, 8, "lex.header.major_version")?;
    let _minor_version = read_u16(input, 10, "lex.header.minor_version")?;
    let index_offset_wire = read_i32(input, INDEX_OFFSET_FIELD_OFFSET, "lex.header.index_offset")?;
    let table_offset_wire = read_i32(input, TABLE_OFFSET_FIELD_OFFSET, "lex.header.table_offset")?;
    let file_size_wire = read_i32(input, FILE_SIZE_FIELD_OFFSET, "lex.header.file_size")?;
    read_bytes(
        input,
        FILE_SIZE_FIELD_OFFSET + 4,
        HEADER_SIZE - (FILE_SIZE_FIELD_OFFSET + 4),
        "lex.header.metadata",
    )?;

    let file_size = validate_file_size(file_size_wire, input.len())?;
    let index_offset = validate_signed_offset(
        "lex.header.index_offset",
        index_offset_wire,
        HEADER_SIZE,
        file_size,
        INDEX_OFFSET_FIELD_OFFSET,
    )?;
    let index_end = index_offset.checked_add(INDEX_SIZE).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "lex index offset plus index size",
        })
        .at_byte_offset(offset_u64(INDEX_OFFSET_FIELD_OFFSET))
    })?;
    if index_end > file_size {
        return Err(invalid_offset(
            "lex.header.index_offset",
            index_offset_wire,
            HEADER_SIZE,
            file_size.saturating_sub(INDEX_SIZE).max(HEADER_SIZE),
            INDEX_OFFSET_FIELD_OFFSET,
        ));
    }
    let table_offset = validate_signed_offset(
        "lex.header.table_offset",
        table_offset_wire,
        index_end,
        file_size,
        TABLE_OFFSET_FIELD_OFFSET,
    )?;

    let alpha_indexes = read_alpha_indexes(input, index_offset)?;
    let (entries, boundaries) = read_records(input, table_offset, file_size, limits)?;
    validate_alpha_indexes(
        &alpha_indexes,
        index_offset,
        file_size - table_offset,
        &boundaries,
    )?;

    Ok(LexiconDocument::new(entries))
}

fn validate_file_size(wire: i32, actual: usize) -> Result<usize, CodecError> {
    if wire < 0 {
        return Err(invalid_offset(
            "lex.header.file_size",
            wire,
            HEADER_SIZE,
            actual.max(HEADER_SIZE),
            FILE_SIZE_FIELD_OFFSET,
        ));
    }

    let value = usize::try_from(wire).map_err(|_| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "lex file size conversion",
        })
        .at_byte_offset(offset_u64(FILE_SIZE_FIELD_OFFSET))
    })?;
    if value < HEADER_SIZE {
        return Err(invalid_offset(
            "lex.header.file_size",
            wire,
            HEADER_SIZE,
            actual.max(HEADER_SIZE),
            FILE_SIZE_FIELD_OFFSET,
        ));
    }
    if value != actual {
        return Err(CodecError::new(CodecErrorKind::MalformedField {
            field: "lex.header.file_size",
            expected: FieldValue::Unsigned(offset_u64(actual)),
            actual: FieldValue::Signed(i64::from(wire)),
        })
        .at_byte_offset(offset_u64(FILE_SIZE_FIELD_OFFSET)));
    }

    Ok(value)
}

fn validate_signed_offset(
    field: &'static str,
    wire: i32,
    minimum: usize,
    maximum: usize,
    field_offset: usize,
) -> Result<usize, CodecError> {
    let value = usize::try_from(wire)
        .map_err(|_| invalid_offset(field, wire, minimum, maximum.max(minimum), field_offset))?;
    if value < minimum || value > maximum {
        return Err(invalid_offset(
            field,
            wire,
            minimum,
            maximum.max(minimum),
            field_offset,
        ));
    }

    Ok(value)
}

fn invalid_offset(
    field: &'static str,
    wire: i32,
    minimum: usize,
    maximum: usize,
    field_offset: usize,
) -> CodecError {
    CodecError::new(CodecErrorKind::InvalidOffset {
        field,
        offset: i64::from(wire),
        minimum: offset_u64(minimum),
        maximum: offset_u64(maximum),
    })
    .at_byte_offset(offset_u64(field_offset))
}

fn read_alpha_indexes(input: &[u8], index_offset: usize) -> Result<[i32; ALPHA_COUNT], CodecError> {
    let mut indexes = [0; ALPHA_COUNT];
    for (alpha, value) in indexes.iter_mut().enumerate() {
        let field_offset = index_offset.checked_add(alpha * 4).ok_or_else(|| {
            CodecError::new(CodecErrorKind::IntegerOverflow {
                operation: "lex alpha index field offset",
            })
            .at_byte_offset(offset_u64(index_offset))
        })?;
        *value = read_i32(input, field_offset, "lex.alpha_index")?;
    }
    Ok(indexes)
}

fn read_records(
    input: &[u8],
    table_offset: usize,
    file_size: usize,
    limits: DecodeLimits,
) -> Result<(Vec<LexiconEntry>, Vec<RecordBoundary>), CodecError> {
    let mut entries = Vec::new();
    let mut boundaries = Vec::new();
    let mut position = table_offset;
    let mut previous_code: Option<String> = None;

    while position < file_size {
        let length = usize::from(read_u16(input, position, "lex.record.length")?);
        if length < RECORD_FIXED_BYTES || length % 2 != 0 {
            return Err(malformed_unsigned(
                "lex.record.length",
                "an even integer in 16..=65535",
                length as u64,
                position,
            ));
        }
        let remaining = file_size - position;
        if length > remaining {
            return Err(CodecError::new(CodecErrorKind::UnexpectedEof {
                field: "lex.record",
                needed: length,
                remaining,
            })
            .at_byte_offset(offset_u64(position)));
        }
        let record_end = position.checked_add(length).ok_or_else(|| {
            CodecError::new(CodecErrorKind::IntegerOverflow {
                operation: "lex record end",
            })
            .at_byte_offset(offset_u64(position))
        })?;

        let weight_offset = position + 2;
        let weight_value = read_u16(input, weight_offset, "lex.record.weight")?;
        if weight_value == 0 {
            return Err(malformed_unsigned(
                "lex.record.weight",
                "1..=65535",
                0,
                weight_offset,
            ));
        }

        let code_length_offset = position + 4;
        let code_length = usize::from(read_u16(
            input,
            code_length_offset,
            "lex.record.code_length",
        )?);
        if !(1..=4).contains(&code_length) {
            return Err(malformed_unsigned(
                "lex.record.code_length",
                "1..=4",
                code_length as u64,
                code_length_offset,
            ));
        }

        let code = read_code(input, position, code_length)?;
        if previous_code
            .as_deref()
            .is_some_and(|previous| previous > code.as_str())
        {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "lex.record.code_order",
                expected: FieldValue::Text("lexicographic nondecreasing order".to_owned()),
                actual: FieldValue::Text(code.clone()),
            })
            .at_byte_offset(offset_u64(position + 6)));
        }

        let text_offset = position + RECORD_TEXT_OFFSET;
        let text_bytes_len = length - RECORD_FIXED_BYTES;
        if text_bytes_len == 0 {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "lex.record.text",
                expected: FieldValue::Text("nonempty UTF-16LE text".to_owned()),
                actual: FieldValue::Unsigned(0),
            })
            .at_byte_offset(offset_u64(text_offset)));
        }
        let terminator_offset = record_end - 2;
        let text_bytes = read_bytes(input, text_offset, text_bytes_len, "lex.record.text")?;
        let terminator = read_u16(input, terminator_offset, "lex.record.terminator")?;
        if terminator != 0 {
            return Err(malformed_expected_unsigned(
                "lex.record.terminator",
                0,
                u64::from(terminator),
                terminator_offset,
            ));
        }
        let text = decode_utf16le(text_bytes, "lex.record.text", text_offset)?;
        if text.is_empty() {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "lex.record.text",
                expected: FieldValue::Text("nonempty UTF-16LE text".to_owned()),
                actual: FieldValue::Unsigned(0),
            })
            .at_byte_offset(offset_u64(text_offset)));
        }

        let next_count = entries.len().checked_add(1).ok_or_else(|| {
            CodecError::new(CodecErrorKind::IntegerOverflow {
                operation: "lex decoded entry count",
            })
            .at_byte_offset(offset_u64(position))
        })?;
        limits
            .check_expanded_entries(next_count)
            .map_err(|error| error.at_byte_offset(offset_u64(position)))?;

        let lex_code = LexCode::new(code.clone())
            .map_err(|error| error.at_byte_offset(offset_u64(position + 6)))?;
        let weight = Weight::new(weight_value)
            .map_err(|error| error.at_byte_offset(offset_u64(weight_offset)))?;
        let entry = LexiconEntry::new(lex_code, text, Some(weight))
            .map_err(|error| error.at_byte_offset(offset_u64(text_offset)))?;

        boundaries.push(RecordBoundary {
            relative_offset: position - table_offset,
            first_alpha: usize::from(code.as_bytes()[0] - b'a'),
        });
        entries.push(entry);
        previous_code = Some(code);
        position = record_end;
    }

    Ok((entries, boundaries))
}

fn read_code(input: &[u8], record_offset: usize, code_length: usize) -> Result<String, CodecError> {
    let mut code = String::with_capacity(code_length);
    for slot in 0..4 {
        let unit_offset = record_offset + 6 + slot * 2;
        let unit = read_u16(input, unit_offset, "lex.record.code")?;
        if slot < code_length {
            if !u8::try_from(unit).is_ok_and(|byte| byte.is_ascii_lowercase()) {
                return Err(malformed_unsigned(
                    "lex.record.code",
                    "a lowercase ASCII UTF-16 unit",
                    u64::from(unit),
                    unit_offset,
                ));
            }
            code.push(char::from(unit as u8));
        } else if unit != 0 {
            return Err(malformed_expected_unsigned(
                "lex.record.code_padding",
                0,
                u64::from(unit),
                unit_offset,
            ));
        }
    }
    Ok(code)
}

fn decode_utf16le(
    bytes: &[u8],
    field: &'static str,
    start_offset: usize,
) -> Result<String, CodecError> {
    let mut units = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([pair[0], pair[1]]));
    }

    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit) {
            let valid_pair = units
                .get(index + 1)
                .is_some_and(|next| (0xDC00..=0xDFFF).contains(next));
            if !valid_pair {
                return Err(invalid_utf16(field, unit, start_offset + index * 2));
            }
            index += 2;
        } else if (0xDC00..=0xDFFF).contains(&unit) {
            return Err(invalid_utf16(field, unit, start_offset + index * 2));
        } else {
            index += 1;
        }
    }

    String::from_utf16(&units)
        .map_err(|_| invalid_utf16(field, units.first().copied().unwrap_or(0), start_offset))
}

fn invalid_utf16(field: &'static str, unit: u16, offset: usize) -> CodecError {
    CodecError::new(CodecErrorKind::InvalidUtf16 {
        field,
        unpaired_surrogate: unit,
    })
    .at_byte_offset(offset_u64(offset))
}

fn validate_alpha_indexes(
    wire_indexes: &[i32; ALPHA_COUNT],
    index_offset: usize,
    record_bytes: usize,
    boundaries: &[RecordBoundary],
) -> Result<(), CodecError> {
    for (alpha, wire) in wire_indexes.iter().copied().enumerate() {
        let field_offset = index_offset + alpha * 4;
        let actual = usize::try_from(wire)
            .map_err(|_| invalid_offset("lex.alpha_index", wire, 0, record_bytes, field_offset))?;
        if actual > record_bytes {
            return Err(invalid_offset(
                "lex.alpha_index",
                wire,
                0,
                record_bytes,
                field_offset,
            ));
        }

        let expected = boundaries
            .iter()
            .find(|boundary| boundary.first_alpha >= alpha)
            .map_or(record_bytes, |boundary| boundary.relative_offset);
        if actual != expected {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "lex.alpha_index",
                expected: FieldValue::Unsigned(offset_u64(expected)),
                actual: FieldValue::Signed(i64::from(wire)),
            })
            .at_byte_offset(offset_u64(field_offset)));
        }
    }

    Ok(())
}

fn read_bytes<'a>(
    input: &'a [u8],
    offset: usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], CodecError> {
    let end = offset.checked_add(needed).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "lex field end",
        })
        .at_byte_offset(offset_u64(offset))
    })?;
    if end > input.len() {
        return Err(CodecError::new(CodecErrorKind::UnexpectedEof {
            field,
            needed,
            remaining: input.len().saturating_sub(offset),
        })
        .at_byte_offset(offset_u64(offset)));
    }
    Ok(&input[offset..end])
}

fn read_u16(input: &[u8], offset: usize, field: &'static str) -> Result<u16, CodecError> {
    let bytes = read_bytes(input, offset, 2, field)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_i32(input: &[u8], offset: usize, field: &'static str) -> Result<i32, CodecError> {
    let bytes = read_bytes(input, offset, 4, field)?;
    Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn malformed_unsigned(
    field: &'static str,
    expected: &'static str,
    actual: u64,
    offset: usize,
) -> CodecError {
    CodecError::new(CodecErrorKind::MalformedField {
        field,
        expected: FieldValue::Text(expected.to_owned()),
        actual: FieldValue::Unsigned(actual),
    })
    .at_byte_offset(offset_u64(offset))
}

fn malformed_expected_unsigned(
    field: &'static str,
    expected: u64,
    actual: u64,
    offset: usize,
) -> CodecError {
    CodecError::new(CodecErrorKind::MalformedField {
        field,
        expected: FieldValue::Unsigned(expected),
        actual: FieldValue::Unsigned(actual),
    })
    .at_byte_offset(offset_u64(offset))
}
