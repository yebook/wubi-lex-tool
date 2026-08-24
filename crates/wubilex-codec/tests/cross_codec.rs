mod support;

use std::{collections::HashSet, fs};

use support::{PREPARATION_HINT, fixture_directory, load_fixture_manifest};
use wubilex_codec::{
    DecodeLimits, LexCode, LexiconDocument, LexiconEntry, LexiconTextFormat, Weight, eudp, lex,
    phrase_text, text,
};

const TIMESTAMP: i32 = 1_700_000_321;
#[test]
fn all_seven_text_layouts_match_independently_authored_complete_strings() {
    let document = LexiconDocument::new(vec![
        lexicon_entry("b", "B", 1),
        lexicon_entry("a", "X", 2),
        lexicon_entry("a", "Y", 3),
        lexicon_entry("a", "X", 2),
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
}

#[test]
fn representative_real_lexicon_has_deterministic_seven_format_semantic_projections() {
    let fixture_directory = fixture_directory();
    let manifest = load_fixture_manifest(&fixture_directory);
    let fixture = manifest
        .fixtures
        .iter()
        .find(|fixture| fixture.scheme == "wubi86")
        .expect("fixture manifest must contain the Wubi 86 representative");
    let fixture_path = fixture_directory.join(&fixture.lex_file);
    let bytes = fs::read(&fixture_path).unwrap_or_else(|error| {
        panic!(
            "real fixture is unavailable at {}: {error}; {PREPARATION_HINT}",
            fixture_path.display()
        )
    });
    let document = lex::decode(&bytes, DecodeLimits::default())
        .unwrap_or_else(|error| panic!("real Wubi 86 fixture must decode: {error}"));
    let expected_pairs = semantic_pairs(&document);
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
        let first = text::format(&document, format)
            .unwrap_or_else(|error| panic!("real fixture must format as {format:?}: {error}"));
        let second = text::format(&document, format)
            .unwrap_or_else(|error| panic!("real fixture must repeat as {format:?}: {error}"));
        assert_eq!(first, second, "{format:?} output must be deterministic");
        assert!(first.ends_with("\r\n"), "{format:?} must use CRLF");
        assert!(
            !first.replace("\r\n", "").contains(['\r', '\n']),
            "{format:?} must not contain bare line endings"
        );

        let decoded = text::decode(first.as_bytes(), DecodeLimits::default())
            .unwrap_or_else(|error| panic!("{format:?} output must decode: {error}"));
        assert!(decoded.warnings().is_empty());
        assert_eq!(
            semantic_pairs(decoded.document()),
            expected_pairs,
            "{format:?} must preserve the documented code/text projection"
        );
    }
}

#[test]
fn phrase_text_eudp_phrase_text_round_trip_preserves_canonical_semantics() {
    let source = concat!(
        "aa,1=#$year|$year_yy|$month_mm|$month|$day|$day_dd|$fullhour|$minute|$second%20🤝\n",
        "aa,3=#gap%0Aline\n",
        "bb=$[甲 🤝]\n",
        "cc=\n",
        "多行第一段\n",
        "第二段 $[literal]\n",
        "dd duplicate 1\n",
        "dd duplicate 1\n",
    );
    let decoded_source = phrase_text::decode(source.as_bytes(), DecodeLimits::default())
        .expect("cross-codec phrase source must decode");
    assert!(decoded_source.warnings().is_empty());
    assert_eq!(
        decoded_source.document().entries()[0].text(),
        "%yyyy%|%yy%|%MM%|%M%|%dd%|%dd%|%HH%|%mm%|%ss% 🤝"
    );

    let source_canonical = phrase_text::format(decoded_source.document())
        .expect("source phrase document must format canonically");
    let source_document = phrase_text::decode(source_canonical.as_bytes(), DecodeLimits::default())
        .expect("canonical source text must decode")
        .into_parts()
        .0;
    let wire = eudp::encode(&source_document, TIMESTAMP)
        .expect("canonical phrase document must encode to EUDP");
    let wire_document =
        eudp::decode(&wire, DecodeLimits::default()).expect("generated EUDP must decode");
    assert_eq!(eudp::encode(&wire_document, TIMESTAMP), Ok(wire));

    let canonical_text =
        phrase_text::format(&wire_document).expect("wire phrase document must format");
    let reparsed = phrase_text::decode(canonical_text.as_bytes(), DecodeLimits::default())
        .expect("canonical phrase text must decode");
    assert!(reparsed.warnings().is_empty());
    assert_eq!(reparsed.document(), &source_document);
    assert!(
        reparsed
            .document()
            .entries()
            .windows(2)
            .any(|entries| entries[0] == entries[1]),
        "duplicate phrases must survive both codecs"
    );
    assert!(
        canonical_text.contains("aa\tgap%0Aline\t3\r\n"),
        "candidate gaps must remain explicit"
    );
}

#[test]
fn s0_code_weight_zero_dead_branch_omission_preserves_surrounding_d_e_f_records() {
    let decoded = text::decode(
        "合法D aa 7\n合法E bb\n合法F cc dd\n".as_bytes(),
        DecodeLimits::default(),
    )
    .expect("valid D, E, and F records must decode without the dead branch");

    assert!(decoded.warnings().is_empty());
    assert_eq!(
        decoded
            .document()
            .entries()
            .iter()
            .map(|entry| (
                entry.code().as_str(),
                entry.text(),
                entry.weight().map(Weight::get)
            ))
            .collect::<Vec<_>>(),
        vec![
            ("aa", "合法D", Some(5000)),
            ("bb", "合法E", None),
            ("cc", "合法F", None),
            ("dd", "合法F", None),
        ]
    );
}

fn semantic_pairs(document: &LexiconDocument) -> HashSet<(String, String)> {
    document
        .entries()
        .iter()
        .map(|entry| (entry.code().as_str().to_owned(), entry.text().to_owned()))
        .collect()
}

fn lexicon_entry(code: &str, value: &str, weight: u16) -> LexiconEntry {
    LexiconEntry::new(
        LexCode::new(code).expect("test lexicon code must be valid"),
        value,
        Some(Weight::new(weight).expect("test lexicon weight must be valid")),
    )
    .expect("test lexicon entry must be valid")
}
