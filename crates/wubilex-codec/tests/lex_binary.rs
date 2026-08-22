use wubilex_codec::{
    CodecErrorKind, DecodeLimits, FieldValue, LexCode, LexiconDocument, LexiconEntry, ResourceKind,
    SourceLocation, Weight,
    lex::{decode, encode},
};

const HEADER_SIZE: usize = 64;
const TABLE_OFFSET: usize = 168;

#[test]
fn known_wire_bytes_decode_and_encode_exactly() {
    let bytes = known_lex_bytes();

    let document = decode(&bytes, DecodeLimits::default()).expect("known bytes must decode");

    assert_eq!(document.len(), 3);
    assert_entry(&document.entries()[0], "a", "甲", 1);
    assert_entry(&document.entries()[1], "a", "乙", 2);
    assert_entry(&document.entries()[2], "cdef", "🤝", 7);
    assert_eq!(
        encode(&document).expect("known document must encode"),
        bytes
    );
}

#[test]
fn empty_document_has_an_exact_canonical_header_and_index() {
    let expected = empty_lex_bytes();

    let encoded = encode(&LexiconDocument::default()).expect("empty document must encode");

    assert_eq!(encoded, expected);
    assert!(
        decode(&encoded, DecodeLimits::default())
            .expect("empty bytes must decode")
            .is_empty()
    );
}

#[test]
fn encode_stably_sorts_codes_preserves_duplicates_and_builds_index_holes() {
    let first_c = entry("c", "先", Some(20));
    let first_a = entry("a", "甲", None);
    let second_c = entry("c", "后", None);
    let duplicate_a = entry("a", "甲", Some(9));
    let document = LexiconDocument::new(vec![
        first_c.clone(),
        first_a.clone(),
        second_c.clone(),
        duplicate_a.clone(),
    ]);

    let encoded = encode(&document).expect("document must encode");
    let decoded = decode(&encoded, DecodeLimits::default()).expect("encoded bytes must decode");

    assert_eq!(
        decoded
            .entries()
            .iter()
            .map(|entry| (
                entry.code().as_str(),
                entry.text(),
                entry.weight().map(Weight::get)
            ))
            .collect::<Vec<_>>(),
        vec![
            ("a", "甲", Some(1)),
            ("a", "甲", Some(9)),
            ("c", "先", Some(20)),
            ("c", "后", Some(21)),
        ]
    );
    assert_eq!(read_i32(&encoded, HEADER_SIZE), 0);
    assert_eq!(read_i32(&encoded, HEADER_SIZE + 4), 36);
    assert_eq!(read_i32(&encoded, HEADER_SIZE + 8), 36);
    assert_eq!(read_i32(&encoded, HEADER_SIZE + 12), 72);
}

#[test]
fn maximum_code_and_non_bmp_text_round_trip() {
    let document = LexiconDocument::new(vec![entry("zzzz", "A🤝中", Some(u16::MAX))]);

    let decoded = decode(
        &encode(&document).expect("document must encode"),
        DecodeLimits::default(),
    )
    .expect("encoded bytes must decode");

    assert_eq!(decoded, document);
}

#[test]
fn tolerant_header_metadata_is_normalized_by_encode() {
    let canonical = known_lex_bytes();
    let mut noncanonical = canonical.clone();
    write_u16(&mut noncanonical, 8, 99);
    write_u16(&mut noncanonical, 10, 42);
    write_i32(&mut noncanonical, 24, -123);
    noncanonical[28..64].fill(0xA5);

    let document = decode(&noncanonical, DecodeLimits::default())
        .expect("non-layout metadata must be tolerated");

    assert_eq!(encode(&document).expect("document must encode"), canonical);
}

