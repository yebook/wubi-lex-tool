use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;
use wubilex_codec::{DecodeLimits, lex};

const MANIFEST_SCHEMA_VERSION: u32 = 1;
const MANIFEST_PATH: &str = "crates/wubilex-codec/tests/fixtures/manifest.json";
const FIXTURE_DIRECTORY: &str = "crates/wubilex-codec/tests/fixtures";
const MAX_ARCHIVE_BYTES: usize = 16 * 1024 * 1024;
const MAX_DECODED_BYTES: usize = wubilex_codec::limits::DEFAULT_MAX_INPUT_BYTES;
const MAX_REDIRECTS: u32 = 5;
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
const CHECK_HINT: &str = "run `cargo xtask fixtures` to prepare the real codec fixtures";

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureManifest {
    schema_version: u32,
    fixtures: Vec<FixtureEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureEntry {
    id: String,
    scheme: String,
    name: String,
    url: String,
    archive_file: String,
    lex_file: String,
    archive_size: usize,
    archive_sha256: String,
    lex_size: usize,
    lex_sha256: String,
    source: String,
    license_note: String,
}

pub(crate) fn run(check: bool) -> Result<(), String> {
    let repository_root = repository_root()?;
    let fixture_directory = repository_root.join(FIXTURE_DIRECTORY);
    let manifest = load_manifest(&repository_root.join(MANIFEST_PATH))?;

    if !check {
        fs::create_dir_all(&fixture_directory)
            .map_err(|error| format!("fixture directory create stage failed: {error}"))?;
    }

    let agent = if check { None } else { Some(build_agent()?) };
    for entry in &manifest.fixtures {
        match verify_cached(entry, &fixture_directory) {
            Ok(()) => println!("fixture {}: verified cached files", entry.id),
            Err(error) if check => return Err(format!("{error}; {CHECK_HINT}")),
            Err(error) => {
                println!(
                    "fixture {}: cache unavailable ({error}); fetching",
                    entry.id
                );
                let agent = agent
                    .as_ref()
                    .ok_or_else(|| "fixture HTTP client was not initialized".to_owned())?;
                fetch_and_install(agent, entry, &fixture_directory)?;
                println!("fixture {}: downloaded and verified", entry.id);
            }
        }
    }

    Ok(())
}

fn repository_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest directory has no repository parent".to_owned())
}

fn load_manifest(path: &Path) -> Result<FixtureManifest, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "fixture manifest read stage failed for {}: {error}",
            path.display()
        )
    })?;
    let manifest: FixtureManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("fixture manifest parse stage failed: {error}"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &FixtureManifest) -> Result<(), String> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(format!(
            "fixture manifest validation stage failed: expected schema version {MANIFEST_SCHEMA_VERSION}, got {}",
            manifest.schema_version
        ));
    }
    if manifest.fixtures.len() != REQUIRED_SCHEMES.len() {
        return Err(format!(
            "fixture manifest validation stage failed: expected {} fixtures, got {}",
            REQUIRED_SCHEMES.len(),
            manifest.fixtures.len()
        ));
    }

    let mut ids = HashSet::new();
    let mut schemes = HashSet::new();
    let mut paths = HashSet::new();
    for entry in &manifest.fixtures {
        validate_entry(entry)?;
        if !ids.insert(entry.id.as_str()) {
            return Err(manifest_error(entry, "duplicate fixture id"));
        }
        if !schemes.insert(entry.scheme.as_str()) {
            return Err(manifest_error(entry, "duplicate fixture scheme"));
        }
        for path in [&entry.archive_file, &entry.lex_file] {
            if !paths.insert(path.as_str()) {
                return Err(manifest_error(entry, "duplicate fixture path"));
            }
        }
    }

    let expected = REQUIRED_SCHEMES.into_iter().collect::<HashSet<_>>();
    if schemes != expected {
        return Err("fixture manifest validation stage failed: scheme set must be exactly wubi86, wubi98, wubi06, wubi091, wubi092, zhengma, xiaohe, and biaoxingma".to_owned());
    }
    Ok(())
}

