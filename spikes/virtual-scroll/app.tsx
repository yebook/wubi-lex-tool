import { useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import {
  calculateFps,
  deriveRow,
  percentile,
  SPIKE_CONFIG,
  type MemoryObservation,
  type RunMetrics,
} from "./benchmark";

interface PerformanceWithMemory extends Performance {
  memory?: MemoryObservation;
}

interface SpikeController {
  ready: boolean;
  run: (run: number) => Promise<RunMetrics>;
}

declare global {
  interface Window {
    __virtualScrollSpike?: SpikeController;
  }
}

function nextFrame(): Promise<number> {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}

async function animateScroll(
  element: HTMLDivElement,
  durationMs: number,
  collect: boolean,
): Promise<Omit<RunMetrics, "run" | "errors">> {
  const timestamps: number[] = [];
  let visibleThroughout = document.visibilityState === "visible";
  let blankRowObserved = false;
  let maximumDomRows = 0;
  let minimumScrollOffset = Number.POSITIVE_INFINITY;
  let maximumScrollOffset = 0;
  const start = await nextFrame();
  let now = start;

  while (now - start < durationMs) {
    const progress = Math.min(1, (now - start) / durationMs);
    const triangle = progress <= 0.5 ? progress * 2 : (1 - progress) * 2;
    const maximumOffset = Math.max(0, element.scrollHeight - element.clientHeight);
    element.scrollTop = triangle * maximumOffset;
    const rows = Array.from(element.querySelectorAll<HTMLElement>("[data-virtual-row]"));
    maximumDomRows = Math.max(maximumDomRows, rows.length);
    blankRowObserved ||=
      rows.length === 0 || rows.some((row) => (row.textContent ?? "").trim().length === 0);
    visibleThroughout &&= document.visibilityState === "visible";
    minimumScrollOffset = Math.min(minimumScrollOffset, element.scrollTop);
    maximumScrollOffset = Math.max(maximumScrollOffset, element.scrollTop);
    if (collect) {
      timestamps.push(now);
    }
    now = await nextFrame();
  }

  const elapsedMs = now - start;
  const intervals = timestamps.slice(1).map((value, index) => value - (timestamps[index] ?? value));
  const memory = (performance as PerformanceWithMemory).memory;
  return {
    elapsedMs,
    frameCount: timestamps.length,
    fps: calculateFps(timestamps.length, elapsedMs),
    p95FrameIntervalMs: percentile(intervals, 0.95),
    maximumFrameIntervalMs: intervals.length === 0 ? 0 : Math.max(...intervals),
    maximumDomRows,
    minimumScrollOffset: Number.isFinite(minimumScrollOffset) ? minimumScrollOffset : 0,
    maximumScrollOffset,
    finalScrollOffset: element.scrollTop,
    visibleThroughout,
    blankRowObserved,
    memory: memory
      ? {
          jsHeapSizeLimit: memory.jsHeapSizeLimit,
          totalJSHeapSize: memory.totalJSHeapSize,
          usedJSHeapSize: memory.usedJSHeapSize,
        }
      : null,
  };
}

export function VirtualScrollSpike() {
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: SPIKE_CONFIG.count,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => SPIKE_CONFIG.rowHeight,
    overscan: SPIKE_CONFIG.overscan,
  });

  useEffect(() => {
    window.__virtualScrollSpike = {
      ready: true,
      run: async (run) => {
        const element = scrollRef.current;
        if (!element) {
          throw new Error("scroll viewport is unavailable");
        }
        element.scrollTop = 0;
        await nextFrame();
        await nextFrame();
        await animateScroll(element, SPIKE_CONFIG.warmupMs, false);
        const metrics = await animateScroll(element, SPIKE_CONFIG.sampleMs, true);
        return { run, ...metrics, errors: [] };
      },
    };
    document.documentElement.dataset.spikeReady = "true";
    return () => {
      delete window.__virtualScrollSpike;
      delete document.documentElement.dataset.spikeReady;
    };
  }, []);

  const items = virtualizer.getVirtualItems();
  return (
    <main>
      <h1>300,000-row virtual scroll benchmark</h1>
      <div className="status">
        Rows {SPIKE_CONFIG.count.toLocaleString()} / row {SPIKE_CONFIG.rowHeight}px / overscan {SPIKE_CONFIG.overscan}
      </div>
      <div
        ref={scrollRef}
        data-scroll-viewport
        className="viewport"
        style={{ height: SPIKE_CONFIG.viewportHeight }}
      >
        <div className="spacer" style={{ height: virtualizer.getTotalSize() }}>
          {items.map((item) => (
            <div
              key={item.key}
              data-virtual-row
              className="row"
              style={{
                height: item.size,
                transform: `translateY(${item.start}px)`,
              }}
            >
              <span>{item.index + 1}</span>
              <strong>{deriveRow(item.index)}</strong>
            </div>
          ))}
        </div>
      </div>
    </main>
  );
}
