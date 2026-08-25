# S0 Integration Baseline

Captured on 2026-08-25 after pushing `main` through commit `771de0a`.

## Archived Prerequisites

| Order | Task | Status |
|---:|---|---|
| 0 | `s0-docs-spec-alignment` | completed and archived |
| 1 | `s0-workspace-toolchain` | completed and archived |
| 2 | `s0-codec-model` | completed and archived |
| 3 | `s0-lex-binary` | completed and archived |
| 4 | `s0-eudp` | completed and archived |
| 5 | `s0-lex-text` | completed and archived |
| 6 | `s0-phrase-aux` | completed and archived |
| 7 | `s0-fixtures-regressions` | completed and archived |
| 8 | `s0-xtask-ci` | completed and archived |
| 9 | `s0-risk-spikes` | completed and archived |

At the pre-task baseline, the parent task metadata contained all ten prerequisite
children and reported `10/10 done`. The parent document reserved order 10 for
`s0-integration`; after this task was created, the current parent metadata
correctly reports ten completed children out of eleven.

## Current Verified Baseline

- Before the pnpm decision changed, the final risk-spike review passed Rust
  workspace gates, codec line coverage at 90.12%, cargo-deny,
  fixtures/bindings/docs, the then-approved frozen pnpm install,
  official-registry audit, TypeScript, ESLint, Vitest, Trellis validation, and
  safety/diff scans.
- Risk-spike evidence records all three Windows live exit codes as 0 and the
  Edge aggregate as `passed=true`; the integration task consumes this evidence
  read-only.
- Git was clean and `main` matched `origin/main` at `771de0a` before task
  creation.

## Revised Toolchain Baseline

- On 2026-08-25 the user replaced the user-level pnpm policy with a project-level
  Volta pin and explicitly rejected `packageManager`.
- Planning recheck: Node `24.18.1`, Volta `2.0.2`, project-directory
  `pnpm --version` `11.19.0`, `volta list pnpm` `11.18.0`.
- Implementation preflight later resolved project-directory `pnpm --version`
  to `11.23.0`; `volta list pnpm` still reported `11.18.0`. The changing
  unpinned command strengthens the recorded drift finding without changing the
  target version.
- Before migration, `package.json` contained `engines.pnpm = 11.18.0` and no
  `volta.pnpm`; this stale state explained why the running command did not
  match the approved project pin.
- The pre-migration `pnpm-lock.yaml` SHA-256 is
  `7CEB34F975BE75DDFCD83E0877E73ABD89ED2ECB84F34B25A4E7B4F3D8D0122D`.
- Implemented state: `package.json.volta.pnpm = 11.18.0` is the sole pnpm
  version source; `engines.pnpm`, `packageManager`, Corepack, and a user-level
  pnpm policy are absent.
- The user persisted `VOLTA_FEATURE_PNPM=1`; implementation copied it into the
  current process, ran `volta pin pnpm@11.18.0` successfully, and changed the
  project-directory resolution from `11.23.0` to `11.18.0` without changing
  the Node pin.
- `pnpm install --frozen-lockfile --force` completed with pnpm `11.18.0` and
  retained SHA-256
  `7CEB34F975BE75DDFCD83E0877E73ABD89ED2ECB84F34B25A4E7B4F3D8D0122D`.

## Initial Integration Gaps And Disposition

1. Parent acceptance and implementation checkboxes lagged behind completed
   child evidence; the integration pass reconciled them, then reopened only
   the migrated final-gate criterion.
2. `00-bootstrap-guidelines` initially had three unchecked completion boxes;
   the integration evidence now supports all three without inventing product
   conventions.
3. Database, logging, component, Hook, and state-management specs initially
   contained empty scaffolding; they now record honest pending-evidence
   boundaries and update triggers.
4. A post-migration final same-tree integration gate and unified S1 entry
   verdict have not yet been recorded.
5. An independent Phase 2.2 review must recheck the migrated contracts before
   S0 closure and archive operations.
6. Volta `2.0.2` still requires the experimental `VOLTA_FEATURE_PNPM=1`
   prerequisite on each machine and in CI; npm/yarn and Corepack remain
   forbidden fallbacks.

The remaining items are integration-state work. They do not authorize new S1
product behavior or machine-state mutation.
