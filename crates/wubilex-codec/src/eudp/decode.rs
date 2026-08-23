use crate::{
    Candidate, CodecError, CodecErrorKind, DecodeLimits, FieldValue, PhraseCode, PhraseDocument,
    PhraseEntry,
};

use super::{
    COUNT_FIELD_OFFSET, HEADER_SIZE, MAGIC, PHRASE_END_FIELD_OFFSET,
    PHRASE_OFFSET_START_FIELD_OFFSET, PHRASE_START_FIELD_OFFSET, RECORD_HEADER_SIZE, offset_u64,
};

const MINIMUM_RECORD_SIZE: usize = RECORD_HEADER_SIZE + 8;

/// Decodes one complete Microsoft Wubi EUDP byte stream.
///
/// Active records retain their wire order and duplicates. Structurally valid
/// deleted records are omitted from the returned document.
pub fn decode(input: &[u8], limits: DecodeLimits) -> Result<PhraseDocument, CodecError> {
    limits.check_input_bytes(input.len())?;

    let magic = read_bytes(input, 0, MAGIC.len(), "eudp.header.magic")?;
    if magic != MAGIC {
        return Err(CodecError::new(CodecErrorKind::MagicMismatch {
            expected: MAGIC.to_vec(),
            actual: magic.to_vec(),
        })
        .at_byte_offset(0));
    }

    // Metadata that does not determine boundaries is deliberately tolerated.
    let _magic2 = read_i32(input, 8, "eudp.header.magic2")?;
    let _version = read_i32(input, 12, "eudp.header.version")?;
    let phrase_offset_start_wire = read_i32(
        input,
        PHRASE_OFFSET_START_FIELD_OFFSET,
        "eudp.header.phrase_offset_start",
    )?;
    let phrase_start_wire = read_i32(input, PHRASE_START_FIELD_OFFSET, "eudp.header.phrase_start")?;
    let phrase_end_wire = read_i32(input, PHRASE_END_FIELD_OFFSET, "eudp.header.phrase_end")?;
    let count_wire = read_i32(input, COUNT_FIELD_OFFSET, "eudp.header.count")?;
    let _timestamp = read_i32(input, 32, "eudp.header.timestamp")?;
    read_bytes(input, 36, HEADER_SIZE - 36, "eudp.header.metadata")?;

    let phrase_end = validate_file_end(phrase_end_wire, input.len())?;
    let count = validate_count(count_wire)?;
    limits
        .check_expanded_entries(count)
        .map_err(|error| error.at_byte_offset(offset_u64(COUNT_FIELD_OFFSET)))?;
    let phrase_offset_start = validate_signed_offset(
        "eudp.header.phrase_offset_start",
        phrase_offset_start_wire,
        HEADER_SIZE,
        phrase_end,
        PHRASE_OFFSET_START_FIELD_OFFSET,
    )?;
    let phrase_start = validate_signed_offset(
        "eudp.header.phrase_start",
        phrase_start_wire,
        phrase_offset_start,
        phrase_end,
        PHRASE_START_FIELD_OFFSET,
    )?;

    let table_bytes = count.checked_mul(4).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "eudp offset table length",
        })
        .at_byte_offset(offset_u64(COUNT_FIELD_OFFSET))
    })?;
    let expected_phrase_start = phrase_offset_start
        .checked_add(table_bytes)
        .ok_or_else(|| {
            CodecError::new(CodecErrorKind::IntegerOverflow {
                operation: "eudp offset table end",
            })
            .at_byte_offset(offset_u64(PHRASE_START_FIELD_OFFSET))
        })?;
    if phrase_start != expected_phrase_start {
        return Err(CodecError::new(CodecErrorKind::MalformedField {
            field: "eudp.header.phrase_start",
            expected: FieldValue::Unsigned(offset_u64(expected_phrase_start)),
            actual: FieldValue::Signed(i64::from(phrase_start_wire)),
        })
        .at_byte_offset(offset_u64(PHRASE_START_FIELD_OFFSET)));
    }

    let record_bytes = phrase_end - phrase_start;
    if count == 0 {
        if record_bytes != 0 {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "eudp.header.phrase_end",
                expected: FieldValue::Unsigned(offset_u64(phrase_start)),
                actual: FieldValue::Signed(i64::from(phrase_end_wire)),
            })
            .at_byte_offset(offset_u64(PHRASE_END_FIELD_OFFSET)));
        }
        return Ok(PhraseDocument::default());
    }

    let offsets = read_offsets(input, phrase_offset_start, count, record_bytes)?;
    read_records(input, phrase_start, &offsets)
}