#[test]
fn implicit_weight_after_maximum_is_a_structured_overflow() {
    let document = LexiconDocument::new(vec![
        entry("a", "甲", Some(u16::MAX)),
        entry("a", "乙", None),
    ]);

    let error = encode(&document).expect_err("implicit weight must not wrap");

    assert_eq!(
        error.kind(),
        &CodecErrorKind::IntegerOverflow {
            operation: "lex implicit weight",
        }
    );
    assert_eq!(error.location(), None);
}

#[test]
fn implicit_weight_restarts_at_one_for_each_code() {
    let document = LexiconDocument::new(vec![entry("a", "甲", None), entry("b", "乙", None)]);

    let decoded = decode(
        &encode(&document).expect("document must encode"),
        DecodeLimits::default(),
    )
    .expect("encoded bytes must decode");

    assert_eq!(decoded.entries()[0].weight().map(Weight::get), Some(1));
    assert_eq!(decoded.entries()[1].weight().map(Weight::get), Some(1));
}

#[test]
fn text_longer_than_the_record_length_field_is_rejected() {
    let text = "字".repeat(32_760);
    let document = LexiconDocument::new(vec![entry("a", &text, Some(1))]);

    let error = encode(&document).expect_err("record length must fit u16");

    assert_eq!(
        error.kind(),
        &CodecErrorKind::IntegerOverflow {
            operation: "lex record length",
        }
    );
}

#[test]
fn maximum_even_record_length_round_trips() {
    let text = "字".repeat(32_759);
    let document = LexiconDocument::new(vec![entry("a", &text, Some(1))]);

    let encoded = encode(&document).expect("maximum record length must encode");

    assert_eq!(read_u16(&encoded, TABLE_OFFSET), 65_534);
    assert_eq!(
        decode(&encoded, DecodeLimits::default()).expect("maximum record length must decode"),
        document
    );
}

#[test]
fn input_and_expanded_entry_limits_are_checked_at_exact_boundaries() {
    let bytes = known_lex_bytes();
    let exact = DecodeLimits::new(bytes.len(), 3);
    assert_eq!(
        decode(&bytes, exact).expect("exact limits must pass").len(),
        3
    );

    let input_error = decode(&bytes, DecodeLimits::new(bytes.len() - 1, 3))
        .expect_err("input limit must reject one byte over");
    assert_eq!(
        input_error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::InputBytes,
            limit: bytes.len() - 1,
            actual: bytes.len(),
        }
    );
    assert_eq!(input_error.location(), None);

    let entry_error = decode(&bytes, DecodeLimits::new(bytes.len(), 2))
        .expect_err("entry limit must reject the third record");
    assert_eq!(
        entry_error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 2,
            actual: 3,
        }
    );
    assert_eq!(
        entry_error.location(),
        Some(SourceLocation::ByteOffset(204))
    );
}

#[test]
fn magic_and_declared_size_errors_preserve_header_offsets() {
    let mut wrong_magic = known_lex_bytes();
    wrong_magic[..8].copy_from_slice(b"notalex!");
    assert_error(
        &wrong_magic,
        CodecErrorKind::MagicMismatch {
            expected: b"imscwubi".to_vec(),
            actual: b"notalex!".to_vec(),
        },
        0,
    );

    let mut wrong_size = known_lex_bytes();
    write_i32(&mut wrong_size, 20, 223);
    assert_error(
        &wrong_size,
        CodecErrorKind::MalformedField {
            field: "lex.header.file_size",
            expected: FieldValue::Unsigned(224),
            actual: FieldValue::Signed(223),
        },
        20,
    );
}

