use wubilex_codec::{
    Candidate, CodecErrorKind, DecodeLimits, FieldValue, PhraseCode, PhraseDocument, PhraseEntry,
    ResourceKind, SourceLocation,
    eudp::{decode, encode},
};

const HEADER_SIZE: usize = 64;
const PHRASE_START: usize = 80;
const FIRST_RECORD: usize = PHRASE_START;
const SECOND_RECORD: usize = 104;
const THIRD_RECORD: usize = 144;
const FILE_SIZE: usize = 200;
const TIMESTAMP: i32 = 1_700_000_000;

#[test]
fn hand_authored_wire_bytes_decode_and_encode_exactly() {
    let bytes = known_eudp_bytes();
    let document = decode(&bytes, DecodeLimits::default()).expect("known EUDP bytes must decode");

    assert_eq!(document.len(), 4);
    assert_entry(&document.entries()[0], "a", "甲", 2);
    assert_entry(&document.entries()[1], "a", "🤝\n%yyyy%", 255);
    assert_entry(&document.entries()[2], "zz", "重复", 1);
    assert_entry(&document.entries()[3], "zz", "重复", 1);
    assert_eq!(document.entries()[2], document.entries()[3]);
    assert_eq!(
        encode(&document, TIMESTAMP).expect("document must encode"),
        bytes
    );
}

#[test]
fn empty_document_has_an_exact_canonical_header() {
    let timestamp = -123;
    let document = PhraseDocument::default();
    let encoded = encode(&document, timestamp).expect("empty document must encode");

    let mut expected = vec![0; HEADER_SIZE];
    expected[..8].copy_from_slice(b"mschxudp");
    write_i32(&mut expected, 8, 0x0060_0002);
    write_i32(&mut expected, 12, 1);
    write_i32(&mut expected, 16, 64);
    write_i32(&mut expected, 20, 64);
    write_i32(&mut expected, 24, 64);
    write_i32(&mut expected, 28, 0);
    write_i32(&mut expected, 32, timestamp);

    assert_eq!(encoded, expected);
    assert_eq!(
        decode(&encoded, DecodeLimits::default()).expect("empty bytes must decode"),
        document
    );
}

#[test]
fn encode_stably_sorts_codes_and_preserves_equal_code_order_and_duplicates() {
    let first_z = entry("z", "first", 200);
    let duplicate_z = entry("z", "duplicate", 1);
    let document = PhraseDocument::new(vec![
        first_z.clone(),
        entry("a", "middle", 7),
        duplicate_z.clone(),
        duplicate_z.clone(),
    ]);

    let encoded = encode(&document, 42).expect("document must encode");
    let decoded = decode(&encoded, DecodeLimits::default()).expect("encoded bytes must decode");

    assert_eq!(document.entries()[0], first_z);
    assert_entry(&decoded.entries()[0], "a", "middle", 7);
    assert_eq!(decoded.entries()[1], first_z);
    assert_eq!(decoded.entries()[2], duplicate_z);
    assert_eq!(decoded.entries()[3], duplicate_z);
    assert_eq!(read_i32(&encoded, 64), 0);
    assert!(read_i32(&encoded, 68) > 0);
    assert!(read_i32(&encoded, 72) > read_i32(&encoded, 68));
    assert!(read_i32(&encoded, 76) > read_i32(&encoded, 72));
}

#[test]
fn maximum_text_offset_code_length_round_trips_and_the_next_length_overflows() {
    let maximum_code = "a".repeat(32_758);
    let maximum_document = PhraseDocument::new(vec![entry(&maximum_code, "x", 1)]);
    let encoded = encode(&maximum_document, 0).expect("maximum text offset must encode");

    assert_eq!(read_u16(&encoded, 68 + 4), 65_534);
    assert_eq!(
        decode(&encoded, DecodeLimits::default()).expect("maximum code must decode"),
        maximum_document
    );

    let too_long_code = "a".repeat(32_759);
    let too_long_document = PhraseDocument::new(vec![entry(&too_long_code, "x", 1)]);
    let error = encode(&too_long_document, 0).expect_err("text offset must fit u16");
    assert_eq!(
        error.kind(),
        &CodecErrorKind::IntegerOverflow {
            operation: "eudp text offset",
        }
    );
    assert_eq!(error.location(), None);
}

