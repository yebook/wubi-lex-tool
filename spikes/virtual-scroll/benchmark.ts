export const SPIKE_CONFIG = {
  count: 300_000,
  rowHeight: 32,
  viewportHeight: 640,
  overscan: 12,
  maximumDomRows: 64,
  minimumFps: 55,
  warmupMs: 1_000,
  sampleMs: 5_000,
  runCount: 3,
} as const;

export interface MemoryObservation {
  jsHeapSizeLimit: number;
  totalJSHeapSize: number;
  usedJSHeapSize: number;
}

export interface RunMetrics {
  run: number;
  elapsedMs: number;
  frameCount: number;
  fps: number;
  p95FrameIntervalMs: number;
  maximumFrameIntervalMs: number;
  maximumDomRows: number;
  minimumScrollOffset: number;
  maximumScrollOffset: number;
  finalScrollOffset: number;
  visibleThroughout: boolean;
  blankRowObserved: boolean;
  memory: MemoryObservation | null;
  errors: string[];
}

export interface RunVerdict {
  valid: boolean;
  passed: boolean;
  reasons: string[];
}

export function deriveRow(index: number): string {
  return `Synthetic row ${String(index + 1).padStart(6, "0")} / code-${index.toString(36)}`;
}

export function percentile(values: readonly number[], quantile: number): number {
  if (values.length === 0) {
    return 0;
  }
  const sorted = [...values].sort((left, right) => left - right);
  const rawIndex = Math.ceil(sorted.length * quantile) - 1;
  const index = Math.max(0, Math.min(sorted.length - 1, rawIndex));
  return sorted[index] ?? 0;
}

export function calculateFps(frameCount: number, elapsedMs: number): number {
  return elapsedMs > 0 ? (1_000 * frameCount) / elapsedMs : 0;
}

export function evaluateRun(metrics: RunMetrics): RunVerdict {
  const reasons = [...metrics.errors];
  if (!metrics.visibleThroughout) {
    reasons.push("document was not visible throughout the sample");
  }
  if (metrics.elapsedMs < 4_900) {
    reasons.push(`sample was too short: ${metrics.elapsedMs.toFixed(1)} ms`);
  }
  if (metrics.maximumScrollOffset - metrics.minimumScrollOffset < SPIKE_CONFIG.viewportHeight * 10) {
    reasons.push("scroll range was not meaningful");
  }
  if (metrics.maximumDomRows > SPIKE_CONFIG.maximumDomRows) {
    reasons.push(`DOM row bound exceeded: ${metrics.maximumDomRows}`);
  }
  if (metrics.maximumDomRows === 0) {
    reasons.push("scroll viewport rendered zero rows");
  }
  if (metrics.blankRowObserved) {
    reasons.push("a rendered row was blank");
  }
  const valid = reasons.length === 0;
  return {
    valid,
    passed: valid && metrics.fps >= SPIKE_CONFIG.minimumFps,
    reasons: valid && metrics.fps < SPIKE_CONFIG.minimumFps
      ? [`fps ${metrics.fps.toFixed(2)} is below ${SPIKE_CONFIG.minimumFps}`]
      : reasons,
  };
}

export function allRunsPass(runs: readonly RunMetrics[]): boolean {
  return runs.length === SPIKE_CONFIG.runCount && runs.every((run) => evaluateRun(run).passed);
}
