use std::num::NonZeroUsize;

use wubilex_codec::{
    CodecError, CodecErrorKind, DecodeLimits, FieldValue, ResourceKind, SourceLocation,
    TextEncoding,
};

#[test]
fn error_kind_is_independent_from_byte_location() {
    let kind = CodecErrorKind::MagicMismatch {
        expected: b"imscwubi".to_vec(),
        actual: b"mschxudp".to_vec(),
    };
    let error = CodecError::new(kind.clone()).at_byte_offset(12);

    assert_eq!(error.kind(), &kind);
    assert_eq!(error.location(), Some(SourceLocation::ByteOffset(12)));
}

#[test]
fn error_supports_one_based_text_line_and_column() {
    let line = NonZeroUsize::new(3).expect("test line is nonzero");
    let column = NonZeroUsize::new(7).expect("test column is nonzero");
    let kind = CodecErrorKind::InvalidTextEncoding {
        encoding: TextEncoding::Gbk,
    };
    let error = CodecError::new(kind.clone()).at_text(line, Some(column));

    assert_eq!(error.kind(), &kind);
    assert_eq!(
        error.location(),
        Some(SourceLocation::Text {
            line,
            column: Some(column),
        })
    );
}

#[test]
fn binary_failures_preserve_field_values_lengths_and_offset_bounds() {
    let eof = CodecErrorKind::UnexpectedEof {
        field: "lex.header.file_size",
        needed: 4,
        remaining: 2,
    };
    let malformed = CodecErrorKind::MalformedField {
        field: "eudp.entry.cb_size",
        expected: FieldValue::Unsigned(16),
        actual: FieldValue::Unsigned(12),
    };
    let offset = CodecErrorKind::InvalidOffset {
        field: "eudp.header.phrase_start",
        offset: -1,
        minimum: 64,
        maximum: 4_096,
    };

    assert!(matches!(
        eof,
        CodecErrorKind::UnexpectedEof {
            field: "lex.header.file_size",
            needed: 4,
            remaining: 2,
        }
    ));
    assert_eq!(
        malformed,
        CodecErrorKind::MalformedField {
            field: "eudp.entry.cb_size",
            expected: FieldValue::Unsigned(16),
            actual: FieldValue::Unsigned(12),
        }
    );
    assert!(matches!(
        offset,
        CodecErrorKind::InvalidOffset {
            field: "eudp.header.phrase_start",
            offset: -1,
            minimum: 64,
            maximum: 4_096,
        }
    ));
}

#[test]
fn decoding_and_format_failures_preserve_parser_context() {
    let utf16 = CodecErrorKind::InvalidUtf16 {
        field: "eudp.entry.text",
        unpaired_surrogate: 0xD800,
    };
    let unsupported = CodecErrorKind::UnsupportedFormat {
        format: "eudp",
        variant: "cbSize=12".to_owned(),
    };
    let overflow = CodecErrorKind::IntegerOverflow {
        operation: "phrase_start + count * 4",
    };

    assert!(matches!(
        utf16,
        CodecErrorKind::InvalidUtf16 {
            field: "eudp.entry.text",
            unpaired_surrogate: 0xD800,
        }
    ));
    assert!(matches!(
        unsupported,
        CodecErrorKind::UnsupportedFormat {
            format: "eudp",
            ref variant,
        } if variant == "cbSize=12"
    ));
    assert!(matches!(
        overflow,
        CodecErrorKind::IntegerOverflow {
            operation: "phrase_start + count * 4",
        }
    ));
}

#[test]
fn default_limits_match_the_approved_contract() {
    let limits = DecodeLimits::default();

    assert_eq!(limits.max_input_bytes(), 64 * 1024 * 1024);
    assert_eq!(limits.max_expanded_entries(), 500_000);
    assert!(limits.check_input_bytes(64 * 1024 * 1024).is_ok());
    assert!(limits.check_expanded_entries(500_000).is_ok());
}

#[test]
fn custom_input_limit_reports_structured_actual_and_limit() {
    let limits = DecodeLimits::new(4, 2);
    let error = limits
        .check_input_bytes(5)
        .expect_err("five input bytes must exceed a four-byte limit");

    assert_eq!(
        error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::InputBytes,
            limit: 4,
            actual: 5,
        }
    );
    assert_eq!(error.location(), None);
}

#[test]
fn custom_entry_limit_and_zero_limits_are_enforced() {
    let limits = DecodeLimits::new(0, 2);
    let error = limits
        .check_expanded_entries(3)
        .expect_err("three entries must exceed a two-entry limit");

    assert_eq!(
        error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 2,
            actual: 3,
        }
    );
    assert!(limits.check_input_bytes(0).is_ok());
    assert!(limits.check_input_bytes(1).is_err());
}
