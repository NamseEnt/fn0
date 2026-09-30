import { alignGaugeRows, alignedRange, emptyColumn } from "./history.ts";
import { COLLECTY_DROPPED_SEGMENTS, COLLECTY_QUEUE_BYTES, COLLECTY_REFUSED_SEGMENTS } from "./metrics.ts";
import type { TelemetryHistoryResponse } from "./schema.ts";
import type { Dependencies } from "./runtime.ts";
import { SignyClient } from "./signy.ts";
import { type TimeWindow, stepGrid } from "./windows.ts";

export async function telemetrySeries(dependencies: Dependencies, window: TimeWindow): Promise<TelemetryHistoryResponse> {
  const signy = new SignyClient(dependencies);
  const grid = stepGrid(window, dependencies.nowMs());
  const gridRange = {
    startSeconds: grid[0]!,
    endSeconds: grid[grid.length - 1]!,
    stepSeconds: window.stepSeconds,
  };
  const range = `${window.stepSeconds}s`;
  const [queue, dropped, refused] = await Promise.all([
    signy.range({ metric: COLLECTY_QUEUE_BYTES, agg: "sum" }, gridRange),
    signy.range({ metric: COLLECTY_DROPPED_SEGMENTS, func: "increase", range, agg: "sum" }, gridRange),
    signy.range({ metric: COLLECTY_REFUSED_SEGMENTS, func: "increase", range, agg: "sum" }, gridRange),
  ]);
  const empty = () => emptyColumn(grid.length);
  return {
    window: window.name,
    step_seconds: window.stepSeconds,
    generated_at: new Date(dependencies.nowMs()).toISOString(),
    timestamps_ms: grid.map((seconds) => seconds * 1000),
    queue: {
      telemetry: queue.ok ? "ok" : "unavailable",
      bytes: queue.ok ? alignGaugeRows(grid, queue.value, window.stepSeconds) : empty(),
    },
    segments: {
      telemetry: dropped.ok && refused.ok ? "ok" : "unavailable",
      dropped: dropped.ok ? alignedRange(grid, dropped.value, window.stepSeconds) : empty(),
      refused: refused.ok ? alignedRange(grid, refused.value, window.stepSeconds) : empty(),
    },
  };
}
