# S0 xtask And CI Research

## Repository Baseline

- No `.github/workflows/` or `deny.toml` exists.
- `xtask` currently exposes only `fixtures [--check]` and has six passing tests.
- `src-tauri` is a compile-only library with Tauri 2.11.5, `default-features = false`, no commands and an empty `src/bindings/` scaffold.
- `src/types/generated/` contains only `.gitkeep`.
- The completed fixture task established eight manifest-driven real samples, 130 codec tests and 90.12% codec line coverage with cargo-llvm-cov 0.9.0.
- Baseline SHA-256: Cargo.lock `a2f228761124fb1a3b48a2047641fd43c114ef9bcaacf0a437fceb98f7e7f506`; pnpm-lock.yaml `79e9717c72fcfc226afcec89bd7c3b91bde9b5ffa56a81cb8c032e13b2e5d702`.
- Tool baseline: Node 24.18.1, global pnpm 11.18.0, Cargo/Rust 1.97.1, Python 3.12.13.

## Authoritative Requirements

- `docs/02-architecture.md` D11 makes Rust/tauri-specta the IPC source of truth and requires committed TypeScript plus `xtask bindings --check`.
- D17 and section 8.5 make `package.json.volta.node`, `package.json.engines.pnpm` and `rust-toolchain.toml` the only Node/pnpm/Rust version sources. pnpm remains a directly invoked global command.
- The approved CI order is fail-fast: Rust format/lint/test, dependency audit, bindings/docs, frozen frontend install and TypeScript/ESLint/Vitest.
- `NFR-MAINT-002/003/006` require 90% codec coverage, eight real schemes and compile/Clippy/fmt/test/audit CI.
- `NFR-SEC-013` requires dependency vulnerability scanning. The project-level Rust tool is cargo-deny; the npm ecosystem equivalent must be invoked as `pnpm audit`, never `npm audit`.
- `.trellis/spec/guides/requirement-id-conventions.md` fixes counts at 414/101/115/630 and defines uniqueness, dangling-reference, placeholder and anchor checks.

## Binding Compatibility

Crates.io research on 2026-08-24 found:

| Crate | Version | License | Relevant contract |
|---|---:|---|---|
| `tauri-specta` | `2.0.0-rc.25` | MIT | Depends on Tauri 2 with defaults off; `typescript` feature; documents empty command/event collectors and generic `Builder<R: Runtime>` export |
| `specta-typescript` | `0.0.12` | MIT | TypeScript exporter used by tauri-specta |
| `tauri` | `2.11.5` | MIT OR Apache-2.0 | Existing exact dependency; MSRV 1.77.2; empty `test` feature exposes `MockRuntime` without enabling `wry` |

This satisfies architecture R54 without selecting the `ts-rs` fallback. The registry can be generic over `tauri::Runtime`; xtask exports through `tauri::test::MockRuntime`, and future application startup can use the same registry with Wry. No fake command is needed to make the output nonempty.

Windows `core.autocrlf=true` makes a raw generated-file comparison unstable unless generated TypeScript has an explicit EOL contract. A narrow `.gitattributes` rule for `src/types/generated/*.ts text eol=lf` is the preferred solution.

## Document Gate Findings

Current unique counts were measured with the corrected digit-aware grammar:

| Category | Count |
|---|---:|
| Modules M1..M8 | 414 |
| NFR | 101 |
| UX | 115 |
| Total | 630 |

The existing `.trellis/scripts/check_anchors.py` is verified project code with two non-obvious GitHub slug/path fixes. Reimplementing it inside xtask would create two sources of truth. `check-docs` should implement ID/count/reference/placeholder checks in Rust, then execute the existing script and propagate its exit status.

The placeholder examples in docs intentionally contain their own `grep -rn` command. Only those self-check lines may be excluded; a broad file or code-block exclusion could hide a real placeholder.

## Audit Findings

- `cargo-deny 0.20.2` is MIT OR Apache-2.0 and declares MSRV 1.88.0, compatible with Rust 1.97.1. It can cover advisories, licenses, bans and sources in one policy.
- The current developer npm registry is `npmmirror`, whose advisory bulk endpoint returns `ERR_PNPM_AUDIT_ENDPOINT_NOT_EXISTS`. A one-command `--registry=https://registry.npmjs.org` diagnostic returned `No known vulnerabilities found` without changing config.
- Do not commit an `.npmrc` solely for audit. GitHub-hosted CI uses the default official registry. Local documentation should distinguish dependency findings from a mirror missing the endpoint.
- cargo-deny is not installed globally. Implementation must install the exact version under ignored `target/tools`; CI installation is ephemeral.

## CI Decisions

