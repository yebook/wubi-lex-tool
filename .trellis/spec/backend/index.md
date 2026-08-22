# Backend Development Guidelines

> Project-specific contracts for the Rust workspace and Tauri application layer.

---

## Status Model

- **Baseline; examples pending** means the rule is already fixed by approved architecture, requirements, or the scaffolded repository layout. Real source examples must be added after S0 produces compilable code.
- **Pending implementation evidence** means the project has not established an actual convention yet. Do not infer one from templates or `.gitkeep` files.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Workspace membership, crate ownership, and dependency boundaries | **Baseline; examples pending** |
| [Database Guidelines](./database-guidelines.md) | Persistence, schema, query, and migration patterns | **Pending implementation evidence** |
| [Error Handling](./error-handling.md) | Library errors, command-boundary errors, and failure context | **Baseline; examples pending** |
| [Quality Guidelines](./quality-guidelines.md) | Rust gates, test obligations, and binary parsing restrictions | **Baseline; examples pending** |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging fields, levels, and redaction | **Pending implementation evidence** |

The baseline guides cite their current sources instead of presenting placeholder code as an established pattern. Keep `00-bootstrap-guidelines` active until real S0 implementation examples can replace this evidence-only baseline.

---

**Language**: All documentation in this directory must be written in **English**.
