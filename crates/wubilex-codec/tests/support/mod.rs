use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use wubilex_codec::limits::DEFAULT_MAX_INPUT_BYTES;

pub(crate) const PREPARATION_HINT: &str =
    "run `cargo xtask fixtures` to prepare real codec fixtures";
const REQUIRED_SCHEMES: [&str; 8] = [
    "wubi86",
    "wubi98",
    "wubi06",
    "wubi091",
    "wubi092",
    "zhengma",
    "xiaohe",
    "biaoxingma",
];

#[derive(Debug, Deserialize)]
pub(crate) struct FixtureManifest {
    schema_version: u32,
    pub(crate) fixtures: Vec<FixtureEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FixtureEntry {
    pub(crate) id: String,
    pub(crate) scheme: String,
    pub(crate) lex_file: String,
    pub(crate) lex_size: usize,
    pub(crate) lex_sha256: String,
}

pub(crate) fn fixture_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub(crate) fn load_fixture_manifest(directory: &Path) -> FixtureManifest {
    let path = directory.join("manifest.json");
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read fixture manifest {}: {error}; {PREPARATION_HINT}",
            path.display()
        )
    });
    let manifest: FixtureManifest = serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("fixture manifest must be valid JSON: {error}"));
    validate_manifest(&manifest);
    manifest
}

fn validate_manifest(manifest: &FixtureManifest) {
    assert_eq!(manifest.schema_version, 1, "unsupported fixture schema");
    assert_eq!(
        manifest.fixtures.len(),
        REQUIRED_SCHEMES.len(),
        "fixture manifest must contain all eight schemes"
    );

    let mut ids = HashSet::new();
    let mut schemes = HashSet::new();
    let mut paths = HashSet::new();
    for fixture in &manifest.fixtures {
        assert!(!fixture.id.is_empty(), "fixture ids must be nonempty");
        assert!(
            ids.insert(fixture.id.as_str()),
            "fixture ids must be unique"
        );
        assert!(
            schemes.insert(fixture.scheme.as_str()),
            "fixture schemes must be unique"
        );
        assert!(
            paths.insert(fixture.lex_file.as_str()),
            "fixture lex paths must be unique"
        );
        assert_portable_lex_file(fixture);
        assert!(
            (1..=DEFAULT_MAX_INPUT_BYTES).contains(&fixture.lex_size),
            "fixture {} has an invalid decoded size",
            fixture.id
        );
        assert!(
            fixture.lex_sha256.len() == 64
                && fixture
                    .lex_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "fixture {} has an invalid decoded SHA-256",
            fixture.id
        );
    }

    assert_eq!(
        schemes,
        REQUIRED_SCHEMES.into_iter().collect(),
        "fixture scheme set must match the eight-scheme contract"
    );
}

fn assert_portable_lex_file(fixture: &FixtureEntry) {
    let file = fixture.lex_file.as_str();
    assert!(
        !file.is_empty()
            && file.ends_with(".lex")
            && file.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
            })
            && !file.contains(['/', '\\'])
            && !matches!(file, "." | ".."),
        "fixture {} has an invalid decoded file name",
        fixture.id
    );
}
