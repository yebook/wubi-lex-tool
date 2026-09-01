# Research: Windows atomic versioned-config persistence

- Query: Define the Windows persistence contract for versioned TOML configuration: same-directory exclusive staging, flush/write-through, replacement with backup, first creation, failure ownership/cleanup, and deterministic testing behind an injectable filesystem boundary. Verify usable Win32 APIs and already locked Rust dependencies.
- Scope: mixed (repository, locked dependency source, Rust standard-library source, and Microsoft Win32 documentation)
- Date: 2026-08-28

## Findings

### Recommended contract

Treat the filesystem adapter as an opaque-byte transaction boundary. TOML serialization, schema migration, and validation must finish before it receives bytes. A save must use this order:

1. Resolve one target parent and create every staging/backup path in that same directory. This is stricter than the Win32 same-volume requirement and prevents a move from degrading into copy/delete.
2. Create a collision-resistant staging name with `OpenOptions::create_new(true)`. On Windows also use `OpenOptionsExt::share_mode(0)` while the staging handle is open. Arm ownership cleanup only after creation succeeds.
3. Write all canonical UTF-8 TOML bytes, flush any user-space writer, call `File::sync_all()`, and close the handle before installation. Rust 1.97.1 maps Windows `sync_all`/`sync_data` to `FlushFileBuffers` (`std/src/sys/fs/windows.rs:400`). A `Write::flush` alone is not the durability boundary.
4. If the target exists, call `ReplaceFileW(target, staging, unique_backup, flags = 0, NULL, NULL)`. Do not pass either ignore-ACL flag. Do not pass `REPLACEFILE_WRITE_THROUGH`: Microsoft explicitly documents it as unsupported. On success, the new bytes own the target name, the old target is the backup, the staging name is consumed, and the staging cleanup guard is disarmed.
5. For initial creation, call `MoveFileExW(staging, target, MOVEFILE_WRITE_THROUGH)` without `MOVEFILE_REPLACE_EXISTING` and without `MOVEFILE_COPY_ALLOWED`. The absence of `REPLACE_EXISTING` makes a concurrent target creation a visible failure instead of an overwrite. Same-directory placement keeps this on the rename path.
6. Update the in-memory snapshot/revision and emit the change event only after the filesystem operation reports success. Any filesystem failure keeps the prior memory/revision/event state.

Do not implement existing-file replacement as remove-then-rename. The current bindings generator does exactly that (`xtask/src/bindings.rs:97`), so it is useful staging/cleanup precedent but is not a product-config atomicity precedent.

### Critical `ReplaceFileW` failure semantics

`ReplaceFileW` is the strongest available Win32 single-call replacement primitive because it combines replacement, old-file backup, and metadata/ACL preservation. However, the official page does not use an unconditional atomic/durable guarantee, and a returned error does not always mean all names are unchanged:

| Native result | Documented namespace state | Required application response |
|---|---|---|
| Success | Replacement takes the target name; original is at the requested backup path | Disarm staging cleanup; retain the validated backup |
| 1175 `ERROR_UNABLE_TO_REMOVE_REPLACED` | Target and replacement retain their original names | Keep memory unchanged; explicitly remove only the owned staging file |
| 1176 `ERROR_UNABLE_TO_MOVE_REPLACEMENT` with a backup path | Target and replacement retain their original names | Keep memory unchanged; explicitly remove only the owned staging file |
| 1177 `ERROR_UNABLE_TO_MOVE_REPLACEMENT_2` | Replacement remains at its staging name but inherits target streams/attributes; the original target has moved to the backup name | Treat backup as the last-valid file; attempt an explicit no-clobber/write-through restore from backup to the now-missing target, and retain all evidence if restore fails |
| Other documented errors | Original names remain; no backup exists, though staging metadata/streams are not guaranteed unchanged | Keep memory unchanged; remove only the owned staging path |

This means the PRD statement that every failed replacement leaves the current file unchanged cannot be satisfied by merely returning the `ReplaceFileW` error. The adapter needs a structured failure disposition (at least `names_unchanged`, `original_at_backup`, and `unknown`) and a recovery branch for 1177. If recovery itself fails, the honest guarantee is that the last-valid bytes remain at the reported backup path, not that the target pathname is intact. The combined primary/recovery/cleanup error and all involved paths must remain visible.