fn validate_file_end(wire: i32, actual: usize) -> Result<usize, CodecError> {
    let value = usize::try_from(wire).map_err(|_| {
        invalid_offset(
            "eudp.header.phrase_end",
            wire,
            HEADER_SIZE,
            actual.max(HEADER_SIZE),
            PHRASE_END_FIELD_OFFSET,
        )
    })?;
    if value < HEADER_SIZE {
        return Err(invalid_offset(
            "eudp.header.phrase_end",
            wire,
            HEADER_SIZE,
            actual.max(HEADER_SIZE),
            PHRASE_END_FIELD_OFFSET,
        ));
    }
    if value != actual {
        return Err(CodecError::new(CodecErrorKind::MalformedField {
            field: "eudp.header.phrase_end",
            expected: FieldValue::Unsigned(offset_u64(actual)),
            actual: FieldValue::Signed(i64::from(wire)),
        })
        .at_byte_offset(offset_u64(PHRASE_END_FIELD_OFFSET)));
    }

    Ok(value)
}

fn validate_count(wire: i32) -> Result<usize, CodecError> {
    usize::try_from(wire).map_err(|_| {
        CodecError::new(CodecErrorKind::MalformedField {
            field: "eudp.header.count",
            expected: FieldValue::Text("a nonnegative wire entry count".to_owned()),
            actual: FieldValue::Signed(i64::from(wire)),
        })
        .at_byte_offset(offset_u64(COUNT_FIELD_OFFSET))
    })
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

fn read_offsets(
    input: &[u8],
    table_start: usize,
    count: usize,
    record_bytes: usize,
) -> Result<Vec<usize>, CodecError> {
    let capacity = count.checked_add(1).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "eudp offset count plus sentinel",
        })
        .at_byte_offset(offset_u64(COUNT_FIELD_OFFSET))
    })?;
    let mut offsets = Vec::with_capacity(capacity);

    for index in 0..count {
        let field_offset = table_start
            .checked_add(index.checked_mul(4).ok_or_else(|| {
                CodecError::new(CodecErrorKind::IntegerOverflow {
                    operation: "eudp offset table field offset",
                })
                .at_byte_offset(offset_u64(table_start))
            })?)
            .ok_or_else(|| {
                CodecError::new(CodecErrorKind::IntegerOverflow {
                    operation: "eudp offset table field offset",
                })
                .at_byte_offset(offset_u64(table_start))
            })?;
        let wire = read_i32(input, field_offset, "eudp.offset_table")?;
        let value = usize::try_from(wire).map_err(|_| {
            invalid_offset(
                "eudp.offset_table",
                wire,
                0,
                record_bytes.saturating_sub(1),
                field_offset,
            )
        })?;
        if value >= record_bytes && record_bytes != 0 {
            return Err(invalid_offset(
                "eudp.offset_table",
                wire,
                0,
                record_bytes - 1,
                field_offset,
            ));
        }
        if record_bytes == 0 && value != 0 {
            return Err(invalid_offset(
                "eudp.offset_table",
                wire,
                0,
                0,
                field_offset,
            ));
        }

        if index == 0 && value != 0 {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "eudp.offset_table.first",
                expected: FieldValue::Unsigned(0),
                actual: FieldValue::Signed(i64::from(wire)),
            })
            .at_byte_offset(offset_u64(field_offset)));
        }
        if let Some(previous) = offsets.last().copied()
            && value <= previous
        {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "eudp.offset_table.order",
                expected: FieldValue::Text("strictly increasing relative offsets".to_owned()),
                actual: FieldValue::Signed(i64::from(wire)),
            })
            .at_byte_offset(offset_u64(field_offset)));
        }

        offsets.push(value);
    }
    offsets.push(record_bytes);
    Ok(offsets)
}