fn validate_entry(entry: &FixtureEntry) -> Result<(), String> {
    if entry.id.is_empty()
        || entry.name.trim().is_empty()
        || entry.source.trim().is_empty()
        || entry.license_note.trim().is_empty()
    {
        return Err(manifest_error(entry, "text fields must be nonempty"));
    }

    let url = Url::parse(&entry.url)
        .map_err(|_| manifest_error(entry, "URL must be a valid absolute HTTPS URL"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(manifest_error(
            entry,
            "URL must use HTTPS without credentials or a fragment",
        ));
    }

    validate_file_name(entry, &entry.archive_file, ".lex.lzma")?;
    validate_file_name(entry, &entry.lex_file, ".lex")?;
    validate_size(entry, "archive_size", entry.archive_size, MAX_ARCHIVE_BYTES)?;
    validate_size(entry, "lex_size", entry.lex_size, MAX_DECODED_BYTES)?;
    validate_digest(entry, "archive_sha256", &entry.archive_sha256)?;
    validate_digest(entry, "lex_sha256", &entry.lex_sha256)?;
    Ok(())
}

fn validate_file_name(entry: &FixtureEntry, value: &str, suffix: &str) -> Result<(), String> {
    let allowed = value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte));
    if value.is_empty()
        || !allowed
        || value.contains(['/', '\\'])
        || matches!(value, "." | "..")
        || !value.ends_with(suffix)
    {
        return Err(manifest_error(
            entry,
            "fixture paths must be lowercase portable file names with the expected suffix",
        ));
    }
    Ok(())
}

fn validate_size(
    entry: &FixtureEntry,
    field: &str,
    value: usize,
    maximum: usize,
) -> Result<(), String> {
    if value == 0 || value > maximum {
        return Err(manifest_error(
            entry,
            &format!("{field} must be in 1..={maximum}, got {value}"),
        ));
    }
    Ok(())
}

fn validate_digest(entry: &FixtureEntry, field: &str, value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(manifest_error(
            entry,
            &format!("{field} must be 64 lowercase hexadecimal characters"),
        ));
    }
    Ok(())
}

fn manifest_error(entry: &FixtureEntry, detail: &str) -> String {
    format!(
        "fixture {} manifest validation stage failed: {detail}",
        entry.id
    )
}

fn build_agent() -> Result<ureq::Agent, String> {
    let tls = ureq::native_tls::TlsConnector::new()
        .map_err(|_| "fixture HTTPS client initialization stage failed".to_owned())?;
    Ok(ureq::AgentBuilder::new()
        .redirects(MAX_REDIRECTS)
        .https_only(true)
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(120))
        .tls_connector(Arc::new(tls))
        .build())
}

fn verify_cached(entry: &FixtureEntry, directory: &Path) -> Result<(), String> {
    verify_file(
        entry,
        &directory.join(&entry.archive_file),
        "archive cache verification",
        entry.archive_size,
        &entry.archive_sha256,
        MAX_ARCHIVE_BYTES,
    )?;
    verify_lex(
        entry,
        &directory.join(&entry.lex_file),
        "lex cache verification",
    )
}

fn verify_lex(entry: &FixtureEntry, path: &Path, stage: &str) -> Result<(), String> {
    verify_file(
        entry,
        path,
        stage,
        entry.lex_size,
        &entry.lex_sha256,
        MAX_DECODED_BYTES,
    )?;
    let bytes = fs::read(path).map_err(|error| fixture_error(entry, stage, &error.to_string()))?;
    lex::decode(&bytes, DecodeLimits::default()).map_err(|error| {
        fixture_error(
            entry,
            stage,
            &format!("strict imscwubi decode failed: {error}"),
        )
    })?;
    Ok(())
}

fn verify_file(
    entry: &FixtureEntry,
    path: &Path,
    stage: &str,
    expected_size: usize,
    expected_digest: &str,
    maximum_size: usize,
) -> Result<(), String> {
    let (actual_size, actual_digest) = digest_file(entry, path, stage, maximum_size)?;
    if actual_size != expected_size {
        return Err(fixture_error(
            entry,
            stage,
            &format!("expected {expected_size} bytes, got {actual_size}"),
        ));
    }
    if actual_digest != expected_digest {
        return Err(fixture_error(
            entry,
            stage,
            &format!("expected SHA-256 {expected_digest}, got {actual_digest}"),
        ));
    }
    Ok(())
}

