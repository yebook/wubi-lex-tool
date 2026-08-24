use std::num::NonZeroUsize;

use wubilex_codec::{
    CodecErrorKind, DecodeLimits, InvalidInputReason, ResourceKind, SourceLocation,
    SplitTableDocument, SplitTableEntry, TextEncoding, Weight, WordFrequencyDocument,
    WordFrequencyEntry, split_table, weight,
};

#[test]
fn auxiliary_models_preserve_order_duplicates_and_validate_tokens() {
    let first = WordFrequencyEntry::new("重复", weight_value(1))
        .expect("test frequency entry must be valid");
    let frequencies = WordFrequencyDocument::new(vec![first.clone(), first.clone()]);
    assert_eq!(frequencies.entries(), &[first.clone(), first]);
    assert_eq!(frequencies.len(), 2);
    assert_eq!(frequencies.clone().into_entries(), frequencies.entries());

    let split = SplitTableEntry::new("𠮷", "󰀖🤝").expect("test split entry must be valid");
    let splits = SplitTableDocument::new(vec![split.clone(), split.clone()]);
    assert_eq!(splits.entries(), &[split.clone(), split]);
    assert_eq!(splits.len(), 2);
    assert_eq!(splits.clone().into_entries(), splits.entries());

    for (field, result, index, character) in [
        (
            "word frequency word",
            WordFrequencyEntry::new("bad word", weight_value(1)).map(|_| ()),
            3,
            ' ',
        ),
        (
            "split table term",
            SplitTableEntry::new("bad\tterm", "root").map(|_| ()),
            3,
            '\t',
        ),
        (
            "split table roots",
            SplitTableEntry::new("term", "bad root").map(|_| ()),
            3,
            ' ',
        ),
    ] {
        let error = result.expect_err("whitespace must be rejected by public models");
        assert_eq!(
            error.kind(),
            &CodecErrorKind::InvalidInput {
                field,
                reason: InvalidInputReason::ContainsWhitespace { index, character },
            }
        );
    }
}

#[test]
fn word_frequency_decode_preserves_order_duplicates_and_boundaries() {
    let decoded = weight::decode(
        "甲\t1\r\n重复\u{3000}42\n重复 65535\n\n".as_bytes(),
        DecodeLimits::default(),
    )
    .expect("valid frequency text must decode");
    assert_eq!(
        decoded
            .entries()
            .iter()
            .map(|entry| (entry.word(), entry.weight().get()))
            .collect::<Vec<_>>(),
        vec![("甲", 1), ("重复", 42), ("重复", 65535)]
    );
    assert_eq!(
        weight::format(&decoded).expect("valid frequency document must format"),
        "甲\t1\n重复\t42\n重复\t65535\n"
    );
    assert_eq!(
        weight::format(&WordFrequencyDocument::default()),
        Ok(String::new())
    );
    assert!(
        weight::decode(b"", DecodeLimits::default())
            .expect("empty frequency text must decode")
            .is_empty()
    );
}

#[test]
fn split_table_decode_preserves_pua_non_bmp_order_and_duplicates() {
    let decoded = split_table::decode(
        "偎\t亻田一󰀖\r\n𠮷 口士🤝\n偎\t亻田一󰀖\n".as_bytes(),
        DecodeLimits::default(),
    )
    .expect("valid split table must decode");
    assert_eq!(
        decoded
            .entries()
            .iter()
            .map(|entry| (entry.term(), entry.roots()))
            .collect::<Vec<_>>(),
        vec![("偎", "亻田一󰀖"), ("𠮷", "口士🤝"), ("偎", "亻田一󰀖")]
    );
    assert_eq!(
        split_table::format(&decoded).expect("valid split document must format"),
        "偎\t亻田一󰀖\n𠮷\t口士🤝\n偎\t亻田一󰀖\n"
    );
    assert_eq!(
        split_table::format(&SplitTableDocument::default()),
        Ok(String::new())
    );
    assert!(
        split_table::decode(b"", DecodeLimits::default())
            .expect("empty split table text must decode")
            .is_empty()
    );
}