#[test]
fn deleted_records_are_validated_then_skipped() {
    let mut deleted = known_eudp_bytes();
    deleted[FIRST_RECORD + 9] = 1;
    let document = decode(&deleted, DecodeLimits::default()).expect("valid tombstone must decode");
    assert_eq!(document.len(), 3);
    assert_entry(&document.entries()[0], "a", "🤝\n%yyyy%", 255);

    let mut zero_candidate = deleted.clone();
    zero_candidate[FIRST_RECORD + 6] = 0;
    assert_malformed_unsigned(
        &zero_candidate,
        "eudp.entry.candidate",
        "1..=255",
        0,
        FIRST_RECORD + 6,
    );

    let mut invalid_utf16 = deleted.clone();
    write_u16(&mut invalid_utf16, FIRST_RECORD + 20, 0xD800);
    assert_error(
        &invalid_utf16,
        CodecErrorKind::InvalidUtf16 {
            field: "eudp.entry.text",
            unpaired_surrogate: 0xD800,
        },
        FIRST_RECORD + 20,
    );

    let mut bad_terminator = deleted.clone();
    write_u16(&mut bad_terminator, FIRST_RECORD + 18, 1);
    assert_malformed_expected_unsigned(
        &bad_terminator,
        "eudp.entry.code_terminator",
        0,
        1,
        FIRST_RECORD + 18,
    );

    let mut empty_text = deleted;
    write_u16(&mut empty_text, FIRST_RECORD + 4, 22);
    write_u16(&mut empty_text, FIRST_RECORD + 18, u16::from(b'b'));
    write_u16(&mut empty_text, FIRST_RECORD + 20, 0);
    assert_malformed_unsigned(
        &empty_text,
        "eudp.entry.text",
        "nonempty UTF-16LE text without U+0000",
        0,
        FIRST_RECORD + 22,
    );
}

#[test]
fn nonstructural_metadata_is_tolerated_and_canonicalized() {
    let canonical = known_eudp_bytes();
    let mut noncanonical = canonical.clone();
    write_i32(&mut noncanonical, 8, -1);
    write_i32(&mut noncanonical, 12, 99);
    write_i32(&mut noncanonical, 32, -123);
    noncanonical[36..64].fill(0xA5);
    write_u16(&mut noncanonical, FIRST_RECORD + 2, 99);
    noncanonical[FIRST_RECORD + 7] = 99;
    noncanonical[FIRST_RECORD + 8] = 99;
    write_u16(&mut noncanonical, FIRST_RECORD + 10, 99);
    write_i32(&mut noncanonical, FIRST_RECORD + 12, -1);

    let document = decode(&noncanonical, DecodeLimits::default())
        .expect("non-layout metadata must be tolerated");

    assert_eq!(
        encode(&document, TIMESTAMP).expect("document must encode"),
        canonical
    );
}

#[test]
fn a_later_offset_table_start_is_accepted_and_normalized() {
    let canonical = known_eudp_bytes();
    let mut extended = canonical.clone();
    extended.splice(HEADER_SIZE..HEADER_SIZE, [0xA5; 4]);
    write_i32(&mut extended, 16, 68);
    write_i32(&mut extended, 20, 84);
    write_i32(&mut extended, 24, 204);

    let document = decode(&extended, DecodeLimits::default())
        .expect("header extension before the offset table must decode");

    assert_eq!(
        encode(&document, TIMESTAMP).expect("document must encode"),
        canonical
    );
}