fn digest_file(
    entry: &FixtureEntry,
    path: &Path,
    stage: &str,
    maximum_size: usize,
) -> Result<(usize, String), String> {
    let file = File::open(path).map_err(|error| fixture_error(entry, stage, &error.to_string()))?;
    let metadata = file
        .metadata()
        .map_err(|error| fixture_error(entry, stage, &error.to_string()))?;
    let metadata_size = usize::try_from(metadata.len()).map_err(|_| {
        fixture_error(
            entry,
            stage,
            "file size is not representable on this platform",
        )
    })?;
    if metadata_size > maximum_size {
        return Err(fixture_error(
            entry,
            stage,
            &format!("file exceeds the {maximum_size}-byte safety limit"),
        ));
    }

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut total = 0usize;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| fixture_error(entry, stage, &error.to_string()))?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count)
            .ok_or_else(|| fixture_error(entry, stage, "byte count overflow"))?;
        if total > maximum_size {
            return Err(fixture_error(
                entry,
                stage,
                &format!("file exceeds the {maximum_size}-byte safety limit"),
            ));
        }
        hasher.update(&buffer[..count]);
    }

    Ok((total, format!("{:x}", hasher.finalize())))
}

fn fetch_and_install(
    agent: &ureq::Agent,
    entry: &FixtureEntry,
    directory: &Path,
) -> Result<(), String> {
    let archive_temp = temporary_path(directory, &entry.archive_file);
    let lex_temp = temporary_path(directory, &entry.lex_file);

    let mut archive_guard = download_archive(agent, entry, &archive_temp)?;
    let mut lex_guard = decompress_archive(entry, &archive_temp, &lex_temp)?;
    verify_file(
        entry,
        &archive_temp,
        "downloaded archive verification",
        entry.archive_size,
        &entry.archive_sha256,
        MAX_ARCHIVE_BYTES,
    )?;
    verify_lex(entry, &lex_temp, "decompressed lex verification")?;

    let archive_target = directory.join(&entry.archive_file);
    let lex_target = directory.join(&entry.lex_file);
    remove_existing(entry, &archive_target, "archive replacement")?;
    remove_existing(entry, &lex_target, "lex replacement")?;
    fs::rename(&archive_temp, &archive_target)
        .map_err(|error| fixture_error(entry, "archive placement", &error.to_string()))?;
    archive_guard.keep();
    fs::rename(&lex_temp, &lex_target)
        .map_err(|error| fixture_error(entry, "lex placement", &error.to_string()))?;
    lex_guard.keep();
    Ok(())
}

fn download_archive(
    agent: &ureq::Agent,
    entry: &FixtureEntry,
    path: &Path,
) -> Result<CleanupFile, String> {
    let response = agent.get(&entry.url).call().map_err(|error| match error {
        ureq::Error::Status(status, _) => fixture_error(
            entry,
            "HTTPS response",
            &format!("expected a successful status, got {status}"),
        ),
        ureq::Error::Transport(error) => fixture_error(
            entry,
            "HTTPS transport",
            &format!(
                "request failed ({:?}): {}",
                error.kind(),
                error.message().unwrap_or("no additional detail")
            ),
        ),
    })?;
    if !(200..300).contains(&response.status()) {
        return Err(fixture_error(
            entry,
            "HTTPS response",
            &format!("expected a 2xx status, got {}", response.status()),
        ));
    }
    let final_url = Url::parse(response.get_url())
        .map_err(|_| fixture_error(entry, "HTTPS redirect", "final response URL is invalid"))?;
    if final_url.scheme() != "https" {
        return Err(fixture_error(
            entry,
            "HTTPS redirect",
            "final response URL is not HTTPS",
        ));
    }
    if let Some(value) = response.header("Content-Length") {
        let content_length = value.parse::<usize>().map_err(|_| {
            fixture_error(entry, "HTTPS headers", "Content-Length is not an integer")
        })?;
        if content_length != entry.archive_size || content_length > MAX_ARCHIVE_BYTES {
            return Err(fixture_error(
                entry,
                "HTTPS headers",
                &format!(
                    "expected Content-Length {}, got {content_length}",
                    entry.archive_size
                ),
            ));
        }
    }

    let mut source = response.into_reader();
    let (file, guard) = create_temporary_file(entry, path, "archive temporary file")?;
    let mut target = BufWriter::new(file);
    let mut hasher = Sha256::new();
    let mut total = 0usize;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = source
            .read(&mut buffer)
            .map_err(|error| fixture_error(entry, "archive download body", &error.to_string()))?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count)
            .ok_or_else(|| fixture_error(entry, "archive download body", "byte count overflow"))?;
        if total > entry.archive_size || total > MAX_ARCHIVE_BYTES {
            return Err(fixture_error(
                entry,
                "archive download body",
                &format!(
                    "body exceeded expected {} bytes or safety limit {MAX_ARCHIVE_BYTES}",
                    entry.archive_size
                ),
            ));
        }
        target
            .write_all(&buffer[..count])
            .map_err(|error| fixture_error(entry, "archive temporary write", &error.to_string()))?;
        hasher.update(&buffer[..count]);
    }
    target
        .flush()
        .map_err(|error| fixture_error(entry, "archive temporary flush", &error.to_string()))?;

    let digest = format!("{:x}", hasher.finalize());
    if total != entry.archive_size || digest != entry.archive_sha256 {
        return Err(fixture_error(
            entry,
            "archive download integrity",
            &format!(
                "expected {} bytes and SHA-256 {}, got {total} bytes and {digest}",
                entry.archive_size, entry.archive_sha256
            ),
        ));
    }
    Ok(guard)
}

