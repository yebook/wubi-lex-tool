use std::num::NonZeroUsize;

use wubilex_codec::{
    Candidate, CodecErrorKind, DecodeLimits, InvalidInputReason, PhraseCode, PhraseDocument,
    PhraseEntry, PhraseTextWarningKind, ResourceKind, SourceLocation, TextEncoding, phrase_text,
};

#[test]
fn supported_encodings_produce_the_same_phrase_document() {
    let utf8 = "aa=甲\r\n".as_bytes().to_vec();
    let utf8_bom = [vec![0xEF, 0xBB, 0xBF], utf8.clone()].concat();
    let utf16le = vec![
        0xFF, 0xFE, 0x61, 0x00, 0x61, 0x00, 0x3D, 0x00, 0x32, 0x75, 0x0D, 0x00, 0x0A, 0x00,
    ];
    let utf16be = vec![
        0xFE, 0xFF, 0x00, 0x61, 0x00, 0x61, 0x00, 0x3D, 0x75, 0x32, 0x00, 0x0D, 0x00, 0x0A,
    ];
    let gbk = vec![0x61, 0x61, 0x3D, 0xBC, 0xD7, 0x0D, 0x0A];
    let cases = [
        (utf8, TextEncoding::Utf8, false),
        (utf8_bom, TextEncoding::Utf8, true),
        (utf16le, TextEncoding::Utf16Le, true),
        (utf16be, TextEncoding::Utf16Be, true),
        (gbk, TextEncoding::Gbk, false),
    ];

    for (input, encoding, has_bom) in cases {
        let decoded = decode(&input);
        assert_eq!(decoded.detected_encoding().encoding(), encoding);
        assert_eq!(decoded.detected_encoding().has_bom(), has_bom);
        assert_eq!(entries(decoded.document()), vec![("aa", "甲", 1)]);
    }
}

#[test]
fn malformed_encoded_bytes_report_original_offsets() {
    let cases = [
        (vec![0xEF, 0xBB, 0xBF, b'a', 0xFF], TextEncoding::Utf8, 4),
        (vec![0xFF, 0xFE, b'a', 0x00, 0xFF], TextEncoding::Utf16Le, 4),
        (
            vec![0xFE, 0xFF, 0x00, b'a', 0xD8, 0x00],
            TextEncoding::Utf16Be,
            4,
        ),
        (vec![0x81, 0x30], TextEncoding::Gbk, 0),
    ];

    for (input, encoding, offset) in cases {
        let error = phrase_text::decode(&input, DecodeLimits::default())
            .expect_err("malformed text must fail");
        assert_eq!(
            error.kind(),
            &CodecErrorKind::InvalidTextEncoding { encoding }
        );
        assert_eq!(error.location(), Some(SourceLocation::ByteOffset(offset)));
    }
}

#[test]
fn each_phrase_dialect_decodes_independently() {
    let cases = [
        ("aa=2,甲\n", ("aa", "甲", 2)),
        ("bb,3=#\n乙\n", ("bb", "乙", 3)),
        ("cc=\n丙\n", ("cc", "丙", 1)),
        ("dd 丁 4\n", ("dd", "丁", 4)),
        ("ee 戊\n", ("ee", "戊", 1)),
        ("己 ff\n", ("ff", "己", 1)),
    ];

    for (input, expected) in cases {
        let decoded = decode(input.as_bytes());
        assert_eq!(entries(decoded.document()), vec![expected]);
    }
}

#[test]
fn p1_through_p6_follow_the_fixed_priority() {
    let decoded = decode(
        concat!(
            "aa=2,甲\n",
            "bb,3=#乙\n",
            "cc=丙\n",
            "dd 丁 4\n",
            "ee 戊\n",
            "己 ff\n",
        )
        .as_bytes(),
    );

    assert_eq!(
        entries(decoded.document()),
        vec![
            ("aa", "甲", 2),
            ("bb", "乙", 3),
            ("cc", "丙", 1),
            ("dd", "丁", 4),
            ("ee", "戊", 1),
            ("ff", "己", 1),
        ]
    );
}

