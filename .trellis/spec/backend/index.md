# Backend Development Guidelines

> Project-specific contracts for the Rust workspace and Tauri application layer.

---

## Status Model

- **Baseline; examples pending** means the rule is already fixed by approved architecture, requirements, or the scaffolded repository layout. Real source examples must be added only after the relevant product area produces compilable code.
- **Baseline with S0 implementation evidence** means the baseline cites compiled S0 source, tests, or repository automation while examples for the remaining product crates are still pending.
- **Baseline with S1 application evidence** means the rule now cites the runnable lifecycle, transactional configuration, generated command errors, or feature catalog while later domain and system-operation examples remain pending.
- **Pending implementation evidence** means the project has not established an
  actual convention yet. The guide must still record approved boundaries,
  unselected decisions, forbidden assumptions and the event that will update
  it; do not infer examples from templates or `.gitkeep` files.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Workspace membership, crate ownership, and dependency boundaries | **Baseline with S1 application evidence** |
| [Database Guidelines](./database-guidelines.md) | Persistence, schema, query, and migration patterns | **Pending implementation evidence** |
| [Error Handling](./error-handling.md) | Library errors, command-boundary errors, and failure context | **Baseline with S1 application evidence** |
| [Quality Guidelines](./quality-guidelines.md) | Rust gates, test obligations, repository commands, and CI contracts | **Baseline with S1 application evidence** |
| [Repository Quality And CI](./repository-quality-ci.md) | xtask signatures, generated bindings, document checks, audits, caches, Windows workflow, and runtime smoke | **Established** |
| [Windows System Integration](./windows-system-integration.md) | Elevation detection, reversible TSF/ACL/Scheduler contracts, and forbidden companion-tool boundary | **Baseline with S1 runtime evidence** |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging fields, retention, levels, and redaction | **Established by S1 runtime** |
| [Window Coordinator](./window-coordinator.md) | Native window/tray lifecycle, placement persistence, and window IPC | **Established by S1 window/tray** |

The baseline guides include compiled S0 examples from the codecs, test-only
real-fixture automation, the Rust-owned IPC binding registry, document
validation, dependency policy, the Windows quality workflow, reversible Windows
risk-spike evidence, and the S1 runtime lifecycle. Bootstrap completeness means
every guide is non-placeholder and honest about its evidence state; database
decisions remain pending while logging and native window coordination now have
executable product evidence.

---

**Language**: All documentation in this directory must be written in **English**.
