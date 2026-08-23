use std::num::NonZeroUsize;

use wubilex_codec::{
    CodecErrorKind, DecodeLimits, InvalidInputReason, ResourceKind, SourceLocation, TextEncoding,
    Weight, text,
};

#[test]
fn four_supported_encodings_decode_to_the_same_document() {
    let utf8 = "a\t甲\r\n".as_bytes().to_vec();
    let utf8_bom = [vec![0xEF, 0xBB, 0xBF], utf8.clone()].concat();
    let utf16le = vec![
        0xFF, 0xFE, 0x61, 0x00, 0x09, 0x00, 0x32, 0x75, 0x0D, 0x00, 0x0A, 0x00,
    ];
    let utf16be = vec![
        0xFE, 0xFF, 0x00, 0x61, 0x00, 0x09, 0x75, 0x32, 0x00, 0x0D, 0x00, 0x0A,
    ];
    let gbk = vec![0x61, 0x09, 0xBC, 0xD7, 0x0D, 0x0A];

    let cases = [
        (utf8, TextEncoding::Utf8, false),
        (utf8_bom, TextEncoding::Utf8, true),
        (utf16le, TextEncoding::Utf16Le, true),
        (utf16be, TextEncoding::Utf16Be, true),
        (gbk, TextEncoding::Gbk, false),
    ];

    for (bytes, encoding, has_bom) in cases {
        let decoded = decode(&bytes);
        assert_eq!(decoded.detected_encoding().encoding(), encoding);
        assert_eq!(decoded.detected_encoding().has_bom(), has_bom);
        assert_eq!(entries(&decoded), vec![("a", "甲", None)]);
    }
}

#[test]
fn malformed_encodings_report_original_zero_based_byte_offsets() {
    let cases = [
        (vec![0xEF, 0xBB, 0xBF, b'a', 0xFF], TextEncoding::Utf8, 4),
        (
            vec![0xEF, 0xBB, 0xBF, b'a', 0xE2, 0x82],
            TextEncoding::Utf8,
            4,
        ),
        (vec![0xFF, 0xFE, b'a', 0x00, 0xFF], TextEncoding::Utf16Le, 4),
        (
            vec![0xFF, 0xFE, b'a', 0x00, 0x00, 0xD8],
            TextEncoding::Utf16Le,
            4,
        ),
        (
            vec![0xFE, 0xFF, 0x00, b'a', 0xD8, 0x00],
            TextEncoding::Utf16Be,
            4,
        ),
        (vec![0xFE, 0xFF, 0x00, b'a', 0xFF], TextEncoding::Utf16Be, 4),
        (vec![0x81, 0x30], TextEncoding::Gbk, 0),
    ];

    for (bytes, encoding, offset) in cases {
        let error = text::decode(&bytes, DecodeLimits::default())
            .expect_err("malformed encoded input must fail");
        assert_eq!(
            error.kind(),
            &CodecErrorKind::InvalidTextEncoding { encoding }
        );
        assert_eq!(error.location(), Some(SourceLocation::ByteOffset(offset)));
    }
}

#[test]
fn yaml_comments_and_description_keep_original_line_locations() {
    let input = concat!(
        "---\n",
        "name: sample\n",
        "...\n",
        "# ignored\n",
        "description\n",
        "[Text]\n",
        "??? ???\n",
        "ab valid\n",
    );

    let decoded = decode(input.as_bytes());
    assert_eq!(entries(&decoded), vec![("ab", "valid", None)]);
    assert_eq!(decoded.warnings().len(), 1);
    assert_eq!(decoded.warnings()[0].preview(), "??? ???");
    assert_eq!(decoded.warnings()[0].location(), text_location(7, 1));
}

#[test]
fn each_a_to_f_dialect_preserves_its_expansion_and_weight_semantics() {
    let input = concat!(
        "aa alpha 7\n",
        "bb=8,beta\n",
        "cc gamma delta\n",
        "反转 dd 65534 tag\n",
        "单码 ee\n",
        "多码 ff gg\n",
    );

    let decoded = decode(input.as_bytes());
    assert_eq!(
        entries(&decoded),
        vec![
            ("aa", "alpha", Some(7)),
            ("bb", "beta", Some(8)),
            ("cc", "gamma", None),
            ("cc", "delta", None),
            ("dd", "反转", Some(1)),
            ("ee", "单码", None),
            ("ff", "多码", None),
            ("gg", "多码", None),
        ]
    );
}

