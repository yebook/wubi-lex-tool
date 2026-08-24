use std::{collections::BTreeSet, fs, path::Path};

use regex::Regex;

const ACTION_PINS: [(&str, &str, &str); 5] = [
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
        "pnpm/action-setup",
        "a7487c7e89a18df4991f7f222e4898a00d66ddda",
        "v4.1.0",
    ),
    (
        "taiki-e/install-action",
        "6cd13508893c0e7eab5f273c2575d3859bd7229a",
        "v2.86.6",
    ),
];

#[test]
fn windows_quality_workflow_has_the_complete_fail_closed_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must have a repository parent");
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("the Windows quality workflow must exist");

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
        "engines.pnpm",
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
        "corepack",
        "volta.pnpm",
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

    let forbidden_command = Regex::new(r"(?m)^\s+(?:npm|yarn|npx|corepack)(?:\s|$)")
        .expect("forbidden command regex must compile");
    assert!(!forbidden_command.is_match(&workflow));
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
