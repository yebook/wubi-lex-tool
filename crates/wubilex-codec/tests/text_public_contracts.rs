use wubilex_codec::{
    DecodeLimits, LexiconDocument, LexiconTextFormat, LexiconTextWarningKind, TextEncoding,
    escape::{escape_whitespace, unescape_whitespace},
    text,
};

#[test]
fn whitespace_escape_is_symmetric_for_all_six_ascii_characters() {
    let text = "space tab\tline\nreturn\rvertical\u{000b}form\u{000c}";
    let escaped = "space%20tab%09line%0Areturn%0Dvertical%0Bform%0C";

    assert_eq!(escape_whitespace(text), escaped);
    assert_eq!(unescape_whitespace(escaped), text);
}

#[test]
fn whitespace_unescape_leaves_unknown_and_lowercase_sequences_literal() {
    let encoded = "%20%0B%0C%0b%0c%25%2F%";

    assert_eq!(
        unescape_whitespace(encoded),
        " \u{000b}\u{000c}%0b%0c%25%2F%"
    );
}

#[test]
fn public_text_decode_result_exposes_encoding_document_and_warnings() {
    let decoded = text::decode(b"abcd\tword\r\n", DecodeLimits::default())
        .expect("valid UTF-8 text must decode");

    assert_eq!(decoded.detected_encoding().encoding(), TextEncoding::Utf8);
    assert!(!decoded.detected_encoding().has_bom());
    assert_eq!(decoded.document().len(), 1);
    assert!(decoded.warnings().is_empty());
}

#[test]
fn seven_formats_are_explicit_public_variants() {
    let formats = [
        LexiconTextFormat::CodeThenText,
        LexiconTextFormat::CodeThenTexts,
        LexiconTextFormat::CodeThenTextWeight,
        LexiconTextFormat::TextThenCode,
        LexiconTextFormat::TextThenCodes,
        LexiconTextFormat::TextThenCodeDescendingWeight,
        LexiconTextFormat::PhraseAscendingCandidate,
    ];

    for format in formats {
        assert_eq!(
            text::format(&LexiconDocument::default(), format),
            Ok(String::new())
        );
    }
}

#[test]
fn warning_kind_is_structured() {
    let decoded = text::decode("??? ???\nabcd word\n".as_bytes(), DecodeLimits::default())
        .expect("an unknown line must not hide a later valid entry");

    assert_eq!(decoded.warnings().len(), 1);
    assert_eq!(
        decoded.warnings()[0].kind(),
        LexiconTextWarningKind::UnrecognizedLine
    );
}
