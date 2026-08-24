use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use regex::Regex;

const EXPECTED_COUNTS: Counts = Counts {
    modules: 414,
    nfr: 101,
    ux: 115,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counts {
    modules: usize,
    nfr: usize,
    ux: usize,
}

impl Counts {
    fn total(self) -> usize {
        self.modules + self.nfr + self.ux
    }
}

#[derive(Debug)]
struct AnchorOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

pub(crate) fn run() -> Result<(), String> {
    let root = crate::repository_root()?;
    let counts = validate_document_tree(&root, EXPECTED_COUNTS, run_anchor_checker)?;
    println!(
        "docs: modules={}, NFR={}, UX={}, total={}; dangling=0, placeholders=0, anchors=0",
        counts.modules,
        counts.nfr,
        counts.ux,
        counts.total()
    );
    Ok(())
}

fn validate_document_tree(
    root: &Path,
    expected: Counts,
    anchor_runner: impl FnOnce(&Path) -> Result<AnchorOutput, String>,
) -> Result<Counts, String> {
    let docs = root.join("docs");
    let files = markdown_files(&docs)?;
    let id_pattern = Regex::new(r"\b(?:M[1-8]|NFR|UX)-[A-Z0-9]+-[0-9]{3}\b")
        .map_err(|error| format!("document validator regex stage failed: {error}"))?;
    let definition_pattern =
        Regex::new(r"^\s*\|\s*`?((?:M[1-8]|NFR|UX)-[A-Z0-9]+-[0-9]{3})`?\s*\|")
            .map_err(|error| format!("document validator regex stage failed: {error}"))?;
    let definition_candidate_pattern =
        Regex::new(r"^\s*\|\s*`?((?:M[0-9]+|NFR|UX)-[^`|\s]+)`?\s*\|")
            .map_err(|error| format!("document validator regex stage failed: {error}"))?;

    let mut definitions = BTreeMap::<String, Vec<Location>>::new();
    let mut references = BTreeMap::<String, Vec<Location>>::new();
    let mut issues = Vec::new();

    for path in files {
        let relative = relative_display(root, &path);
        let owner = definition_owner(root, &path);
        let text = fs::read_to_string(&path).map_err(|error| {
            format!("document read stage failed for {}: {error}", path.display())
        })?;
        for (index, line) in text.lines().enumerate() {
            let location = Location {
                path: relative.clone(),
                line: index + 1,
            };
            for found in id_pattern.find_iter(line) {
                references
                    .entry(found.as_str().to_owned())
                    .or_default()
                    .push(location.clone());
            }
            if let Some(owner) = owner {
                match (
                    definition_candidate_pattern.captures(line),
                    definition_pattern.captures(line),
                ) {
                    (_, Some(captures)) => {
                        let Some(id) = captures.get(1).map(|value| value.as_str()) else {
                            issues.push(format!(
                                "definition parser did not capture an ID at {}:{}",
                                location.path, location.line
                            ));
                            continue;
                        };
                        if owner.owns(id) {
                            definitions
                                .entry(id.to_owned())
                                .or_default()
                                .push(location.clone());
                        } else {
                            issues.push(format!(
                                "requirement definition {id} has the wrong document owner at {}:{}",
                                location.path, location.line
                            ));
                        }
                    }
                    (Some(captures), None) => {
                        let candidate = captures
                            .get(1)
                            .map_or("<unreadable>", |value| value.as_str());
                        issues.push(format!(
                            "invalid requirement definition ID {candidate} at {}:{}",
                            location.path, location.line
                        ));
                    }
                    (None, None) => {}
                }
            }
            if contains_placeholder(line) && !is_placeholder_self_check(line) {
                issues.push(format!(
                    "placeholder found at {}:{}",
                    location.path, location.line
                ));
            }
        }
    }

    for (id, locations) in &definitions {
        if locations.len() > 1 {
            let found_at = locations
                .iter()
                .map(Location::display)
                .collect::<Vec<_>>()
                .join(", ");
            issues.push(format!("duplicate definition {id}: {found_at}"));
        }
    }

    let unique_definitions = definitions.keys().cloned().collect::<BTreeSet<_>>();
    let counts = count_definitions(&unique_definitions);
    if counts != expected {
        issues.push(format!(
            "definition count drift: expected modules/NFR/UX/total {}/{}/{}/{}, got {}/{}/{}/{}",
            expected.modules,
            expected.nfr,
            expected.ux,
            expected.total(),
            counts.modules,
            counts.nfr,
            counts.ux,
            counts.total()
        ));
    }

    for (id, locations) in references {
        if !unique_definitions.contains(&id) {
            let found_at = locations
                .iter()
                .map(Location::display)
                .collect::<Vec<_>>()
                .join(", ");
            issues.push(format!("dangling requirement reference {id}: {found_at}"));
        }
    }

    match anchor_runner(root) {
        Ok(output) if output.success => {}
        Ok(output) => {
            let evidence = [output.stdout.trim(), output.stderr.trim()]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ");
            issues.push(format!("anchor checker failed: {evidence}"));
        }
        Err(error) => issues.push(format!("anchor checker spawn failed: {error}")),
    }

    if issues.is_empty() {
        Ok(counts)
    } else {
        Err(format!(
            "document invariant check failed with {} issue(s):\n- {}",
            issues.len(),
            issues.join("\n- ")
        ))
    }
}

fn markdown_files(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![directory.to_path_buf()];
    let mut files = Vec::new();
    while let Some(current) = pending.pop() {
        let entries = fs::read_dir(&current).map_err(|error| {
            format!(
                "document discovery stage failed for {}: {error}",
                current.display()
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "document discovery stage failed for {}: {error}",
                    current.display()
                )
            })?;
            let file_type = entry.file_type().map_err(|error| {
                format!(
                    "document discovery stage failed for {}: {error}",
                    entry.path().display()
                )
            })?;
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some("md")
            {
                files.push(entry.path());
            }
        }
    }
    files.sort();
    Ok(files)
}

