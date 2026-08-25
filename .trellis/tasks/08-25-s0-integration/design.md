# S0 Integration Design

## 1. Integration Boundary

This task is an evidence and specification closure pass. It does not add product
behavior. Its writable scope is limited to the root toolchain manifest, CI and
workflow-contract code/tests, architecture and Trellis specifications, active
task records, and integration evidence. Archived task records remain historical.
A product-code failure discovered here is a blocker to classify, not permission
to expand S0 silently.

## 2. Evidence Model

Create `research/integration-results.md` as the single integration record. It
contains:

- one row for each of the ten archived child tasks;
- one row for each parent requirement and acceptance criterion;
- exact validation commands and outcomes;
- the S1 entry verdict and remaining environment limitations.

Evidence priority is current executable checks, committed source/specs, archived
task results, then task metadata. A checked box without one of these sources is
not evidence.

On 2026-08-25, the user explicitly approved skipping the independent
byte-for-byte comparison against aardio/original-project output. The accepted
S0 evidence for the seven text formats is the existing complete canonical
string assertions, deterministic real-fixture projections, strict encoding,
whitespace escaping, and named regression coverage. This criterion change does
not claim that a legacy golden or aardio runtime exists and does not modify any
raw `.lex` or EUDP byte-level requirement.

## 3. Specification Closure

Every backend/frontend spec file must be in one explicit state:

1. **Established**: executable implementation and tests exist.
2. **Baseline with evidence**: approved boundary exists and S0 supplies partial
   implementation evidence.
3. **Pending implementation evidence**: no pattern exists yet; the document
   states the fixed boundary, decisions that remain unselected, forbidden
   premature assumptions, and the event that must update the spec.

The third state is intentional documentation, not an empty template. Replace
all `(To be filled by the team)` sections, but do not add hypothetical database,
logging, component, hook, or store examples. Existing S0 code examples in the
codec, repository automation, binding, CI, Windows, and virtualization specs
satisfy the bootstrap requirement for real examples.

## 4. Parent And Bootstrap Data Flow

```text
archived child tasks + current repository + full gates
  -> integration evidence matrix
  -> spec placeholder closure
  -> bootstrap checklist update
  -> parent PRD/implementation checklist update
  -> final quality review
  -> archive integration, bootstrap, and parent
```

Parent checkboxes are updated only after the evidence matrix is complete.
Bootstrap is archived only when every spec file is non-placeholder and its
status is honest about the available implementation evidence.

## 5. Validation Contract

The final gate mirrors the checked-in Windows CI and repository quality specs:

- Rust formatting, workspace check, strict Clippy, all tests, warnings-denied
  Rustdoc, and codec coverage at or above 90%;
- offline fixture verification, cargo-deny, binding freshness, document
  validation, and independent actionlint syntax validation;
- Volta-pinned pnpm forced frozen reinstall, security audit, TypeScript,
  ESLint, and Vitest;
- Trellis context validation, placeholder scan, safety scan, and diff check.

No Windows `--live` or visible Edge command is part of this integration rerun.
The already committed raw evidence is verified structurally and against its
reports.

## 6. Project-Level pnpm Migration

`package.json` is the only Node/pnpm version contract:

```text
package.json.volta.node + package.json.volta.pnpm
  -> local Volta shim resolution
  -> pnpm frozen forced reinstall
  -> frontend gates
```

Run `volta pin pnpm@11.18.0`, remove `engines.pnpm`, and keep
`packageManager` absent. The checked-in CI uses `volta-cli/action` to hydrate
both project pins; it removes the separate pnpm-version reader and
`pnpm/action-setup` path. Repository contract tests must require
`volta.pnpm`, reject `engines.pnpm`, `packageManager`, and Corepack, and compare
the running `pnpm --version` with the pin.

Volta `2.0.2`, the latest published release, still gates pnpm support behind
`VOLTA_FEATURE_PNPM=1`. Without it the pin command fails before changing the
manifest. The user has enabled this prerequisite in the Windows user
environment. Set it explicitly in CI and also copy the persisted value into the
current implementation process because already-running processes do not
inherit a newly persisted user variable. This flag enables Volta's pnpm
resolver; it is not a pnpm version source and does not replace
`package.json.volta.pnpm`.

Capture the `pnpm-lock.yaml` SHA-256 before migration, reinstall with
`pnpm install --frozen-lockfile --force`, and require the same hash afterward.
The forced install satisfies the requested reinstall while the frozen lockfile
prevents dependency resolution drift.

## 7. Failure And Rollback

- A code/test gate failure keeps the parent and bootstrap active and records the
  exact blocker. Fix only a narrow S0 regression that is clearly within scope;
  otherwise create a separately reviewed child plan.
- A spec contradiction returns to planning before checklist changes.
- A failed project pin or reinstall restores the pre-migration `package.json`
  and leaves the task active; no global package-manager installation is added.
- If the experimental Volta prerequisite is missing on another machine, fail
  with the required environment setup instead of hand-writing a pin that the
  local shim cannot honor.
- Task/spec documentation edits can be reverted as one integration work commit.
- Archive commits happen only after the integration work commit, so task closure
  never hides uncommitted fixes.

## 8. Completion Sequence

After a full-scope `trellis-check` passes, update specs if the integration found
new executable knowledge, commit the integration artifacts, archive the current
child, archive the completed bootstrap and parent tasks, record the journal, and
push only when the user authorizes the resulting commit sequence.