fn read_records(
    input: &[u8],
    phrase_start: usize,
    offsets: &[usize],
) -> Result<PhraseDocument, CodecError> {
    let record_count = offsets.len().checked_sub(1).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "eudp offset sentinel count",
        })
        .at_byte_offset(offset_u64(phrase_start))
    })?;
    let mut entries = Vec::with_capacity(record_count);
    let mut previous_code: Option<String> = None;

    for index in 0..record_count {
        let record_start = phrase_start.checked_add(offsets[index]).ok_or_else(|| {
            CodecError::new(CodecErrorKind::IntegerOverflow {
                operation: "eudp record start",
            })
            .at_byte_offset(offset_u64(phrase_start))
        })?;
        let record_end = phrase_start
            .checked_add(offsets[index + 1])
            .ok_or_else(|| {
                CodecError::new(CodecErrorKind::IntegerOverflow {
                    operation: "eudp record end",
                })
                .at_byte_offset(offset_u64(record_start))
            })?;
        let parsed = read_record(input, record_start, record_end)?;

        if previous_code
            .as_deref()
            .is_some_and(|previous| previous > parsed.code.as_str())
        {
            return Err(CodecError::new(CodecErrorKind::MalformedField {
                field: "eudp.entry.code_order",
                expected: FieldValue::Text("lexicographic nondecreasing order".to_owned()),
                actual: FieldValue::Text(parsed.code.clone()),
            })
            .at_byte_offset(offset_u64(record_start + RECORD_HEADER_SIZE)));
        }
        previous_code = Some(parsed.code.clone());

        if parsed.deleted == 0 {
            let code = PhraseCode::new(parsed.code).map_err(|error| {
                error.at_byte_offset(offset_u64(record_start + RECORD_HEADER_SIZE))
            })?;
            let candidate = Candidate::new(parsed.candidate)
                .map_err(|error| error.at_byte_offset(offset_u64(record_start + 6)))?;
            let entry = PhraseEntry::new(code, parsed.text, candidate).map_err(|error| {
                error.at_byte_offset(offset_u64(record_start + parsed.text_offset))
            })?;
            entries.push(entry);
        }
    }

    Ok(PhraseDocument::new(entries))
}

struct ParsedRecord {
    code: String,
    text: String,
    candidate: u8,
    deleted: u8,
    text_offset: usize,
}