#[test]
fn structural_offsets_reject_negative_and_out_of_range_values() {
    let mut negative_size = known_lex_bytes();
    write_i32(&mut negative_size, 20, -1);
    assert_invalid_offset(&negative_size, "lex.header.file_size", -1, 20);

    let mut negative_index = known_lex_bytes();
    write_i32(&mut negative_index, 12, -1);
    assert_invalid_offset(&negative_index, "lex.header.index_offset", -1, 12);

    let mut index_beyond_file = known_lex_bytes();
    write_i32(&mut index_beyond_file, 12, 224);
    assert_invalid_offset(&index_beyond_file, "lex.header.index_offset", 224, 12);

    let mut negative_table = known_lex_bytes();
    write_i32(&mut negative_table, 16, -1);
    assert_invalid_offset(&negative_table, "lex.header.table_offset", -1, 16);

    let mut overlapping_table = known_lex_bytes();
    write_i32(&mut overlapping_table, 16, 167);
    assert_invalid_offset(&overlapping_table, "lex.header.table_offset", 167, 16);

    let mut table_beyond_file = known_lex_bytes();
    write_i32(&mut table_beyond_file, 16, 225);
    assert_invalid_offset(&table_beyond_file, "lex.header.table_offset", 225, 16);
}

#[test]
fn alpha_index_must_match_record_boundaries_and_partitions() {
    let mut negative = known_lex_bytes();
    write_i32(&mut negative, HEADER_SIZE + 4, -1);
    assert_invalid_offset(&negative, "lex.alpha_index", -1, HEADER_SIZE + 4);

    let mut beyond_records = known_lex_bytes();
    write_i32(&mut beyond_records, HEADER_SIZE + 4, 57);
    assert_invalid_offset(&beyond_records, "lex.alpha_index", 57, HEADER_SIZE + 4);

    let mut inside_record = known_lex_bytes();
    write_i32(&mut inside_record, HEADER_SIZE + 4, 1);
    assert_error(
        &inside_record,
        CodecErrorKind::MalformedField {
            field: "lex.alpha_index",
            expected: FieldValue::Unsigned(36),
            actual: FieldValue::Signed(1),
        },
        HEADER_SIZE + 4,
    );

    let mut wrong_partition_boundary = known_lex_bytes();
    write_i32(&mut wrong_partition_boundary, HEADER_SIZE + 4, 0);
    assert_error(
        &wrong_partition_boundary,
        CodecErrorKind::MalformedField {
            field: "lex.alpha_index",
            expected: FieldValue::Unsigned(36),
            actual: FieldValue::Signed(0),
        },
        HEADER_SIZE + 4,
    );
}

#[test]
fn record_length_rejects_small_odd_and_truncated_records() {
    let mut too_small = known_lex_bytes();
    write_u16(&mut too_small, TABLE_OFFSET, 14);
    assert_malformed_unsigned(
        &too_small,
        "lex.record.length",
        "an even integer in 16..=65535",
        14,
        TABLE_OFFSET,
    );

    let mut odd = known_lex_bytes();
    write_u16(&mut odd, TABLE_OFFSET, 17);
    assert_malformed_unsigned(
        &odd,
        "lex.record.length",
        "an even integer in 16..=65535",
        17,
        TABLE_OFFSET,
    );

    let mut beyond_end = known_lex_bytes();
    write_u16(&mut beyond_end, 204, 22);
    assert_error(
        &beyond_end,
        CodecErrorKind::UnexpectedEof {
            field: "lex.record",
            needed: 22,
            remaining: 20,
        },
        204,
    );
}

#[test]
fn record_weight_and_code_length_are_strict() {
    let mut zero_weight = known_lex_bytes();
    write_u16(&mut zero_weight, TABLE_OFFSET + 2, 0);
    assert_malformed_unsigned(
        &zero_weight,
        "lex.record.weight",
        "1..=65535",
        0,
        TABLE_OFFSET + 2,
    );

    for invalid in [0, 5] {
        let mut bytes = known_lex_bytes();
        write_u16(&mut bytes, TABLE_OFFSET + 4, invalid);
        assert_malformed_unsigned(
            &bytes,
            "lex.record.code_length",
            "1..=4",
            u64::from(invalid),
            TABLE_OFFSET + 4,
        );
    }
}

