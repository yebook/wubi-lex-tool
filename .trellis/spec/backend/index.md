# Backend Development Guidelines

> Project-specific contracts for the Rust workspace and Tauri application layer.

---

## Status Model

- **Baseline; examples pending** means the rule is already fixed by approved architecture, requirements, or the scaffolded repository layout. Real source examples must be added after S0 produces compilable code.
- **Baseline with S0 implementation evidence** means the baseline cites compiled S0 source, tests, or repository automation while examples for the remaining product crates are still pending.
- **Pending implementation evidence** means the project has not established an actual convention yet. Do not infer one from templates or `.gitkeep` files.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Workspace membership, crate ownership, and dependency boundaries | **Baseline with S0 implementation evidence** |
| [Database Guidelines](./database-guidelines.md) | Persistence, schema, query, and migration patterns | **Pending implementation evidence** |
| [Error Handling](./error-handling.md) | Library errors, command-boundary errors, and failure context | **Baseline with S0 implementation evidence** |
| [Quality Guidelines](./quality-guidelines.md) | Rust gates, test obligations, repository commands, and CI contracts | **Baseline with S0 implementation evidence** |
| [Repository Quality And CI](./repository-quality-ci.md) | xtask signatures, generated bindings, document checks, audits, caches, and Windows workflow | **Established** |
| [Windows System Integration](./windows-system-integration.md) | Reversible TSF, ACL, Scheduler API, evidence, and restoration contracts | **Baseline with S0 risk-spike evidence** |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging fields, levels, and redaction | **Pending implementation evidence** |

The baseline guides now include compiled S0 examples from the codecs, test-only
real-fixture automation, the Rust-owned IPC binding registry, document
validation, dependency policy, the Windows quality workflow, and reversible
Windows risk-spike evidence. Keep `00-bootstrap-guidelines` active until the
remaining product crates provide equivalent implementation evidence.

---

**Language**: All documentation in this directory must be written in **English**.
