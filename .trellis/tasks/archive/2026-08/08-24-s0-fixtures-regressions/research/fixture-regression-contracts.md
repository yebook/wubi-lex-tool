# Fixture And Regression Research

## Authoritative Contracts

- `docs/02-architecture.md` assigns real codec fixture acquisition to `cargo xtask fixtures`, places downloaded files under `crates/wubilex-codec/tests/fixtures/`, and forbids committing the binaries.
- `docs/22-roadmap.md` requires eight real schemes, byte-identical `.lex` round trips, the three S0 defect regressions, and at least 90% codec line coverage.
- `.trellis/spec/backend/quality-guidelines.md` forbids machine-local fixtures and silent skipping. It requires reproducibly fetched 86, 98, 06, 091, 092, Zhengma, Xiaohe, and Biaoxingma samples.
- `wubi-lex/lib/app/lexNetContents.aardio` is behavioral evidence for the legacy catalog names and paths. Its HTTP transport is explicitly not reusable because `NFR-SEC-003` requires HTTPS.

## Selected Upstream Samples

Planning performed HTTPS HEAD requests only; fixture bodies were not downloaded. All selected endpoints returned `200 OK` on 2026-08-24.

| Scheme | Legacy catalog item | HTTPS path | Compressed bytes |
|---|---|---|---:|
| 86 | Microsoft Wubi 86 minimal | `/download/lex/ChsWubi86.min.lex.lzma` | 46,580 |
| 98 | Wubi 98 minimal | `/download/lex/ChsWubi98.min.lex.lzma` | 155,720 |
| 06 | New Century minimal | `/download/lex/06.min.lex.lzma` | 146,934 |
| 091 | Dianer 091 full | `/download/lex/091.lex.lzma` | 2,226,131 |
| 092 | Wubi 092 regular | `/download/lex/092/092wb.lex.lzma` | 784,134 |
| Zhengma | Zhengma 6.6 | `/download/lex/zhengma/zhengma6.6.lex.lzma` | 417,307 |
| Xiaohe | Xiaohe minimal | `/download/lex/xhyx.min.lex.lzma` | 58,724 |
| Biaoxingma | Biaoxingma Wei | `/download/lex/bxm.wei.lex.lzma` | 516,574 |

The common origin is `https://wubi.aardio.com`; total compressed size is 4,352,104 bytes. The live `index.json` is 6,569 bytes and was last modified 2022-12-13, but its entries still contain HTTP URLs and no checksums. The implementation must use committed, reviewed HTTPS URLs and hashes rather than trust the live index.

## Verified Bootstrap And Fixture Results

Implementation completed the required two-pass bootstrap on 2026-08-24. The first pass fetched into an isolated temporary directory only to establish exact compressed/decompressed sizes and SHA-256 values; that directory was deleted. The second pass started from an empty fixture cache and used only the committed manifest plus `cargo xtask fixtures`. A subsequent warm-cache run with proxy variables removed reused all eight verified files, and `cargo xtask fixtures --check` completed without network access.

The native Rustls path could not complete TLS through the current standard environment proxy. The committed xtask therefore uses `ureq` with the system `native-tls` backend and `proxy-from-env`; it does not hard-code proxy hosts or credentials. The downloader still requires HTTPS for the initial and final URL, limits redirects to five, caps compressed input at 16 MiB and decompressed output at 64 MiB, and validates both size and SHA-256 before final placement.

All eight decoded files passed strict `.lex` decode, expected scheme detection, model validation, and byte-identical canonical re-encode:

| ID | `.lex` bytes | Entries | Distinct codes | `.lex` SHA-256 |
|---|---:|---:|---:|---|
| `wubi86` | 199,608 | 11,080 | 10,798 | `5073d519aabea4038e20b344c889a100ef1aa0011cfa59046e54978d6d73a22f` |
| `wubi98` | 583,038 | 32,381 | 26,027 | `bec634fcae2223b9a5b6ef3b22b39bf5e75f739a41d7e00a0f447eaf2e396359` |
| `wubi06` | 554,712 | 30,808 | 24,868 | `f181a9e0cef2455412f90a0285b3771e874cf5ac9fc01553e66d81f3b2befe83` |
| `wubi091` | 6,635,468 | 307,742 | 248,471 | `804592b48f7a364a62e399e4ac502c2bfa030790b195bccac4729df8ac18b5d0` |
| `wubi092` | 2,312,746 | 111,985 | 86,309 | `20a99e7b07654cf840f04e6b7b803416d651e96200fbfa6d37550d1cbb4d5bbc` |
| `zhengma` | 1,313,146 | 65,519 | 56,420 | `83d1218323dea83a3b96f1099ae9b266393a471d4e6d1402161ef8011baaf54c` |
| `xiaohe` | 230,284 | 12,765 | 11,557 | `a3de391041a15928140c9a666d062a255cea90307f158849c975fd8b689ddf9d` |
| `biaoxingma` | 1,491,916 | 72,582 | 56,755 | `599c428e7fb059b7f3c7467130420b5b9846ba715d27062c351c407c8fa2a8d9` |

No production codec compatibility defect was exposed. The independent aardio runtime/golden comparison remains deferred to `s0-integration` as planned.

## Integrity And Licensing

- Upstream URLs are not content-addressed. Pin compressed and decompressed SHA-256 plus exact sizes, then reject drift until a human reviews and updates the manifest.
- The `aardio/wubi-lex` source repository declares MIT. The online catalog does not provide a per-dictionary license. Do not infer that every third-party dictionary is MIT.
- This task downloads on demand for tests and commits no dictionary bytes. Record source and the missing per-item license evidence; product redistribution is a separate S5 decision.
- Root `resource/` is ignored user data. It is not a manifest source, cache, test fallback, or integrity oracle.

## Test Evidence Already Present

- 117 codec tests cover hand-authored `.lex` and EUDP bytes, all text dialects and outputs, phrase dialects, auxiliary tables, scheme branches, malformed fields, limits, and the `xfxy` / `%0B` / `%0C` regressions.
- The missing S0 defect test is an explicitly named regression proving removal of the dead `codeWeight[0]` branch does not discard valid text records.
- Existing exact text expectations are independent hand-written strings. No aardio runtime or upstream golden output is available locally.
- Approved scope decision: combine those exact strings with real-document semantic projections now; defer an independently generated aardio golden comparison to `s0-integration`.

## Tool And Dependency Findings

- `wubilex-codec` production dependencies did not change. Property/manifest support is dev-only; HTTP, TLS, LZMA, filesystem and hashing behavior remains in `xtask`.
- Exact direct additions and registry license metadata are:

| Package | Version | Scope | License |
|---|---:|---|---|
| `lzma-rs` | 0.3.0 | xtask | MIT |
| `serde` | 1.0.228 | xtask + codec dev | MIT OR Apache-2.0 |
| `serde_json` | 1.0.145 | xtask + codec dev | MIT OR Apache-2.0 |
| `sha2` | 0.10.9 | xtask + codec dev | MIT OR Apache-2.0 |
| `ureq` | 2.12.1 | xtask, `native-tls` + `proxy-from-env` only | MIT OR Apache-2.0 |
| `url` | 2.5.7 | xtask | MIT OR Apache-2.0 |
| `proptest` | 1.8.0 | codec dev, `std` only | MIT OR Apache-2.0 |

- Global `pnpm` remains the only pnpm invocation route.
- `cargo-llvm-cov 0.9.0` is installed only under ignored `target/tools/cargo-llvm-cov-0.9.0/`; `llvm-tools-preview` is installed for Rust 1.97.1. The next CI task still owns runner installation and caching.
- The post-change `Cargo.lock` SHA-256 is `a2f228761124fb1a3b48a2047641fd43c114ef9bcaacf0a437fceb98f7e7f506`.

