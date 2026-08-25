import { describe, expect, it } from "vitest";

import {
  allRunsPass,
  calculateFps,
  deriveRow,
  evaluateRun,
  percentile,
  type RunMetrics,
} from "./benchmark";

function passingRun(run: number): RunMetrics {
  return {
    run,
    elapsedMs: 5_000,
    frameCount: 300,
    fps: 60,
    p95FrameIntervalMs: 17,
    maximumFrameIntervalMs: 20,
    maximumDomRows: 44,
    minimumScrollOffset: 0,
    maximumScrollOffset: 9_000_000,
    finalScrollOffset: 0,
    visibleThroughout: true,
    blankRowObserved: false,
    memory: null,
    errors: [],
  };
}

describe("virtual-scroll spike metrics", () => {
  it("derives labels directly from an index", () => {
    expect(deriveRow(0)).toBe("Synthetic row 000001 / code-0");
    expect(deriveRow(299_999)).toContain("300000");
  });

  it("calculates fps and nearest-rank percentile", () => {
    expect(calculateFps(300, 5_000)).toBe(60);
    expect(percentile([8, 16, 24, 32], 0.95)).toBe(32);
  });

  it("rejects an unbounded DOM even when fps passes", () => {
    const metrics = { ...passingRun(1), maximumDomRows: 65 };
    expect(evaluateRun(metrics)).toMatchObject({ valid: false, passed: false });
  });

  it("rejects a blank viewport with zero rendered rows", () => {
    const metrics = { ...passingRun(1), maximumDomRows: 0 };
    expect(evaluateRun(metrics)).toMatchObject({ valid: false, passed: false });
  });

  it("requires every one of three runs to pass", () => {
    const runs = [passingRun(1), passingRun(2), passingRun(3)];
    expect(allRunsPass(runs)).toBe(true);
    runs[1] = { ...runs[1]!, fps: 54.99 };
    expect(allRunsPass(runs)).toBe(false);
  });
});