#[derive(Clone, Copy)]
enum DefinitionOwner {
    Module,
    Nfr,
    Ux,
}

impl DefinitionOwner {
    fn owns(self, id: &str) -> bool {
        match self {
            Self::Module => id.starts_with('M'),
            Self::Nfr => id.starts_with("NFR-"),
            Self::Ux => id.starts_with("UX-"),
        }
    }
}

fn definition_owner(root: &Path, path: &Path) -> Option<DefinitionOwner> {
    let relative = path.strip_prefix(root).ok()?;
    let normalized = relative.to_string_lossy().replace('\\', "/");
    if normalized.starts_with("docs/modules/M") && normalized.ends_with(".md") {
        Some(DefinitionOwner::Module)
    } else if normalized == "docs/20-nonfunctional.md" {
        Some(DefinitionOwner::Nfr)
    } else if normalized == "docs/21-ui-ux.md" {
        Some(DefinitionOwner::Ux)
    } else {
        None
    }
}

fn count_definitions(definitions: &BTreeSet<String>) -> Counts {
    let mut counts = Counts::default();
    for id in definitions {
        if id.starts_with("NFR-") {
            counts.nfr += 1;
        } else if id.starts_with("UX-") {
            counts.ux += 1;
        } else if id.starts_with('M') {
            counts.modules += 1;
        }
    }
    counts
}

fn contains_placeholder(line: &str) -> bool {
    line.contains("TBD") || line.contains("\u{5f85}\u{8865}\u{5145}")
}

fn is_placeholder_self_check(line: &str) -> bool {
    matches!(
        line.trim(),
        "grep -rn \"TBD\\|\u{5f85}\u{8865}\u{5145}\" docs/ --include=\"*.md\""
            | "grep -rn \"TBD\\|\u{5f85}\u{8865}\u{5145}\" docs/ --include=\"*.md\" | grep -v \"grep -rn\""
    )
}

