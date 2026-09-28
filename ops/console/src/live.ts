import { probeCanary } from "./canary.ts";
import { STALE_AFTER_SECONDS, assessHealth } from "./health.ts";
import {
  COLLECTY_DROPPED_SEGMENTS,
  COLLECTY_QUEUE_BYTES,
  COLLECTY_REFUSED_SEGMENTS,
  DISPATCH_REJECTIONS,
  REJECTION_REASONS,
  SERVICE_INSTANCE_ID_LABEL,
  SIGNY_INGEST_ERRORS,
  SIGNY_REMOTE_HEALTHY,
  WORKER_DRAINING,
  WORKER_IN_FLIGHT_REQUESTS,
  WORKER_MANIFEST_LOADED,
  WORKER_WEBSOCKET_CONNECTIONS,
} from "./metrics.ts";
import type { Dependencies } from "./runtime.ts";
import type { DispatchRejections, LiveResponse, WorkerInstance } from "./schema.ts";
import { type InstantRow, SignyClient, type SignyResult } from "./signy.ts";

const RECENT_REJECTIONS_SECONDS = 300;
const GAUGE_LOOKBACK = "10m";

/** A counter increase: no series means nothing was counted. */
function increaseOf(result: SignyResult<InstantRow[]>): number | null {
  return result.ok ? result.value.reduce((total, row) => total + row.value, 0) : null;
}

/** A gauge: no series means no reading, not zero. */
function gaugeSumOf(result: SignyResult<InstantRow[]>): number | null {
  return result.ok && result.value.length > 0
    ? result.value.reduce((total, row) => total + row.value, 0)
    : null;
}

export function rejectionsByReason(rows: InstantRow[]): DispatchRejections {
  const rejections: DispatchRejections = { queue_full: 0, project_admission_full: 0, closed: 0 };
  for (const row of rows) {
    const reason = row.labels.reason;
    if (reason !== undefined && (REJECTION_REASONS as readonly string[]).includes(reason)) {
      rejections[reason as keyof DispatchRejections] += row.value;
    }
  }
  return rejections;
}

function workerInstances(
  nowMs: number,
  manifestLoaded: InstantRow[],
  draining: InstantRow[],
  inFlight: InstantRow[],
  websockets: InstantRow[],
): WorkerInstance[] {
  const byInstance = new Map<string, WorkerInstance>();
  const instanceOf = (row: InstantRow): WorkerInstance | null => {
    const id = row.labels[SERVICE_INSTANCE_ID_LABEL];
    if (id === undefined) {
      return null;
    }
    const ageSeconds = Math.max(0, (nowMs - row.timestampMs) / 1000);
    const existing = byInstance.get(id);
    if (existing) {
      existing.telemetry_age_seconds = Math.min(existing.telemetry_age_seconds, ageSeconds);
      return existing;
    }
    const created: WorkerInstance = {
      service_instance_id: id,
      telemetry_age_seconds: ageSeconds,
      manifest_loaded: null,
      draining: null,
      in_flight_requests: null,
      websocket_connections: null,
    };
    byInstance.set(id, created);
    return created;
  };
  for (const row of manifestLoaded) {
    const instance = instanceOf(row);
    if (instance) instance.manifest_loaded = row.value === 1;
  }
  for (const row of draining) {
    const instance = instanceOf(row);
    if (instance) instance.draining = row.value === 1;
  }
  for (const row of inFlight) {
    const instance = instanceOf(row);
    if (instance) instance.in_flight_requests = row.value;
  }
  for (const row of websockets) {
    const instance = instanceOf(row);
    if (instance) instance.websocket_connections = row.value;
  }
  return [...byInstance.values()].sort(
    (left, right) => left.telemetry_age_seconds - right.telemetry_age_seconds,
  );
}

