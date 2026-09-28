import type {
  ComponentHealth,
  DispatchRejections,
  HealthState,
  ProbeResult,
  TelemetryQueryStatus,
  WorkerInstance,
} from "./schema.ts";

/** The worker exports every 60 s; three missed exports is stale. */
export const STALE_AFTER_SECONDS = 180;

export interface HealthInputs {
  canary: {
    runtime: ProbeResult;
    doc_db: ProbeResult;
    doc_db_write: ProbeResult;
    storage: ProbeResult;
  };
  signyQuery: TelemetryQueryStatus;
  signyReady: ProbeResult;
  signyRemoteHealthy: boolean | null;
  workerTelemetryAgeSeconds: number | null;
  collectyLostSegmentsRecent: number | null;
  instances: WorkerInstance[];
  recentDispatchRejections: DispatchRejections | null;
}

export interface HealthAssessment {
  state: HealthState;
  reasons: string[];
  components: {
    runtime: ComponentHealth;
    doc_db: ComponentHealth;
    doc_db_write: ComponentHealth;
    storage: ComponentHealth;
    telemetry: ComponentHealth;
    worker: ComponentHealth;
    capacity: ComponentHealth;
  };
}

function component(state: HealthState, reason: string | null = null): ComponentHealth {
  return { state, reason };
}

function describe(probe: ProbeResult): string {
  switch (probe.status) {
    case "ok":
      return "ok";
    case "failed":
      return `canary reported ${probe.failure}`;
    case "timeout":
      return "no answer before the probe deadline";
    case "unexpected":
      return `unexpected answer (HTTP ${probe.http_status ?? "?"})`;
    case "unauthorized":
      return `the ops credential was refused (HTTP ${probe.http_status ?? "?"})`;
    case "unreachable":
      return "the request could not be sent";
  }
}

/**
 * The runtime probe is the only evidence that users can be served. Its
 * failure through fn0 (a timeout, a 5xx page) is `down`; a refusal of the
 * ops credential, or a request that could not leave the console, says
 * nothing about fn0 and is `unknown`.
 */
function assessRuntime(probe: ProbeResult): ComponentHealth {
  switch (probe.status) {
    case "ok":
      return component("healthy");
    case "failed":
    case "timeout":
    case "unexpected":
      return component("down", describe(probe));
    case "unauthorized":
    case "unreachable":
      return component("unknown", describe(probe));
  }
}

/** A dependency probe rides the runtime path, so it only says something
 * about its dependency once the runtime probe on its own succeeded. */
function assessDependency(probe: ProbeResult, runtime: ComponentHealth): ComponentHealth {
  if (runtime.state !== "healthy") {
    return component("unknown", "the runtime path itself is not answering");
  }
  switch (probe.status) {
    case "ok":
      return component("healthy");
    case "failed":
    case "timeout":
    case "unexpected":
      return component("down", describe(probe));
    case "unauthorized":
    case "unreachable":
      return component("unknown", describe(probe));
  }
}

function assessTelemetry(inputs: HealthInputs): ComponentHealth {
  if (inputs.signyQuery === "unavailable") {
    return component("down", "Signy queries are failing");
  }
  if (inputs.signyReady.status !== "ok") {
    return component("degraded", `Signy /ready: ${describe(inputs.signyReady)}`);
  }
  if (inputs.workerTelemetryAgeSeconds === null) {
    return component("degraded", "no worker telemetry in the platform tenant");
  }
  if (inputs.workerTelemetryAgeSeconds > STALE_AFTER_SECONDS) {
    return component(
      "degraded",
      `worker telemetry is ${Math.round(inputs.workerTelemetryAgeSeconds)} s old`,
    );
  }
  if (inputs.signyRemoteHealthy === false) {
    return component("degraded", "Signy reports its R2 remote unhealthy");
  }
  if (inputs.collectyLostSegmentsRecent !== null && inputs.collectyLostSegmentsRecent > 0) {
    return component("degraded", "collecty dropped or had refused telemetry segments");
  }
  if (inputs.signyRemoteHealthy === null || inputs.collectyLostSegmentsRecent === null) {
    return component("unknown", "part of the telemetry pipeline could not be read");
  }
  return component("healthy");
}

function telemetryIsTrustworthy(telemetry: ComponentHealth, inputs: HealthInputs): boolean {
  return (
    inputs.signyQuery === "ok" &&
    inputs.workerTelemetryAgeSeconds !== null &&
    inputs.workerTelemetryAgeSeconds <= STALE_AFTER_SECONDS &&
    telemetry.state !== "down"
  );
}

function assessWorker(inputs: HealthInputs, trustworthy: boolean): ComponentHealth {
  if (!trustworthy) {
    return component("unknown", "worker state comes from telemetry that is missing or stale");
  }
  const fresh = inputs.instances.filter(
    (instance) => instance.telemetry_age_seconds <= STALE_AFTER_SECONDS,
  );
  if (fresh.some((instance) => instance.manifest_loaded === true && instance.draining === false)) {
    return component("healthy");
  }
  if (fresh.length > 0 && fresh.every((instance) => instance.draining === true)) {
    return component("degraded", "every reporting worker instance is draining");
  }
  return component("degraded", "no reporting worker instance has its manifest loaded");
}

/**
 * `queue_full` is the worker's own capacity and `closed` a broken worker
 * thread; both degrade the platform. `project_admission_full` is one project
 * reaching its own concurrency limit and does not.
 */
function assessCapacity(inputs: HealthInputs, trustworthy: boolean): ComponentHealth {
  if (!trustworthy || inputs.recentDispatchRejections === null) {
    return component("unknown", "rejection counts come from telemetry that is missing or stale");
  }
  const { queue_full, closed } = inputs.recentDispatchRejections;
  if (closed > 0) {
    return component("degraded", `${closed} requests hit a closed worker thread`);
  }
  if (queue_full > 0) {
    return component("degraded", `${queue_full} requests were refused: worker queue full`);
  }
  return component("healthy");
}

export function assessHealth(inputs: HealthInputs): HealthAssessment {
  const runtime = assessRuntime(inputs.canary.runtime);
  const telemetry = assessTelemetry(inputs);
  const trustworthy = telemetryIsTrustworthy(telemetry, inputs);
  const components = {
    runtime,
    doc_db: assessDependency(inputs.canary.doc_db, runtime),
    doc_db_write: assessDependency(inputs.canary.doc_db_write, runtime),
    storage: assessDependency(inputs.canary.storage, runtime),
    telemetry,
    worker: assessWorker(inputs, trustworthy),
    capacity: assessCapacity(inputs, trustworthy),
  };

  if (runtime.state === "down") {
    return { state: "down", reasons: [`runtime canary: ${runtime.reason}`], components };
  }
  if (runtime.state === "unknown") {
    return {
      state: "unknown",
      reasons: [`runtime canary could not be measured: ${runtime.reason}`],
      components,
    };
  }
  const reasons = Object.entries(components)
    .filter(([, health]) => health.state !== "healthy")
    .map(([name, health]) => `${name}: ${health.reason ?? health.state}`);
  return { state: reasons.length === 0 ? "healthy" : "degraded", reasons, components };
}