#[test]
fn magic_declared_end_and_count_errors_preserve_header_offsets() {
    let mut wrong_magic = known_eudp_bytes();
    wrong_magic[..8].copy_from_slice(b"notudp!!");
    assert_error(
        &wrong_magic,
        CodecErrorKind::MagicMismatch {
            expected: b"mschxudp".to_vec(),
            actual: b"notudp!!".to_vec(),
        },
        0,
    );

    let mut wrong_end = known_eudp_bytes();
    write_i32(&mut wrong_end, 24, 199);
    assert_error(
        &wrong_end,
        CodecErrorKind::MalformedField {
            field: "eudp.header.phrase_end",
            expected: FieldValue::Unsigned(FILE_SIZE as u64),
            actual: FieldValue::Signed(199),
        },
        24,
    );

    let mut negative_end = known_eudp_bytes();
    write_i32(&mut negative_end, 24, -1);
    assert_invalid_offset(&negative_end, "eudp.header.phrase_end", -1, 24);

    let mut negative_count = known_eudp_bytes();
    write_i32(&mut negative_count, 28, -1);
    assert_error(
        &negative_count,
        CodecErrorKind::MalformedField {
            field: "eudp.header.count",
            expected: FieldValue::Text("a nonnegative wire entry count".to_owned()),
            actual: FieldValue::Signed(-1),
        },
        28,
    );
}

#[test]
fn structural_header_offsets_and_table_length_are_strict() {
    let mut negative_table_start = known_eudp_bytes();
    write_i32(&mut negative_table_start, 16, -1);
    assert_invalid_offset(
        &negative_table_start,
        "eudp.header.phrase_offset_start",
        -1,
        16,
    );

    let mut table_inside_header = known_eudp_bytes();
    write_i32(&mut table_inside_header, 16, 60);
    assert_invalid_offset(
        &table_inside_header,
        "eudp.header.phrase_offset_start",
        60,
        16,
    );

    let mut negative_phrase_start = known_eudp_bytes();
    write_i32(&mut negative_phrase_start, 20, -1);
    assert_invalid_offset(&negative_phrase_start, "eudp.header.phrase_start", -1, 20);

    let mut wrong_table_end = known_eudp_bytes();
    write_i32(&mut wrong_table_end, 20, 84);
    assert_error(
        &wrong_table_end,
        CodecErrorKind::MalformedField {
            field: "eudp.header.phrase_start",
            expected: FieldValue::Unsigned(PHRASE_START as u64),
            actual: FieldValue::Signed(84),
        },
        20,
    );

    let mut zero_count_with_records = known_eudp_bytes();
    write_i32(&mut zero_count_with_records, 20, 64);
    write_i32(&mut zero_count_with_records, 28, 0);
    assert_error(
        &zero_count_with_records,
        CodecErrorKind::MalformedField {
            field: "eudp.header.phrase_end",
            expected: FieldValue::Unsigned(64),
            actual: FieldValue::Signed(FILE_SIZE as i64),
        },
        24,
    );
}

#[test]
fn offset_table_requires_zero_strictly_increasing_in_range_boundaries() {
    let mut nonzero_first = known_eudp_bytes();
    write_i32(&mut nonzero_first, 64, 1);
    assert_error(
        &nonzero_first,
        CodecErrorKind::MalformedField {
            field: "eudp.offset_table.first",
            expected: FieldValue::Unsigned(0),
            actual: FieldValue::Signed(1),
        },
        64,
    );

    let mut negative = known_eudp_bytes();
    write_i32(&mut negative, 68, -1);
    assert_invalid_offset(&negative, "eudp.offset_table", -1, 68);

    let mut repeated = known_eudp_bytes();
    write_i32(&mut repeated, 68, 0);
    assert_error(
        &repeated,
        CodecErrorKind::MalformedField {
            field: "eudp.offset_table.order",
            expected: FieldValue::Text("strictly increasing relative offsets".to_owned()),
            actual: FieldValue::Signed(0),
        },
        68,
    );

    let mut beyond_section = known_eudp_bytes();
    write_i32(&mut beyond_section, 76, 120);
    assert_invalid_offset(&beyond_section, "eudp.offset_table", 120, 76);

    let mut inside_record = known_eudp_bytes();
    write_i32(&mut inside_record, 68, 22);
    assert_error(
        &inside_record,
        CodecErrorKind::UnexpectedEof {
            field: "eudp.entry",
            needed: 24,
            remaining: 22,
        },
        FIRST_RECORD,
    );
}

