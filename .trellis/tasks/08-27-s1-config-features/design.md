# Design - S1 配置与功能目录

## 1. Delivery Boundary

本子任务建立后续 S1 子任务共同依赖的配置、错误和功能目录合同，但不实现窗口协调、主题渲染、路由、动作语义或系统写入。

```text
Tauri app_config_dir
  -> config codec / validator / migration
  -> ConfigService (serialized transaction + revision)
  -> thin config_* commands
  -> generated ConfigSnapshot / ConfigChangedEvent

Cargo cfg features
  -> AppFeatureCatalog
  -> app_features command
  -> generated TypeScript
  -> feature client
  -> Zustand feature store
  -> later route / action / placeholder consumers
```

Rust 是配置 wire type、错误、feature ID、里程碑和 command/event 的唯一事实来源。TOML 是唯一持久配置；Zustand 只缓存有界 feature catalog，不复制或持久化配置。

## 2. Module Ownership

计划新增或扩展：

```text
src-tauri/src/
  config/
    mod.rs          public service and snapshot contract
    model.rs        schema v1 types, defaults and validation
    codec.rs        bounded UTF-8 TOML decode/encode
    migration.rs    version discriminator and adjacent-step registry
    storage.rs      file-operation port and Windows implementation
  error/mod.rs      generated AppError contract and conversions
  features/mod.rs   stable feature catalog projected from Cargo cfg
  commands/app/
    config.rs       thin async config command adapters
    features.rs     app_features adapter
  events/mod.rs     ConfigChangedEvent beside existing launch event
  bindings/mod.rs   one canonical registry

crates/wubilex-winime/src/
  filesystem.rs     typed exclusive staging and Win32 replace/move wrappers

src/
  lib/features-client.ts
  stores/features.ts
  stores/features.test.ts
```

Config parsing, validation, migration and transaction policy stay out of command handlers. Application-owned path and recovery policy live in `src-tauri/config`; direct `ReplaceFileW`, `MoveFileExW` and Windows-exclusive staging calls stay in `wubilex-winime` per the repository's single direct-Windows boundary.

## 3. Schema V1 Contract

All Rust types derive `Serialize`, `Deserialize`, `specta::Type` where they cross IPC, use camelCase external names, reject unknown fields, and own explicit defaults.

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub struct AppConfig {
    pub schema_version: u32,
    pub window: WindowConfig,
    pub ui: UiConfig,
    pub keymap: KeymapConfig,
}

pub struct WindowConfig {
    pub bounds: Option<WindowBounds>,
    pub maximized: bool,
    pub close_action: CloseAction,
}

pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

pub struct UiConfig {
    pub theme: ThemePreference,
    pub density: Density,
    pub locale: AppLocale,
    pub sidebar_collapsed: bool,
    pub onboarding_version: u32,
}

pub struct KeymapConfig {
    pub bindings: BTreeMap<String, BindingOverride>,
}

pub enum BindingOverride {
    Custom { accelerator: String },
    Unbound,
}
```

Defaults are: no stored bounds, not maximized, minimize to tray, system theme, standard density, `zh-CN`, expanded sidebar, onboarding version 0, and no overrides. An absent keymap entry means "use the action default"; `Unbound` means the user explicitly cleared it.

Validation is deliberately bounded:

- input file maximum: 1 MiB;
- bounds width/height: 1..=32768; coordinates: -1,000,000..=1,000,000;
- finite scale factor: 0.5..=8.0;
- keymap overrides: at most 512;
- action ID: 1..=96 ASCII lowercase letters, digits, `.`, `_`, `-`;
- accelerator: 1..=128 Unicode scalar values, with no control characters.

This task validates only the durable container. `s1-window-tray` owns monitor correction; `s1-actions-keymap` owns registered action IDs, accelerator parsing, conflicts and defaults.

Canonical output is UTF-8 without BOM, LF-only, no trailing whitespace, deterministic struct/BTreeMap order, and exactly one final newline.

## 4. Version And Load State Machine

The decoder first reads `schemaVersion` from a bounded TOML value, then routes through adjacent migration steps and finally deserializes/validates the current model.

Schema v1 is the first application-owned TOML, so the initial migration registry is empty. Missing, zero and future versions are rejected; no fake v0 is invented. A future schema bump must add the real vN fixture and `vN -> vN+1` transformation in the same change.

Startup behavior:

| Input state | File action | In-memory state | Persistence |
|---|---|---|---|
| File missing | Transactionally create defaults | Defaults, revision 1 | Ready |
| Live file missing, owned backup present | Validate and restore the newest valid backup without clobber | Recovered config plus notice | Ready if restore succeeds, otherwise read-only |
| Valid v1 | None | Loaded config, revision 1 | Ready |
| Supported old version in future | Migrate and transactionally persist with backup | Migrated config, revision 1 | Ready |
| Malformed/invalid | Move source to unique `.corrupt` path, then create defaults | Defaults plus visible recovery notice | Ready if default write succeeds |
| Corrupt preservation fails | Leave source untouched | Defaults plus failure notice | Read-only |
| Future schema | Leave source untouched | Defaults plus unsupported-version notice | Read-only |
| Config directory/read/create fails | Do not touch unrelated paths | Defaults plus failure notice | Read-only |

Routine updates fail visibly in read-only mode. An explicit, fully validated whole-document import may leave read-only mode by replacing the live file through the normal transaction, preserving the previous file as backup.

## 5. Transactional Persistence

`ConfigService` owns a `Mutex<ConfigData>` containing snapshot, persistence mode and owned paths. It holds the lock across validation and persistence so concurrent commands cannot interleave revisions or disk writes. Poisoned state returns a structured state error; it does not silently continue a write from uncertain state.

`ConfigFileOps` is an injectable app-layer port covering bounded read, exclusive same-directory temp creation, sync, atomic replace/create, corrupt preservation and owned cleanup. Its production implementation delegates raw native operations to the typed `wubilex-winime` filesystem adapter; tests use a real temp directory plus a stage-failing wrapper.

Transaction order:

1. Clone current data and validate the complete candidate.
2. Canonically encode to bytes.
3. Create a unique same-directory temp file with exclusive ownership.
4. Write all bytes, flush the writer, call `File::sync_all()`, and close the staging handle.
5. If the live file exists, call the lower adapter's `ReplaceFileW` wrapper with flags 0 and a unique non-existing backup path; otherwise install with no-clobber `MoveFileExW(MOVEFILE_WRITE_THROUGH)`.
6. Only after filesystem success, replace memory state and increment revision with `checked_add`.
7. Return the complete snapshot; the command adapter emits `config://changed` after releasing the service lock.