#[test]
fn comments_and_multiline_keep_original_locations_and_state_boundaries() {
    let input = concat!(
        "/* opening\n",
        "ignored */ aa=甲\n",
        "bb=\n",
        " 第一行 \n",
        "\n",
        "第二行\n",
        "cc=丙\n",
        "??? ???\n",
    );
    let decoded = decode(input.as_bytes());

    assert_eq!(
        entries(decoded.document()),
        vec![
            ("aa", "甲", 1),
            ("bb", "第一行\n第二行", 1),
            ("cc", "丙", 1)
        ]
    );
    assert_eq!(decoded.warnings().len(), 1);
    assert_eq!(
        decoded.warnings()[0].kind(),
        PhraseTextWarningKind::UnrecognizedLine
    );
    assert_eq!(decoded.warnings()[0].location(), text_location(8, 1));
}

#[test]
fn arrays_use_unicode_scalars_or_ascii_space_tokens_and_reset_to_candidate_one() {
    let decoded = decode(
        concat!(
            "aa=9,existing\n",
            "aa=$[甲乙🤝]\n",
            "aa=after\n",
            "bb=$[乾☰  兑☱   离☲]\n",
            "cc=4,$[甲乙]\n",
        )
        .as_bytes(),
    );

    assert_eq!(
        entries(decoded.document()),
        vec![
            ("aa", "existing", 9),
            ("aa", "甲", 1),
            ("aa", "乙", 2),
            ("aa", "🤝", 3),
            ("aa", "after", 10),
            ("bb", "乾☰", 1),
            ("bb", "兑☱", 2),
            ("bb", "离☲", 3),
            ("cc", "$[甲乙]", 4),
        ]
    );
}

#[test]
fn p2_rewrites_only_known_time_aliases_and_unescapes_whitespace() {
    let decoded = decode(
        concat!(
            "aa,1=$year-$year_yy-$month_mm-$month-$day-$day_dd\n",
            "bb,1=$fullhour:$minute:$second:$unknown%20literal%09tab\n",
        )
        .as_bytes(),
    );

    assert_eq!(
        entries(decoded.document()),
        vec![
            ("aa", "%yyyy%-%yy%-%MM%-%M%-%dd%-%dd%", 1),
            ("bb", "%HH%:%mm%:%ss%:$unknown literal\ttab", 1),
        ]
    );
}

#[test]
fn recognized_invalid_fields_and_unclosed_comments_are_strict() {
    let cases = [
        ("aa=0,text\n", "phrase candidate", 1, 4),
        ("aa=+1,text\n", "phrase candidate", 1, 4),
        ("aa,+1=text\n", "phrase candidate", 1, 4),
        ("aa text +1\n", "phrase candidate", 1, 9),
        ("aa=256,text\n", "phrase candidate", 1, 4),
        ("aa=$[]\n", "phrase text", 1, 4),
        ("Aa=1,text\n", "phrase code", 1, 1),
        ("aa=1,\n", "phrase text", 1, 6),
        ("aa=\n", "phrase text", 1, 4),
    ];

    for (input, field, line, column) in cases {
        let error = phrase_text::decode(input.as_bytes(), DecodeLimits::default())
            .expect_err("recognized invalid input must fail");
        assert!(matches!(
            error.kind(),
            CodecErrorKind::InvalidInput { field: actual, .. }
                | CodecErrorKind::MalformedField { field: actual, .. }
                if actual == &field
        ));
        assert_eq!(error.location(), Some(text_location(line, column)));
    }

    let error = phrase_text::decode(b"aa=ok\n/* missing", DecodeLimits::default())
        .expect_err("an unclosed comment must fail");
    assert!(matches!(
        error.kind(),
        CodecErrorKind::MalformedField {
            field: "phrase_text.comment",
            ..
        }
    ));
    assert_eq!(error.location(), Some(text_location(2, 1)));

    let pending_first = phrase_text::decode(b"aa=\nbb=0,text\n", DecodeLimits::default())
        .expect_err("an empty pending phrase must fail before the next invalid record");
    assert!(matches!(
        pending_first.kind(),
        CodecErrorKind::InvalidInput {
            field: "phrase text",
            reason: InvalidInputReason::Empty,
        }
    ));
    assert_eq!(pending_first.location(), Some(text_location(1, 4)));
}