#[test]
fn record_header_variant_offset_and_candidate_are_strict() {
    let mut unsupported = known_eudp_bytes();
    write_u16(&mut unsupported, FIRST_RECORD, 12);
    assert_error(
        &unsupported,
        CodecErrorKind::UnsupportedFormat {
            format: "eudp",
            variant: "cbSize=12".to_owned(),
        },
        FIRST_RECORD,
    );

    let mut odd_offset = known_eudp_bytes();
    write_u16(&mut odd_offset, FIRST_RECORD + 4, 21);
    assert_malformed_unsigned(
        &odd_offset,
        "eudp.entry.text_offset",
        "an even relative byte offset",
        21,
        FIRST_RECORD + 4,
    );

    for invalid in [18, 24] {
        let mut bytes = known_eudp_bytes();
        write_u16(&mut bytes, FIRST_RECORD + 4, invalid);
        let error = decode(&bytes, DecodeLimits::default()).expect_err("offset must be rejected");
        assert!(matches!(
            error.kind(),
            CodecErrorKind::InvalidOffset {
                field: "eudp.entry.text_offset",
                offset,
                minimum: 20,
                maximum: 22,
            } if *offset == i64::from(invalid)
        ));
        assert_eq!(
            error.location(),
            Some(SourceLocation::ByteOffset((FIRST_RECORD + 4) as u64))
        );
    }

    let mut zero_candidate = known_eudp_bytes();
    zero_candidate[FIRST_RECORD + 6] = 0;
    assert_malformed_unsigned(
        &zero_candidate,
        "eudp.entry.candidate",
        "1..=255",
        0,
        FIRST_RECORD + 6,
    );

    let mut odd_record = known_eudp_bytes();
    write_i32(&mut odd_record, 68, 25);
    assert_malformed_unsigned(
        &odd_record,
        "eudp.entry.length",
        "an even byte length",
        25,
        FIRST_RECORD,
    );
}

#[test]
fn code_is_strict_lowercase_ascii_utf16_and_nul_terminated() {
    let mut uppercase = known_eudp_bytes();
    write_u16(&mut uppercase, FIRST_RECORD + 16, u16::from(b'A'));
    assert_malformed_unsigned(
        &uppercase,
        "eudp.entry.code",
        "a nonempty lowercase ASCII UTF-16 string without U+0000",
        u64::from(b'A'),
        FIRST_RECORD + 16,
    );

    let mut embedded_nul = known_eudp_bytes();
    write_u16(&mut embedded_nul, FIRST_RECORD + 16, 0);
    assert_malformed_unsigned(
        &embedded_nul,
        "eudp.entry.code",
        "a nonempty lowercase ASCII UTF-16 string without U+0000",
        0,
        FIRST_RECORD + 16,
    );

    let mut invalid_utf16 = known_eudp_bytes();
    write_u16(&mut invalid_utf16, FIRST_RECORD + 16, 0xD800);
    assert_error(
        &invalid_utf16,
        CodecErrorKind::InvalidUtf16 {
            field: "eudp.entry.code",
            unpaired_surrogate: 0xD800,
        },
        FIRST_RECORD + 16,
    );

    let mut terminator = known_eudp_bytes();
    write_u16(&mut terminator, FIRST_RECORD + 18, 1);
    assert_malformed_expected_unsigned(
        &terminator,
        "eudp.entry.code_terminator",
        0,
        1,
        FIRST_RECORD + 18,
    );
}

