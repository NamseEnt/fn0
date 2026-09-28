/**
 * The platform-tenant metric contract the console reads (see
 * docs/fn0/operations-dashboard.md). Label keys are as Signy stores them:
 * dots in attribute names become underscores, metric names keep their dots.
 */
export const PLATFORM_REQUEST_DURATION = "fn0.platform.request.duration";
export const PLATFORM_REQUEST_COUNT = `${PLATFORM_REQUEST_DURATION}_count`;
export const PLATFORM_REQUEST_BUCKET = `${PLATFORM_REQUEST_DURATION}_bucket`;
export const INSTANCE_CPU_BUDGET_EXCEEDED = "fn0.platform.cpu_timeouts";
export const DISPATCH_REJECTIONS = "fn0.worker.dispatch.rejections";
export const FAILURES = "fn0.failures";
export const DEADLINE_EXCEEDED_ATTRIBUTES = [
  "component=executor",
  "error_type=deadline_exceeded",
] as const;

export const WORKER_MANIFEST_LOADED = "fn0.worker.manifest_loaded";
export const WORKER_DRAINING = "fn0.worker.draining";
export const WORKER_IN_FLIGHT_REQUESTS = "fn0.worker.requests.in_flight";
export const WORKER_WEBSOCKET_CONNECTIONS = "fn0.worker.websocket.connections";

export const COLLECTY_QUEUE_BYTES = "collecty_queue_bytes";
export const COLLECTY_DROPPED_SEGMENTS = "collecty_queue_dropped_segments_total";
export const COLLECTY_REFUSED_SEGMENTS = "collecty_segments_refused_total";

export const SIGNY_REMOTE_HEALTHY = "signy_remote_healthy";
export const SIGNY_INGEST_ERRORS = "signy_ingest_errors_total";

export const SERVICE_INSTANCE_ID_LABEL = "service_instance_id";
export const OUTCOMES = ["ok", "client_error", "server_error", "failed"] as const;
export const REJECTION_REASONS = ["queue_full", "project_admission_full", "closed"] as const;
