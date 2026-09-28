import { OUTCOMES } from "./metrics.ts";
import { type CumulativeBucket, histogramQuantile, parseUpperBound } from "./quantile.ts";
import type {
  DispatchRejections,
  InvocationCounts,
  LatencySeconds,
  OverviewResponse,
} from "./schema.ts";

export function invocationCounts(
  byOutcome: ReadonlyMap<string, number>,
): InvocationCounts {
  const counts: InvocationCounts = { total: 0, ok: 0, client_error: 0, server_error: 0, failed: 0 };
  for (const outcome of OUTCOMES) {
    const value = byOutcome.get(outcome) ?? 0;
    counts[outcome] = value;
    counts.total += value;
  }
  return counts;
}

export function latencyFromBuckets(buckets: readonly CumulativeBucket[]): LatencySeconds {
  return {
    p50: histogramQuantile(0.5, buckets),
    p95: histogramQuantile(0.95, buckets),
    p99: histogramQuantile(0.99, buckets),
  };
}

export function bucketsFromLabels(
  rows: readonly { labels: Record<string, string>; value: number }[],
): CumulativeBucket[] {
  const buckets: CumulativeBucket[] = [];
  for (const row of rows) {
    const upperBound = parseUpperBound(row.labels.le);
    if (upperBound !== null) {
      buckets.push({ upperBound, count: row.value });
    }
  }
  return buckets;
}

/**
 * The two ratios the console shows, kept apart because they answer
 * different questions. `guest_server_error` is the application's own 5xx.
 * `unanswered` is fn0 not getting an answer at all: a failed invocation, a
 * 504 deadline, or a 503 dispatch rejection. The last two never reach the
 * invocation histogram, so they are added to its denominator here.
 */
export function ratios(
  invocations: InvocationCounts,
  deadlineExceeded: number,
  rejections: DispatchRejections,
): OverviewResponse["ratios"] {
  const rejected = rejections.queue_full + rejections.project_admission_full + rejections.closed;
  const attempted = invocations.total + deadlineExceeded + rejected;
  return {
    guest_server_error:
      invocations.total > 0 ? invocations.server_error / invocations.total : null,
    unanswered:
      attempted > 0 ? (invocations.failed + deadlineExceeded + rejected) / attempted : null,
  };
}