fn read_record(
    input: &[u8],
    record_start: usize,
    record_end: usize,
) -> Result<ParsedRecord, CodecError> {
    let record_len = record_end.checked_sub(record_start).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "eudp record length",
        })
        .at_byte_offset(offset_u64(record_start))
    })?;
    if record_len < MINIMUM_RECORD_SIZE {
        return Err(CodecError::new(CodecErrorKind::UnexpectedEof {
            field: "eudp.entry",
            needed: MINIMUM_RECORD_SIZE,
            remaining: record_len,
        })
        .at_byte_offset(offset_u64(record_start)));
    }
    if record_len % 2 != 0 {
        return Err(malformed_unsigned(
            "eudp.entry.length",
            "an even byte length",
            offset_u64(record_len),
            record_start,
        ));
    }
    read_bytes(input, record_start, record_len, "eudp.entry")?;

    let cb_size = read_u16(input, record_start, "eudp.entry.cb_size")?;
    if cb_size != RECORD_HEADER_SIZE as u16 {
        return Err(CodecError::new(CodecErrorKind::UnsupportedFormat {
            format: "eudp",
            variant: format!("cbSize={cb_size}"),
        })
        .at_byte_offset(offset_u64(record_start)));
    }
    let _cb_size2 = read_u16(input, record_start + 2, "eudp.entry.cb_size2")?;
    let text_offset_wire = read_u16(input, record_start + 4, "eudp.entry.text_offset")?;
    let text_offset = usize::from(text_offset_wire);
    if text_offset % 2 != 0 {
        return Err(malformed_unsigned(
            "eudp.entry.text_offset",
            "an even relative byte offset",
            u64::from(text_offset_wire),
            record_start + 4,
        ));
    }
    let maximum_text_offset = record_len - 2;
    if text_offset < RECORD_HEADER_SIZE + 4 || text_offset > maximum_text_offset {
        return Err(CodecError::new(CodecErrorKind::InvalidOffset {
            field: "eudp.entry.text_offset",
            offset: i64::from(text_offset_wire),
            minimum: offset_u64(RECORD_HEADER_SIZE + 4),
            maximum: offset_u64(maximum_text_offset),
        })
        .at_byte_offset(offset_u64(record_start + 4)));
    }

    let candidate = input[record_start + 6];
    if candidate == 0 {
        return Err(malformed_unsigned(
            "eudp.entry.candidate",
            "1..=255",
            0,
            record_start + 6,
        ));
    }
    let deleted = input[record_start + 9];

    let code_start = record_start + RECORD_HEADER_SIZE;
    let code_terminator_offset = record_start + text_offset - 2;
    let code_terminator = read_u16(input, code_terminator_offset, "eudp.entry.code_terminator")?;
    if code_terminator != 0 {
        return Err(malformed_expected_unsigned(
            "eudp.entry.code_terminator",
            0,
            u64::from(code_terminator),
            code_terminator_offset,
        ));
    }
    let code_bytes = read_bytes(
        input,
        code_start,
        code_terminator_offset - code_start,
        "eudp.entry.code",
    )?;
    if code_bytes.is_empty() {
        return Err(malformed_unsigned(
            "eudp.entry.code",
            "a nonempty lowercase ASCII UTF-16 string without U+0000",
            0,
            code_start,
        ));
    }
    let code = decode_utf16le(code_bytes, "eudp.entry.code", code_start)?;
    for (index, pair) in code_bytes.chunks_exact(2).enumerate() {
        let unit = u16::from_le_bytes([pair[0], pair[1]]);
        if !u8::try_from(unit).is_ok_and(|byte| byte.is_ascii_lowercase()) {
            return Err(malformed_unsigned(
                "eudp.entry.code",
                "a nonempty lowercase ASCII UTF-16 string without U+0000",
                u64::from(unit),
                code_start + index * 2,
            ));
        }
    }

    let text_start = record_start + text_offset;
    let text_terminator_offset = record_end - 2;
    let text_terminator = read_u16(input, text_terminator_offset, "eudp.entry.text_terminator")?;
    if text_terminator != 0 {
        return Err(malformed_expected_unsigned(
            "eudp.entry.text_terminator",
            0,
            u64::from(text_terminator),
            text_terminator_offset,
        ));
    }
    let text_bytes = read_bytes(
        input,
        text_start,
        text_terminator_offset - text_start,
        "eudp.entry.text",
    )?;
    if text_bytes.is_empty() {
        return Err(malformed_unsigned(
            "eudp.entry.text",
            "nonempty UTF-16LE text without U+0000",
            0,
            text_start,
        ));
    }
    for (index, pair) in text_bytes.chunks_exact(2).enumerate() {
        if u16::from_le_bytes([pair[0], pair[1]]) == 0 {
            return Err(malformed_unsigned(
                "eudp.entry.text",
                "nonempty UTF-16LE text without U+0000",
                0,
                text_start + index * 2,
            ));
        }
    }
    let text = decode_utf16le(text_bytes, "eudp.entry.text", text_start)?;

    Ok(ParsedRecord {
        code,
        text,
        candidate,
        deleted,
        text_offset,
    })
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

fn read_bytes<'a>(
    input: &'a [u8],
    offset: usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], CodecError> {
    let end = offset.checked_add(needed).ok_or_else(|| {
        CodecError::new(CodecErrorKind::IntegerOverflow {
            operation: "eudp field end",
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