A successful replacement can retain one unique backup per transaction. A unique, non-existing backup path avoids relying on undocumented behavior for an already-existing `lpBackupFileName`. If the product wants a single stable `.bak`, old-backup pruning or promotion must happen only after a new transaction has fully succeeded; the current requirement does not define retention.

### Exclusive staging and cleanup ownership

- `create_new(true)` is the atomic name claim. Rust's own filesystem guidance recommends it instead of exists-then-create to avoid TOCTOU (`std/src/fs.rs:15`, `std/src/fs.rs:26`), and the Windows implementation maps it to `CREATE_NEW` (`std/src/sys/fs/windows.rs:317`).
- Rust's Windows default share mode allows read, write, and delete/rename (`std/src/os/windows/fs.rs:168`). `share_mode(0)` prevents other opens while staging is being written. The staging handle must then be closed before `ReplaceFileW`, whose replacement-file open uses no sharing mode.
- Construct the cleanup guard only after successful creation. The project already follows that ordering in `xtask/src/bindings.rs:139` and `src-tauri/src/recovery/mod.rs:67`; it has a collision regression proving failed `create_new` never deletes the pre-existing path (`src-tauri/src/recovery/mod.rs:237`). The backend error spec explicitly rejects arming before creation (`.trellis/spec/backend/error-handling.md:73`).
- The guard owns one exact path and is disarmed only after successful installation. A failed create owns nothing. A failed write/flush/sync/close or ordinary install failure owns only staging. A 1177 failure also owns staging, but must preserve the backup containing the original.
- `Drop` may provide best-effort cleanup, but cannot be the only path because it cannot report failure. Run explicit cleanup when returning a primary error; if cleanup fails, return/record both failures. Never let a cleanup error replace the earlier write/replace error.
- Use an injected unique-name source (process id plus monotonic sequence/random suffix), with bounded collision retries. Clock injection alone is insufficient because multiple writes can share one timestamp.

The guarantee assumes the application-owned config directory is not being maliciously raced. Once the exclusive staging handle is closed, path-only cleanup cannot cryptographically prove that an attacker did not replace that path. An unguessable name, application-private directory ACL, single-instance process, and immediate cleanup are the practical boundary. Stronger hostile-directory protection would require handle/file-ID verification and is not established in this repository.

### Flush and write-through details

- `File::sync_all()` is sufficient for staging data/metadata flushing at the Rust layer on Windows because the standard library calls `FlushFileBuffers` (`std/src/sys/fs/windows.rs:400`). The local binding staging code already writes then syncs before rename (`xtask/src/bindings.rs:151`, `xtask/src/bindings.rs:157`), and session markers do the same (`src-tauri/src/recovery/mod.rs:72`, `src-tauri/src/recovery/mod.rs:82`).
- Microsoft documents `FlushFileBuffers` as writing buffered information for the file to the device. It also notes hardware/cache limitations and recommends batching writes before one flush.
- Microsoft documents `MOVEFILE_WRITE_THROUGH` as not returning until the move is performed on disk and as flushing copy/delete moves. Use it for first installation even though same-directory placement should avoid copy/delete.
- Microsoft documents `REPLACEFILE_WRITE_THROUGH` as unsupported. Therefore the supported existing-target sequence is stage `sync_all` -> close -> `ReplaceFileW(flags = 0)`. Do not claim power-loss-proof namespace durability beyond this supported contract. The API gives a no-partial-content replacement boundary for normal operation; sudden power loss, filesystem/filter-driver behavior, or unsupported media caches remain outside deterministic unit guarantees.
- Do not mark the staging config `FILE_ATTRIBUTE_TEMPORARY`; Microsoft notes that this attribute encourages retaining data in cache rather than writing it to storage, which conflicts with durable configuration intent.

### Injectable filesystem boundary