The implementation must use the Windows replacement APIs validated in research. `REPLACEFILE_WRITE_THROUGH` is unsupported and must not be used. It must not implement replacement as delete-live-then-rename. Cleanup may delete only a temp path whose exclusive creation succeeded. Export uses the same safe destination transaction but never aliases the live config, backup or owned temp/corrupt paths.

The native replace wrapper returns a structured namespace disposition. For normal failures, clean only the owned staging file. Error 1177 can leave the old target at backup and the target name absent; the service must attempt a no-clobber/write-through restore from backup. If restore fails, memory/revision remain unchanged, the service enters read-only mode, the valid backup is retained, and the combined replace/restore/cleanup evidence and paths remain visible. Startup recognizes and validates owned backups so a missing live path can recover. Do not claim an unconditional unchanged target pathname or power-loss-proof directory metadata.

An event emit failure is logged with event/stage metadata but does not claim the completed persistence failed and does not attempt a destructive rollback. Consumers recover by calling `config_snapshot`.

## 6. Whole-Document Import And Export

The user selected whole-document replacement:

```text
read import (<= 1 MiB)
  -> decode version
  -> apply every registered adjacent migration
  -> default allowed missing fields
  -> validate complete candidate
  -> normal live-config transaction
  -> new full snapshot/event
```

No field or group is inherited from the current config. Failure before or during commit leaves the memory snapshot and revision unchanged; the last-valid bytes remain at the live path or a reported owned backup after the 1177 recovery branch. If commit succeeds, the previous live config becomes the write-before-replace backup.

Export reads the authoritative in-memory snapshot, excludes revision/notices/persistence/runtime data, canonically encodes only `AppConfig`, and safely writes the user-selected destination. Import/export requests accept an OS path serialized by the generated IPC contract; empty paths, directories and destinations aliasing config-owned artifacts are rejected.

## 7. IPC And AppError

### Commands

```text
config_snapshot() -> Result<ConfigSnapshot, AppError>
config_update_window(WindowConfig) -> Result<ConfigSnapshot, AppError>
config_update_ui(UiConfig) -> Result<ConfigSnapshot, AppError>
config_update_keymap(KeymapConfig) -> Result<ConfigSnapshot, AppError>
config_restore_defaults(ConfigGroup) -> Result<ConfigSnapshot, AppError>
config_import(ConfigPathRequest) -> Result<ConfigSnapshot, AppError>
config_export(ConfigPathRequest) -> Result<ConfigExportResult, AppError>
app_features() -> AppFeatureCatalog
```

Config disk operations run through `tauri::async_runtime::spawn_blocking`; the command layer clones managed service ownership and emits only after success. `app_features` is synchronous and state-free.

`ConfigGroup` is `window | ui | keymap | all`. Each update or restore submits a complete group, avoiding a loosely typed key-path setter.

### Snapshot And Event

```rust
pub struct ConfigSnapshot {
    pub revision: u64,
    pub config: AppConfig,
    pub persistence: ConfigPersistence,
    pub notices: Vec<ConfigNotice>,
}

pub struct ConfigChangedEvent {
    pub snapshot: ConfigSnapshot,
}
```

`ConfigChangedEvent::NAME` is `config://changed`. Notices are bounded and typed; details contain stages, error kinds/codes and owned paths, never TOML payloads, shortcut values or other user content.

### AppError

The first real command error contract is:

```rust
pub struct AppError {
    pub code: AppErrorCode,
    pub kind: AppErrorKind,
    pub module: RequirementModule,
    pub message: String,
    pub detail: Option<String>,
    pub recoverable: bool,
}
```

