use std::panic::{AssertUnwindSafe, catch_unwind};

use proptest::{collection::vec, prelude::*, test_runner::Config};
use wubilex_codec::{
    Candidate, DecodeLimits, LexCode, LexiconDocument, LexiconEntry, LexiconTextFormat, PhraseCode,
    PhraseDocument, PhraseEntry, Weight, escape, eudp, lex, text,
};

const TIMESTAMP: i32 = 1_700_000_123;

proptest! {
    #![proptest_config(Config::with_cases(64))]

    #[test]
    fn lex_round_trip_preserves_the_canonical_duplicate_projection(
        document in lexicon_document_strategy()
    ) {
        let bytes = lex::encode(&document).expect("generated lexicon must encode");
        let decoded = lex::decode(&bytes, DecodeLimits::default())
            .expect("encoded lexicon must decode");
        let mut expected = document.entries().to_vec();
        expected.sort_by(|left, right| left.code().as_str().cmp(right.code().as_str()));

        prop_assert_eq!(&decoded, &LexiconDocument::new(expected));
        prop_assert_eq!(lex::encode(&decoded), Ok(bytes));
    }

    #[test]
    fn eudp_round_trip_preserves_codes_candidates_duplicates_and_non_bmp_text(
        document in phrase_document_strategy()
    ) {
        let bytes = eudp::encode(&document, TIMESTAMP).expect("generated phrases must encode");
        let decoded = eudp::decode(&bytes, DecodeLimits::default())
            .expect("encoded phrases must decode");
        let mut expected = document.entries().to_vec();
        expected.sort_by(|left, right| left.code().as_str().cmp(right.code().as_str()));

        prop_assert_eq!(&decoded, &PhraseDocument::new(expected));
        prop_assert_eq!(eudp::encode(&decoded, TIMESTAMP), Ok(bytes));
    }

    #[test]
    fn six_ascii_whitespace_escapes_round_trip(
        value in whitespace_text_strategy()
    ) {
        let escaped = escape::escape_whitespace(&value);
        prop_assert_eq!(escape::unescape_whitespace(&escaped), value);
    }

    #[test]
    fn weighted_lexicon_text_round_trip_preserves_its_canonical_projection(
        document in text_lexicon_document_strategy()
    ) {
        let formatted = text::format(&document, LexiconTextFormat::CodeThenTextWeight)
            .expect("generated lexicon text must format");
        let decoded = text::decode(formatted.as_bytes(), DecodeLimits::default())
            .expect("formatted lexicon text must decode");
        let mut expected = document.entries().to_vec();
        expected.sort_by(|left, right| {
            left.code()
                .as_str()
                .cmp(right.code().as_str())
                .then_with(|| {
                    left.weight()
                        .map(Weight::get)
                        .cmp(&right.weight().map(Weight::get))
                })
        });

        prop_assert!(decoded.warnings().is_empty());
        prop_assert_eq!(decoded.document(), &LexiconDocument::new(expected));
        prop_assert_eq!(
            text::format(decoded.document(), LexiconTextFormat::CodeThenTextWeight),
            Ok(formatted)
        );
    }

    #[test]
    fn bounded_arbitrary_bytes_never_panic_binary_decoders(
        bytes in vec(any::<u8>(), 0..=2048)
    ) {
        let limits = DecodeLimits::new(2048, 128);
        let lex_result = catch_unwind(AssertUnwindSafe(|| lex::decode(&bytes, limits)));
        let eudp_result = catch_unwind(AssertUnwindSafe(|| eudp::decode(&bytes, limits)));

        prop_assert!(lex_result.is_ok());
        prop_assert!(eudp_result.is_ok());
        if let Ok(Ok(document)) = lex_result {
            prop_assert!(lex::encode(&document).is_ok());
        }
        if let Ok(Ok(document)) = eudp_result {
            prop_assert!(eudp::encode(&document, TIMESTAMP).is_ok());
        }
    }
}

#[test]
fn unknown_incomplete_and_lowercase_percent_sequences_remain_literal() {
    let input = "%25%2F%0b%0c%2%tail";
    assert_eq!(escape::unescape_whitespace(input), input);
}

