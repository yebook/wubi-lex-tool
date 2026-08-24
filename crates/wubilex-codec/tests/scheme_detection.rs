use wubilex_codec::{LexCode, LexScheme, LexiconDocument, LexiconEntry, Weight, detect};

#[test]
fn direct_feature_groups_cover_five_branches_in_priority_order() {
    let cases = [
        (
            vec![("q", "月"), ("e", "世")],
            LexScheme::Zhengma { formation: false },
        ),
        (
            vec![("aakk", "啊"), ("hedn", "鹤")],
            LexScheme::XiaoheSoundShape,
        ),
        (
            vec![("qv", "月"), ("ev", "世")],
            LexScheme::Zhengma { formation: true },
        ),
        (
            vec![("sr", "版"), ("ks", "吃"), ("ms", "见")],
            LexScheme::Wubi092,
        ),
        (vec![("hodd", "够"), ("opto", "啊")], LexScheme::Biaoxingma),
    ];

    for (features, expected) in cases {
        assert_eq!(detect::scheme(&document(&features)), expected);
    }

    let all = document(&[
        ("q", "月"),
        ("e", "世"),
        ("aakk", "啊"),
        ("hedn", "鹤"),
        ("qv", "月"),
        ("ev", "世"),
    ]);
    assert_eq!(
        detect::scheme(&all),
        LexScheme::Zhengma { formation: false }
    );
}

#[test]
fn scored_features_cover_86_98_06_and_091() {
    let cases = [
        (Vec::new(), LexScheme::Wubi86),
        (
            vec![
                ("teb", "笔"),
                ("othc", "煅"),
                ("tuwb", "舱"),
                ("khdy", "跋"),
            ],
            LexScheme::Wubi98,
        ),
        (vec![("khdy", "跋"), ("xfxy", "线")], LexScheme::Wubi06),
        (
            vec![("ks", "整"), ("lm", "示"), ("ms", "刺")],
            LexScheme::Wubi091,
        ),
    ];

    for (features, expected) in cases {
        assert_eq!(detect::scheme(&document(&features)), expected);
    }
}

#[test]
fn lowercase_xfxy_is_a_non_tautological_06_regression() {
    assert_eq!(detect::scheme(&document(&[])), LexScheme::Wubi86);
    assert_eq!(
        detect::scheme(&document(&[("xfxy", "线")])),
        LexScheme::Wubi06
    );
}

#[test]
fn an_explicit_98_06_score_tie_falls_back_to_86() {
    assert_eq!(
        detect::scheme(&document(&[("teb", "笔"), ("khdy", "跋")])),
        LexScheme::Wubi86
    );
}

#[test]
fn detection_ignores_weights_source_order_and_duplicates() {
    let first = entry("q", "月", None);
    let second = entry("e", "世", Some(u16::MAX));
    let document = LexiconDocument::new(vec![second, first.clone(), first]);

    assert_eq!(
        detect::scheme(&document),
        LexScheme::Zhengma { formation: false }
    );
}

fn document(features: &[(&str, &str)]) -> LexiconDocument {
    LexiconDocument::new(
        features
            .iter()
            .map(|(code, text)| entry(code, text, None))
            .collect(),
    )
}

fn entry(code: &str, text: &str, weight: Option<u16>) -> LexiconEntry {
    LexiconEntry::new(
        LexCode::new(code).expect("test code must be valid"),
        text,
        weight.map(|value| Weight::new(value).expect("test weight must be valid")),
    )
    .expect("test entry must be valid")
}