#[test]
fn warnings_are_bounded_ordered_and_share_the_entry_budget() {
    let long = "界".repeat(161);
    let input = format!("??? first\n{long}\naa=valid\n");
    let decoded = decode(input.as_bytes());
    assert_eq!(decoded.warnings().len(), 2);
    assert_eq!(decoded.warnings()[0].preview(), "??? first");
    assert_eq!(decoded.warnings()[1].preview().chars().count(), 160);
    assert!(decoded.warnings()[1].is_truncated());

    let error = phrase_text::decode(input.as_bytes(), DecodeLimits::new(input.len(), 2))
        .expect_err("two warnings and an entry must exceed a two-item budget");
    assert_eq!(
        error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 2,
            actual: 3,
        }
    );
    assert_eq!(error.location(), Some(text_location(3, 1)));

    let warning_only = phrase_text::decode(b"??? ???\n", DecodeLimits::default())
        .expect_err("a warning-only body must not become empty success");
    assert!(matches!(
        warning_only.kind(),
        CodecErrorKind::MalformedField {
            field: "phrase_text.body",
            ..
        }
    ));
}

#[test]
fn arrays_and_auto_candidates_enforce_the_complete_u8_boundary() {
    let array_255 = format!("aa=$[{}]\n", "甲".repeat(255));
    let decoded = decode(array_255.as_bytes());
    assert_eq!(decoded.document().len(), 255);
    assert_eq!(decoded.document().entries()[254].candidate().get(), 255);

    let array_256 = format!("aa=$[{}]\n", "甲".repeat(256));
    let error = phrase_text::decode(array_256.as_bytes(), DecodeLimits::default())
        .expect_err("the 256th array candidate must fail");
    assert!(matches!(
        error.kind(),
        CodecErrorKind::IntegerOverflow { .. }
    ));

    let error = phrase_text::decode(b"aa=255,last\naa=overflow\n", DecodeLimits::default())
        .expect_err("automatic candidate 256 must fail");
    assert!(matches!(
        error.kind(),
        CodecErrorKind::IntegerOverflow { .. }
    ));
    assert_eq!(error.location(), Some(text_location(2, 1)));

    let budget_error = phrase_text::decode("aa=$[甲乙丙]\n".as_bytes(), DecodeLimits::new(100, 2))
        .expect_err("the third array entry must exceed a two-entry budget");
    assert_eq!(
        budget_error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::ExpandedEntries,
            limit: 2,
            actual: 3,
        }
    );
    assert_eq!(budget_error.location(), Some(text_location(1, 1)));
}

#[test]
fn p6_requires_an_exact_lowercase_one_to_four_letter_code() {
    let decoded = decode("??? a1\n??? abcde\n有效 cc\n尾序 dd 1 2\n紧随 ee12\n".as_bytes());
    assert_eq!(
        entries(decoded.document()),
        vec![
            ("a", "???", 1),
            ("cc", "有效", 1),
            ("dd", "尾序", 1),
            ("ee", "紧随", 1),
        ]
    );
    assert_eq!(decoded.warnings().len(), 1);
    assert_eq!(decoded.warnings()[0].preview(), "??? abcde");
}