#[test]
fn microsoft_branch_expands_codes_and_returns_early() {
    let input = "Microsoft Wubi\n[Text]\n词%20条 aa bb\n另一个 cc\n";

    let decoded = decode(input.as_bytes());
    assert_eq!(
        entries(&decoded),
        vec![
            ("aa", "词 条", None),
            ("bb", "词 条", None),
            ("cc", "另一个", None),
        ]
    );
}

#[test]
fn descending_weights_are_normalized_only_for_d_entries() {
    let input = "aa explicit 6000\n反转 bb 0\n其次 cc 1tag\n";

    let decoded = decode(input.as_bytes());
    assert_eq!(
        entries(&decoded),
        vec![
            ("aa", "explicit", Some(6000)),
            ("bb", "反转", Some(5001)),
            ("cc", "其次", Some(5000)),
        ]
    );
}

#[test]
fn signed_weight_fails_at_the_weight_field() {
    let cases = [("ab word -1\n", 9), ("abcd  -1\n", 7), ("词 ab -1\n", 6)];

    for (input, column) in cases {
        let error = text::decode(input.as_bytes(), DecodeLimits::default())
            .expect_err("a negative weight must not become an unweighted entry or warning");

        assert!(matches!(
            error.kind(),
            CodecErrorKind::MalformedField {
                field: "lexicon weight",
                ..
            }
        ));
        assert_eq!(error.location(), Some(text_location(1, column)));
    }
}

#[test]
fn descending_layout_rejects_more_than_one_trailing_suffix_token() {
    let decoded = decode("词 ab 1 tag extra\ncd valid\n".as_bytes());

    assert_eq!(entries(&decoded), vec![("cd", "valid", None)]);
    assert_eq!(decoded.warnings().len(), 1);
    assert_eq!(decoded.warnings()[0].preview(), "词 ab 1 tag extra");
}

#[test]
fn jidian_cleanup_removes_markers_strips_tilde_and_clears_weights() {
    let input = concat!(
        "~:生僻字词\n",
        "^:用户词组\n",
        "[Text]\n",
        "aa ~保留 9\n",
        "aa ^删除 10\n",
        "bb $删除 11\n",
        "cc !删除 12\n",
    );

    let decoded = decode(input.as_bytes());
    assert_eq!(entries(&decoded), vec![("aa", "保留", None)]);
}

#[test]
fn warnings_are_ordered_bounded_and_charged_to_the_output_budget() {
    let long = "界".repeat(161);
    let input = format!("\t??? ???\n{long}\nab valid\n");
    let decoded = decode(input.as_bytes());

    assert_eq!(decoded.warnings().len(), 2);
    assert_eq!(decoded.warnings()[0].location(), text_location(1, 2));
    assert_eq!(decoded.warnings()[0].preview(), "\t??? ???");
    assert!(!decoded.warnings()[0].is_truncated());
    assert_eq!(decoded.warnings()[1].preview().chars().count(), 160);
    assert!(decoded.warnings()[1].is_truncated());

    let error = text::decode(input.as_bytes(), DecodeLimits::new(input.len(), 2))
        .expect_err("two warnings plus one entry must exceed the shared budget");
    assert_eq!(
        error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 2,
            actual: 3,
        }
    );
    assert_eq!(error.location(), Some(text_location(3, 1)));
}

#[test]
fn recognized_invalid_fields_fail_instead_of_becoming_warnings() {
    let cases = [
        ("abcde word\n", "lexicon code", 1, 1),
        ("abcd  1\n", "lexicon text", 1, 6),
        ("abcd word 0\n", "lexicon weight", 1, 11),
        ("abcd=1,\n", "lexicon text", 1, 8),
        ("词 ab 65535\n", "lexicon weight", 1, 6),
        ("词 Ab\n", "lexicon code", 1, 3),
    ];

    for (input, field, line, column) in cases {
        let error = text::decode(input.as_bytes(), DecodeLimits::default())
            .expect_err("recognized invalid syntax must fail");
        assert!(matches!(
            error.kind(),
            CodecErrorKind::InvalidInput {
                field: actual_field,
                reason: InvalidInputReason::Empty
                    | InvalidInputReason::Zero
                    | InvalidInputReason::NotLowercaseAscii { .. }
                    | InvalidInputReason::TooLong { .. },
            } if *actual_field == field
        ));
        assert_eq!(error.location(), Some(text_location(line, column)));
    }
}

