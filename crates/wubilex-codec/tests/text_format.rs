use wubilex_codec::{
    CodecErrorKind, LexCode, LexiconDocument, LexiconEntry, LexiconTextFormat, Weight, text,
};

#[test]
fn all_seven_formats_use_one_canonical_stable_projection() {
    let document = document(&[
        ("b", "B", None),
        ("a", "X", Some(2)),
        ("a", "Y", None),
        ("a", "X", Some(2)),
    ]);
    let cases = [
        (
            LexiconTextFormat::CodeThenText,
            "a\tX\r\na\tX\r\na\tY\r\nb\tB\r\n",
        ),
        (LexiconTextFormat::CodeThenTexts, "a\tX\tY\r\nb\tB\r\n"),
        (
            LexiconTextFormat::CodeThenTextWeight,
            "a\tX\t2\r\na\tX\t2\r\na\tY\t3\r\nb\tB\t1\r\n",
        ),
        (
            LexiconTextFormat::TextThenCode,
            "X\ta\r\nX\ta\r\nY\ta\r\nB\tb\r\n",
        ),
        (LexiconTextFormat::TextThenCodes, "X\ta\r\nY\ta\r\nB\tb\r\n"),
        (
            LexiconTextFormat::TextThenCodeDescendingWeight,
            "X\ta\t65533\r\nX\ta\t65533\r\nY\ta\t65532\r\nB\tb\t65534\r\n",
        ),
        (
            LexiconTextFormat::PhraseAscendingCandidate,
            "a=1,X\r\na=2,X\r\na=3,Y\r\nb=1,B\r\n",
        ),
    ];

    for (format, expected) in cases {
        assert_eq!(text::format(&document, format).as_deref(), Ok(expected));
    }

    assert_eq!(
        document.entries()[0].code().as_str(),
        "b",
        "formatting must not mutate source order"
    );
}

#[test]
fn code_aggregation_folds_only_adjacent_equal_texts() {
    let document = document(&[
        ("a", "X", Some(1)),
        ("a", "Y", Some(1)),
        ("a", "X", Some(1)),
    ]);

    assert_eq!(
        text::format(&document, LexiconTextFormat::CodeThenTexts).as_deref(),
        Ok("a\tX\tY\tX\r\n")
    );
}

#[test]
fn every_output_text_uses_symmetric_whitespace_escaping() {
    let document = document(&[("a", " \t\n\r\u{000b}\u{000c}", None)]);

    assert_eq!(
        text::format(&document, LexiconTextFormat::CodeThenText).as_deref(),
        Ok("a\t%20%09%0A%0D%0B%0C\r\n")
    );
}

#[test]
fn omitted_weight_after_maximum_is_a_structured_overflow() {
    let document = document(&[("a", "max", Some(u16::MAX)), ("a", "next", None)]);

    let error = text::format(&document, LexiconTextFormat::CodeThenText)
        .expect_err("an omitted weight after 65535 cannot be represented");
    assert_eq!(
        error.kind(),
        &CodecErrorKind::IntegerOverflow {
            operation: "lexicon effective weight increment",
        }
    );
    assert_eq!(error.location(), None);
}

#[test]
fn phrase_candidate_uses_the_full_nonzero_u16_range() {
    let entries = (1..=256)
        .map(|index| ("a", format!("word{index}"), Some(1)))
        .map(|(code, value, weight)| entry(code, &value, weight))
        .collect();
    let document = LexiconDocument::new(entries);

    let formatted = text::format(&document, LexiconTextFormat::PhraseAscendingCandidate)
        .expect("the 256th candidate is a valid lexicon weight");
    assert!(formatted.ends_with("a=256,word256\r\n"));

    let entries = (0..=u16::MAX)
        .map(|_| entry("a", "word", Some(1)))
        .collect();
    let document = LexiconDocument::new(entries);

    let error = text::format(&document, LexiconTextFormat::PhraseAscendingCandidate)
        .expect_err("the 65536th candidate cannot round-trip through a u16 weight");
    assert_eq!(
        error.kind(),
        &CodecErrorKind::IntegerOverflow {
            operation: "phrase candidate index",
        }
    );
}

#[test]
fn utf16le_output_has_a_bom_and_exact_little_endian_units() {
    let empty = text::encode_utf16le(&LexiconDocument::default(), LexiconTextFormat::CodeThenText)
        .expect("empty output must encode");
    assert_eq!(empty, vec![0xFF, 0xFE]);

    let document = document(&[("a", "甲", None)]);
    let encoded = text::encode_utf16le(&document, LexiconTextFormat::CodeThenText)
        .expect("formatted text must encode");
    assert_eq!(
        encoded,
        vec![
            0xFF, 0xFE, 0x61, 0x00, 0x09, 0x00, 0x32, 0x75, 0x0D, 0x00, 0x0A, 0x00,
        ]
    );
}

#[test]
fn text_aggregation_groups_codes_in_canonical_first_encounter_order() {
    let document = document(&[
        ("b", "shared", None),
        ("a", "other", None),
        ("a", "shared", Some(2)),
        ("b", "shared", Some(1)),
    ]);

    assert_eq!(
        text::format(&document, LexiconTextFormat::TextThenCodes).as_deref(),
        Ok("other\ta\r\nshared\ta b\r\n")
    );
}

#[test]
fn descending_output_allows_zero_for_the_maximum_effective_weight() {
    let document = document(&[("a", "max", Some(u16::MAX))]);

    assert_eq!(
        text::format(&document, LexiconTextFormat::TextThenCodeDescendingWeight).as_deref(),
        Ok("max\ta\t0\r\n")
    );
}

#[test]
fn formatted_whitespace_round_trips_through_the_decoder() {
    let original = "space tab\tline\nreturn\rvertical\u{000b}form\u{000c}";
    let document = document(&[("a", original, None)]);
    let formatted =
        text::format(&document, LexiconTextFormat::CodeThenText).expect("document must format");
    let decoded = text::decode(formatted.as_bytes(), wubilex_codec::DecodeLimits::default())
        .expect("formatted document must decode");

    assert_eq!(decoded.document(), &document);
}

#[test]
fn utf16le_output_encodes_non_bmp_text_as_a_surrogate_pair() {
    let document = document(&[("a", "🤝", None)]);
    let encoded = text::encode_utf16le(&document, LexiconTextFormat::CodeThenText)
        .expect("emoji output must encode");

    assert_eq!(
        encoded,
        vec![
            0xFF, 0xFE, 0x61, 0x00, 0x09, 0x00, 0x3E, 0xD8, 0x1D, 0xDD, 0x0D, 0x00, 0x0A, 0x00,
        ]
    );
}

fn document(values: &[(&str, &str, Option<u16>)]) -> LexiconDocument {
    LexiconDocument::new(
        values
            .iter()
            .map(|(code, value, weight)| entry(code, value, *weight))
            .collect(),
    )
}

fn entry(code: &str, value: &str, weight: Option<u16>) -> LexiconEntry {
    LexiconEntry::new(
        LexCode::new(code).expect("test code must be valid"),
        value,
        weight.map(|value| Weight::new(value).expect("test weight must be valid")),
    )
    .expect("test entry must be valid")
}