Keep policy in `src-tauri/config` and direct Win32 calls behind a lower Windows adapter. A testable shape is a generic `ConfigService<F: ConfigFileSystem>` (the production state uses the concrete adapter) where the boundary exposes stages rather than one undifferentiated `save` call:

```rust
trait ConfigFileSystem {
    type Pending: std::io::Write;

    fn create_dir_all(&self, directory: &Path) -> Result<(), ConfigIoError>;
    fn create_temp_exclusive(&self, path: &Path) -> Result<Self::Pending, ConfigIoError>;
    fn sync(&self, pending: &mut Self::Pending) -> Result<(), ConfigIoError>;
    fn close(&self, pending: Self::Pending) -> Result<(), ConfigIoError>;
    fn replace_with_backup(
        &self,
        target: &Path,
        staging: &Path,
        backup: &Path,
    ) -> Result<(), ReplaceFailure>;
    fn install_new_noreplace(&self, staging: &Path, target: &Path)
        -> Result<(), ConfigIoError>;
    fn restore_backup_noreplace(&self, backup: &Path, target: &Path)
        -> Result<(), ConfigIoError>;
    fn remove_owned(&self, path: &Path) -> Result<(), ConfigIoError>;
}
```

The exact trait may instead use an owned pending-file wrapper, but it must expose create, write/flush, sync, close, replace/backup, first install, restore, and cleanup as separately injectable stages. `ReplaceFailure` must preserve Win32 code/message and namespace disposition. Do not make unit tests parse error display strings.

Recommended deterministic tests:

| Case | Assertions |
|---|---|
| Existing target success | Ordered create/write/flush/sync/close/replace; target is exactly new bytes; backup is exactly old validated bytes; staging absent; one revision/event |
| Initial create success | No backup/replace call; no-clobber move with write-through; target equals staged bytes |
| Temp collision | `AlreadyExists`; pre-existing collision bytes unchanged; no cleanup authorization; bounded retry behavior |
| Failure at create/write/flush/sync/close | Target and memory/revision unchanged; cleanup only after successful create; primary stage retained |
| Failure at ordinary replace/install | Target unchanged (or concurrently created target retained); owned staging cleaned; memory/revision/event unchanged |
| Simulated native 1177 | Mock moves original to backup before returning error; service attempts no-clobber restore; success restores old target, while restore failure retains backup and reports both errors/paths |
| Cleanup failure | Primary error remains primary; cleanup error is attached/returned as secondary evidence |
| External target appears during initial install | No-clobber move fails and never overwrites it |

The in-memory fake should model path contents and state transitions, not merely record that methods were called; otherwise it cannot catch the 1177 partial-name state. Add a Windows-only integration test using an actual temporary directory for first install and existing replacement/backup. Unit fault injection remains the authoritative coverage for every failure stage; a stress reader that accepts only complete old/new bytes is useful evidence but cannot prove all filesystem/power-loss behavior.

### Dependency and ownership fit

- `windows = 0.61.3` is already exactly selected in `crates/wubilex-winime/Cargo.toml:11` and locked at `Cargo.lock:4617`. Its generated bindings contain `FlushFileBuffers`, `MoveFileExW`, `ReplaceFileW`, `MOVEFILE_WRITE_THROUGH`, and the replace flags (`windows-0.61.3/.../Win32/Storage/FileSystem/mod.rs:894`, `:1710`, `:2291`, `:6074`, `:7411`). Enabling its `Win32_Storage_FileSystem` feature is sufficient to expose that module; no new Win32 crate is needed.
- The project spec requires all direct Win32/COM work to remain in `wubilex-winime` (`.trellis/spec/backend/directory-structure.md:48`), while configuration policy belongs under `src-tauri/config` (`.trellis/spec/backend/directory-structure.md:50`). Therefore the Windows adapter should be exposed by the lower crate (most naturally under its system-side-effect boundary), and `ConfigService` should consume it rather than adding direct `windows` calls to command/config policy code.
- `tempfile 3.27.0` is present only transitively (`Cargo.lock:3779`), not as a direct app dependency. Its Windows persist implementation uses `MoveFileExW`, optional replace-existing, and no write-through/backup (`tempfile-3.27.0/src/file/imp/windows.rs:92`), so it does not satisfy existing-config backup or the 1177 recovery contract. It is unnecessary for production staging; tests may use it only after declaring an intentional direct dev-dependency.
- No locked `atomicwrites`, `atomic-write-file`, or `fs4` package was found.

