import { bucketsFromLabels, latencyFromBuckets } from "./accounting.ts";
import {
  DEADLINE_EXCEEDED_ATTRIBUTES,
  DISPATCH_REJECTIONS,
  FAILURES,
  OUTCOMES,
  PLATFORM_REQUEST_BUCKET,
  PLATFORM_REQUEST_COUNT,
  REJECTION_REASONS,
} from "./metrics.ts";
import type { Dependencies } from "./runtime.ts";
import type { SeriesResponse } from "./schema.ts";
import { type RangeRow, SignyClient } from "./signy.ts";
import { type TimeWindow, stepGrid } from "./windows.ts";

type Column = (number | null)[];

/**
 * Lines a series up with the requested grid. A step Signy omitted is `null`
 * — no sample reached it — while a series that does not exist at all is a
 * counter that was never incremented, and its steps are 0.
 */
export function alignToGrid(
  gridSeconds: readonly number[],
  row: RangeRow | undefined,
  stepSeconds: number,
): Column {
  if (row === undefined) {
    return gridSeconds.map(() => 0);
  }
  const byStep = new Map<number, number>();
  for (const [timestampMs, value] of row.samples) {
    byStep.set(Math.round(timestampMs / 1000 / stepSeconds), value);
  }
  return gridSeconds.map((seconds) => byStep.get(Math.round(seconds / stepSeconds)) ?? null);
}

function rowWithLabel(rows: readonly RangeRow[], key: string, value: string): RangeRow | undefined {
  return rows.find((row) => row.labels[key] === value);
}

export async function series(
  dependencies: Dependencies,
  window: TimeWindow,
): Promise<SeriesResponse> {
  const signy = new SignyClient(dependencies);
  const grid = stepGrid(window, dependencies.nowMs());
  const gridRange = {
    startSeconds: grid[0]!,
    endSeconds: grid[grid.length - 1]!,
    stepSeconds: window.stepSeconds,
  };
  const range = `${window.stepSeconds}s`;
  const [byOutcome, buckets, deadline, rejections] = await Promise.all([
    signy.range(
      { metric: PLATFORM_REQUEST_COUNT, func: "increase", range, agg: "sum", by: ["outcome"] },
      gridRange,
    ),
    signy.range(
      { metric: PLATFORM_REQUEST_BUCKET, func: "increase", range, agg: "sum", by: ["le"] },
      gridRange,
    ),
    signy.range(
      {
        metric: FAILURES,
        attributes: DEADLINE_EXCEEDED_ATTRIBUTES,
        func: "increase",
        range,
        agg: "sum",
      },
      gridRange,
    ),
    signy.range(
      { metric: DISPATCH_REJECTIONS, func: "increase", range, agg: "sum", by: ["reason"] },
      gridRange,
    ),
  ]);

  const unavailable = !byOutcome.ok || !buckets.ok || !deadline.ok || !rejections.ok;
  const empty = (): Column => grid.map(() => null);
  const perMinute = (column: Column): Column =>
    column.map((value) => (value === null ? null : value / (window.stepSeconds / 60)));
  const response: SeriesResponse = {
    window: window.name,
    step_seconds: window.stepSeconds,
    generated_at: new Date(dependencies.nowMs()).toISOString(),
    telemetry: unavailable ? "unavailable" : "ok",
    timestamps_ms: grid.map((seconds) => seconds * 1000),
    invocations_per_minute: { ok: empty(), client_error: empty(), server_error: empty(), failed: empty() },
    deadline_exceeded: empty(),
    dispatch_rejections: { queue_full: empty(), project_admission_full: empty(), closed: empty() },
    latency_seconds: { p50: empty(), p95: empty(), p99: empty() },
  };
  if (unavailable) {
    return response;
  }

  for (const outcome of OUTCOMES) {
    response.invocations_per_minute[outcome] = perMinute(
      alignToGrid(grid, rowWithLabel(byOutcome.value, "outcome", outcome), window.stepSeconds),
    );
  }
  response.deadline_exceeded = alignToGrid(grid, deadline.value[0], window.stepSeconds);
  for (const reason of REJECTION_REASONS) {
    response.dispatch_rejections[reason] = alignToGrid(
      grid,
      rowWithLabel(rejections.value, "reason", reason),
      window.stepSeconds,
    );
  }

  const bucketColumns = buckets.value.map((row) => ({
    labels: row.labels,
    column: alignToGrid(grid, row, window.stepSeconds),
  }));
  grid.forEach((_, index) => {
    const stepBuckets = bucketsFromLabels(
      bucketColumns
        .filter((bucket) => bucket.column[index] !== null)
        .map((bucket) => ({ labels: bucket.labels, value: bucket.column[index] as number })),
    );
    const latency = latencyFromBuckets(stepBuckets);
    response.latency_seconds.p50[index] = latency.p50;
    response.latency_seconds.p95[index] = latency.p95;
    response.latency_seconds.p99[index] = latency.p99;
  });
  return response;
}