#[test]
fn text_is_strict_nonempty_utf16_without_embedded_nul() {
    let mut embedded_nul = known_eudp_bytes();
    write_u16(&mut embedded_nul, THIRD_RECORD + 22, 0);
    assert_malformed_unsigned(
        &embedded_nul,
        "eudp.entry.text",
        "nonempty UTF-16LE text without U+0000",
        0,
        THIRD_RECORD + 22,
    );

    let mut invalid_utf16 = known_eudp_bytes();
    write_u16(&mut invalid_utf16, SECOND_RECORD + 22, u16::from(b'x'));
    assert_error(
        &invalid_utf16,
        CodecErrorKind::InvalidUtf16 {
            field: "eudp.entry.text",
            unpaired_surrogate: 0xD83E,
        },
        SECOND_RECORD + 20,
    );

    let mut terminator = known_eudp_bytes();
    write_u16(&mut terminator, FIRST_RECORD + 22, 1);
    assert_malformed_expected_unsigned(
        &terminator,
        "eudp.entry.text_terminator",
        0,
        1,
        FIRST_RECORD + 22,
    );
}

#[test]
fn records_must_be_sorted_by_code_but_not_by_candidate() {
    let document = PhraseDocument::new(vec![entry("a", "high", 255), entry("a", "low", 1)]);
    let encoded = encode(&document, 0).expect("candidate order need not be sorted");
    let decoded = decode(&encoded, DecodeLimits::default()).expect("bytes must decode");
    assert_eq!(decoded, document);

    let mut descending = known_eudp_bytes();
    write_u16(&mut descending, FIRST_RECORD + 16, u16::from(b'b'));
    assert_error(
        &descending,
        CodecErrorKind::MalformedField {
            field: "eudp.entry.code_order",
            expected: FieldValue::Text("lexicographic nondecreasing order".to_owned()),
            actual: FieldValue::Text("a".to_owned()),
        },
        SECOND_RECORD + 16,
    );
}

#[test]
fn input_and_declared_wire_entry_limits_are_checked_at_exact_boundaries() {
    let bytes = known_eudp_bytes();
    assert_eq!(
        decode(&bytes, DecodeLimits::new(bytes.len(), 4))
            .expect("exact limits must pass")
            .len(),
        4
    );

    let input_error = decode(&bytes, DecodeLimits::new(bytes.len() - 1, 4))
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

    let mut with_deleted = bytes;
    with_deleted[FIRST_RECORD + 9] = 1;
    let entry_error = decode(&with_deleted, DecodeLimits::new(FILE_SIZE, 3))
        .expect_err("tombstones must count toward the wire entry limit");
    assert_eq!(
        entry_error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 3,
            actual: 4,
        }
    );
    assert_eq!(entry_error.location(), Some(SourceLocation::ByteOffset(28)));
}

#[test]
fn encode_rejects_embedded_nul_without_inventing_a_wire_location() {
    let document = PhraseDocument::new(vec![entry("a", "left\0right", 1)]);
    let error = encode(&document, 0).expect_err("ambiguous NUL text must be rejected");

    assert_eq!(
        error.kind(),
        &CodecErrorKind::MalformedField {
            field: "eudp.entry.text",
            expected: FieldValue::Text("text without U+0000".to_owned()),
            actual: FieldValue::Text("embedded U+0000 at UTF-16 code unit index 4".to_owned()),
        }
    );
    assert_eq!(error.location(), None);
}

#[test]
fn every_truncated_prefix_returns_a_structured_error_without_panicking() {
    let bytes = known_eudp_bytes();

    for length in 0..bytes.len() {
        let result = std::panic::catch_unwind(|| decode(&bytes[..length], DecodeLimits::default()));
        let decode_result = result.expect("decoder must not panic for any truncated prefix");
        assert!(decode_result.is_err(), "prefix of {length} bytes must fail");
    }
}