export async function live(dependencies: Dependencies): Promise<LiveResponse> {
  const signy = new SignyClient(dependencies);
  const [
    runtime,
    docDb,
    storage,
    ready,
    exposition,
    manifestLoaded,
    draining,
    inFlight,
    websockets,
    queueBytes,
    droppedSegments,
    refusedSegments,
    rejections,
  ] = await Promise.all([
    probeCanary(dependencies, "runtime"),
    probeCanary(dependencies, "dodb"),
    probeCanary(dependencies, "storage"),
    signy.ready(),
    signy.exposition(),
    signy.instant({ metric: WORKER_MANIFEST_LOADED, lookback: GAUGE_LOOKBACK }),
    signy.instant({ metric: WORKER_DRAINING, lookback: GAUGE_LOOKBACK }),
    signy.instant({ metric: WORKER_IN_FLIGHT_REQUESTS, lookback: GAUGE_LOOKBACK }),
    signy.instant({ metric: WORKER_WEBSOCKET_CONNECTIONS, lookback: GAUGE_LOOKBACK }),
    signy.instant({ metric: COLLECTY_QUEUE_BYTES, lookback: GAUGE_LOOKBACK, agg: "sum" }),
    signy.instant({ metric: COLLECTY_DROPPED_SEGMENTS, func: "increase", range: "15m", agg: "sum" }),
    signy.instant({ metric: COLLECTY_REFUSED_SEGMENTS, func: "increase", range: "15m", agg: "sum" }),
    signy.instant({
      metric: DISPATCH_REJECTIONS,
      func: "increase",
      range: `${RECENT_REJECTIONS_SECONDS}s`,
      agg: "sum",
      by: ["reason"],
    }),
  ]);

  const nowMs = dependencies.nowMs();
  const metricQueries = [
    manifestLoaded,
    draining,
    inFlight,
    websockets,
    queueBytes,
    droppedSegments,
    refusedSegments,
    rejections,
  ];
  const signyQuery = metricQueries.every((result) => result.ok) ? "ok" : "unavailable";
  const instances = workerInstances(
    nowMs,
    manifestLoaded.ok ? manifestLoaded.value : [],
    draining.ok ? draining.value : [],
    inFlight.ok ? inFlight.value : [],
    websockets.ok ? websockets.value : [],
  );
  const workerTelemetryAgeSeconds =
    manifestLoaded.ok && instances.length > 0
      ? Math.min(...instances.map((instance) => instance.telemetry_age_seconds))
      : null;
  const dropped = increaseOf(droppedSegments);
  const refused = increaseOf(refusedSegments);
  const collectyLostSegmentsRecent = dropped === null || refused === null ? null : dropped + refused;
  const recentDispatchRejections = rejections.ok ? rejectionsByReason(rejections.value) : null;
  const remoteHealthyValue = exposition.ok ? exposition.value.get(SIGNY_REMOTE_HEALTHY) : undefined;
  const signyRemoteHealthy = remoteHealthyValue === undefined ? null : remoteHealthyValue === 1;
  const canary = { runtime, doc_db: docDb, storage };

  const assessment = assessHealth({
    canary,
    signyQuery,
    signyReady: ready,
    signyRemoteHealthy,
    workerTelemetryAgeSeconds,
    collectyLostSegmentsRecent,
    instances,
    recentDispatchRejections,
  });

  return {
    generated_at: new Date(nowMs).toISOString(),
    health: { state: assessment.state, reasons: assessment.reasons },
    components: assessment.components,
    canary,
    signy: {
      query: signyQuery,
      ready,
      remote_healthy: signyRemoteHealthy,
      ingest_errors_total: exposition.ok ? (exposition.value.get(SIGNY_INGEST_ERRORS) ?? null) : null,
    },
    telemetry: {
      worker_telemetry_age_seconds: workerTelemetryAgeSeconds,
      stale_after_seconds: STALE_AFTER_SECONDS,
      collecty_queue_bytes: gaugeSumOf(queueBytes),
      collecty_lost_segments_recent: collectyLostSegmentsRecent,
    },
    worker: { instances },
    capacity: {
      window_seconds: RECENT_REJECTIONS_SECONDS,
      dispatch_rejections: recentDispatchRejections,
    },
  };
}
