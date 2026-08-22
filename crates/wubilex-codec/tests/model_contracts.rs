use wubilex_codec::{
    Candidate, CodecErrorKind, DetectedTextEncoding, InvalidInputReason, LexCode, LexScheme,
    LexiconDocument, LexiconEntry, PhraseCode, PhraseDocument, PhraseEntry, TextEncoding, Weight,
};

#[test]
fn lex_code_accepts_one_and_four_lowercase_ascii_letters() {
    assert_eq!(lex_code("a").as_str(), "a");
    assert_eq!(lex_code("xfxy").as_str(), "xfxy");
}

#[test]
fn lex_code_rejects_every_invalid_boundary_without_truncation() {
    assert_invalid_input("lexicon code", LexCode::new(""), InvalidInputReason::Empty);
    assert_invalid_input(
        "lexicon code",
        LexCode::new("Abcd"),
        InvalidInputReason::NotLowercaseAscii {
            index: 0,
            character: 'A',
        },
    );
    assert_invalid_input(
        "lexicon code",
        LexCode::new("五笔"),
        InvalidInputReason::NotLowercaseAscii {
            index: 0,
            character: '五',
        },
    );
    assert_invalid_input(
        "lexicon code",
        LexCode::new("abcde"),
        InvalidInputReason::TooLong { max: 4, actual: 5 },
    );
}

#[test]
fn weight_preserves_optional_and_full_nonzero_u16_range() {
    let omitted = lexicon_entry("a", "甲", None);
    let minimum = lexicon_entry("a", "乙", Some(weight(1)));
    let maximum = lexicon_entry("a", "丙", Some(weight(u16::MAX)));

    assert_eq!(omitted.weight(), None);
    assert_eq!(minimum.weight().map(Weight::get), Some(1));
    assert_eq!(maximum.weight().map(Weight::get), Some(u16::MAX));
    assert_invalid_input("lexicon weight", Weight::new(0), InvalidInputReason::Zero);
}

#[test]
fn lexicon_document_preserves_source_order_and_duplicates() {
    let first = lexicon_entry("ab", "重复", Some(weight(1)));
    let duplicate = first.clone();
    let last = lexicon_entry("aa", "末项", None);
    let document = LexiconDocument::new(vec![first.clone(), duplicate, last.clone()]);

    assert_eq!(document.len(), 3);
    assert_eq!(document.entries(), &[first.clone(), first, last]);
}

#[test]
fn lexicon_entry_rejects_empty_text_and_empty_document_is_valid() {
    let error = LexiconEntry::new(lex_code("a"), "", None);

    assert_invalid_input("lexicon text", error, InvalidInputReason::Empty);
    assert!(LexiconDocument::default().is_empty());
    assert!(LexiconDocument::new(Vec::new()).into_entries().is_empty());
}

#[test]
fn phrase_code_has_no_lex_specific_four_letter_limit() {
    let code = phrase_code("abcdefgh");

    assert_eq!(code.as_str(), "abcdefgh");
    assert_invalid_input(
        "phrase code",
        PhraseCode::new(""),
        InvalidInputReason::Empty,
    );
    assert_invalid_input(
        "phrase code",
        PhraseCode::new("abc1"),
        InvalidInputReason::NotLowercaseAscii {
            index: 3,
            character: '1',
        },
    );
}

#[test]
fn candidate_covers_the_complete_nonzero_u8_range() {
    assert_eq!(candidate(1).get(), 1);
    assert_eq!(candidate(u8::MAX).get(), u8::MAX);
    assert_invalid_input(
        "phrase candidate",
        Candidate::new(0),
        InvalidInputReason::Zero,
    );
}

#[test]
fn phrase_utf16_length_is_derived_for_bmp_and_emoji_text() {
    let bmp = phrase_entry("z", "符号", 1);
    let emoji = phrase_entry("z", "🤝", 2);
    let mixed = phrase_entry("z", "A🤝中", 3);

    assert_eq!(bmp.utf16_len(), 2);
    assert_eq!(emoji.utf16_len(), 2);
    assert_eq!(mixed.utf16_len(), 4);
}