### Files found

- `.trellis/tasks/08-27-s1-config-features/prd.md` - fixes transactional persistence, injected directory/clock/filesystem, backup, rollback, and full-import requirements.
- `.trellis/spec/backend/directory-structure.md` - assigns configuration policy to `src-tauri/config` and direct Win32 calls to `wubilex-winime`.
- `.trellis/spec/backend/error-handling.md` - requires staged technical evidence and ownership-based cleanup.
- `.trellis/spec/backend/quality-guidelines.md` - requires Windows integration behavior through deterministic boundaries and serialization/error contract tests.
- `xtask/src/bindings.rs` - local create-new, owned cleanup, write, and `sync_all` precedent; its remove-then-rename replacement is not safe for product config.
- `src-tauri/src/recovery/mod.rs` - application-side create-new ownership guard and collision regression.
- `crates/wubilex-winime/Cargo.toml` - existing exact `windows 0.61.3` dependency and current feature set.
- `crates/wubilex-winime/src/security.rs` - established pattern for a typed adapter, staged Win32 error evidence, and non-Windows behavior.
- `Cargo.lock` - locked `windows 0.61.3`, transitive `tempfile 3.27.0`, and `windows-sys 0.61.2`.

### External references

- Microsoft Learn, `ReplaceFileW`, updated 2025-07-01: <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew>. Verified backup/same-volume requirements, unsupported write-through flag, ACL behavior, and error codes 1175/1176/1177.
- Microsoft Learn, `MoveFileExW`, updated 2025-07-01: <https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw>. Verified replace-existing and write-through flags.
- Microsoft Learn, `FlushFileBuffers`, updated 2025-07-01: <https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers>. Verified buffered-data flush semantics and cache caveats.
- Microsoft Learn, `CreateFileW`, updated 2025-07-01: <https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew>. Verified zero sharing mode, `CREATE_NEW`, write-through caching behavior, and temporary-file caching warning.
- Installed Rust 1.97.1 standard-library source under `E:/env/rust/rustup/toolchains/1.97.1-x86_64-pc-windows-msvc/lib/rustlib/src/rust/library/std/`.
- Installed locked crate sources under `E:/env/rust/cargo/registry/src/rsproxy.cn-e3de039b2554c837/`.

### Related specs

- `.trellis/spec/backend/directory-structure.md:24` - crate responsibilities and direct-Windows ownership.
- `.trellis/spec/backend/error-handling.md:27` - typed propagation and native code/message preservation.
- `.trellis/spec/backend/error-handling.md:62` - existing owned-partial cleanup rule.
- `.trellis/spec/backend/quality-guidelines.md:22` - deterministic backend test obligations.
- `.trellis/spec/backend/windows-system-integration.md` - broader reversible Windows mutation and combined primary/restore failure precedent.

## Caveats / Not Found

- Microsoft documents `ReplaceFileW` as a single replacement operation with backup, but does not state an unconditional atomic or power-loss-durability guarantee. The research therefore does not claim more than complete staged bytes plus a single supported namespace replacement call in normal operation.
- `REPLACEFILE_WRITE_THROUGH` exists in headers/bindings but is explicitly unsupported. There is no verified supported flag that makes `ReplaceFileW` namespace metadata power-loss durable.
- Existing-backup-path overwrite behavior is not specified on the verified `ReplaceFileW` page. Use a unique backup path or define and separately verify a retention protocol; do not silently assume a fixed `.bak` will be replaced safely.
- A returned 1177 can move the old target to the backup path. Absolute "failed call leaves target pathname unchanged" is not a valid native assumption; the PRD/design must acknowledge explicit restore or the weaker preserved-at-backup guarantee.
- No existing product config module or injectable product filesystem interface was found. The cited repository code is precedent only, not an implementation to reuse wholesale.
- No root `resource/` content was read or used.
