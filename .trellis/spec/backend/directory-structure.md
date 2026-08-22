# Directory Structure

> The approved Rust workspace layout and ownership boundaries.

---

## Current Status

The workspace, `wubilex-codec` shared contracts, and the raw Microsoft Wubi `.lex` codec compile. Other product modules may still contain only their S0 shell or scaffolded directories, so the ownership table remains the placement contract for work that has not started.

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
| `xtask/` | Reproducible repository automation | Workspace tooling | Product behavior |

`wubilex-codec` and `wubilex-core` remain synchronous and must not depend on Tokio. Long-running work is moved to blocking workers by `src-tauri`; cancellation and progress are adapted at that boundary.

## Required Module Locations

- Shared codec contracts live in `crates/wubilex-codec/src/error.rs`, `limits.rs`, and `model/`. The `model/` directory contains only format-neutral ordered documents and validated scalar types; it must not contain core indexes or parser placeholders.
- Codec format code stays under `crates/wubilex-codec/src/{lex,eudp,text,weight,split_table,detect,escape}/`. Real codec fixtures belong under `crates/wubilex-codec/tests/fixtures/` and are fetched reproducibly by `xtask fixtures` rather than committed as implicit machine-local data.
- Raw `.lex` byte decoding and canonical encoding live in `crates/wubilex-codec/src/lex/`. That module is synchronous and memory-to-memory; filesystem access and `.lex.lzma` or `.lex.zst` handling remain in `wubilex-resource`.
- Domain operations stay under the matching `crates/wubilex-core/src/` area. Cross-layer exits owned by core are declared in `crates/wubilex-core/src/ports/`.
- Direct Win32 or COM work stays in `crates/wubilex-winime/`. System side effects are centralized behind `src/sysops/`; service, scheduler, and ACL implementations use their existing dedicated directories.
- Resource HTTP abstraction belongs in `crates/wubilex-resource/src/http/`; catalog, download, archive, cache, and verification code stays in its named module.
- Tauri commands are grouped below `src-tauri/src/commands/` by their documented command prefix. Shared application concerns use the existing `state/`, `events/`, `task/`, `config/`, `error/`, `features/`, `recovery/`, and `bindings/` directories.

Rust module and directory names follow the scaffolded `snake_case` form, including `split_table` and `double_pinyin`. Command names use `<module_prefix>_<action>` and must stay in the command directory assigned to that prefix.

## Sources

- [`docs/02-architecture.md` sections 1, 2, D10, and 9](../../../docs/02-architecture.md)
- Existing crate READMEs and scaffolded directories under [`crates/`](../../../crates/)
- [`src-tauri/README.md`](../../../src-tauri/README.md)
- [`wubilex-codec` shared contract modules](../../../crates/wubilex-codec/src/lib.rs)
- [`wubilex-codec` `.lex` format module](../../../crates/wubilex-codec/src/lex/mod.rs)

The codec contract and raw `.lex` format layout are established implementation evidence. Add examples for the remaining crates only after those crates contain real behavior.