#[test]
fn auxiliary_text_requires_bomless_strict_utf8() {
    for bom in [
        [0xEF, 0xBB, 0xBF].as_slice(),
        [0xFF, 0xFE].as_slice(),
        [0xFE, 0xFF].as_slice(),
    ] {
        let error = weight::decode(bom, DecodeLimits::default()).expect_err("BOM must be rejected");
        assert!(matches!(
            error.kind(),
            CodecErrorKind::UnsupportedFormat {
                format: "word-frequency",
                ..
            }
        ));
        assert_eq!(error.location(), Some(SourceLocation::ByteOffset(0)));
    }

    for bom in [
        [0xEF, 0xBB, 0xBF].as_slice(),
        [0xFF, 0xFE].as_slice(),
        [0xFE, 0xFF].as_slice(),
    ] {
        let error =
            split_table::decode(bom, DecodeLimits::default()).expect_err("BOM must be rejected");
        assert!(matches!(
            error.kind(),
            CodecErrorKind::UnsupportedFormat {
                format: "split-table",
                ..
            }
        ));
        assert_eq!(error.location(), Some(SourceLocation::ByteOffset(0)));
    }

    let error = weight::decode(b"ok 1\n\xFF", DecodeLimits::default())
        .expect_err("malformed UTF-8 must fail");
    assert_eq!(
        error.kind(),
        &CodecErrorKind::InvalidTextEncoding {
            encoding: TextEncoding::Utf8,
        }
    );
    assert_eq!(error.location(), Some(SourceLocation::ByteOffset(5)));
}

#[test]
fn malformed_auxiliary_lines_fail_at_the_owning_field() {
    let frequency_cases = [
        ("onlyword\n", "word_frequency.line", 1),
        ("word 0\n", "word frequency weight", 6),
        ("word 65536\n", "word frequency weight", 6),
        ("word +1\n", "word frequency weight", 6),
        ("word nope\n", "word frequency weight", 6),
        ("word 1 extra\n", "word_frequency.line", 8),
    ];
    for (input, field, column) in frequency_cases {
        let error = weight::decode(input.as_bytes(), DecodeLimits::default())
            .expect_err("malformed frequency line must fail");
        assert!(matches!(
            error.kind(),
            CodecErrorKind::InvalidInput { field: actual, .. }
                | CodecErrorKind::MalformedField { field: actual, .. }
                if actual == &field
        ));
        assert_eq!(error.location(), Some(text_location(1, column)));
    }

    for (input, column) in [("term\n", 1), ("term roots extra\n", 12)] {
        let error = split_table::decode(input.as_bytes(), DecodeLimits::default())
            .expect_err("malformed split line must fail");
        assert!(matches!(
            error.kind(),
            CodecErrorKind::MalformedField {
                field: "split_table.line",
                ..
            }
        ));
        assert_eq!(error.location(), Some(text_location(1, column)));
    }

    let unicode_column =
        split_table::decode("词 roots extra\n".as_bytes(), DecodeLimits::default())
            .expect_err("columns must count Unicode scalars rather than UTF-8 bytes");
    assert_eq!(unicode_column.location(), Some(text_location(1, 9)));
}

#[test]
fn auxiliary_input_and_entry_limits_are_checked_at_exact_boundaries() {
    let input = b"a 1\nb 2\n";
    assert!(weight::decode(input, DecodeLimits::new(input.len(), 2)).is_ok());

    let bytes_error = weight::decode(input, DecodeLimits::new(input.len() - 1, 2))
        .expect_err("input one byte over limit must fail");
    assert_eq!(
        bytes_error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::InputBytes,
            limit: input.len() - 1,
            actual: input.len(),
        }
    );

    let entry_error = weight::decode(input, DecodeLimits::new(input.len(), 1))
        .expect_err("second entry must exceed limit");
    assert_eq!(
        entry_error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 1,
            actual: 2,
        }
    );
    assert_eq!(entry_error.location(), Some(text_location(2, 1)));

    let split_input = b"a b\nc d\n";
    assert!(split_table::decode(split_input, DecodeLimits::new(split_input.len(), 2),).is_ok());
    let split_error = split_table::decode(split_input, DecodeLimits::new(split_input.len(), 1))
        .expect_err("the second split entry must exceed the limit");
    assert_eq!(
        split_error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 1,
            actual: 2,
        }
    );
    assert_eq!(split_error.location(), Some(text_location(2, 1)));
}

#[test]
fn every_auxiliary_utf8_prefix_returns_without_panicking() {
    let frequency = "词 1\n".as_bytes();
    let split = "𠮷 口士🤝\n".as_bytes();
    for end in 0..frequency.len() {
        let _ = weight::decode(&frequency[..end], DecodeLimits::default());
    }
    for end in 0..split.len() {
        let _ = split_table::decode(&split[..end], DecodeLimits::default());
    }
}

fn weight_value(value: u16) -> Weight {
    Weight::new(value).expect("test weight must be valid")
}

fn text_location(line: usize, column: usize) -> SourceLocation {
    SourceLocation::Text {
        line: NonZeroUsize::new(line).expect("test line must be nonzero"),
        column: Some(NonZeroUsize::new(column).expect("test column must be nonzero")),
    }
}
