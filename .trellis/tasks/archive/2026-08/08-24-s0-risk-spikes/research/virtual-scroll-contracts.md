# Virtual Scroll Spike Contracts

## Scope

This spike proves only the S1 entry assumption behind `UX-BIGDATA-001`: a fixed-height virtual list can keep 300,000 synthetic rows scrollable in Microsoft Edge at the roadmap threshold. It does not implement the S2 product table, Rust-side paging, sorting, filtering, editing, selection, or visual design.

Repository evidence:

- `docs/22-roadmap.md` requires the pre-S1 spike and sets the pass threshold to at least 55 fps.
- `docs/21-ui-ux.md` defines the eventual product contracts in `UX-BIGDATA-001..008`; only virtualized rendering is exercised here.
- `.trellis/spec/frontend/directory-structure.md` reserves `src/components/virtual-table/` for the later product component, so the disposable harness belongs under `spikes/virtual-scroll/` rather than that production directory.
- `.trellis/spec/frontend/quality-guidelines.md` requires strict TypeScript, no hidden failures, and focused checks for frontend behavior.

## Selected Dependencies

Registry inspection on 2026-08-24 returned:

| Package | Exact version | Use |
|---|---:|---|
| `@tanstack/react-virtual` | `3.14.10` | Mature React virtualizer used by the harness. |
| `playwright-core` | `1.62.1` | Drives the installed Edge channel without downloading a Playwright browser. |

Both are development-only dependencies. Use the repository's global `pnpm` command and exact versions. Do not use Volta pnpm, Corepack, npm, yarn, or npx.

`playwright-core` is sufficient: the spike needs one deterministic runner, not Playwright Test fixtures or another bundled Chromium. Launch with `chromium.launch({ channel: "msedge", headless: false })`; the machine has Edge at `C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe`, version `151.0.4129.101`.

## Harness Boundary

Create an isolated Vite entry below `spikes/virtual-scroll/`. The runner starts Vite on loopback with an ephemeral port, launches visible Edge, runs the benchmark, writes a caller-selected JSON result, and closes both Edge and Vite in `finally`.

The harness uses:

- `count = 300_000` without allocating a 300,000-element object array;
- row values derived from the virtual item index at render time;
- a fixed `32px` row estimate;
- a fixed `640px` scroll viewport at a Playwright viewport of `1440 x 1000`;
- `overscan = 12`;
- stable row keys from the integer index;
- a total-size spacer and absolutely translated visible rows, following the TanStack Virtual contract.

The rendered DOM must remain bounded. With 20 visible rows and 12 rows of overscan on each side, the expected steady-state count is about 44. The assertion allows at most 64 `[data-virtual-row]` nodes so small boundary effects do not make the test flaky while a full or window-sized render still fails decisively.

## Measurement Protocol

The Playwright runner must bring the page to the foreground and wait for a harness-ready marker before sampling. It then performs three independent scripted runs in the same visible Edge session:

1. Reset `scrollTop` to zero and wait for two animation frames.
2. Warm up for 1,000 ms using the same `requestAnimationFrame` scroll loop used for measurement.
3. Sample for 5,000 ms while moving the viewport through a triangular, continuously changing scroll path. The script writes `scrollTop` once per animation frame and records the frame timestamp.
4. Record elapsed time, frame count, calculated fps (`1000 * frame_count / elapsed_ms`), p95 frame interval, maximum frame interval, maximum DOM row count, final scroll offset, and `performance.memory` when Edge exposes it.
5. Wait 500 ms before the next run so cleanup and layout settle.

An individual run is valid only when:

- the document is visible for the full sample;
- elapsed sample time is at least 4,900 ms;
- the scroll offset changes over a meaningful range;
- no page error, console error, or blank-row assertion occurs;
- maximum DOM row count is at most 64.

The spike passes only when all three valid runs report at least 55 fps. Report all raw run values; do not average a failing run away. This is a local feasibility threshold, not a cross-machine CI performance gate.

## Commands And Evidence

Planned commands use global pnpm:

```powershell
pnpm install --frozen-lockfile
pnpm run spike:virtual-scroll -- --output .trellis/tasks/08-24-s0-risk-spikes/research/results/virtual-scroll.json
pnpm run typecheck
pnpm run lint
pnpm run test
```

The live benchmark is intentionally not part of default tests or CI because visible Edge timing is machine- and foreground-dependent. Pure calculation helpers and DOM-bound logic should have deterministic Vitest coverage; the runner remains an explicit manual quality gate for this task.

## Failure Interpretation

- If DOM rows exceed 64, the harness integration is wrong; fix the spike before drawing an architecture conclusion.
- If any valid run is below 55 fps after eliminating page errors and background throttling, record the failure without lowering the threshold. Re-evaluate row rendering cost, paging boundaries, and the selected virtualizer before S1.
- If installed Edge cannot be automated, record the automation/environment failure separately from rendering performance. Do not substitute a downloaded browser and claim the Edge criterion passed.

