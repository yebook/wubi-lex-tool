# Type Safety

> Type ownership at the Rust-to-TypeScript boundary.

---

## Single Source Of Truth

Rust owns every shared Tauri command signature, event payload, and IPC data type. The single generic registry in `src-tauri/src/bindings/mod.rs` exports those contracts through `tauri-specta` into `src/types/generated/bindings.ts`.

Generated files are committed with LF line endings so contract changes are visible in review. `cargo xtask bindings --check` is the non-mutating freshness gate and must pass in CI.

## Change Flow

1. Change the Rust command, event, or serialized data structure at its owning boundary.
2. Regenerate the TypeScript bindings.
3. Review both the Rust change and generated TypeScript diff.
4. Update frontend consumers without redefining the payload shape.
5. Run the binding freshness check and TypeScript compiler.

The `ts-rs` fallback is allowed only if the documented `tauri-specta` compatibility risk is triggered. That fallback requires handwritten command signatures plus contract tests; it is not a parallel convention.

## Frontend-Owned Types

- View-only props and local UI state may be TypeScript-owned when they do not cross IPC.
- IPC wrappers in `src/lib/` import generated types and expose typed results; they do not recast raw payload fields in each consumer.
- Feature availability is received from the backend contract and stored in the Zustand feature store. Frontend code must not duplicate it as Vite constants or locally maintained unions.
- File formats and parsed lexicon or phrase models are Rust-owned. The frontend receives only the typed command results required for its view, including paged lexicon data.

## Runtime Validation Boundary

The frontend does not parse user lexicon or phrase files. Rust performs format validation and returns structured `AppError` values. No frontend runtime-validation library has been selected yet; establish that convention only when a real non-IPC external-data boundary appears.

## Scenario: Rust-Owned IPC Binding Generation

### 1. Scope / Trigger

Apply this contract whenever a Tauri command, event, serialized IPC type, binding dependency, generated TypeScript file, or frontend IPC consumer changes. The S0 registry is intentionally empty; its generated output proves the pipeline without inventing product behavior.

### 2. Signatures

```rust
pub fn builder<R: tauri::Runtime>() -> tauri_specta::Builder<R>;
pub fn export_mock(path: impl AsRef<Path>) -> Result<(), specta_typescript::Error>;
```

```text
cargo xtask bindings
cargo xtask bindings --check
```

### 3. Contracts

- `builder<R>()` is the only command/event collector. Runtime startup and repository export must reuse it instead of maintaining parallel lists.
- Repository export instantiates `MockRuntime`, so binding generation does not enable `wry`, create a window, or register a fake command.
- The compatible stack is exact `tauri-specta 2.0.0-rc.25`, `specta 2.0.0-rc.25` with function support, and `specta-typescript 0.0.12`.
- The generated header identifies `cargo xtask bindings` as the owner. Output is valid UTF-8, LF-only, ends in a newline, and is committed at `src/types/generated/bindings.ts`.
- Default generation may replace only that owned target through a staged file. `--check` exports to ignored temporary storage, byte-compares canonical output, cleans up, and never repairs the target.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Rust registry and generated TypeScript match | `bindings --check` succeeds without changing the target |
| Generated target is missing or stale | Fail with `cargo xtask bindings`; preserve the target bytes |
| Export emits non-UTF-8 or cannot normalize/stage/replace | Fail with the exact stage and path; do not write partial output as current |
| A frontend consumer needs a new command/event/type | Add it to the Rust registry, regenerate, review the diff, then update the consumer |
| `tauri-specta` compatibility fails in a future upgrade | Return to the documented `ts-rs` decision; do not create a second live registry |

### 5. Good / Base / Bad Cases

- Good: a new typed Rust command is collected once, regenerated, imported by its frontend wrapper, and passes freshness plus TypeScript checks.
- Base: the empty S0 registry generates a deterministic committed TypeScript file and passes repeated generation and `--check`.
- Bad: TypeScript is handwritten to unblock a consumer, `wry` is enabled only for generation, or check mode silently rewrites a stale file.

### 6. Tests Required

- Assert repeat generation is byte-identical, UTF-8, LF-only, and ends with a newline.
- Mutate the committed-target analogue, run check mode, and assert nonzero failure plus byte-for-byte non-modification.
- Extend workflow contract tests whenever action pins, binding commands, or generated paths change.
- For every real command/event added in S1+, add serialization contract coverage and compile the frontend consumer against regenerated types.

### 7. Wrong vs Correct

```typescript
// Wrong: a handwritten mirror can drift from Rust.
type ImportRequest = { path: string };

// Correct: import the contract generated from the single Rust registry.
import type { ImportRequest } from "../types/generated/bindings";
```

## Forbidden Patterns

- Handwriting a second TypeScript version of a Rust IPC type.
- Editing files under `src/types/generated/` directly.
- Using local casts to compensate for stale or mismatched generated bindings.
- Parsing the same raw command or event payload independently in multiple components.
- Treating compile-time TypeScript types as a substitute for validation at an actual untrusted-data boundary.

## Sources

- [`docs/02-architecture.md` sections 3.5, 6.2, D11, and D16](../../../docs/02-architecture.md)
- [`docs/02-architecture.md` section 8.5](../../../docs/02-architecture.md)
- [Canonical Rust binding registry](../../../src-tauri/src/bindings/mod.rs)
- [Generated TypeScript baseline](../../../src/types/generated/bindings.ts)
- [`xtask` generation and freshness implementation](../../../xtask/src/bindings.rs)

The empty generated baseline and freshness behavior are established. Real command/event payloads and frontend consumer examples remain pending until S1.
