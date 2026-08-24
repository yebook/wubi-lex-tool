# Quality Guidelines

> Frontend quality gates and responsibility boundaries.

---

## Required Gates

Frontend changes use the global pnpm command only and must pass frozen installation, `pnpm audit --audit-level high`, the TypeScript compiler with no emit, ESLint with zero warnings, and Vitest. Node comes from `package.json.volta.node`; pnpm must match `package.json.engines.pnpm`. Do not introduce a Volta project-level pnpm pin, `VOLTA_FEATURE_PNPM`, npm, yarn, npx, corepack, `.nvmrc`, or a competing `packageManager` version source.

When a change touches IPC, `cargo xtask bindings --check` is part of the frontend gate even if TypeScript compilation succeeds. `cargo xtask check-docs` also guards requirement references consumed by frontend work.

The checked-in Windows workflow reads versions from those repository fields, keys the pnpm store cache by `pnpm-lock.yaml`, and runs every frontend command after the Rust, audit, binding, and documentation gates. Cache restoration never replaces `pnpm install --frozen-lockfile`. A developer mirror that lacks the npm audit endpoint may be diagnosed with a command-local official `--registry` override; do not commit an `.npmrc` or report the missing endpoint as a clean audit.

## Required Boundaries

- Keep transformations, slimming, word generation, and file parsing in Rust. The frontend issues typed commands and renders results.
- Fetch only the current page or viewport of a large lexicon. Do not hold a second complete lexicon in frontend state.
- Read feature availability from the backend-populated Zustand store. Do not infer availability from missing commands or duplicate flags.
- Use `src/styles/theme.css` as the Tailwind v4 token source and follow the established Tailwind token guide.
- Import shared IPC contracts from generated bindings rather than redefining or casting payloads locally.

## Testing Requirements

- Vitest covers frontend units and components as they are introduced.
- Component tests cover empty, loading, failure, disabled, and placeholder states when those states are part of the component contract.
- Cross-layer tests verify command and event serialization through generated types rather than duplicating fixture interfaces in TypeScript.
- End-to-end tests are required for the documented critical flow once the runnable shell and relevant stages exist: load a lexicon, edit it, and install it.
- Feature-placeholder behavior is tested with backend feature switches disabled so unfinished commands cannot appear active.

## Forbidden Patterns

- Business or codec logic in React components, hooks, stores, or IPC wrappers.
- Full-file parsing or full-lexicon ownership in the WebView.
- Handwritten IPC type mirrors or edits to generated output.
- A second toolchain version source or non-pnpm package-manager commands.
- A floating third-party Action ref, `continue-on-error`, or a cache condition that bypasses install, audit, typecheck, lint, or tests.
- Treating a scaffold directory as proof of a component, hook, or state convention.

## Review Checklist

- Is the change in the route, shared component, infrastructure, or generated-type directory that owns it?
- Does data cross IPC through the generated contract and a single typed boundary?
- Are loading, empty, failure, cancellation, and feature-disabled outcomes handled where relevant?
- Are large collections paged or virtualized rather than copied into frontend state?
- Do the TypeScript, ESLint, Vitest, and binding checks cover the changed surface?
- Does styling use the approved Tailwind v4 token mechanism?

## Sources

- [`docs/02-architecture.md` sections 6.2, D9, D11, D16, 8, and 8.5](../../../docs/02-architecture.md)
- [`docs/20-nonfunctional.md`](../../../docs/20-nonfunctional.md)
- [`docs/21-ui-ux.md`](../../../docs/21-ui-ux.md)
- [Tailwind v4 Token Convention](./tailwind-v4-tokens.md)
- [Windows quality workflow](../../../.github/workflows/ci.yml)
- [Frontend toolchain version sources](../../../package.json)

The binding freshness and global-pnpm CI gates are established. Component and browser-flow examples remain pending until the frontend shell exists.