#[test]
fn systematic_binary_mutations_return_without_panicking() {
    let lex_bytes = lex::encode(&LexiconDocument::new(vec![lexicon_entry("a", "甲🤝", 1)]))
        .expect("test lexicon must encode");
    let eudp_bytes = eudp::encode(
        &PhraseDocument::new(vec![phrase_entry("aa", "甲🤝\n", 1)]),
        TIMESTAMP,
    )
    .expect("test phrases must encode");

    for bytes in [&lex_bytes, &eudp_bytes] {
        for index in (0..bytes.len()).step_by((bytes.len() / 32).max(1)) {
            let mut mutated = bytes.clone();
            mutated[index] ^= 0xFF;
            let result = catch_unwind(AssertUnwindSafe(|| {
                let limits = DecodeLimits::new(mutated.len(), 128);
                let _ = lex::decode(&mutated, limits);
                let _ = eudp::decode(&mutated, limits);
            }));
            assert!(result.is_ok(), "mutation at byte {index} must not panic");
        }
    }
}

fn lexicon_document_strategy() -> impl Strategy<Value = LexiconDocument> {
    vec(
        (lex_code_strategy(), model_text_strategy(), 1u16..=u16::MAX),
        1..=10,
    )
    .prop_map(|values| {
        let mut entries = values
            .into_iter()
            .map(|(code, value, weight)| lexicon_entry(&code, &value, weight))
            .collect::<Vec<_>>();
        entries.push(entries[0].clone());
        LexiconDocument::new(entries)
    })
}

fn phrase_document_strategy() -> impl Strategy<Value = PhraseDocument> {
    vec(
        (phrase_code_strategy(), model_text_strategy(), 1u8..=u8::MAX),
        1..=10,
    )
    .prop_map(|values| {
        let mut entries = values
            .into_iter()
            .map(|(code, value, candidate)| phrase_entry(&code, &value, candidate))
            .collect::<Vec<_>>();
        entries.push(entries[0].clone());
        PhraseDocument::new(entries)
    })
}

fn text_lexicon_document_strategy() -> impl Strategy<Value = LexiconDocument> {
    vec(
        (lex_code_strategy(), text_field_strategy(), 1u16..=u16::MAX),
        1..=10,
    )
    .prop_map(|values| {
        let mut entries = values
            .into_iter()
            .map(|(code, value, weight)| lexicon_entry(&code, &value, weight))
            .collect::<Vec<_>>();
        entries.push(entries[0].clone());
        LexiconDocument::new(entries)
    })
}

fn lex_code_strategy() -> impl Strategy<Value = String> {
    vec(b'a'..=b'z', 1..=4).prop_map(|bytes| bytes.into_iter().map(char::from).collect())
}

fn phrase_code_strategy() -> impl Strategy<Value = String> {
    vec(b'a'..=b'z', 1..=8).prop_map(|bytes| bytes.into_iter().map(char::from).collect())
}

fn model_text_strategy() -> impl Strategy<Value = String> {
    vec(
        prop_oneof![
            6 => prop::sample::select(vec!['甲', '乙', '中', '🤝', '\n', '\t', '\u{e000}']),
            1 => any::<char>().prop_filter("text excludes embedded NUL", |character| *character != '\0'),
        ],
        1..=16,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

fn whitespace_text_strategy() -> impl Strategy<Value = String> {
    vec(
        prop_oneof![
            Just(' '),
            Just('\t'),
            Just('\n'),
            Just('\r'),
            Just('\u{000b}'),
            Just('\u{000c}'),
            any::<char>().prop_filter("ordinary text excludes percent and NUL", |character| {
                *character != '%' && *character != '\0' && !character.is_whitespace()
            }),
        ],
        0..=64,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

fn text_field_strategy() -> impl Strategy<Value = String> {
    vec(
        prop_oneof![
            Just(' '),
            Just('\t'),
            Just('\n'),
            Just('\r'),
            Just('\u{000b}'),
            Just('\u{000c}'),
            any::<char>().prop_filter(
                "text fields exclude ambiguous percent, NUL, and unsupported whitespace",
                |character| {
                    *character != '%' && *character != '\0' && !character.is_whitespace()
                }
            ),
        ],
        1..=16,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

fn lexicon_entry(code: &str, value: &str, weight: u16) -> LexiconEntry {
    LexiconEntry::new(
        LexCode::new(code).expect("generated lexicon code must be valid"),
        value,
        Some(Weight::new(weight).expect("generated weight must be nonzero")),
    )
    .expect("generated lexicon text must be nonempty")
}

fn phrase_entry(code: &str, value: &str, candidate: u8) -> PhraseEntry {
    PhraseEntry::new(
        PhraseCode::new(code).expect("generated phrase code must be valid"),
        value,
        Candidate::new(candidate).expect("generated candidate must be nonzero"),
    )
    .expect("generated phrase text must be nonempty")
}