#[test]
fn every_phrase_section_prefix_is_safe_with_a_matching_declared_end() {
    let bytes = known_eudp_bytes();

    for length in HEADER_SIZE..bytes.len() {
        let mut prefix = bytes[..length].to_vec();
        write_i32(
            &mut prefix,
            24,
            i32::try_from(length).expect("test length must fit i32"),
        );

        let result = std::panic::catch_unwind(|| decode(&prefix, DecodeLimits::default()));
        let decode_result = result.expect("decoder must not panic inside a truncated section");
        assert!(
            decode_result.is_err(),
            "phrase-section prefix of {length} bytes must fail"
        );
    }
}

fn known_eudp_bytes() -> Vec<u8> {
    let mut bytes = vec![0; PHRASE_START];
    bytes[..8].copy_from_slice(b"mschxudp");
    write_i32(&mut bytes, 8, 0x0060_0002);
    write_i32(&mut bytes, 12, 1);
    write_i32(&mut bytes, 16, HEADER_SIZE as i32);
    write_i32(&mut bytes, 20, PHRASE_START as i32);
    write_i32(&mut bytes, 24, FILE_SIZE as i32);
    write_i32(&mut bytes, 28, 4);
    write_i32(&mut bytes, 32, TIMESTAMP);
    for (index, offset) in [0, 24, 64, 92].into_iter().enumerate() {
        write_i32(&mut bytes, HEADER_SIZE + index * 4, offset);
    }

    bytes.extend_from_slice(&[
        16, 0, 16, 0, 20, 0, 2, 6, 0, 0, 0, 0, 0, 0, 0, 0, b'a', 0, 0, 0, 0x32, 0x75, 0, 0,
    ]);
    bytes.extend_from_slice(&[
        16, 0, 16, 0, 20, 0, 255, 6, 0, 0, 0, 0, 0, 0, 0, 0, b'a', 0, 0, 0, 0x3E, 0xD8, 0x1D, 0xDD,
        0x0A, 0, b'%', 0, b'y', 0, b'y', 0, b'y', 0, b'y', 0, b'%', 0, 0, 0,
    ]);
    let duplicate = [
        16, 0, 16, 0, 22, 0, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, b'z', 0, b'z', 0, 0, 0, 0xCD, 0x91,
        0x0D, 0x59, 0, 0,
    ];
    bytes.extend_from_slice(&duplicate);
    bytes.extend_from_slice(&duplicate);

    assert_eq!(bytes.len(), FILE_SIZE, "hand-authored fixture size drifted");
    bytes
}

fn entry(code: &str, text: &str, candidate: u8) -> PhraseEntry {
    PhraseEntry::new(
        PhraseCode::new(code).expect("test code must be valid"),
        text,
        Candidate::new(candidate).expect("test candidate must be valid"),
    )
    .expect("test entry must be valid")
}

fn assert_entry(entry: &PhraseEntry, code: &str, text: &str, candidate: u8) {
    assert_eq!(entry.code().as_str(), code);
    assert_eq!(entry.text(), text);
    assert_eq!(entry.candidate().get(), candidate);
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

fn assert_error(bytes: &[u8], expected_kind: CodecErrorKind, expected_offset: usize) {
    let error = decode(bytes, DecodeLimits::default()).expect_err("bytes must be rejected");
    assert_eq!(error.kind(), &expected_kind);
    assert_eq!(
        error.location(),
        Some(SourceLocation::ByteOffset(expected_offset as u64))
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
        Some(SourceLocation::ByteOffset(offset as u64))
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
            expected: FieldValue::Text(expected.to_owned()),
            actual: FieldValue::Unsigned(actual),
        },
        offset,
    );
}

fn assert_malformed_expected_unsigned(
    bytes: &[u8],
    field: &'static str,
    expected: u64,
    actual: u64,
    offset: usize,
) {
    assert_error(
        bytes,
        CodecErrorKind::MalformedField {
            field,
            expected: FieldValue::Unsigned(expected),
            actual: FieldValue::Unsigned(actual),
        },
        offset,
    );
}
