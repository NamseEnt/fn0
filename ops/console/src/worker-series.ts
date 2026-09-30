import { alignGaugeRows, emptyColumn } from "./history.ts";
import type { WorkerHistoryResponse } from "./schema.ts";
import type { Dependencies } from "./runtime.ts";
import { SignyClient } from "./signy.ts";
import { type TimeWindow, stepGrid } from "./windows.ts";

export async function workerSeries(dependencies: Dependencies, window: TimeWindow): Promise<WorkerHistoryResponse> {
  const signy = new SignyClient(dependencies);
  const grid = stepGrid(window, dependencies.nowMs());
  const gridRange = {
    startSeconds: grid[0]!,
    endSeconds: grid[grid.length - 1]!,
    stepSeconds: window.stepSeconds,
  };
  const [inFlight, websockets] = await Promise.all([
    signy.range({ metric: "fn0.worker.requests.in_flight", agg: "sum" }, gridRange),
    signy.range({ metric: "fn0.worker.websocket.connections", agg: "sum" }, gridRange),
  ]);
  const empty = () => emptyColumn(grid.length);
  return {
    window: window.name,
    step_seconds: window.stepSeconds,
    generated_at: new Date(dependencies.nowMs()).toISOString(),
    timestamps_ms: grid.map((seconds) => seconds * 1000),
    telemetry: inFlight.ok && websockets.ok ? "ok" : "unavailable",
    in_flight_requests: inFlight.ok ? alignGaugeRows(grid, inFlight.value, window.stepSeconds) : empty(),
    websocket_connections: websockets.ok ? alignGaugeRows(grid, websockets.value, window.stepSeconds) : empty(),
  };
}
