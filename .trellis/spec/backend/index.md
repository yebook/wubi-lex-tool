# Backend Development Guidelines

> Project-specific contracts for the Rust workspace and Tauri application layer.

---

## Status Model

- **Baseline; examples pending** means the rule is already fixed by approved architecture, requirements, or the scaffolded repository layout. Real source examples must be added after S0 produces compilable code.
- **Baseline with S0 codec evidence** means the baseline now cites compiled `wubilex-codec` source and tests, while examples for other backend crates remain pending.
- **Pending implementation evidence** means the project has not established an actual convention yet. Do not infer one from templates or `.gitkeep` files.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Workspace membership, crate ownership, and dependency boundaries | **Baseline with S0 codec evidence** |
| [Database Guidelines](./database-guidelines.md) | Persistence, schema, query, and migration patterns | **Pending implementation evidence** |
| [Error Handling](./error-handling.md) | Library errors, command-boundary errors, and failure context | **Baseline with S0 codec evidence** |
| [Quality Guidelines](./quality-guidelines.md) | Rust gates, test obligations, and binary parsing restrictions | **Baseline with S0 codec evidence** |
| [Logging Guidelines](./logging-guidelines.md) | Structured logging fields, levels, and redaction | **Pending implementation evidence** |

The three baseline guides now include compiled S0 examples from the shared models and the raw `.lex`, EUDP, and community lexicon text codecs in `wubilex-codec`. Keep `00-bootstrap-guidelines` active until the remaining S0 crates provide equivalent implementation evidence.

---

**Language**: All documentation in this directory must be written in **English**.
