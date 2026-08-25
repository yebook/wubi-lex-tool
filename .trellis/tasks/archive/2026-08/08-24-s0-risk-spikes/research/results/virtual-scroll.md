# Virtual Scroll Spike Result

## Status

`PASS`

Captured on 2026-08-25 using global pnpm 11.18.0, Node 24.18.1, and
Microsoft Edge 151.0.4129.101.

## Commands And Evidence

```powershell
pnpm run typecheck
pnpm run lint
pnpm run test --run
pnpm run spike:virtual-scroll -- --output .trellis/tasks/08-24-s0-risk-spikes/research/results/virtual-scroll.json
```

- TypeScript project checks passed, including `spikes/virtual-scroll/tsconfig.json`.
- ESLint passed with zero warnings.
- Vitest passed three files and 15 tests, covering derived rows, FPS/percentile
  calculations, bounded DOM rejection, zero-row viewport rejection, three-run
  aggregate verdicts, runner arguments, and the inline favicon contract.

The harness derives each row from its virtual index and does not allocate a
300,000-object row array.

## Measurements

| Run | FPS | p95 frame interval | Maximum frame interval | Maximum DOM rows | Visible | Blank row | Errors |
|---:|---:|---:|---:|---:|---|---|---|
| 1 | 119.60 | 8.50 ms | 16.80 ms | 45 | yes | no | none |
| 2 | 93.40 | 16.80 ms | 25.30 ms | 45 | yes | no | none |
| 3 | 110.02 | 16.70 ms | 25.20 ms | 45 | yes | no | none |

All runs traversed approximately 9.60 million pixels. Reported JavaScript heap
usage was 23.7 MB, 31.0 MB, and 40.5 MB respectively, against a 4.40 GB heap
limit. The raw, machine-readable evidence is retained in `virtual-scroll.json`.

## Verdict And Cleanup

All three valid runs exceeded the 55 fps threshold, stayed below the 64-row DOM
bound, remained visible, rendered no blank rows, and reported no page or console
errors. The inline favicon fix was therefore verified by a clean foreground
rerun, and the virtual-scroll spike passes SPIKE-R06 and SPIKE-R07 without
weakening either threshold.

The runner closed its Edge instance and Vite server in
`finally`; it created no persistent product state beyond the requested result
file.

This is a foreground benchmark on the current development machine, not a
deterministic CI gate or a guarantee for lower-spec hardware.
