# Quality Guidelines

> Rust quality gates and the S0 codec verification baseline.

---

## Required Gates

Backend changes must compile and pass formatting, Clippy with warnings denied, tests, and dependency license and vulnerability checks. CI obtains Rust from `rust-toolchain.toml`; workflow files must not hard-code another Rust version.

The approved CI sequence also checks generated IPC bindings and documentation contracts. A local check is not complete if it skips a gate affected by the change.

## Forbidden Patterns

- Do not map external `.lex` or EUDP bytes into `#[repr(C)]` structures with `transmute` or equivalent memory reinterpretation. Their variable lengths and alignment are not a Rust ABI contract; parse fields explicitly with bounds checks.
- Do not use `unwrap()` or `expect()` on production paths.
- Do not put domain transformations in Tauri command handlers or frontend callbacks.
- Do not add Tauri, Windows, network, or Tokio dependencies across the boundaries defined in the backend directory guide.
- Do not use machine-local fixtures or silently skip missing regression data.
- Do not treat the aardio implementation as a Rust implementation template. It is evidence for behavior only.

## Testing Requirements

| Area | Minimum evidence |
|---|---|
| `wubilex-codec` | Unit and property tests for round trips, dialects, version detection, boundaries, resource limits, and malformed input; measured coverage >= 90% |
| Codec fixtures | At least one reproducibly fetched real lexicon for each of 86, 98, 06, 091, 092, Zhengma, Xiaohe, and Biaoxingma |
| S0 regressions | Failure-to-pass tests for uppercase `XFXY`, asymmetric whitespace escaping, and the removed `codeWeight[0]` dead branch |
| `wubilex-core` | Input/output assertions for every implemented transform, slimming, weighting, and word-generation operation |
| `wubilex-winime` | Operation-sequence tests through recording dry-run behavior; real execution only in an isolated Windows CI environment |
| `wubilex-resource` | Mocked HTTP and hostile archive tests, including path traversal |
| `wubilex-app` | Serialization contract tests for commands and shared errors |

## Established S0 Codec Evidence

- `crates/wubilex-codec/tests/model_contracts.rs` fixes the validated model boundaries, ordered duplicate-preserving documents, nonzero weight/candidate ranges, UTF-16 code-unit behavior, scheme shape, and encoding/BOM contract.
- `crates/wubilex-codec/tests/error_and_limits.rs` fixes structured error evidence and the 64 MiB / 500,000 default limits. Assertions match error kinds and locations rather than rendered messages.
- The crate's only direct dependency at this stage is the exact `thiserror = 2.0.20` contract. Parser, encoding, platform, async, serialization, and network dependencies are introduced only by the task that uses them.
- Root-level user samples are not fixtures. Tests must use committed synthetic data or reproducibly fetched files under `crates/wubilex-codec/tests/fixtures/`; they must never depend on a machine-local `resource/` directory.

The remaining three known defect regressions belong to S4: EUDP drag-and-drop dispatch, duplicate Zhengma word generation, and the non-incrementing `unique()` loop.

## Review Checklist

- Does the change stay within its crate's dependency and responsibility boundary?
- Does every parser validate offsets, lengths, UTF-16 units, and resource limits before access or allocation?
- Are behavior contracts backed by round-trip, boundary, malformed-input, or regression tests as appropriate?
- Do failures preserve actionable context without panicking or becoming an empty success?
- If a Rust IPC contract changed, were generated TypeScript bindings regenerated and checked?
- Did the change preserve the requirement ID and documentation-anchor contracts?

## Sources

- [`docs/01-data-formats.md` implementation notes](../../../docs/01-data-formats.md)
- [`docs/02-architecture.md` sections 0, 8, and 8.5](../../../docs/02-architecture.md)
- [`NFR-REL-004`, `NFR-MAINT-001..006`](../../../docs/20-nonfunctional.md)
- [`docs/22-roadmap.md` S0 and S4](../../../docs/22-roadmap.md)
- [`wubilex-codec` contract tests](../../../crates/wubilex-codec/tests/model_contracts.rs)

These model, error, and limit tests are established examples. Parser round trips, real fixtures, and coverage evidence remain obligations for the later S0 format tasks.
