# Directory Structure

> The approved Rust workspace layout and ownership boundaries.

---

## Current Status

The workspace, `wubilex-codec` contracts and codecs, test-only real-fixture automation, Rust-owned IPC binding registry, document checks, dependency policy, and Windows CI workflow compile or validate. Other product modules may still contain only their S0 shell or scaffolded directories, so the ownership table remains the placement contract for work that has not started.

## Workspace Membership

The root `Cargo.toml` is a virtual workspace. Its members are exactly:

- `crates/wubilex-codec`
- `crates/wubilex-core`
- `crates/wubilex-winime`
- `crates/wubilex-resource`
- `src-tauri`
- `xtask`

`crates/wubilex-learn` is deliberately scaffolded but must remain outside workspace members until S8. Keep the conventional `src-tauri/` path; it is the `wubilex-app` crate and must not be moved under `crates/`.

## Ownership And Dependencies

| Location | Responsibility | Allowed dependencies | Forbidden dependencies or work |
|---|---|---|---|
| `crates/wubilex-codec/` | Bytes or text to typed models and back | Standard library plus parsing and encoding crates | Tauri, Windows APIs, network, and non-filesystem I/O |
| `crates/wubilex-core/` | Domain models, indexes, transformations, slimming, weighting, and word generation | `wubilex-codec` | Tauri, Windows APIs, and network |
| `crates/wubilex-winime/` | Windows IME, TSF, registry, service, scheduler, and ACL integration | The `windows` crate with required features only | Tauri and domain logic |
| `crates/wubilex-resource/` | HTTP, archives, integrity checks, and cache management | HTTP, compression, and hashing crates | Tauri and domain logic |
| `src-tauri/` | Commands, events, application state, task orchestration, and recovery | All active lower crates | Domain logic inside command handlers |
| `xtask/` | Reproducible repository automation, including fixtures, IPC binding generation, and document validation | Workspace tooling plus dependencies required by those commands | Product resource behavior, product runtime code, or a direct Tauri runtime dependency |

`wubilex-codec` and `wubilex-core` remain synchronous and must not depend on Tokio. Long-running work is moved to blocking workers by `src-tauri`; cancellation and progress are adapted at that boundary.

## Required Module Locations

- Shared codec contracts live in `crates/wubilex-codec/src/error.rs`, `limits.rs`, and `model/`. The `model/` directory contains only format-neutral ordered documents and validated scalar types; it must not contain core indexes or parser placeholders.
- Codec format code stays under `crates/wubilex-codec/src/{lex,eudp,text,weight,split_table,detect,escape}/`. The committed real-fixture contract is `crates/wubilex-codec/tests/fixtures/{manifest.json,README.md,.gitignore}`; downloaded `.lex` and `.lex.lzma` payloads remain ignored. Acquisition, cache verification, bounded LZMA-alone decoding, and strict codec validation live in `xtask/src/fixtures.rs` and are invoked through `cargo xtask fixtures [--check]`.
- Integration tests that consume real fixtures share `crates/wubilex-codec/tests/support/mod.rs` as their manifest loader. Do not duplicate fixture paths or maintain a second scheme list in an individual test.
- Raw `.lex` byte decoding and canonical encoding live in `crates/wubilex-codec/src/lex/`. That module is synchronous and memory-to-memory; filesystem access and `.lex.lzma` or `.lex.zst` handling remain in `wubilex-resource`.
- Raw EUDP byte decoding and canonical encoding live in `crates/wubilex-codec/src/eudp/`. The caller supplies the wire timestamp explicitly. System clock access, v1/v2 path selection and dual writes, Windows version checks, TSF coordination, backup, and rollback remain outside the codec.
- Community lexicon text detection, strict byte decoding, dialect parsing, warnings, and canonical formatting live in `crates/wubilex-codec/src/text/`; the six-character whitespace escape contract lives in `src/escape/` so phrase text can reuse it. Both modules are synchronous and memory-to-memory. Paths, filesystem writes, containers, scheme detection, indexes, and UI diagnostics remain outside them.
- Phrase text parsing and formatting live in `crates/wubilex-codec/src/text/phrase/` and are exposed through the crate-root `phrase_text` facade. Shared strict byte decoding remains in `src/text/encoding.rs`, while shared two-column parsing for auxiliary formats remains crate-private in `src/text/auxiliary.rs`.
- BOM-less UTF-8 word-frequency and split-table codecs live in `src/weight/` and `src/split_table/`; their ordered value objects live in `src/model/`. Content-only scheme detection lives in `src/detect/` and may inspect a `LexiconDocument`, but it must not build a domain index or perform filesystem work.
- Domain operations stay under the matching `crates/wubilex-core/src/` area. Cross-layer exits owned by core are declared in `crates/wubilex-core/src/ports/`.
- Direct Win32 or COM work stays in `crates/wubilex-winime/`. System side effects are centralized behind `src/sysops/`; service, scheduler, and ACL implementations use their existing dedicated directories.
- Resource HTTP abstraction belongs in `crates/wubilex-resource/src/http/`; catalog, download, archive, cache, and verification code stays in its named module.
- Tauri commands are grouped below `src-tauri/src/commands/` by their documented command prefix. Shared application concerns use the existing `state/`, `events/`, `task/`, `config/`, `error/`, `features/`, `recovery/`, and `bindings/` directories.
- `src-tauri/src/bindings/mod.rs` owns the single generic `tauri_specta::Builder<R: tauri::Runtime>` registry. Repository generation instantiates it with `tauri::test::MockRuntime`; do not enable `wry` or create a fake command merely to make the generated file nonempty.
- `xtask/src/{fixtures,bindings,check_docs}.rs` own the three repository command families. Generated TypeScript is written only to `src/types/generated/bindings.ts`; document checks reuse `.trellis/scripts/check_anchors.py` rather than cloning its slug logic.

Rust module and directory names follow the scaffolded `snake_case` form, including `split_table` and `double_pinyin`. Command names use `<module_prefix>_<action>` and must stay in the command directory assigned to that prefix.

## Sources

- [`docs/02-architecture.md` sections 1, 2, D10, and 9](../../../docs/02-architecture.md)
- Existing crate READMEs and scaffolded directories under [`crates/`](../../../crates/)
- [`src-tauri/README.md`](../../../src-tauri/README.md)
- [`wubilex-codec` shared contract modules](../../../crates/wubilex-codec/src/lib.rs)
- [`wubilex-codec` `.lex` format module](../../../crates/wubilex-codec/src/lex/mod.rs)
- [`wubilex-codec` EUDP format module](../../../crates/wubilex-codec/src/eudp/mod.rs)
- [`wubilex-codec` community text module](../../../crates/wubilex-codec/src/text/mod.rs)
- [`wubilex-codec` phrase text module](../../../crates/wubilex-codec/src/text/phrase/mod.rs)
- [`wubilex-codec` auxiliary text codecs](../../../crates/wubilex-codec/src/weight/mod.rs)
- [`wubilex-codec` scheme detector](../../../crates/wubilex-codec/src/detect/mod.rs)
- [`xtask` fixture automation](../../../xtask/src/fixtures.rs)
- [`xtask` binding generation](../../../xtask/src/bindings.rs)
- [`xtask` document validation](../../../xtask/src/check_docs.rs)
- [Canonical Tauri binding registry](../../../src-tauri/src/bindings/mod.rs)
- [Real fixture manifest](../../../crates/wubilex-codec/tests/fixtures/manifest.json)

The codec contracts, test-only fixture automation, binding registry/export path, document validator, and CI policy are established implementation evidence. Add examples for the remaining crates only after those crates contain real behavior.