`AppErrorKind` keeps the approved `io | parse | network | permission | system | validation | cancelled` categories. `RequirementModule` freezes `m1..m8`; this task emits `m7`. Stable config codes distinguish unavailable state, invalid path, read, parse, unsupported version, validation, preservation, backup/write/replace, import and export failures. Lower errors retain stage, `io::ErrorKind` and `raw_os_error` in bounded technical detail.

## 8. Feature Catalog

The catalog is a const ordered Rust table. Every entry maps a generated `AppFeatureId` to exactly one declared Cargo feature and target milestone:

| ID | Cargo feature | Target |
|---|---|---|
| `lexiconRead` | `feat-m1-read` | S2 |
| `phraseRead` | `feat-m2-read` | S2 |
| `reverseLookup` | `feat-m3-lookup` | S2 |
| `systemWrite` | `feat-m4-system-write` | S3 |
| `lexiconEdit` | `feat-m1-edit` | S4 |
| `phraseEdit` | `feat-m2-edit` | S4 |
| `radicalReference` | `feat-m5-radicals` | S5 |
| `resourceSync` | `feat-m6-resource-sync` | S5 |
| `systemSettings` | `feat-m4-settings` | S6 |
| `resourceUpdate` | `feat-m6-update` | S6 |
| `legacyMigration` | `feat-m7-legacy-migration` | S7 |
| `selfLearning` | `feat-m8-learning` | S8 |

The S1 default build leaves all future features disabled. Future milestones enable commands and catalog availability under the same Cargo feature. Records expose `id`, `available`, `targetMilestone` and optional typed `unavailableReason`, not raw Cargo feature strings.

S1 shell capabilities are already present and are not placeholders. ImTip is omitted entirely rather than represented as permanently unavailable.

## 9. Frontend Feature Store

Add exact Zustand `5.0.15`. The store uses generated `AppFeatureCatalog`, `AppFeatureId` and `AppError` types through an injected feature client.

```text
status: loading | ready | failed
catalog: generated records only
error: generated AppError or bounded invocation failure
initialize(): deduplicated in-flight fetch
retry(): new fetch after failure
isAvailable(id): typed selector
feature(id): typed selector
replace(catalog): full snapshot replacement for tests/bootstrap
```

No Zustand persistence, devtools middleware, optimistic updates or Vite flags are added. React StrictMode may invoke bootstrap twice, so the store/client shares one in-flight promise and applies a response only once. Tests inject a client and cover concurrent initialize, lookup, failed state, retry and full replacement.

The current runtime view triggers feature bootstrap without adding final route or placeholder UI. Those consumers arrive in `s1-routing-shell`.

## 10. Startup Integration

Tauri setup order becomes:

1. Resolve application config directory.
2. Load `ConfigService`, falling back to read-only defaults plus notice on failure.
3. Manage config state before the WebView can invoke commands.
4. Continue existing logging, session, privilege and window setup.
5. Frontend obtains runtime snapshot and initializes the feature store; later consumers separately request config snapshot.

Config failure never prevents runtime startup. No config stage performs S2/S3 data access or system mutation.

## 11. Verification Strategy

### Rust Unit And Integration Tests

- canonical v1 roundtrip, LF/final newline, deterministic BTreeMap order;
- missing/zero/future schema, unknown field, invalid enum/range and size limits;
- defaultable fields and explicit unbound keymap override;
- missing-file create, valid load, corrupt preserve/default, preservation failure and read-only future behavior;
- real Windows initial install and atomic replace in a temp directory;
- failure injection at create, write, sync, backup/replace, native 1177 restore and cleanup stages;
- update/import failure keeps snapshot and revision exact and preserves last-valid bytes at live or owned backup;
- full import replaces omitted/defaultable fields rather than merging current values;
- concurrent updates serialize and revisions increase once each;
- complete stable feature ordering and cfg-to-availability projection;
- `AppError` and snapshot JSON/TypeScript field contract.

### Frontend Tests

- generated type consumption without handwritten unions;
- loading/ready/failed/retry states;
- StrictMode-style concurrent initialization makes one command call;
- enabled/disabled typed selectors and unknown lookup behavior;
- full catalog replacement does not retain stale entries.

### Repository And Static Gates

- complete Rust, Rustdoc, bindings, docs, deny and frontend gates;
- official-registry high audit after frozen install;
- package-manager contract and exact Volta pnpm pin;
- case-insensitive production search proves no ImTip config/feature surface;
- no Vite feature source, WebView config persistence or non-atomic delete/rename fallback.

## 12. Rollback And Compatibility

- Before schema v1 ships beyond this task, its shape may be corrected in the same task. After downstream consumers or test users receive it, every change requires a new version and adjacent migration.
- If native replacement cannot preserve a validated backup across ordinary and 1177 failure topologies, stop the task; do not weaken the contract to delete then rename.
- If Zustand integration regresses runtime bootstrap, revert only the feature client/store while retaining the backend catalog; do not create Vite flags.
- Import is whole-document replacement and is not changed to merge without a new user decision and compatibility review.
- Root `resource/`, S2 data and S3 system operations remain untouched.
