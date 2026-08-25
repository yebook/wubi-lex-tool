# Virtualization And Performance Probes

> Large-list ownership and reproducible visible-browser measurement contracts.

---

## Current Status

The isolated S0 TanStack Virtual spike establishes the performance-probe and
bounded-DOM contracts below. The S2 product virtual table remains pending and
must reuse these contracts without copying the spike into a product route.

## Scenario: Large Virtualized Collections

### 1. Scope / Trigger

Apply this contract when rendering a lexicon-sized collection or adding a
browser benchmark for scrolling behavior. Large collections remain paged or
index-derived; the WebView must not own a second full array of row objects.

### 2. Signatures

The local visible benchmark has one supported command:

```text
pnpm run spike:virtual-scroll -- --output <result.json>
```

The project-pinned pnpm 11 forwards the separator as a literal argument. The runner parser
therefore accepts exactly these equivalent vectors and rejects every other one:

```text
--output <result.json>
-- --output <result.json>
```

The page exposes one benchmark controller:

```typescript
interface SpikeController {
  ready: boolean;
  run(runNumber: number): Promise<RunMetrics>;
}
```

### 3. Contracts

- The reference spike uses 300,000 index-derived rows, a 32 px fixed row size,
  a 640 px viewport, overscan 12, and at most 64 rendered row elements. It never
  creates a 300,000-object array.
- Run three foreground samples. Each run has a one-second warm-up followed by a
  sample of at least five seconds. Every valid run must reach at least 55 fps.
- Record elapsed time, frames, FPS, p95/max frame interval, scroll range, DOM
  row bound, visibility, blank-row observation, runtime errors, and available
  JavaScript heap measurements.
- Install pageerror and console-error listeners before navigation. Carry an
  error cursor across initialization, samples, and inter-run gaps so no error
  falls outside a verdict window.
- The harness must be self-contained. Even an implicit missing favicon is a
  runtime error; declare an inline favicon rather than filtering the 404.
- Write JSON only to the caller-selected output and close Edge and Vite in
  `finally`. The visible benchmark is a local manual gate, not a CI command.
- Use the direct `pnpm` executable resolved from `package.json.volta.pnpm` with
  `VOLTA_FEATURE_PNPM=1`. Do not add Corepack, npm, yarn, npx,
  `engines.pnpm`, or another package-manager version source.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Argument vector differs from either accepted form | Exit nonzero with the exact usage string |
| Browser is hidden during any sample | Mark the run invalid |
| Sample is shorter than 4.9 seconds or scroll range is not meaningful | Mark the run invalid |
| DOM row count is zero, blank, or greater than 64 | Mark the run invalid |
| Page/console error occurs before, during, or between samples | Attach it to a run and fail the aggregate |
| Any valid run is below 55 fps | Fail without lowering the threshold |
| Fewer or more than three runs are returned | Fail the aggregate |
| Runner throws or verdict fails | Still close browser/server and retain written evidence when available |

### 5. Good / Base / Bad Cases

- Good: three visible, error-free samples meet the FPS and DOM thresholds and
  produce a structured JSON report.
- Base: deterministic unit tests exercise row derivation and verdict logic
  without starting Vite or a visible browser.
- Bad: rendering all row objects, using a headless result as the local pass,
  clearing initialization errors before run 1, ignoring a 404, or judging
  smoothness by eye.

### 6. Tests Required

- Unit-test derived labels, FPS/percentile math, sample validity, blank and
  unbounded DOM rejection, and exact three-run aggregation.
- Unit-test both pnpm/direct argument vectors plus missing, duplicate, unknown,
  and extra arguments.
- Statistically verify the harness HTML declares its self-contained favicon.
- The manual result must include all three raw runs, environment/version fields,
  fixed config, per-run verdicts, and an aggregate `passed` value.
- When runner error capture or harness resources change, rerun visible Edge;
  deterministic tests alone cannot replace the performance evidence.

### 7. Wrong vs Correct

```typescript
// Wrong: initialization errors are discarded before the first sample.
const errorStart = runtimeErrors.length;
const firstRun = await controller.run(1);

// Correct: begin at cursor zero and advance only after assigning every error.
let errorCursor = 0;
const firstRun = await controller.run(1);
firstRun.errors.push(...runtimeErrors.slice(errorCursor));
errorCursor = runtimeErrors.length;
```

## Sources

- [Virtual-scroll spike](../../../spikes/virtual-scroll/)
- [`S0 risk-spike design`](../../tasks/archive/2026-08/08-24-s0-risk-spikes/design.md)
- [`S0 virtual-scroll result`](../../tasks/archive/2026-08/08-24-s0-risk-spikes/research/results/virtual-scroll.md)