fn decompress_archive(
    entry: &FixtureEntry,
    archive: &Path,
    output: &Path,
) -> Result<CleanupFile, String> {
    let source = File::open(archive)
        .map_err(|error| fixture_error(entry, "LZMA archive open", &error.to_string()))?;
    let (target, guard) = create_temporary_file(entry, output, "LZMA temporary file")?;
    let mut reader = BufReader::new(source);
    let limit = entry.lex_size.min(MAX_DECODED_BYTES);
    let mut writer = BoundedWriter::new(BufWriter::new(target), limit);
    lzma_rs::lzma_decompress(&mut reader, &mut writer)
        .map_err(|error| fixture_error(entry, "LZMA-alone decompression", &error.to_string()))?;
    writer
        .flush()
        .map_err(|error| fixture_error(entry, "LZMA output flush", &error.to_string()))?;
    if writer.written() != entry.lex_size {
        return Err(fixture_error(
            entry,
            "LZMA output size",
            &format!(
                "expected {} bytes, got {}",
                entry.lex_size,
                writer.written()
            ),
        ));
    }
    Ok(guard)
}

fn create_temporary_file(
    entry: &FixtureEntry,
    path: &Path,
    stage: &str,
) -> Result<(File, CleanupFile), String> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| fixture_error(entry, stage, &error.to_string()))?;
    Ok((file, CleanupFile::new(path.to_path_buf())))
}

fn remove_existing(entry: &FixtureEntry, path: &Path, stage: &str) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(fixture_error(entry, stage, &error.to_string())),
    }
}

fn temporary_path(directory: &Path, file_name: &str) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    directory.join(format!(
        "{file_name}.part-{}-{sequence}",
        std::process::id()
    ))
}

fn fixture_error(entry: &FixtureEntry, stage: &str, detail: &str) -> String {
    format!(
        "fixture {} ({}) {stage} stage failed: {detail}",
        entry.id, entry.name
    )
}

struct CleanupFile {
    path: PathBuf,
    keep: bool,
}

impl CleanupFile {
    const fn new(path: PathBuf) -> Self {
        Self { path, keep: false }
    }

    fn keep(&mut self) {
        self.keep = true;
    }
}

impl Drop for CleanupFile {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_file(&self.path);
        }
    }
}

struct BoundedWriter<W> {
    inner: W,
    limit: usize,
    written: usize,
}

impl<W> BoundedWriter<W> {
    const fn new(inner: W, limit: usize) -> Self {
        Self {
            inner,
            limit,
            written: 0,
        }
    }

    const fn written(&self) -> usize {
        self.written
    }
}