## Regression And Coverage Results

- `wubilex-codec` now runs 130 tests: synthetic binary/text contracts, seven property tests, four cross-codec tests, and one manifest-driven loop over all eight real fixtures.
- `xtask` now runs six tests, including a regression proving that a `create_new` collision cannot delete a partial file owned by another run.
- The three S0 regressions are explicit: lowercase `xfxy`, `%0B`/`%0C` symmetry as part of all six whitespace escapes, and preservation of valid D/E/F records after removing the `codeWeight[0]` dead branch.
- Initial codec-only line coverage was 89.48%. Minimal public-contract tests for `FieldValue` display, `LexCode`/`PhraseCode` `AsRef<str>`, and auxiliary document `len`/`into_entries` raised it to 90.12% without changing production behavior.
- The reproducible local hard gate is:

```powershell
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov clean --workspace
& .\target\tools\cargo-llvm-cov-0.9.0\bin\cargo-llvm-cov.exe llvm-cov --package wubilex-codec --all-features --summary-only --fail-under-lines 90
```

## Full Validation Evidence

The following gates passed after the final test edits and research update:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo doc -p wubilex-codec --all-features --no-deps`
- `cargo xtask fixtures` and offline `cargo xtask fixtures --check`
- global `pnpm install --frozen-lockfile`, `pnpm lint`, `pnpm typecheck`, and `pnpm test`
- task context validation, documentation anchors, fixture ignore probes, and `git diff --check`

`pnpm-lock.yaml` remained byte-identical across the frozen install with SHA-256 `79e9717c72fcfc226afcec89bd7c3b91bde9b5ffa56a81cb8c032e13b2e5d702`. `cargo tree -p wubilex-codec -e normal` confirms no Tokio, Tauri, Windows, network, TLS, LZMA, filesystem or Serde dependency entered codec production dependencies. Registry metadata confirms every new direct crate uses MIT, Apache-2.0, or the compatible dual license recorded above.

Neither `cargo-deny` nor `cargo-audit` is installed on this machine, and this task did not mutate global Cargo state to install them. Automated advisory-database scanning therefore remains an explicit gate for `s0-xtask-ci`; it is not reported as passed here.

## Independent Review Results

The Phase 2.2 review fixed three issues before repeating the full gate:

- Real and cross-codec tests now share one test-side manifest loader. It validates the eight-scheme set, unique ids/schemes/decoded paths, portable decoded file names, decoded sizes, and decoded SHA-256 values; the representative Wubi 86 test no longer hard-codes its `.lex` file name.
- The weighted text property now uses a format-specific strategy that excludes ambiguous literal percent sequences and unsupported Unicode whitespace while retaining all six supported ASCII whitespace escapes. It also asserts byte-identical canonical text reformatting.
- Temporary cleanup guards are created only after `create_new` succeeds, so a collision cannot delete a stale or concurrently owned partial file. The two final targets are still installed only after both temporary payloads pass size, digest, LZMA, and strict `.lex` validation; an interrupted two-file replacement cannot satisfy `verify_cached` because that check requires both targets.

The independent rerun passed formatting, workspace check, strict Clippy, all workspace tests, Rustdoc, prepared and offline fixture checks, the missing-fixture hard-failure probe, global pnpm gates, Trellis validation, anchors, dependency/license checks, ignore probes, and `git diff --check`. The workspace-local coverage hard gate remained at 90.12% line coverage.

## Failure Policy

- Missing fixtures are a hard, actionable test failure, not an ignored or successful test.
- `cargo xtask fixtures --check` is offline and never repairs; the default command may fetch and repair only after complete temporary-file validation.
- A real fixture that decodes but does not byte-round-trip is a compatibility finding. Diagnose canonical metadata differences before changing the codec or acceptance contract.
- Coverage tests must assert behavior. Do not add tautological calls or weaken strict parsing solely to raise a percentage.
