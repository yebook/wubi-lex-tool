# Repository Quality And CI

> Executable contracts for repository commands, generated bindings, document validation, dependency policy, and the Windows quality workflow.

---

## Scenario: Repository Commands And Windows Quality CI

### 1. Scope / Trigger

Apply this contract when changing the `xtask` command surface, Rust-to-TypeScript binding registry, requirement documents, dependency policy, toolchain version sources, or `.github/workflows/ci.yml`. These are repository-quality boundaries; they must not acquire product resource, release, signing, or Windows system-mutation behavior.

### 2. Signatures

```text
cargo xtask fixtures
cargo xtask fixtures --check
cargo xtask bindings
cargo xtask bindings --check
cargo xtask check-docs
```

Only these five argument vectors are valid. Any missing, duplicate, extra, unknown, or non-Unicode argument fails with the complete usage text.

```rust
pub fn builder<R: tauri::Runtime>() -> tauri_specta::Builder<R>;
pub fn export_mock(path: impl AsRef<Path>) -> Result<(), specta_typescript::Error>;
```

### 3. Contracts

- Repository paths derive from `xtask`'s `CARGO_MANIFEST_DIR`, so commands are independent of the caller's current directory.
- `src-tauri/src/bindings/mod.rs` is the only command/event registry. It remains generic over `tauri::Runtime`; repository export uses `MockRuntime`, exact `tauri-specta 2.0.0-rc.25` and `specta-typescript 0.0.12`, with no `wry` feature and no fake command.
- `cargo xtask bindings` exports through an ignored unique temporary file, normalizes to UTF-8 LF ending in a newline, and stages replacement of `src/types/generated/bindings.ts`. `--check` byte-compares canonical output without changing the committed target.
- `cargo xtask check-docs` scans sorted `docs/**/*.md`, accepts definition IDs only in their owning module/NFR/UX documents, requires unique counts `414/101/115/630`, rejects dangling IDs and real `TBD` or `待补充` placeholders, and delegates anchors to `.trellis/scripts/check_anchors.py`.
- `.github/workflows/ci.yml` runs on pull requests, `main` pushes, and manual dispatch on `windows-latest`, with `contents: read`, per-ref concurrency cancellation, a finite timeout, and no secrets or release steps. Every third-party action is pinned to a reviewed full commit SHA with its release tag in a comment.
- Node, pnpm, and Rust versions come only from `package.json.volta.node`, `package.json.engines.pnpm`, and `rust-toolchain.toml`. CI exposes pnpm as the global command. Do not add `volta.pnpm`, corepack, `packageManager`, `.nvmrc`, npm, yarn, or npx.
- Cargo and pnpm caches are keyed by their lockfiles. The fixture cache is keyed by runner OS plus the fixture manifest digest. A cache hit never replaces preparation, offline verification, tests, coverage, audit, or freshness checks.
- `deny.toml` runs cargo-deny 0.20.2 advisories, licenses, bans, and sources. Advisory ignores are exact reviewed IDs with reasons; licenses are limited to the current lockfile; broad crate skips, git sources, unknown registries, and wildcard workspace dependencies remain forbidden.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Argument vector is not one of the five signatures | Exit nonzero and print all supported forms |
| Generated binding is missing or stale in `--check` | Exit nonzero, leave the target byte-identical, and print `cargo xtask bindings` |
| Binding export, normalization, staging, or replacement fails | Preserve the failing stage and path; do not report freshness success |
| Requirement definition is malformed, duplicated, wrongly owned, missing, dangling, or contains a placeholder | Aggregate every independently detectable issue and exit nonzero |
| Python or anchor checking fails | Preserve spawn/stdout/stderr evidence and fail `check-docs` |
| Codec coverage is below 90%, an audit denies, or a tool version differs | The CI step exits nonzero; no `continue-on-error` or fallback success |
| A local npm mirror has no audit endpoint | Diagnose with command-local official `--registry`; never commit mirror settings or treat the missing endpoint as a clean audit |

### 5. Good / Base / Bad Cases

- Good: a Rust IPC change updates the single registry, regenerates committed LF TypeScript, and passes Rust tests, `bindings --check`, TypeScript, and workflow checks.
- Base: the empty S0 command/event registry exports a deterministic non-handwritten TypeScript baseline and passes freshness checks without enabling a window runtime.
- Bad: a fake command is added to avoid empty output, a generated file is hand-edited, a requirement row is ignored because its ID is malformed, or CI skips verification after a cache hit.

### 6. Tests Required

- Parser tests accept exactly the five forms and reject invalid Unicode and every extra/unknown argument with full usage.
- Binding tests prove repeat generation is byte-identical and LF-only; mutation causes `--check` to fail without repair.
- Document tests cover valid trees, malformed/wrong-owner and duplicate definitions, count drift, dangling references, placeholders, anchor nonzero output, spawn failure, and the live `414/101/115/630` baseline.
- Workflow contract tests parse the checked-in YAML and assert triggers, permissions, concurrency, exact action SHAs, version sources, cache keys, step order, and forbidden bypass/package-manager patterns. Run actionlint as an independent syntax check.
- The final local gate mirrors CI: fmt, check, strict Clippy, fixtures, tests, Rustdoc, coverage, cargo-deny, bindings/docs, frozen pnpm install, audit, typecheck, lint, and Vitest.

### 7. Wrong vs Correct

```yaml
# Wrong: a floating action and a tolerated security gate.
- uses: actions/checkout@v4
- run: cargo deny check
  continue-on-error: true

# Correct: reviewed immutable action code and fail-closed audit.
- uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0
- run: cargo deny check
```

## Sources

- [`xtask` command parser](../../../xtask/src/main.rs)
- [`xtask` binding generator](../../../xtask/src/bindings.rs)
- [`xtask` document validator](../../../xtask/src/check_docs.rs)
- [Canonical Tauri binding registry](../../../src-tauri/src/bindings/mod.rs)
- [Windows quality workflow](../../../.github/workflows/ci.yml)
- [Cargo dependency policy](../../../deny.toml)
- [Frontend toolchain version sources](../../../package.json)