impl<W: Write> Write for BoundedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let next = self
            .written
            .checked_add(buffer.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "decoded size overflow"))?;
        if next > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "decoded fixture exceeds its bounded size",
            ));
        }
        let count = self.inner.write(buffer)?;
        self.written = self
            .written
            .checked_add(count)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "decoded size overflow"))?;
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Cursor, Write},
        path::{Path, PathBuf},
    };

    use sha2::{Digest, Sha256};
    use wubilex_codec::{LexiconDocument, lex};

    use super::{
        BoundedWriter, FixtureEntry, FixtureManifest, REQUIRED_SCHEMES, create_temporary_file,
        validate_manifest, verify_cached,
    };

    #[test]
    fn manifest_rejects_wrong_schema_scheme_set_and_duplicate_paths() {
        let mut manifest = valid_manifest();
        assert!(validate_manifest(&manifest).is_ok());

        manifest.schema_version = 2;
        assert!(validate_manifest(&manifest).is_err());
        manifest.schema_version = 1;

        manifest.fixtures[0].scheme = "unknown".to_owned();
        assert!(validate_manifest(&manifest).is_err());
        manifest.fixtures[0].scheme = REQUIRED_SCHEMES[0].to_owned();

        manifest.fixtures[1].lex_file = manifest.fixtures[0].archive_file.clone();
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn manifest_rejects_insecure_urls_paths_hashes_and_sizes() {
        let cases: [fn(&mut FixtureEntry); 4] = [
            |entry| entry.url = "http://example.test/file.lex.lzma".to_owned(),
            |entry| entry.lex_file = "../fixture.lex".to_owned(),
            |entry| entry.lex_sha256 = "ABC".to_owned(),
            |entry| entry.archive_size = 0,
        ];
        for mutate in cases {
            let mut manifest = valid_manifest();
            mutate(&mut manifest.fixtures[0]);
            assert!(validate_manifest(&manifest).is_err());
        }
    }

    #[test]
    fn bounded_writer_rejects_the_first_byte_beyond_its_limit() {
        let mut writer = BoundedWriter::new(Vec::new(), 3);
        assert_eq!(writer.write(b"abc").expect("three bytes must fit"), 3);
        let error = writer
            .write(b"d")
            .expect_err("the fourth byte must exceed the limit");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn offline_cache_verification_rejects_missing_and_corrupt_files() {
        let directory = TestDirectory::new();
        let lex_bytes = lex::encode(&LexiconDocument::default()).expect("empty lex must encode");
        let mut archive_bytes = Vec::new();
        lzma_rs::lzma_compress(&mut Cursor::new(&lex_bytes), &mut archive_bytes)
            .expect("test archive must compress");
        let entry = fixture_entry("wubi86", 0, &archive_bytes, &lex_bytes);

        assert!(verify_cached(&entry, directory.path()).is_err());
        fs::write(directory.path().join(&entry.archive_file), &archive_bytes)
            .expect("test archive must write");
        fs::write(directory.path().join(&entry.lex_file), &lex_bytes).expect("test lex must write");
        assert!(verify_cached(&entry, directory.path()).is_ok());

        fs::write(
            directory
                .path()
                .join(format!("{}.part-stale", entry.lex_file)),
            b"incomplete",
        )
        .expect("stale partial must write");
        assert!(
            verify_cached(&entry, directory.path()).is_ok(),
            "stale partial files must never replace the verified targets"
        );

        fs::write(directory.path().join(&entry.archive_file), b"corrupt")
            .expect("corrupt archive must write");
        assert!(verify_cached(&entry, directory.path()).is_err());

        fs::write(directory.path().join(&entry.archive_file), &archive_bytes)
            .expect("test archive must restore");
        fs::write(directory.path().join(&entry.lex_file), b"corrupt")
            .expect("corrupt lex must write");
        assert!(verify_cached(&entry, directory.path()).is_err());
    }

    #[test]
    fn temporary_file_collision_does_not_delete_an_existing_partial() {
        let directory = TestDirectory::new();
        let path = directory.path().join("fixture.lex.part-existing");
        fs::write(&path, b"owned by another run").expect("existing partial must write");
        let entry = fixture_entry("wubi86", 0, b"archive", b"imscwubi");

        assert!(create_temporary_file(&entry, &path, "test temporary file").is_err());
        assert_eq!(
            fs::read(path).expect("existing partial must remain"),
            b"owned by another run"
        );
    }

    fn valid_manifest() -> FixtureManifest {
        FixtureManifest {
            schema_version: 1,
            fixtures: REQUIRED_SCHEMES
                .iter()
                .enumerate()
                .map(|(index, scheme)| fixture_entry(scheme, index, b"archive", b"imscwubi"))
                .collect(),
        }
    }

    fn fixture_entry(
        scheme: &str,
        index: usize,
        archive_bytes: &[u8],
        lex_bytes: &[u8],
    ) -> FixtureEntry {
        FixtureEntry {
            id: scheme.to_owned(),
            scheme: scheme.to_owned(),
            name: format!("fixture {scheme}"),
            url: format!("https://example.test/{scheme}.lex.lzma"),
            archive_file: format!("fixture-{index}.lex.lzma"),
            lex_file: format!("fixture-{index}.lex"),
            archive_size: archive_bytes.len(),
            archive_sha256: digest(archive_bytes),
            lex_size: lex_bytes.len(),
            lex_sha256: digest(lex_bytes),
            source: "test source".to_owned(),
            license_note: "test only".to_owned(),
        }
    }

    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "wubilex-xtask-fixtures-{}-{}",
                std::process::id(),
                super::TEMP_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("test directory must be unique");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