#[test]
fn phrase_document_preserves_order_duplicates_and_candidate_positions() {
    let first = phrase_entry("zz", "短语", 2);
    let duplicate = first.clone();
    let document = PhraseDocument::new(vec![first.clone(), duplicate]);

    assert_eq!(document.len(), 2);
    assert_eq!(document.entries(), &[first.clone(), first]);
    assert_eq!(document.entries()[0].candidate().get(), 2);
}

#[test]
fn phrase_entry_rejects_empty_text_and_empty_document_is_valid() {
    let error = PhraseEntry::new(phrase_code("z"), "", candidate(1));

    assert_invalid_input("phrase text", error, InvalidInputReason::Empty);
    assert!(PhraseDocument::default().is_empty());
    assert!(PhraseDocument::new(Vec::new()).into_entries().is_empty());
}

#[test]
fn scheme_contract_covers_eight_schemes_and_scopes_formation_to_zhengma() {
    let schemes = [
        LexScheme::Wubi86,
        LexScheme::Wubi98,
        LexScheme::Wubi06,
        LexScheme::Wubi091,
        LexScheme::Wubi092,
        LexScheme::Zhengma { formation: false },
        LexScheme::XiaoheSoundShape,
        LexScheme::Biaoxingma,
    ];
    let identifiers = schemes.map(scheme_identifier);

    assert_eq!(
        identifiers,
        ["86", "98", "06", "091", "092", "zhengma", "xhyx", "bxm"]
    );
    assert_eq!(
        scheme_identifier(LexScheme::Zhengma { formation: true }),
        "zhengma-formation"
    );
}

#[test]
fn text_detection_contract_separates_encoding_from_bom_presence() {
    let encodings = [
        TextEncoding::Utf8,
        TextEncoding::Utf16Le,
        TextEncoding::Utf16Be,
        TextEncoding::Gbk,
    ];

    for encoding in encodings {
        let without_bom = DetectedTextEncoding::new(encoding, false);
        let with_bom = DetectedTextEncoding::new(encoding, true);

        assert_eq!(without_bom.encoding(), encoding);
        assert!(!without_bom.has_bom());
        assert_eq!(with_bom.encoding(), encoding);
        assert!(with_bom.has_bom());
    }
}

fn lex_code(value: &str) -> LexCode {
    LexCode::new(value).expect("test lexicon code must be valid")
}

fn phrase_code(value: &str) -> PhraseCode {
    PhraseCode::new(value).expect("test phrase code must be valid")
}

fn weight(value: u16) -> Weight {
    Weight::new(value).expect("test weight must be valid")
}

fn candidate(value: u8) -> Candidate {
    Candidate::new(value).expect("test candidate must be valid")
}

fn lexicon_entry(code: &str, text: &str, weight: Option<Weight>) -> LexiconEntry {
    LexiconEntry::new(lex_code(code), text, weight).expect("test lexicon entry must be valid")
}

fn phrase_entry(code: &str, text: &str, candidate_value: u8) -> PhraseEntry {
    PhraseEntry::new(phrase_code(code), text, candidate(candidate_value))
        .expect("test phrase entry must be valid")
}

fn scheme_identifier(scheme: LexScheme) -> &'static str {
    match scheme {
        LexScheme::Wubi86 => "86",
        LexScheme::Wubi98 => "98",
        LexScheme::Wubi06 => "06",
        LexScheme::Wubi091 => "091",
        LexScheme::Wubi092 => "092",
        LexScheme::Zhengma { formation: false } => "zhengma",
        LexScheme::Zhengma { formation: true } => "zhengma-formation",
        LexScheme::XiaoheSoundShape => "xhyx",
        LexScheme::Biaoxingma => "bxm",
    }
}

fn assert_invalid_input<T: std::fmt::Debug>(
    field: &'static str,
    result: Result<T, wubilex_codec::CodecError>,
    reason: InvalidInputReason,
) {
    let error = result.expect_err("value must fail validation");

    assert!(matches!(
        error.kind(),
        CodecErrorKind::InvalidInput {
            field: actual_field,
            reason: actual_reason,
        } if *actual_field == field && *actual_reason == reason
    ));
    assert_eq!(error.location(), None);
}
