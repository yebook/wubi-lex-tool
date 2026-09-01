# Config And Feature Contract Research

## Scope

This note records repository evidence and planning decisions for the first
versioned application config, the first command-boundary `AppError`, the Cargo
feature catalog, and the first Zustand store. It does not use the root
`resource/` directory.

## Repository Evidence

- `docs/modules/M7-app-shell.md` owns `M7-CONF-001..007`: grouped persistent
  config, defaults, immediate save, corrupt-file recovery, schema migration,
  application-owned paths, and import/export.
- `docs/02-architecture.md` D11 makes Rust and the canonical
  `tauri-specta` registry the only IPC type source. D16 makes `src-tauri` Cargo
  features the only availability source and requires one startup
  `app_features` fetch into Zustand.
- `.trellis/tasks/08-25-s1-shell-ui/design.md` requires a versioned TOML with
  `window`, `ui`, and `keymap` groups, transactional persistence, a revisioned
  `config://changed` event, and no runtime/session data in exports.
- `.trellis/spec/backend/error-handling.md` requires thin commands, typed
  errors, preserved failure stages, and no production `unwrap()` / `expect()`.
- `.trellis/spec/frontend/state-management.md` forbids WebView persistence,
  handwritten feature unions, Vite feature flags, and a second config store.
- The current application has only the `desktop` Cargo feature and no Zustand
  dependency or product store. The first config schema therefore has no prior
  application-owned TOML version to migrate.

## Dependency Evidence

- The lockfile already contains `toml 1.1.4+spec-1.1.0` and
  `tempfile 3.27.0`. `wubilex-winime` already pins `windows 0.61.3`; enabling
  its `Win32_Storage_FileSystem` feature exposes the required native APIs
  without adding a second Windows binding crate.
- An official-registry read on 2026-08-28 returned Zustand `5.0.15`. The task
  should add exactly `zustand: 5.0.15` with project pnpm and update the lockfile.
- `package.json.volta.pnpm` remains the only pnpm version source. Do not add
  `packageManager`, `engines.pnpm`, Corepack, npm, yarn, or npx.

## Schema Version Decision

Schema version 1 is the first real WubiLex application-owned TOML contract.
There is no honest v0 document in the repository, and the aardio script config
is a separate S7 legacy-migration input. Do not invent a fake v0 solely to make
a successful migration test pass.

The migration engine is still mandatory:

1. Read the version discriminator before deserializing the current model.
2. Apply only registered `vN -> vN+1` pure transformations.
3. Refuse a gap, version zero, or a version newer than the application.
4. Require the first future schema bump to add the real predecessor fixture and
   transition test in the same change.

At schema v1, the registered transition set is empty. Tests cover v1, missing
version, zero, and future versions. "Every supported old version" currently
means an empty set rather than a fictional compatibility promise.

## Schema V1 Shape

The planned Rust-owned shape is:

```text
AppConfigV1
  schemaVersion: 1
  window
    bounds: optional logical rectangle plus saved scale factor
    maximized: boolean
    closeAction: minimizeToTray | exit
  ui
    theme: system | light | dark
    density: standard | compact
    locale: zhCn
    sidebarCollapsed: boolean
    onboardingVersion: nonnegative integer
  keymap
    bindings: ordered map<ActionId, BindingOverride>

BindingOverride
  custom { accelerator }
  unbound
```

The binding override shape must distinguish "use the action default" (map key
absent) from "explicitly unbound". This task validates bounded strings and map
size; the later action/keymap task owns registered action IDs, accelerator
parsing, conflict rules, and defaults.

All structs reject unknown fields. Optional/defaultable fields use explicit
defaults. Invalid enum, numeric range, map size, string length, or unknown
field is a validation failure, never a silent partial load.

## Import Decision

The user selected whole-document replacement on 2026-08-28. An import is a
complete versioned config snapshot:

- parse, migrate, default missing allowed fields, and validate in isolation;
- do not merge any group or field from the current config;
- commit through the same transaction path as a normal save;
- on failure, keep the live file, in-memory snapshot, revision, and frontend
  state unchanged;
- on success, publish one new revisioned snapshot.

## Newer And Corrupt Files

A newer schema is not corrupt. Startup preserves it at the live path, loads
defaults in memory, marks persistence read-only, and returns a visible
unsupported-version notice. Routine updates must not overwrite it. A later
explicit import of a supported complete config may replace it only through the
normal validated transaction.

Malformed current/older input is corrupt. Preserve it to a uniquely named
`.corrupt` artifact before creating defaults. If preservation fails, leave the
source untouched, use in-memory defaults in read-only mode, and surface the
failed stage. Never overwrite a file that could not first be preserved.

Existing-file replacement uses the lower crate's verified Windows adapter.
`ReplaceFileW` must use flags 0 because `REPLACEFILE_WRITE_THROUGH` is
documented as unsupported. Native error 1177 can move the old target to the
unique backup before returning failure, so the service must inspect the
structured failure disposition, attempt a no-clobber restore, and preserve the
backup plus combined error evidence if restore also fails. The honest failure
guarantee is recoverable last-valid bytes at the live or owned backup path,
not an unconditional unchanged target pathname.

## IPC Contract

Planned command surface:

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

`ConfigSnapshot` includes the monotonic revision, full config, persistence
mode, and bounded startup/recovery notices. `ConfigChangedEvent` contains the
complete snapshot and uses `config://changed`; an event receiver can always
recover by calling `config_snapshot`.

Event emission happens after the transaction commits. Emit failure is logged
without lying that persistence failed or attempting an unsafe rollback; the
snapshot command is the recovery path.

`AppError` adds a stable generated error code to the approved category,
requirement module, Chinese message, optional technical detail, and
recoverability fields. Config failures keep parse, validation, version, path,
preservation, backup, write, replace, import, export, and lock stages distinct.

## Feature Catalog

S1 shell capabilities are not placeholders, and ImTip must not exist as any
catalog entry. The initial future-capability catalog is ordered and maps every
ID to one Cargo feature and target milestone:

| AppFeatureId | Cargo feature | Target |
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

All are unavailable in the S1 default build until their owning milestone adds
and enables command implementations under the same Cargo feature. Catalog
records contain `id`, `available`, `targetMilestone`, and optional typed
`unavailableReason`; they do not expose Cargo feature strings to the frontend.

## Zustand Store Contract

- Place the first reviewed store under `src/stores/features.ts` and provide a
  factory with an injected generated-command adapter for deterministic tests.
- State is `loading | ready | failed`, stores only the bounded generated
  catalog, and has typed ID selectors. It never persists to local/session
  storage.
- Bootstrap is idempotent and deduplicates the in-flight request so React
  StrictMode cannot trigger duplicate startup commands. Failure remains
  visible and retryable.
- A later snapshot fetch replaces the complete catalog; events or local code
  do not patch individual feature records.
- Tests cover ordering-independent lookup, enabled/disabled selection,
  concurrent initialization, failure, retry, and full snapshot replacement.

## Planning Consequences

- This child installs Zustand because it is the first real consumer. The later
  UI-foundation child installs only its remaining dependencies.
- Config disk I/O stays Rust-owned and must use a blocking worker for commands;
  the frontend store is not optimistic and does not persist config.
- Configuration policy remains in `src-tauri/config`; direct `ReplaceFileW`,
  `MoveFileExW` and exclusive Windows staging wrappers remain in
  `wubilex-winime` per the established direct-Windows ownership boundary.
- The implementation must update backend error/directory specs and frontend
  state/type specs with real evidence after quality checks.
