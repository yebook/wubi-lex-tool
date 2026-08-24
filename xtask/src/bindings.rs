use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const GENERATED_PATH: &str = "src/types/generated/bindings.ts";
const CHECK_HINT: &str = "run `cargo xtask bindings` and commit the generated result";

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn run(check: bool) -> Result<(), String> {
    let root = crate::repository_root()?;
    sync_generated(&root, check, export_bindings)
}

fn export_bindings(path: &Path) -> Result<(), String> {
    wubilex_app::bindings::export_mock(path)
        .map_err(|error| format!("binding export stage failed: {error}"))
}

fn sync_generated(
    root: &Path,
    check: bool,
    export: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let temporary_directory = root.join("target/xtask/bindings");
    fs::create_dir_all(&temporary_directory).map_err(|error| {
        format!(
            "binding temporary-directory stage failed for {}: {error}",
            temporary_directory.display()
        )
    })?;
    let temporary_path = unique_path(&temporary_directory, "export", "ts");
    let _temporary_guard = CleanupFile::new(temporary_path.clone());

    export(&temporary_path)?;
    let generated = normalize_generated_file(&temporary_path)?;
    fs::write(&temporary_path, &generated).map_err(|error| {
        format!(
            "binding normalization stage failed for {}: {error}",
            temporary_path.display()
        )
    })?;

    let target = root.join(GENERATED_PATH);
    let existing = match fs::read(&target) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "binding target read stage failed for {}: {error}",
                target.display()
            ));
        }
    };

    if check {
        match existing {
            Some(bytes) if bytes == generated => {
                println!("bindings: generated TypeScript is current");
                return Ok(());
            }
            Some(_) => {
                return Err(format!(
                    "binding freshness check failed: {} is stale; {CHECK_HINT}",
                    target.display()
                ));
            }
            None => {
                return Err(format!(
                    "binding freshness check failed: {} is missing; {CHECK_HINT}",
                    target.display()
                ));
            }
        }
    }

    if existing.as_deref() == Some(generated.as_slice()) {
        println!("bindings: generated TypeScript is unchanged");
        return Ok(());
    }

    let parent = target
        .parent()
        .ok_or_else(|| "binding target has no parent directory".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "binding target-directory stage failed for {}: {error}",
            parent.display()
        )
    })?;

    let staged_path = unique_path(parent, ".bindings", "tmp");
    let mut staged_guard = create_owned_file(&staged_path, &generated)?;
    if target.exists() {
        fs::remove_file(&target).map_err(|error| {
            format!(
                "binding replacement stage failed to remove {}: {error}",
                target.display()
            )
        })?;
    }
    fs::rename(&staged_path, &target).map_err(|error| {
        format!(
            "binding replacement stage failed for {}: {error}",
            target.display()
        )
    })?;
    staged_guard.keep();
    println!("bindings: wrote {}", target.display());
    Ok(())
}

fn normalize_generated_file(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "binding generated-file read stage failed for {}: {error}",
            path.display()
        )
    })?;
    let text = String::from_utf8(bytes)
        .map_err(|_| "binding export stage produced non-UTF-8 TypeScript".to_owned())?;
    let mut normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    Ok(normalized.into_bytes())
}

fn create_owned_file(path: &Path, bytes: &[u8]) -> Result<CleanupFile, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "binding staging-file create stage failed for {}: {error}",
                path.display()
            )
        })?;
    let guard = CleanupFile::new(path.to_path_buf());
    file.write_all(bytes).map_err(|error| {
        format!(
            "binding staging-file write stage failed for {}: {error}",
            path.display()
        )
    })?;
    file.sync_all().map_err(|error| {
        format!(
            "binding staging-file sync stage failed for {}: {error}",
            path.display()
        )
    })?;
    Ok(guard)
}

fn unique_path(directory: &Path, stem: &str, extension: &str) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    directory.join(format!(
        "{stem}-{}-{sequence}.{extension}",
        std::process::id()
    ))
}

struct CleanupFile {
    path: PathBuf,
    remove: bool,
}

impl CleanupFile {
    fn new(path: PathBuf) -> Self {
        Self { path, remove: true }
    }

    fn keep(&mut self) {
        self.remove = false;
    }
}

impl Drop for CleanupFile {
    fn drop(&mut self) {
        if self.remove {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GENERATED_PATH, sync_generated};
    use std::{fs, path::PathBuf};
    use wubilex_app::bindings::GENERATED_HEADER;

    #[test]
    fn repeated_generation_is_byte_identical_and_lf_stable() {
        let directory = TestDirectory::new();
        let export = |path: &std::path::Path| {
            fs::write(path, format!("{GENERATED_HEADER}\r\nexport {{}};\r\n"))
                .map_err(|error| error.to_string())
        };

        sync_generated(directory.path(), false, export).expect("first generation must succeed");
        let target = directory.path().join(GENERATED_PATH);
        let first = fs::read(&target).expect("generated target must exist");
        sync_generated(directory.path(), false, export).expect("second generation must succeed");
        let second = fs::read(target).expect("generated target must remain");

        assert_eq!(first, second);
        assert!(first.starts_with(GENERATED_HEADER.as_bytes()));
        assert!(!first.contains(&b'\r'));
        assert_eq!(first.last(), Some(&b'\n'));
    }

    #[test]
    fn check_rejects_mutation_without_repairing_the_target() {
        let directory = TestDirectory::new();
        let export = |path: &std::path::Path| {
            fs::write(path, format!("{GENERATED_HEADER}\nexport {{}};\n"))
                .map_err(|error| error.to_string())
        };
        sync_generated(directory.path(), false, export).expect("generation must succeed");

        let target = directory.path().join(GENERATED_PATH);
        fs::write(&target, b"manually changed\r\n").expect("mutation must succeed");
        let before = fs::read(&target).expect("mutated target must exist");
        let error = sync_generated(directory.path(), true, export)
            .expect_err("freshness check must reject mutation");
        let after = fs::read(target).expect("check must not remove target");

        assert!(error.contains("stale"));
        assert!(error.contains("cargo xtask bindings"));
        assert_eq!(before, after, "check mode must not repair the target");
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "wubilex-xtask-bindings-{}-{}",
                std::process::id(),
                super::TEMP_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
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