#[test]
fn canonical_format_is_stable_compresses_only_eligible_groups_and_round_trips() {
    let original = PhraseDocument::new(vec![
        phrase_entry("bb", "single", 1),
        phrase_entry("aa", "乙", 2),
        phrase_entry("aa", "甲", 1),
        phrase_entry("cc", "🤝", 1),
        phrase_entry("cc", "双字", 2),
        phrase_entry("dd", "gap", 2),
        phrase_entry("ee", "duplicate-a", 1),
        phrase_entry("ee", "duplicate-b", 1),
        phrase_entry("ff", "A🤝", 1),
        phrase_entry("ff", "乙", 2),
        phrase_entry("gg", "line\nspace ", 1),
        phrase_entry("hh", "high", 255),
    ]);

    let formatted = phrase_text::format(&original).expect("a valid document must format");
    assert_eq!(
        formatted,
        concat!(
            "aa\t$[甲 乙]\r\n",
            "bb\tsingle\t1\r\n",
            "cc\t$[🤝 双字]\r\n",
            "dd\tgap\t2\r\n",
            "ee\tduplicate-a\t1\r\n",
            "ee\tduplicate-b\t1\r\n",
            "ff\tA🤝\t1\r\n",
            "ff\t乙\t2\r\n",
            "gg\tline%0Aspace%20\t1\r\n",
            "hh\thigh\t255\r\n",
        )
    );
    assert_eq!(
        phrase_text::format(&original),
        phrase_text::format(&original)
    );
    assert_eq!(original.entries()[0].code().as_str(), "bb");
    let decoded = decode(formatted.as_bytes());
    assert_eq!(decoded.document().len(), original.len());
    assert_eq!(
        phrase_text::format(decoded.document()),
        Ok(formatted.clone())
    );
}

#[test]
fn phrase_input_limit_is_checked_at_the_exact_byte_boundary() {
    let input = b"aa=valid\n";
    assert!(phrase_text::decode(input, DecodeLimits::new(input.len(), 1)).is_ok());

    let error = phrase_text::decode(input, DecodeLimits::new(input.len() - 1, 1))
        .expect_err("input one byte over the limit must fail");
    assert_eq!(
        error.kind(),
        &CodecErrorKind::ResourceLimitExceeded {
            resource: ResourceKind::InputBytes,
            limit: input.len() - 1,
            actual: input.len(),
        }
    );
    assert_eq!(error.location(), None);
}

#[test]
fn empty_comment_only_and_truncated_prefixes_never_panic() {
    for input in [b"".as_slice(), b"/* comment */\n".as_slice()] {
        let decoded = decode(input);
        assert!(decoded.document().is_empty());
        assert!(decoded.warnings().is_empty());
    }

    let utf16 = [0xFF, 0xFE, 0x61, 0x00, 0x61, 0x00, 0x3D, 0x00, 0x32, 0x75];
    for end in 0..utf16.len() {
        let _ = phrase_text::decode(&utf16[..end], DecodeLimits::default());
    }
}

fn decode(input: &[u8]) -> wubilex_codec::DecodedPhraseText {
    phrase_text::decode(input, DecodeLimits::default()).expect("test phrase text must decode")
}

fn entries(document: &PhraseDocument) -> Vec<(&str, &str, u8)> {
    document
        .entries()
        .iter()
        .map(|entry| (entry.code().as_str(), entry.text(), entry.candidate().get()))
        .collect()
}

fn phrase_entry(code: &str, text: &str, candidate: u8) -> PhraseEntry {
    PhraseEntry::new(
        PhraseCode::new(code).expect("test code must be valid"),
        text,
        Candidate::new(candidate).expect("test candidate must be valid"),
    )
    .expect("test phrase entry must be valid")
}

fn text_location(line: usize, column: usize) -> SourceLocation {
    SourceLocation::Text {
        line: NonZeroUsize::new(line).expect("test line must be nonzero"),
        column: Some(NonZeroUsize::new(column).expect("test column must be nonzero")),
    }
}
