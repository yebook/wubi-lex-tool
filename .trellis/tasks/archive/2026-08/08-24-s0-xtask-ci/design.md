# Design - S0 xtask And CI Gates

## 1. Boundary And Layout

This task extends repository tooling and continuous integration, not product behavior:

```text
.github/workflows/ci.yml             # one Windows quality workflow
.gitattributes                       # generated TypeScript uses LF
deny.toml                            # Cargo advisory/license/ban/source policy
xtask/src/main.rs                    # strict command dispatch
xtask/src/bindings.rs                # generate or non-mutating freshness check
xtask/src/check_docs.rs              # document invariant checks
src-tauri/src/bindings/mod.rs        # generic tauri-specta registry/export entry
src/types/generated/bindings.ts      # committed generated baseline
```

`xtask` may depend on `wubilex-app` to call the Rust-owned binding registry. It remains repository automation. No dependency or implementation is added to the pure production codec crates.

## 2. Command Contract

The parser accepts exactly:

```text
cargo xtask fixtures
cargo xtask fixtures --check
cargo xtask bindings
cargo xtask bindings --check
cargo xtask check-docs
```

The two default commands may write only their owned generated/cache targets. Both `--check` modes are non-mutating. Every other argument vector fails with a complete usage string and exit code 1. Repository paths derive from `CARGO_MANIFEST_DIR`, so commands work from any current directory.

## 3. Binding Generation

`src-tauri/src/bindings/mod.rs` owns one `builder<R: tauri::Runtime>() -> tauri_specta::Builder<R>` registry. At the S0 baseline it collects empty command and event lists; this is a real schema state, not a placeholder command.

The exact compatible stack is `tauri-specta = 2.0.0-rc.25` with its `typescript` feature and `specta-typescript = 0.0.12`. The upstream manifest depends on Tauri 2 with default features disabled and explicitly documents empty collectors. `wubilex-app` enables Tauri's empty `test` feature only so xtask can instantiate `tauri::test::MockRuntime`; it does not enable `wry`.

Generation flow:

```text
generic Rust registry
  -> Builder<MockRuntime>
  -> Typescript exporter to unique target/ temp file
  -> normalize/require LF
  -> default: replace generated bindings.ts
  -> --check: byte-compare committed file, remove temp, fail on drift
```

`.gitattributes` fixes `src/types/generated/*.ts text eol=lf`, avoiding Windows checkout differences. `--check` never repairs. A generated header identifies the file as machine-owned. Future S1 runtime registration must reuse the same builder rather than maintain another command list.

The `ts-rs` fallback is not selected because the researched tauri-specta version supports the current Tauri 2 contract. If compilation or output integration disproves that evidence, planning returns for an R54 decision before changing generators.

## 4. Document Validation

`check_docs.rs` walks sorted `docs/**/*.md` files and uses the fixed requirement grammar `(M[1-8]|NFR|UX)-[A-Z0-9]+-[0-9]{3}`. It distinguishes definition rows from references, reports duplicate definitions with paths/lines, asserts category counts 414/101/115 and total 630, then rejects referenced IDs without a definition.

Placeholder scanning rejects `TBD` and `待补充` while excluding only the documented self-check command lines. It does not grow a general ignore list. Anchor validation delegates to the already verified `.trellis/scripts/check_anchors.py` and propagates missing Python, spawn failure, nonzero exit, stdout, and stderr as a failed `check-docs` stage. This reuses the GitHub-slug and normalized-path behavior instead of cloning it in Rust.

Unit tests operate on bounded temporary document trees and injectable anchor-runner outcomes. The repository integration test asserts the complete live baseline.

## 5. Dependency Policy

`deny.toml` uses cargo-deny configuration compatible with 0.20.2:

- deny known vulnerabilities, unsound advisories, yanked crates, unknown registries, and git sources not explicitly approved;
- allow only licenses observed in the reviewed lockfile and compatible with distribution policy;
- warn on duplicate versions and unmaintained/notice advisories unless a concrete risk requires denial;
- any exception is exact crate/version/advisory with a reason and review note.

Implementation installs `cargo-deny 0.20.2` and any missing local verifier under ignored `target/tools`, never globally. CI is ephemeral and installs the same exact cargo-deny and cargo-llvm-cov versions through immutable actions.

## 6. CI Workflow

One `windows-latest` job matches the pinned MSVC toolchain and future platform direction. It runs for pull requests, pushes to `main`, and `workflow_dispatch`, with `contents: read`, no secrets, and concurrency cancellation per workflow/ref.

All external actions use reviewed full commit SHAs with release comments. The setup flow is:

1. Checkout.
2. Install Volta/Node from `package.json.volta.node` through the approved Volta action.
3. Read `package.json.engines.pnpm` into a step output and let `pnpm/action-setup` expose that version as the global `pnpm` command.
4. Let rustup/Cargo honor `rust-toolchain.toml`; cache Cargo data by `Cargo.lock` without another Rust version field.
5. Install exact cargo-llvm-cov 0.9.0 and cargo-deny 0.20.2 in the ephemeral runner.
6. Restore fixture cache keyed by OS plus `tests/fixtures/manifest.json` hash.

Gate sequence:

```text
verify tool versions and forbidden version sources
  -> cargo fmt --check
  -> cargo check --workspace --all-targets --all-features
  -> cargo clippy --workspace --all-targets --all-features -- -D warnings
  -> cargo xtask fixtures
  -> cargo xtask fixtures --check
  -> cargo test --workspace --all-features
  -> warnings-denied cargo doc --workspace --all-features --no-deps
  -> cargo llvm-cov -p wubilex-codec --all-features --fail-under-lines 90
  -> cargo deny check
  -> cargo xtask bindings --check
  -> cargo xtask check-docs
  -> pnpm install --frozen-lockfile
  -> pnpm audit --audit-level high
  -> pnpm typecheck
  -> pnpm lint
  -> pnpm test --run
```

No gate has `continue-on-error`. Cache hits never replace verification. The workflow does not read root `resource/`, execute product system operations, publish artifacts, or receive credentials.

## 7. Verification Strategy

Implementation first records baseline hashes and adds failing command/parser/generator/doc-policy tests. It then validates the new commands locally, installs exact workspace-local audit tools, and runs the same sequence as CI. The workflow receives static checks for YAML parseability, full action SHA pins, permissions, dynamic version reads, cache keys, forbidden commands, and absence of weakening flags.

The final independent review checks both layers: commands must implement the contracts, and the YAML must actually call them in the required order. Merely containing expected strings is not sufficient evidence if a condition or tolerated failure bypasses a gate.

## 8. Rollback Shape

Binding generation, document validation, dependency policy, and CI workflow are separate review units. A tauri-specta incompatibility rolls back only binding files and returns to the documented R54 decision. A transient fixture/advisory service failure does not justify weakening the manifest, coverage threshold, or security policy. Workflow failure may be debugged on a branch or manual run; `main` gates remain fail-closed.