fn run_anchor_checker(root: &Path) -> Result<AnchorOutput, String> {
    let script = root.join(".trellis/scripts/check_anchors.py");
    if !script.is_file() {
        return Err(format!("anchor checker is missing: {}", script.display()));
    }
    let output = Command::new("python")
        .arg(&script)
        .current_dir(root)
        .output()
        .map_err(|error| format!("could not execute Python for {}: {error}", script.display()))?;
    Ok(AnchorOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[derive(Clone, Debug)]
struct Location {
    path: String,
    line: usize,
}

impl Location {
    fn display(&self) -> String {
        format!("{}:{}", self.path, self.line)
    }
}

#[cfg(test)]
mod tests {
    use super::{AnchorOutput, Counts, validate_document_tree};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    const SMALL_COUNTS: Counts = Counts {
        modules: 1,
        nfr: 1,
        ux: 1,
    };

    #[test]
    fn small_document_tree_accepts_definitions_references_and_self_check_lines() {
        let tree = TestDocuments::valid();
        let counts = validate_document_tree(tree.path(), SMALL_COUNTS, |_| Ok(anchor_success()))
            .expect("valid document tree must pass");
        assert_eq!(counts, SMALL_COUNTS);
    }

    #[test]
    fn duplicate_count_dangling_and_placeholder_problems_are_aggregated() {
        let tree = TestDocuments::valid();
        tree.append(
            "docs/modules/M1-test.md",
            "| `M1-TEST-001` | duplicate |\n| `M1-TEST-999` | TBD |\n",
        );
        tree.append("docs/README.md", "See `NFR-MISSING-999`.\n");

        let error = validate_document_tree(tree.path(), SMALL_COUNTS, |_| Ok(anchor_success()))
            .expect_err("all document invariant violations must fail");

        assert!(error.contains("duplicate definition M1-TEST-001"));
        assert!(error.contains("definition count drift"));
        assert!(error.contains("dangling requirement reference NFR-MISSING-999"));
        assert!(error.contains("placeholder found"));
    }

    #[test]
    fn invalid_and_wrongly_owned_definition_ids_are_rejected() {
        let tree = TestDocuments::valid();
        tree.append(
            "docs/modules/M1-test.md",
            "| `M1-TEST-01` | too short |\n| `M1-TEST-001-extra` | trailing text |\n| `NFR-TEST-001` | wrong owner |\n",
        );

        let error = validate_document_tree(tree.path(), SMALL_COUNTS, |_| Ok(anchor_success()))
            .expect_err("invalid definition IDs must fail");

        assert!(error.contains("invalid requirement definition ID M1-TEST-01"));
        assert!(error.contains("invalid requirement definition ID M1-TEST-001-extra"));
        assert!(error.contains("requirement definition NFR-TEST-001 has the wrong document owner"));
    }

    #[test]
    fn placeholder_self_check_exemption_does_not_hide_extra_placeholder_text() {
        let tree = TestDocuments::valid();
        tree.append(
            "docs/README.md",
            "grep -rn \"TBD\\|\u{5f85}\u{8865}\u{5145}\" docs/ --include=\"*.md\" && echo TBD\n",
        );

        let error = validate_document_tree(tree.path(), SMALL_COUNTS, |_| Ok(anchor_success()))
            .expect_err("a self-check prefix must not exempt additional placeholder text");
        assert!(error.contains("placeholder found"));
    }

    #[test]
    fn count_drift_is_reported_without_a_dangling_false_positive() {
        let tree = TestDocuments::valid();
        let error = validate_document_tree(
            tree.path(),
            Counts {
                modules: 2,
                ..SMALL_COUNTS
            },
            |_| Ok(anchor_success()),
        )
        .expect_err("wrong expected count must fail");
        assert!(error.contains("definition count drift"));
        assert!(!error.contains("dangling requirement reference"));
    }

    #[test]
    fn anchor_nonzero_and_spawn_failures_preserve_evidence() {
        let tree = TestDocuments::valid();
        let nonzero = validate_document_tree(tree.path(), SMALL_COUNTS, |_| {
            Ok(AnchorOutput {
                success: false,
                stdout: "1 broken anchor".to_owned(),
                stderr: "anchor detail".to_owned(),
            })
        })
        .expect_err("nonzero anchor result must fail");
        assert!(nonzero.contains("1 broken anchor"));
        assert!(nonzero.contains("anchor detail"));

        let spawn = validate_document_tree(tree.path(), SMALL_COUNTS, |_| {
            Err("python unavailable".to_owned())
        })
        .expect_err("anchor spawn error must fail");
        assert!(spawn.contains("python unavailable"));
    }

    #[test]
    fn live_repository_has_the_approved_definition_counts() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask must have a repository parent")
            .to_path_buf();
        let counts =
            validate_document_tree(&root, super::EXPECTED_COUNTS, |_| Ok(anchor_success()))
                .expect("live document definitions must satisfy the approved counts");
        assert_eq!(counts.total(), 630);
    }

    fn anchor_success() -> AnchorOutput {
        AnchorOutput {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    struct TestDocuments(PathBuf);

    impl TestDocuments {
        fn valid() -> Self {
            let path = std::env::temp_dir().join(format!(
                "wubilex-xtask-docs-{}-{}",
                std::process::id(),
                TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("test directory must be unique");
            let tree = Self(path);
            tree.write(
                "docs/modules/M1-test.md",
                "# Module\n\n| ID | Text |\n|---|---|\n| `M1-TEST-001` | module |\n",
            );
            tree.write(
                "docs/20-nonfunctional.md",
                "| `NFR-A11Y-001` | nonfunctional |\n",
            );
            tree.write("docs/21-ui-ux.md", "| `UX-TEST-001` | UX |\n");
            tree.write(
                "docs/README.md",
                "Refs: `M1-TEST-001`, `NFR-A11Y-001`, `UX-TEST-001`.\n\ngrep -rn \"TBD\\|\u{5f85}\u{8865}\u{5145}\" docs/ --include=\"*.md\"\n",
            );
            tree
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }

        fn write(&self, relative: &str, text: &str) {
            let target = self.0.join(relative);
            fs::create_dir_all(target.parent().expect("test file must have a parent"))
                .expect("test parent must be creatable");
            fs::write(target, text).expect("test document must be writable");
        }

        fn append(&self, relative: &str, text: &str) {
            use std::io::Write;
            let target = self.0.join(relative);
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(target)
                .expect("test document must exist");
            file.write_all(text.as_bytes())
                .expect("test document must be appendable");
        }
    }

    impl Drop for TestDocuments {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
