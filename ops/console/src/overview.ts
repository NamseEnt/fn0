import { bucketsFromLabels, invocationCounts, latencyFromBuckets, ratios } from "./accounting.ts";
import { rejectionsByReason } from "./live.ts";
import {
  DEADLINE_EXCEEDED_ATTRIBUTES,
  DISPATCH_REJECTIONS,
  FAILURES,
  INSTANCE_CPU_BUDGET_EXCEEDED,
  PLATFORM_REQUEST_BUCKET,
  PLATFORM_REQUEST_COUNT,
} from "./metrics.ts";
import type { Dependencies } from "./runtime.ts";
import type { OverviewResponse } from "./schema.ts";
import { SignyClient } from "./signy.ts";
import type { TimeWindow } from "./windows.ts";

export async function overview(
  dependencies: Dependencies,
  window: TimeWindow,
): Promise<OverviewResponse> {
  const signy = new SignyClient(dependencies);
  const range = `${window.seconds}s`;
  const [byOutcome, buckets, deadline, rejections, cpuBudget] = await Promise.all([
    signy.instant({ metric: PLATFORM_REQUEST_COUNT, func: "increase", range, agg: "sum", by: ["outcome"] }),
    signy.instant({ metric: PLATFORM_REQUEST_BUCKET, func: "increase", range, agg: "sum", by: ["le"] }),
    signy.instant({
      metric: FAILURES,
      attributes: DEADLINE_EXCEEDED_ATTRIBUTES,
      func: "increase",
      range,
      agg: "sum",
    }),
    signy.instant({ metric: DISPATCH_REJECTIONS, func: "increase", range, agg: "sum", by: ["reason"] }),
    signy.instant({ metric: INSTANCE_CPU_BUDGET_EXCEEDED, func: "increase", range, agg: "sum" }),
  ]);
  const base = {
    window: window.name,
    window_seconds: window.seconds,
    generated_at: new Date(dependencies.nowMs()).toISOString(),
  };
  if (!byOutcome.ok || !buckets.ok || !deadline.ok || !rejections.ok || !cpuBudget.ok) {
    return {
      ...base,
      telemetry: "unavailable",
      invocations: null,
      deadline_exceeded: null,
      dispatch_rejections: null,
      ratios: { guest_server_error: null, unanswered: null },
      latency_seconds: null,
      instance_cpu_budget_exceeded: null,
    };
  }
  const invocations = invocationCounts(
    new Map(byOutcome.value.map((row) => [row.labels.outcome ?? "", row.value])),
  );
  const deadlineExceeded = deadline.value.reduce((total, row) => total + row.value, 0);
  const dispatchRejections = rejectionsByReason(rejections.value);
  return {
    ...base,
    telemetry: "ok",
    invocations: { ...invocations, per_minute: invocations.total / (window.seconds / 60) },
    deadline_exceeded: deadlineExceeded,
    dispatch_rejections: dispatchRejections,
    ratios: ratios(invocations, deadlineExceeded, dispatchRejections),
    latency_seconds: latencyFromBuckets(bucketsFromLabels(buckets.value)),
    instance_cpu_budget_exceeded: cpuBudget.value.reduce((total, row) => total + row.value, 0),
  };
}
