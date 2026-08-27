//! Ownership-safe evidence for application sessions that did not exit normally.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use serde::Serialize;

const MARKER_PREFIX: &str = "wubilex-session-";
const MARKER_SUFFIX: &str = ".json";
const MARKER_SCHEMA_VERSION: u8 = 1;

/// Marker creation outcome retained by the process until normal exit.
#[derive(Debug)]
pub struct SessionMarker {
    path: PathBuf,
    previous_abnormal_session_count: usize,
}

/// Managed ownership wrapper that cleans a marker only on a normal Tauri exit.
#[derive(Debug)]
pub struct SessionLifecycle {
    marker: Mutex<Option<SessionMarker>>,
}

impl SessionLifecycle {
    pub fn new(marker: Option<SessionMarker>) -> Self {
        Self {
            marker: Mutex::new(marker),
        }
    }

    pub fn clean_exit(&self) -> io::Result<()> {
        let mut marker = self.lock();
        if let Some(owned) = marker.as_mut() {
            owned.clean_exit()?;
        }
        *marker = None;
        Ok(())
    }

    fn lock(&self) -> MutexGuard<'_, Option<SessionMarker>> {
        self.marker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl SessionMarker {
    /// Creates a unique marker with `create_new` and reports existing owned evidence.
    pub fn create(
        directory: impl AsRef<Path>,
        session_id: &str,
        process_id: u32,
        app_version: &str,
        started_at_unix_ms: u64,
    ) -> io::Result<Self> {
        validate_session_id(session_id)?;
        let directory = directory.as_ref();
        fs::create_dir_all(directory)?;
        let previous_abnormal_session_count = owned_markers(directory)?.len();
        let path = directory.join(format!("{MARKER_PREFIX}{session_id}{MARKER_SUFFIX}"));

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let guard = IncompleteMarkerGuard::new(path.clone());
        write_record(
            &mut file,
            &MarkerRecord {
                schema_version: MARKER_SCHEMA_VERSION,
                session_id,
                process_id,
                app_version,
                started_at_unix_ms,
            },
        )?;
        file.sync_all()?;
        guard.keep();

        Ok(Self {
            path,
            previous_abnormal_session_count,
        })
    }

    /// Number of markers that predated this session.
    pub fn previous_abnormal_session_count(&self) -> usize {
        self.previous_abnormal_session_count
    }

    /// Deletes only the exact marker created by this session.
    pub fn clean_exit(&mut self) -> io::Result<()> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    #[cfg(test)]
    fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MarkerRecord<'a> {
    schema_version: u8,
    session_id: &'a str,
    process_id: u32,
    app_version: &'a str,
    started_at_unix_ms: u64,
}

fn write_record(file: &mut File, record: &MarkerRecord<'_>) -> io::Result<()> {
    serde_json::to_writer(&mut *file, record).map_err(io::Error::other)?;
    file.write_all(b"\n")
}

fn validate_session_id(session_id: &str) -> io::Result<()> {
    let valid = !session_id.is_empty()
        && session_id.len() <= 64
        && session_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
    if valid {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "session id must contain only ASCII letters, digits, or hyphens",
        ))
    }
}

fn owned_markers(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut markers = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        if is_owned_marker_name(&name) {
            markers.push(entry.path());
        }
    }
    markers.sort();
    Ok(markers)
}

fn is_owned_marker_name(name: &std::ffi::OsStr) -> bool {
    name.to_str()
        .and_then(|name| {
            name.strip_prefix(MARKER_PREFIX)
                .and_then(|name| name.strip_suffix(MARKER_SUFFIX))
        })
        .is_some_and(|session_id| validate_session_id(session_id).is_ok())
}

struct IncompleteMarkerGuard {
    path: PathBuf,
    keep: bool,
}

impl IncompleteMarkerGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, keep: false }
    }

    fn keep(mut self) {
        self.keep = true;
    }
}

impl Drop for IncompleteMarkerGuard {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SessionMarker;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn detects_stale_evidence_and_clean_exit_removes_only_owned_marker() {
        let directory = TestDirectory::new();
        let stale = directory.path().join("wubilex-session-stale.json");
        let malformed = directory.path().join("wubilex-session-.json");
        fs::write(&stale, b"{}\n").expect("stale fixture must be writable");
        fs::write(&malformed, b"{}\n").expect("malformed fixture must be writable");

        let mut current = SessionMarker::create(directory.path(), "current", 42, "0.1.0", 100)
            .expect("current marker must be created");
        assert_eq!(current.previous_abnormal_session_count(), 1);
        assert!(current.path().exists());

        current
            .clean_exit()
            .expect("owned marker cleanup must succeed");
        assert!(!current.path().exists());
        assert!(stale.exists(), "normal exit must preserve other evidence");
        assert!(
            malformed.exists(),
            "unowned near-match must remain untouched"
        );
    }

    #[test]
    fn abnormal_drop_retains_the_marker() {
        let directory = TestDirectory::new();
        let path = {
            let marker = SessionMarker::create(directory.path(), "abnormal", 42, "0.1.0", 100)
                .expect("marker must be created");
            marker.path().to_path_buf()
        };
        assert!(path.exists(), "drop is not proof of a normal Tauri exit");
    }

    #[test]
    fn create_new_failure_never_deletes_existing_evidence() {
        let directory = TestDirectory::new();
        let existing = directory.path().join("wubilex-session-collision.json");
        fs::write(&existing, b"existing\n").expect("collision fixture must be writable");

        let error = SessionMarker::create(directory.path(), "collision", 42, "0.1.0", 100)
            .expect_err("create_new must reject an existing marker");
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read(existing).expect("existing evidence must remain"),
            b"existing\n"
        );
    }

    #[test]
    fn rejects_session_ids_that_could_escape_the_owned_directory() {
        let directory = TestDirectory::new();
        for id in ["", "../outside", "has space", "slash/name"] {
            let error = SessionMarker::create(directory.path(), id, 42, "0.1.0", 100)
                .expect_err("invalid id must fail before file creation");
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        }
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "wubilex-session-test-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("test directory must be unique");
            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
