import type { WindowName } from "./windows.ts";

export type HealthState = "healthy" | "degraded" | "down" | "unknown";

/**
 * - `ok`: the probe answered 200 `{"ok":true}`.
 * - `failed`: the canary answered 503 naming the dependency failure it saw.
 * - `timeout`: no answer within the probe deadline.
 * - `unexpected`: any other answer from the edge or fn0 (a 5xx page, a 404).
 * - `unauthorized`: Cloudflare Access refused the ops credential — a
 *   monitoring problem, not an fn0 one.
 * - `unreachable`: the request could not be sent at all.
 */
export type ProbeStatus =
  | "ok"
  | "failed"
  | "timeout"
  | "unexpected"
  | "unauthorized"
  | "unreachable";

export interface ProbeResult {
  status: ProbeStatus;
  latency_ms: number | null;
  http_status: number | null;
  /** The canary's own `failure` value (`missing`, `mismatch`,
   * `unavailable`) when `status` is `failed`. */
  failure: string | null;
}

export interface ComponentHealth {
  state: HealthState;
  reason: string | null;
}

export interface DispatchRejections {
  queue_full: number;
  project_admission_full: number;
  closed: number;
}

export interface WorkerInstance {
  service_instance_id: string;
  telemetry_age_seconds: number;
  manifest_loaded: boolean | null;
  draining: boolean | null;
  in_flight_requests: number | null;
  websocket_connections: number | null;
}

export type TelemetryQueryStatus = "ok" | "unavailable";

export interface LiveResponse {
  generated_at: string;
  health: { state: HealthState; reasons: string[] };
  components: {
    runtime: ComponentHealth;
    doc_db: ComponentHealth;
    storage: ComponentHealth;
    telemetry: ComponentHealth;
    worker: ComponentHealth;
    capacity: ComponentHealth;
  };
  canary: { runtime: ProbeResult; doc_db: ProbeResult; storage: ProbeResult };
  signy: {
    query: TelemetryQueryStatus;
    ready: ProbeResult;
    remote_healthy: boolean | null;
    ingest_errors_total: number | null;
  };
  telemetry: {
    worker_telemetry_age_seconds: number | null;
    stale_after_seconds: number;
    collecty_queue_bytes: number | null;
    collecty_lost_segments_recent: number | null;
  };
  worker: { instances: WorkerInstance[] };
  capacity: {
    window_seconds: number;
    dispatch_rejections: DispatchRejections | null;
  };
}

export interface InvocationCounts {
  total: number;
  ok: number;
  client_error: number;
  server_error: number;
  failed: number;
}

export interface LatencySeconds {
  p50: number | null;
  p95: number | null;
  p99: number | null;
}

export interface OverviewResponse {
  window: WindowName;
  window_seconds: number;
  generated_at: string;
  telemetry: TelemetryQueryStatus;
  /** Guest invocations: HTTP requests plus queue tasks, WebSocket events and
   * cross-project invocations. Excludes 504 deadline and 503 dispatch
   * rejections, which never produce an invocation record. */
  invocations: (InvocationCounts & { per_minute: number }) | null;
  deadline_exceeded: number | null;
  dispatch_rejections: DispatchRejections | null;
  ratios: {
    /** `server_error / invocations.total`: the guest answered 5xx. */
    guest_server_error: number | null;
    /** Invocations fn0 got no guest answer for — `failed`, 504 deadline and
     * every 503 dispatch rejection — over all of them. */
    unanswered: number | null;
  };
  latency_seconds: LatencySeconds | null;
  /** Wasm instances stopped for exceeding their cumulative CPU budget. Not a
   * per-request CPU timeout. */
  instance_cpu_budget_exceeded: number | null;
}

export interface SeriesResponse {
  window: WindowName;
  step_seconds: number;
  generated_at: string;
  telemetry: TelemetryQueryStatus;
  timestamps_ms: number[];
  invocations_per_minute: Record<keyof Omit<InvocationCounts, "total">, (number | null)[]>;
  deadline_exceeded: (number | null)[];
  dispatch_rejections: Record<keyof DispatchRejections, (number | null)[]>;
  latency_seconds: Record<keyof LatencySeconds, (number | null)[]>;
}

export interface PlatformError {
  timestamp: string;
  service: string | null;
  message: string;
  error_type: string | null;
  component: string | null;
  attributes: Record<string, string>;
}

export interface ErrorsResponse {
  window: WindowName;
  generated_at: string;
  telemetry: TelemetryQueryStatus;
  limit: number;
  errors: PlatformError[];
}
