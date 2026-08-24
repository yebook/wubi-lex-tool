mod support;

use std::{collections::HashSet, fs};

use sha2::{Digest, Sha256};
use support::{PREPARATION_HINT, fixture_directory, load_fixture_manifest};
use wubilex_codec::{DecodeLimits, LexScheme, detect, lex};

#[test]
fn all_eight_real_fixtures_decode_detect_and_reencode_byte_identically() {
    let fixture_directory = fixture_directory();
    let manifest = load_fixture_manifest(&fixture_directory);

    for fixture in manifest.fixtures {
        let path = fixture_directory.join(&fixture.lex_file);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "fixture {} is unavailable at {}: {error}; {PREPARATION_HINT}",
                fixture.id,
                path.display()
            )
        });
        assert_eq!(
            bytes.len(),
            fixture.lex_size,
            "fixture {} decoded size drifted",
            fixture.id
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            fixture.lex_sha256,
            "fixture {} decoded digest drifted",
            fixture.id
        );

        let document = lex::decode(&bytes, DecodeLimits::default())
            .unwrap_or_else(|error| panic!("fixture {} must decode strictly: {error}", fixture.id));
        assert!(
            !document.is_empty(),
            "fixture {} must be nonempty",
            fixture.id
        );
        assert!(
            document
                .entries()
                .windows(2)
                .all(|pair| pair[0].code().as_str() <= pair[1].code().as_str()),
            "fixture {} records must be ordered by code",
            fixture.id
        );

        assert_eq!(
            detect::scheme(&document),
            expected_scheme(&fixture.scheme),
            "fixture {} scheme detection drifted",
            fixture.id
        );
        let encoded = lex::encode(&document)
            .unwrap_or_else(|error| panic!("fixture {} must re-encode: {error}", fixture.id));
        assert_eq!(
            encoded, bytes,
            "fixture {} must re-encode byte-identically",
            fixture.id
        );

        let distinct_codes = document
            .entries()
            .iter()
            .map(|entry| entry.code().as_str())
            .collect::<HashSet<_>>()
            .len();
        eprintln!(
            "{}: bytes={}, entries={}, distinct_codes={}, sha256={}",
            fixture.id,
            fixture.lex_size,
            document.len(),
            distinct_codes,
            fixture.lex_sha256
        );
    }
}

fn expected_scheme(scheme: &str) -> LexScheme {
    match scheme {
        "wubi86" => LexScheme::Wubi86,
        "wubi98" => LexScheme::Wubi98,
        "wubi06" => LexScheme::Wubi06,
        "wubi091" => LexScheme::Wubi091,
        "wubi092" => LexScheme::Wubi092,
        "zhengma" => LexScheme::Zhengma { formation: false },
        "xiaohe" => LexScheme::XiaoheSoundShape,
        "biaoxingma" => LexScheme::Biaoxingma,
        other => panic!("unsupported fixture scheme {other}"),
    }
}
