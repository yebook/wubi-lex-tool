use std::{collections::BTreeSet, fs, path::Path};

use regex::Regex;

const ACTION_PINS: [(&str, &str, &str); 4] = [
    (
        "actions/checkout",
        "11d5960a326750d5838078e36cf38b85af677262",
        "v4.4.0",
    ),
    (
        "actions/cache",
        "0057852bfaa89a56745cba8c7296529d2fc39830",
        "v4.3.0",
    ),
    (
        "volta-cli/action",
        "4047d4429228024f9852a9410f32b280e2c7f18f",
        "v4.1.0",
    ),
    (
        "taiki-e/install-action",
        "6cd13508893c0e7eab5f273c2575d3859bd7229a",
        "v2.86.6",
    ),
];

fn contains_forbidden_package_manager_token(command: &str) -> bool {
    Regex::new(r"(?i)(?:^|[^a-z0-9_-])(?:npm|yarn|npx|corepack)(?:$|[^a-z0-9_-])")
        .expect("forbidden package manager regex must compile")
        .is_match(command)
}

#[test]
fn windows_quality_workflow_has_the_complete_fail_closed_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must have a repository parent");
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("the Windows quality workflow must exist");
    let package_text = fs::read_to_string(root.join("package.json"))
        .expect("the root package manifest must exist");
    let package: serde_json::Value =
        serde_json::from_str(&package_text).expect("package.json must be valid JSON");
    let volta = package
        .get("volta")
        .and_then(serde_json::Value::as_object)
        .expect("package.json.volta must be an object");

    let volta_keys = volta.keys().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(
        volta_keys,
        BTreeSet::from(["node", "pnpm"]),
        "package.json.volta must contain only the Node and pnpm version sources"
    );
    for key in ["node", "pnpm"] {
        let version = volta
            .get(key)
            .and_then(serde_json::Value::as_str)
            .expect("Volta pins must be strings");
        assert!(!version.trim().is_empty(), "volta.{key} must not be empty");
    }
    assert!(
        package
            .get("engines")
            .and_then(|engines| engines.get("pnpm"))
            .is_none(),
        "package.json.engines.pnpm is forbidden"
    );
    assert!(
        package.get("packageManager").is_none(),
        "package.json.packageManager is forbidden"
    );
    for (name, command) in package
        .get("scripts")
        .and_then(serde_json::Value::as_object)
        .expect("package.json.scripts must be an object")
    {
        let command = command
            .as_str()
            .expect("package.json script commands must be strings");
        assert!(
            !contains_forbidden_package_manager_token(command),
            "package.json script {name} uses a forbidden package manager"
        );
    }

    for required in [
        "push:",
        "branches:\n      - main",
        "pull_request:",
        "workflow_dispatch:",
        "contents: read",
        "group: ${{ github.workflow }}-${{ github.ref }}",
        "cancel-in-progress: true",
        "runs-on: windows-latest",
        "package.json",
        "VOLTA_FEATURE_PNPM: \"1\"",
        "$package.volta.pnpm",
        "$package.engines.PSObject.Properties['pnpm']",
        "rust-toolchain.toml",
        "hashFiles('Cargo.lock')",
        "hashFiles('pnpm-lock.yaml')",
        "hashFiles('crates/wubilex-codec/tests/fixtures/manifest.json')",
        "cargo-llvm-cov@0.9.0",
        "cargo-deny@0.20.2",
    ] {
        assert!(
            workflow.contains(required),
            "missing workflow contract: {required}"
        );
    }

    for forbidden in [
        "continue-on-error",
        "pnpm/action-setup",
        "corepack",
        "packageManager",
        "24.18.1",
        "11.18.0",
        "1.97.1",
    ] {
        assert!(
            !workflow.contains(forbidden),
            "forbidden workflow contract: {forbidden}"
        );
    }

    assert_eq!(
        workflow.matches("uses: actions/cache@").count(),
        3,
        "Cargo, pnpm and fixture caches must be separate"
    );

    let uses_pattern = Regex::new(r"(?m)^\s*uses:\s+([^\s@]+)@([0-9a-f]{40})\s+#\s+(v[^\s]+)\s*$")
        .expect("action regex must compile");
    let actual_actions = uses_pattern
        .captures_iter(&workflow)
        .map(|capture| {
            (
                capture[1].to_owned(),
                capture[2].to_owned(),
                capture[3].to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual_actions.len(),
        workflow.matches("uses:").count(),
        "every action must use a full commit SHA and release comment"
    );
    let expected_actions = ACTION_PINS
        .into_iter()
        .map(|(action, sha, release)| (action.to_owned(), sha.to_owned(), release.to_owned()))
        .collect::<BTreeSet<_>>();
    let distinct_actions = actual_actions.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(distinct_actions, expected_actions);

    let command_order = [
        "cargo fmt --all -- --check",
        "cargo check --workspace --all-targets --all-features",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo xtask fixtures\n",
        "cargo xtask fixtures --check",
        "cargo test --workspace --all-features",
        "cargo doc --workspace --all-features --no-deps",
        "cargo llvm-cov --package wubilex-codec --all-features --summary-only --fail-under-lines 90",
        "cargo deny check",
        "cargo xtask bindings --check",
        "cargo xtask check-docs",
        "pnpm install --frozen-lockfile",
        "pnpm audit --audit-level high",
        "pnpm typecheck",
        "pnpm lint",
        "pnpm test --run",
    ];
    let mut previous = 0;
    for command in command_order {
        let position = workflow
            .find(command)
            .unwrap_or_else(|| panic!("missing required command: {command}"));
        assert!(position >= previous, "gate is out of order: {command}");
        previous = position;
    }

    assert!(!contains_forbidden_package_manager_token(&workflow));
    for bypass in [
        "--no-fail-fast",
        "--ignore-rust-version",
        "--exclude-from-test",
        "--ignore-filename-regex",
        "|| true",
        "exit 0",
    ] {
        assert!(
            !workflow.contains(bypass),
            "workflow bypass found: {bypass}"
        );
    }
}

#[test]
fn forbidden_package_manager_detection_covers_wrapped_and_mixed_case_commands() {
    for command in [
        "npm run test",
        "cmd /c \"NPM run test\"",
        "powershell -Command corepack enable",
        "tool && yarn lint",
        "npx eslint .",
    ] {
        assert!(
            contains_forbidden_package_manager_token(command),
            "{command}"
        );
    }

    for command in ["pnpm run test", "minimum", "npm-run-all"] {
        assert!(
            !contains_forbidden_package_manager_token(command),
            "{command}"
        );
    }
}
