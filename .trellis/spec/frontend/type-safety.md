# Type Safety

> Type ownership at the Rust-to-TypeScript boundary.

---

## Single Source Of Truth

Rust owns every shared Tauri command signature, event payload, and IPC data type. `tauri-specta` exports those contracts from `src-tauri/src/bindings/` into `src/types/generated/`.

Generated files are committed so contract changes are visible in review. `xtask bindings --check` is the freshness gate and must pass in CI.

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

## Forbidden Patterns

- Handwriting a second TypeScript version of a Rust IPC type.
- Editing files under `src/types/generated/` directly.
- Using local casts to compensate for stale or mismatched generated bindings.
- Parsing the same raw command or event payload independently in multiple components.
- Treating compile-time TypeScript types as a substitute for validation at an actual untrusted-data boundary.

## Sources

- [`docs/02-architecture.md` sections 3.5, 6.2, D11, and D16](../../../docs/02-architecture.md)
- [`docs/02-architecture.md` section 8.5](../../../docs/02-architecture.md)
- The scaffolded [`src/types/generated/`](../../../src/types/generated/) and [`src-tauri/src/bindings/`](../../../src-tauri/src/bindings/) directories

Real generated output and consumer examples remain pending until the S0/S1 implementation exists.