#[test]
fn nonempty_warning_only_body_and_unclosed_yaml_are_errors() {
    let warning_only = text::decode(b"??? ???\n", DecodeLimits::default())
        .expect_err("a nonempty body with no entries must fail");
    assert!(matches!(
        warning_only.kind(),
        CodecErrorKind::MalformedField {
            field: "text.body",
            ..
        }
    ));
    assert_eq!(warning_only.location(), Some(text_location(1, 1)));

    let yaml = text::decode(b"---\nname: missing end\n", DecodeLimits::default())
        .expect_err("unclosed YAML front matter must fail");
    assert!(matches!(
        yaml.kind(),
        CodecErrorKind::MalformedField {
            field: "text.yaml_front_matter",
            ..
        }
    ));
    assert_eq!(yaml.location(), Some(text_location(1, 1)));
}

#[test]
fn empty_and_preprocessing_only_bodies_are_valid_empty_documents() {
    for input in [
        b"".as_slice(),
        b"# comment\n\n".as_slice(),
        b"description\n[Text]\n".as_slice(),
    ] {
        let decoded = decode(input);
        assert!(decoded.document().is_empty());
        assert!(decoded.warnings().is_empty());
    }
}

#[test]
fn input_limit_is_checked_before_encoding_detection() {
    let error = text::decode(&[0xFF], DecodeLimits::new(0, 10))
        .expect_err("a nonempty input must exceed a zero-byte limit first");

    assert_eq!(
        error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::InputBytes,
            limit: 0,
            actual: 1,
        }
    );
    assert_eq!(error.location(), None);
}

#[test]
fn multi_text_expansion_stops_at_the_first_over_budget_entry() {
    let error = text::decode(b"aa one two three\n", DecodeLimits::new(100, 2))
        .expect_err("the third expanded text must exceed a two-entry limit");

    assert_eq!(
        error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 2,
            actual: 3,
        }
    );
    assert_eq!(error.location(), Some(text_location(1, 1)));
}

#[test]
fn ascending_and_descending_weight_endpoints_are_strict() {
    let decoded = decode("aa maximum 65535\n最小 bb 65534\n".as_bytes());
    assert_eq!(
        entries(&decoded),
        vec![("aa", "maximum", Some(65535)), ("bb", "最小", Some(1)),]
    );

    for input in [
        "aa zero 0\n",
        "aa overflow 65536\n",
        "零 bb 65535\n",
        "溢出 bb 65536\n",
    ] {
        assert!(
            text::decode(input.as_bytes(), DecodeLimits::default()).is_err(),
            "weight boundary must fail for {input:?}"
        );
    }
}

#[test]
fn jidian_empty_tilde_and_all_removed_documents_fail() {
    let prefix = "~:生僻字词\n^:用户词组\n[Text]\n";
    let empty_tilde = format!("{prefix}aa ~\n");
    let error = text::decode(empty_tilde.as_bytes(), DecodeLimits::default())
        .expect_err("stripping a lone tilde must not create empty text");
    assert!(matches!(
        error.kind(),
        CodecErrorKind::InvalidInput {
            field: "lexicon text",
            reason: InvalidInputReason::Empty,
        }
    ));
    assert_eq!(error.location(), Some(text_location(4, 4)));

    let all_removed = format!("{prefix}aa ^removed\n");
    let error = text::decode(all_removed.as_bytes(), DecodeLimits::default())
        .expect_err("a nonempty body removed by cleanup must not become empty success");
    assert!(matches!(
        error.kind(),
        CodecErrorKind::MalformedField {
            field: "text.body",
            ..
        }
    ));
}

#[test]
fn unknown_percent_sequences_remain_literal_in_decoded_text() {
    let decoded = decode(b"a %0b%0c%25%2F%\n");

    assert_eq!(entries(&decoded), vec![("a", "%0b%0c%25%2F%", None)]);
}

#[test]
fn every_truncated_encoded_prefix_returns_without_panicking() {
    let utf16le = [
        0xFF, 0xFE, 0x61, 0x00, 0x09, 0x00, 0x32, 0x75, 0x0D, 0x00, 0x0A, 0x00,
    ];

    for end in 0..utf16le.len() {
        let _ = text::decode(&utf16le[..end], DecodeLimits::default());
    }
}

fn decode(input: &[u8]) -> wubilex_codec::DecodedLexiconText {
    text::decode(input, DecodeLimits::default()).expect("test text must decode")
}

fn entries(decoded: &wubilex_codec::DecodedLexiconText) -> Vec<(&str, &str, Option<u16>)> {
    decoded
        .document()
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.code().as_str(),
                entry.text(),
                entry.weight().map(Weight::get),
            )
        })
        .collect()
}

fn text_location(line: usize, column: usize) -> SourceLocation {
    SourceLocation::Text {
        line: NonZeroUsize::new(line).expect("test line must be nonzero"),
        column: Some(NonZeroUsize::new(column).expect("test column must be nonzero")),
    }
}
