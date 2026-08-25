# Database Guidelines

> Persistence boundaries before a database implementation exists.

---

## Current Status

**Pending implementation evidence.** The repository has no database crate,
schema, query layer, migration, or database test. S0 established codec and
repository infrastructure only. This document records the approved persistence
boundary without selecting a database design for a later phase.

## Approved Boundary

- Application configuration is not a database concern. Architecture decision
  D12 assigns it to strongly typed `serde` + TOML under
  `src-tauri/src/config/`, with a schema version, corruption fallback, backup,
  atomic replacement and migration support.
- User lexicon data and disposable resource caches must remain physically
  separate. A cache may be deleted without deleting user-owned data.
- `wubilex-codec` remains synchronous and memory-to-memory. It must not acquire
  database, ORM, filesystem, Tauri or asynchronous dependencies.
- The frontend never reads a persistence schema. Data crosses the application
  boundary through generated command/event contracts and frontend views
  request only the page or viewport they need.

These are placement and ownership constraints, not proof that SQL or another
database is required.

## Decisions Not Yet Established

The project has not selected:

- whether any product data needs a database rather than files and in-memory
  indexes;
- a database engine, ORM, query library, connection model or pooling policy;
- schema, table, column, index or foreign-key naming conventions;
- transaction boundaries, batching rules or concurrency behavior;
- a migration file format, migration runner, rollback policy or test fixture
  strategy.

## Forbidden Premature Assumptions

- Do not add a database dependency merely to fill this guideline or mirror a
  generic Tauri template.
- Do not treat scaffolded directories, config requirements or the legacy
  application's file layout as evidence for a database convention.
- Do not move the D12 TOML configuration contract into a database without an
  explicit architecture review.
- Do not expose persistence rows directly through Tauri commands or duplicate
  a future schema in TypeScript.
- Do not place persistence access in `wubilex-codec` or frontend code.

## Update Trigger

Update this guide in the same reviewed task that first introduces database
persistence. That task must select the owning crate and library, define schema
and migration behavior, add executable tests for queries/transactions/failure
recovery, and cite the resulting source paths here. Until then, no query,
migration or naming example is a project convention.

## Sources

- [`docs/02-architecture.md` D12 and workspace boundaries](../../../docs/02-architecture.md)
- [`docs/modules/M7-app-shell.md` configuration requirements](../../../docs/modules/M7-app-shell.md)
- [`docs/20-nonfunctional.md` data safety requirements](../../../docs/20-nonfunctional.md)
- [Backend directory structure](./directory-structure.md)