- Use a single Windows job because the repository pins the MSVC target and later Windows-specific work will extend this baseline. This task does not execute real system mutations.
- Trigger pull requests, `main` pushes and manual dispatch. Grant only `contents: read` and cancel stale same-ref runs.
- Pin all third-party actions to full audited commit SHAs with release comments. Resolve and record the actual tag-to-SHA mapping during implementation rather than guessing it in planning.
- Use the approved Volta action for Node, read pnpm version from `engines.pnpm` into a step output, and pass it to `pnpm/action-setup`. This prepares a global command without creating a project-level pnpm pin.
- Rustup honors `rust-toolchain.toml`; no workflow Rust version is allowed. Cargo and pnpm caches are keyed by their lockfiles. Fixture cache is keyed by OS and manifest hash, but both default preparation and offline verification still run.
- Install exact cargo-llvm-cov 0.9.0 and cargo-deny 0.20.2 in the ephemeral runner. Do not use a floating tool version.
- No `continue-on-error`, skipped real fixture test, broad advisory ignore, or coverage/source exclusion is permitted.

## Deferred Work

- `xtask resources`, `xtask licenses`, product license UI and resource manifest remain their owning later tasks.
- Real command/event bindings and frontend consumers begin in S1; this task establishes the generator and empty canonical baseline only.
- GitHub deployment, release artifacts, signing, system integration and isolated Windows VM tests remain outside this workflow.
- Independent aardio golden comparison remains `s0-integration` evidence.

## Implemented Evidence

### Binding And Command Baseline

- The final strict command parser accepts only the five documented forms and has Windows coverage for non-Unicode arguments.
- `wubilex-app::bindings::builder<R: tauri::Runtime>()` is the single registry. `export_mock` uses `tauri::test::MockRuntime`; `cargo tree` and the independent review confirmed that `wry` is absent.
- The exact compatible dependencies are `tauri-specta = 2.0.0-rc.25`, `specta = 2.0.0-rc.25` with function support, and `specta-typescript = 0.0.12`.
- The empty registry produces a real 156-byte LF TypeScript baseline. Repeated generation is byte-identical; `--check` rejects mutation without changing the target.
- The final xtask suite has 17 passing tests. Independent review added malformed/wrong-owner requirement-definition cases and narrowed placeholder self-check handling to two exact command lines.

### Immutable Action Pins

The release tags resolve to these workflow commits:

| Action | Release | Commit |
|---|---|---|
| `actions/checkout` | `v4.4.0` | `11d5960a326750d5838078e36cf38b85af677262` |
| `actions/cache` | `v4.3.0` | `0057852bfaa89a56745cba8c7296529d2fc39830` |
| `volta-cli/action` | `v4.1.0` | `4047d4429228024f9852a9410f32b280e2c7f18f` |
| `pnpm/action-setup` | `v4.1.0` | `a7487c7e89a18df4991f7f222e4898a00d66ddda` |
| `taiki-e/install-action` | `v2.86.6` | `6cd13508893c0e7eab5f273c2575d3859bd7229a` |

Annotated tags use their peeled commit, not the tag-object SHA. Fresh `git ls-remote` verification succeeded for checkout, cache, Volta, and install-action. The pnpm tag was verified against the fetched upstream clone after repeated GitHub TLS EOF responses; its annotated tag object is `7088e561eb65bb68695d245aa206f005ef30921d` and peels to the workflow SHA above.

### Dependency Policy

- Final `Cargo.lock` SHA-256 is `1a831480272d3bc8bb187f696346f058f4507edb8685079400526858213f24de`; the pnpm lock remains `79e9717c72fcfc226afcec89bd7c3b91bde9b5ffa56a81cb8c032e13b2e5d702`.
- `cargo-deny 0.20.2 check` passes advisories, bans, licenses, and sources. Duplicate dependency versions remain warnings for reviewed transitive stacks.
- The allowed licenses are Apache-2.0, BSD-2-Clause, BSD-3-Clause, MIT, MPL-2.0, Unicode-3.0, and Zlib.
- Six exact unmaintained-only advisories are ignored with transitive dependency and review reasons: `RUSTSEC-2024-0436`, `RUSTSEC-2025-0075`, `RUSTSEC-2025-0080`, `RUSTSEC-2025-0081`, `RUSTSEC-2025-0098`, and `RUSTSEC-2025-0100`. No vulnerability, unsound, or yanked advisory is hidden, and no broad crate skip is present.

### Final Quality Evidence

- Rust fmt, workspace check, strict Clippy, all workspace tests, and warnings-denied Rustdoc pass. Results include 130 codec tests and 17 xtask tests.
- All eight fixture pairs pass default preparation and offline verification. Codec line coverage is 90.12% with exact workspace-local cargo-llvm-cov 0.9.0 and `--fail-under-lines 90`.
- `cargo xtask bindings`, `bindings --check`, and `check-docs` pass. Document output is modules 414, NFR 101, UX 115, total 630, with zero dangling IDs, placeholders, or broken anchors.
- actionlint 1.7.7, Trellis task validation, the independent anchor command, forbidden-pattern workflow tests, and `git diff --check` pass.
- Global pnpm 11.18.0 frozen install, typecheck, lint, and Vitest pass. The configured npmmirror lacks an audit endpoint; the command-local official registry audit reports no known vulnerabilities and no repository npm configuration was changed.
- The independent Trellis check restored the pre-existing journal `merge=union` attribute and fixed document validation that could previously ignore malformed or wrongly owned definition IDs. All affected gates were rerun after those fixes.
- Root `resource/` was not read. No release, signing, product resource, Wry runtime, fake IPC command, or global Cargo-tool mutation was introduced.