#[test]
fn record_code_rejects_non_lowercase_units_and_nonzero_padding() {
    let mut uppercase = known_lex_bytes();
    write_u16(&mut uppercase, TABLE_OFFSET + 6, u16::from(b'A'));
    assert_malformed_unsigned(
        &uppercase,
        "lex.record.code",
        "a lowercase ASCII UTF-16 unit",
        u64::from(b'A'),
        TABLE_OFFSET + 6,
    );

    let mut padding = known_lex_bytes();
    write_u16(&mut padding, TABLE_OFFSET + 8, u16::from(b'b'));
    assert_error(
        &padding,
        CodecErrorKind::MalformedField {
            field: "lex.record.code_padding",
            expected: FieldValue::Unsigned(0),
            actual: FieldValue::Unsigned(u64::from(b'b')),
        },
        TABLE_OFFSET + 8,
    );
}

#[test]
fn record_text_rejects_empty_invalid_utf16_and_nonzero_terminator() {
    let mut empty = empty_lex_bytes();
    set_file_size(&mut empty, TABLE_OFFSET + 16);
    empty.extend_from_slice(&[16, 0, 1, 0, 1, 0, b'a', 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    set_all_alpha_indexes(&mut empty, 0, 16);
    assert_error(
        &empty,
        CodecErrorKind::MalformedField {
            field: "lex.record.text",
            expected: FieldValue::Text("nonempty UTF-16LE text".to_owned()),
            actual: FieldValue::Unsigned(0),
        },
        TABLE_OFFSET + 14,
    );

    let mut invalid_utf16 = known_lex_bytes();
    write_u16(&mut invalid_utf16, TABLE_OFFSET + 14, 0xD800);
    assert_error(
        &invalid_utf16,
        CodecErrorKind::InvalidUtf16 {
            field: "lex.record.text",
            unpaired_surrogate: 0xD800,
        },
        TABLE_OFFSET + 14,
    );

    let mut terminator = known_lex_bytes();
    write_u16(&mut terminator, TABLE_OFFSET + 16, 1);
    assert_malformed_unsigned(
        &terminator,
        "lex.record.terminator",
        "0",
        1,
        TABLE_OFFSET + 16,
    );
}

#[test]
fn records_must_be_sorted_by_code() {
    let mut bytes = known_lex_bytes();
    write_u16(&mut bytes, 186 + 6, u16::from(b'z'));

    assert_error(
        &bytes,
        CodecErrorKind::MalformedField {
            field: "lex.record.code_order",
            expected: FieldValue::Text("lexicographic nondecreasing order".to_owned()),
            actual: FieldValue::Text("cdef".to_owned()),
        },
        210,
    );
}

#[test]
fn every_truncated_prefix_returns_a_structured_error_without_panicking() {
    let bytes = known_lex_bytes();

    for length in 0..bytes.len() {
        let result = std::panic::catch_unwind(|| decode(&bytes[..length], DecodeLimits::default()));
        let decode_result = result.expect("decoder must not panic for any truncated prefix");
        assert!(decode_result.is_err(), "prefix of {length} bytes must fail");
    }
}

#[test]
fn every_record_section_prefix_is_safe_with_a_matching_declared_size() {
    let bytes = known_lex_bytes();

    for length in TABLE_OFFSET..bytes.len() {
        let mut prefix = bytes[..length].to_vec();
        set_file_size(&mut prefix, length);

        let result = std::panic::catch_unwind(|| decode(&prefix, DecodeLimits::default()));
        let decode_result = result.expect("decoder must not panic inside any truncated record");
        assert!(
            decode_result.is_err(),
            "record-section prefix of {length} bytes must fail"
        );
    }
}

fn known_lex_bytes() -> Vec<u8> {
    let mut bytes = canonical_prefix(224);
    for index in 0..26 {
        let value = match index {
            0 => 0,
            1 | 2 => 36,
            _ => 56,
        };
        write_i32(&mut bytes, HEADER_SIZE + index * 4, value);
    }

    bytes.extend_from_slice(&[
        18, 0, 1, 0, 1, 0, b'a', 0, 0, 0, 0, 0, 0, 0, 0x32, 0x75, 0, 0,
    ]);
    bytes.extend_from_slice(&[
        18, 0, 2, 0, 1, 0, b'a', 0, 0, 0, 0, 0, 0, 0, 0x59, 0x4E, 0, 0,
    ]);
    bytes.extend_from_slice(&[
        20, 0, 7, 0, 4, 0, b'c', 0, b'd', 0, b'e', 0, b'f', 0, 0x3E, 0xD8, 0x1D, 0xDD, 0, 0,
    ]);
    bytes
}

fn empty_lex_bytes() -> Vec<u8> {
    canonical_prefix(TABLE_OFFSET)
}

fn canonical_prefix(file_size: usize) -> Vec<u8> {
    let mut bytes = vec![0; TABLE_OFFSET];
    bytes[..8].copy_from_slice(b"imscwubi");
    write_u16(&mut bytes, 8, 1);
    write_u16(&mut bytes, 10, 1);
    write_i32(&mut bytes, 12, 64);
    write_i32(&mut bytes, 16, 168);
    write_i32(
        &mut bytes,
        20,
        i32::try_from(file_size).expect("test file size must fit i32"),
    );
    write_i32(&mut bytes, 24, 0x7856_3412);
    bytes
}

fn set_file_size(bytes: &mut [u8], file_size: usize) {
    write_i32(
        bytes,
        20,
        i32::try_from(file_size).expect("test file size must fit i32"),
    );
}

fn set_all_alpha_indexes(bytes: &mut [u8], first: i32, rest: i32) {
    write_i32(bytes, HEADER_SIZE, first);
    for index in 1..26 {
        write_i32(bytes, HEADER_SIZE + index * 4, rest);
    }
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("test field must contain four bytes"),
    )
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("test field must contain two bytes"),
    )
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn entry(code: &str, text: &str, weight_value: Option<u16>) -> LexiconEntry {
    LexiconEntry::new(
        LexCode::new(code).expect("test code must be valid"),
        text,
        weight_value.map(|value| Weight::new(value).expect("test weight must be valid")),
    )
    .expect("test entry must be valid")
}

fn assert_entry(entry: &LexiconEntry, code: &str, text: &str, weight_value: u16) {
    assert_eq!(entry.code().as_str(), code);
    assert_eq!(entry.text(), text);
    assert_eq!(entry.weight().map(Weight::get), Some(weight_value));
}

fn assert_error(bytes: &[u8], expected_kind: CodecErrorKind, expected_offset: usize) {
    let error = decode(bytes, DecodeLimits::default()).expect_err("bytes must be rejected");
    assert_eq!(error.kind(), &expected_kind);
    assert_eq!(
        error.location(),
        Some(SourceLocation::ByteOffset(
            u64::try_from(expected_offset).expect("test offset must fit u64")
        ))
    );
}

fn assert_invalid_offset(bytes: &[u8], field: &'static str, value: i64, offset: usize) {
    let error = decode(bytes, DecodeLimits::default()).expect_err("offset must be rejected");
    assert!(matches!(
        error.kind(),
        CodecErrorKind::InvalidOffset {
            field: actual_field,
            offset: actual_value,
            ..
        } if *actual_field == field && *actual_value == value
    ));
    assert_eq!(
        error.location(),
        Some(SourceLocation::ByteOffset(
            u64::try_from(offset).expect("test offset must fit u64")
        ))
    );
}

fn assert_malformed_unsigned(
    bytes: &[u8],
    field: &'static str,
    expected: &'static str,
    actual: u64,
    offset: usize,
) {
    assert_error(
        bytes,
        CodecErrorKind::MalformedField {
            field,
            expected: if expected == "0" {
                FieldValue::Unsigned(0)
            } else {
                FieldValue::Text(expected.to_owned())
            },
            actual: FieldValue::Unsigned(actual),
        },
        offset,
    );
}
