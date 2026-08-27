//! Structured, bounded process logging without user-content payloads.

use std::{
    fs, io,
    path::Path,
    time::{Duration, SystemTime},
};

use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    filter::filter_fn,
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
};

const LOG_PREFIX: &str = "wubilex.";
const LOG_SUFFIX: &str = ".jsonl";
const RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Owns the non-blocking logging worker until the process loop returns.
#[derive(Debug)]
pub struct LoggingGuard {
    _worker_guard: WorkerGuard,
}

/// Bounded initialization failure used for a visible runtime notice.
#[derive(Debug, thiserror::Error)]
#[error("logging stage {stage} failed: {message}")]
pub struct LoggingError {
    stage: &'static str,
    message: String,
}

impl LoggingError {
    pub fn stage(&self) -> &'static str {
        self.stage
    }

    fn new(stage: &'static str, error: impl std::fmt::Display) -> Self {
        Self {
            stage,
            message: error.to_string(),
        }
    }
}

/// Initializes daily JSON logs, a seven-file bound and a redacted panic hook.
pub fn initialize(directory: &Path) -> Result<LoggingGuard, LoggingError> {
    fs::create_dir_all(directory).map_err(|error| LoggingError::new("create_directory", error))?;
    prune_owned_logs(directory, SystemTime::now())
        .map_err(|error| LoggingError::new("prune_retention", error))?;

    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_PREFIX.trim_end_matches('.'))
        .filename_suffix(LOG_SUFFIX.trim_start_matches('.'))
        .max_log_files(7)
        .build(directory)
        .map_err(|error| LoggingError::new("create_appender", error))?;
    let (writer, worker_guard) = tracing_appender::non_blocking(appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .flatten_event(true)
        .with_ansi(false)
        .with_writer(writer)
        .with_filter(filter_fn(is_application_event));

    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(file_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(false)
                .with_writer(io::stderr)
                .with_filter(filter_fn(is_application_event)),
        )
        .try_init()
        .map_err(|error| LoggingError::new("install_subscriber", error))?;

    #[cfg(not(debug_assertions))]
    tracing_subscriber::registry()
        .with(file_layer)
        .try_init()
        .map_err(|error| LoggingError::new("install_subscriber", error))?;

    install_redacted_panic_hook();
    Ok(LoggingGuard {
        _worker_guard: worker_guard,
    })
}

fn install_redacted_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload_type = if info.payload().is::<&str>() {
            "str"
        } else if info.payload().is::<String>() {
            "String"
        } else {
            "unknown"
        };
        let (source_file, source_line) = info.location().map_or(("unknown", 0), |location| {
            (location.file(), location.line())
        });
        tracing::error!(
            event = "panic",
            stage = "panic_hook",
            pid = std::process::id(),
            app_version = env!("CARGO_PKG_VERSION"),
            source_file,
            source_line,
            payload_type
        );
        previous(info);
    }));
}

fn is_application_event(metadata: &tracing::Metadata<'_>) -> bool {
    metadata.target() == "wubilex_app" || metadata.target().starts_with("wubilex_app::")
}

fn prune_owned_logs(directory: &Path, now: SystemTime) -> io::Result<()> {
    let current_day = unix_day(now)?;
    let oldest_retained_day = current_day.saturating_sub(retention_days());
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Some(log_day) = owned_log_day(&entry.file_name()) else {
            continue;
        };
        if log_day < oldest_retained_day {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn owned_log_day(name: &std::ffi::OsStr) -> Option<i64> {
    let name = name.to_str()?;
    let date = name
        .strip_prefix(LOG_PREFIX)
        .and_then(|name| name.strip_suffix(LOG_SUFFIX))?;
    let bytes = date.as_bytes();
    if bytes.len() != 10
        || !bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
    {
        return None;
    }

    let year = date[0..4].parse::<i32>().ok()?;
    let month = date[5..7].parse::<u32>().ok()?;
    let day = date[8..10].parse::<u32>().ok()?;
    if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

fn unix_day(time: SystemTime) -> io::Result<i64> {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("log retention clock predates Unix epoch: {error}"),
            )
        })?;
    i64::try_from(duration.as_secs() / 86_400)
        .map_err(|_| io::Error::other("log retention day does not fit in i64"))
}

const fn retention_days() -> i64 {
    (RETENTION.as_secs() / (24 * 60 * 60)) as i64
}

const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

const fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let mut adjusted_year = year as i64;
    let month = month as i64;
    let day = day as i64;
    if month <= 2 {
        adjusted_year -= 1;
    }
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::{days_from_civil, owned_log_day, prune_owned_logs};
    use std::{
        fs::{self, File},
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{Duration, SystemTime},
    };

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn ownership_filter_accepts_only_daily_json_log_names() {
        assert!(owned_log_day("wubilex.2026-08-26.jsonl".as_ref()).is_some());
        for name in [
            "other.2026-08-26.jsonl",
            "wubilex.latest.jsonl",
            "wubilex.2026-02-30.jsonl",
            "wubilex.2026-08-26.log",
            "wubilex.20260826.jsonl",
        ] {
            assert!(owned_log_day(name.as_ref()).is_none());
        }
    }

    #[test]
    fn pruning_removes_only_owned_files_older_than_seven_days() {
        let directory = TestDirectory::new();
        let old_owned = directory.path().join("wubilex.2026-08-18.jsonl");
        let boundary_owned = directory.path().join("wubilex.2026-08-19.jsonl");
        let unrelated = directory.path().join("notes.jsonl");
        let now = SystemTime::UNIX_EPOCH
            + Duration::from_secs(days_from_civil(2026, 8, 26) as u64 * 24 * 60 * 60);

        for path in [&old_owned, &boundary_owned, &unrelated] {
            File::create(path).expect("fixture file must be created");
        }

        prune_owned_logs(directory.path(), now).expect("pruning must succeed");
        assert!(!old_owned.exists());
        assert!(boundary_owned.exists());
        assert!(unrelated.exists());
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "wubilex-logging-test-{}-{}",
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
