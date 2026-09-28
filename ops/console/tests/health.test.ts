import assert from "node:assert/strict";
import { test } from "node:test";
import { type HealthInputs, assessHealth } from "../src/health.ts";
import type { ProbeResult, ProbeStatus } from "../src/schema.ts";

const probe = (status: ProbeStatus, failure: string | null = null): ProbeResult => ({
  status,
  latency_ms: status === "timeout" || status === "unreachable" ? null : 20,
  http_status:
    status === "ok" ? 200 : status === "failed" ? 503 : status === "unauthorized" ? 401 : null,
  failure,
});

const healthy = (): HealthInputs => ({
  canary: {
    runtime: probe("ok"),
    doc_db: probe("ok"),
    doc_db_write: probe("ok"),
    storage: probe("ok"),
  },
  signyQuery: "ok",
  signyReady: probe("ok"),
  signyRemoteHealthy: true,
  workerTelemetryAgeSeconds: 30,
  collectyLostSegmentsRecent: 0,
  instances: [
    {
      service_instance_id: "instance-a",
      telemetry_age_seconds: 30,
      manifest_loaded: true,
      draining: false,
      in_flight_requests: 1,
      websocket_connections: 0,
    },
  ],
  recentDispatchRejections: { queue_full: 0, project_admission_full: 0, closed: 0 },
});

const signyDown = (inputs: HealthInputs): HealthInputs => ({
  ...inputs,
  signyQuery: "unavailable",
  signyReady: probe("timeout"),
  signyRemoteHealthy: null,
  workerTelemetryAgeSeconds: null,
  collectyLostSegmentsRecent: null,
  instances: [],
  recentDispatchRejections: null,
});

test("runtime OK and everything else OK is HEALTHY", () => {
  const assessment = assessHealth(healthy());
  assert.equal(assessment.state, "healthy");
  assert.deepEqual(assessment.reasons, []);
});

test("runtime OK with Signy down is DEGRADED, never DOWN", () => {
  const assessment = assessHealth(signyDown(healthy()));
  assert.equal(assessment.state, "degraded");
  assert.equal(assessment.components.runtime.state, "healthy");
  assert.equal(assessment.components.telemetry.state, "down");
  assert.equal(assessment.components.worker.state, "unknown");
  assert.equal(assessment.components.capacity.state, "unknown");
});

test("runtime OK with the dodb canary failing is DEGRADED and names dodb", () => {
  const inputs = healthy();
  inputs.canary.doc_db = probe("failed", "missing");
  const assessment = assessHealth(inputs);
  assert.equal(assessment.state, "degraded");
  assert.equal(assessment.components.doc_db.state, "down");
  assert.equal(assessment.components.storage.state, "healthy");
  assert.match(assessment.reasons.join(";"), /doc_db: canary reported missing/);
});

test("runtime OK with the storage canary failing is DEGRADED and names storage", () => {
  const inputs = healthy();
  inputs.canary.storage = probe("timeout");
  const assessment = assessHealth(inputs);
  assert.equal(assessment.state, "degraded");
  assert.equal(assessment.components.storage.state, "down");
  assert.equal(assessment.components.doc_db.state, "healthy");
});

test("runtime failing with Signy healthy is DOWN", () => {
  for (const status of ["timeout", "unexpected", "failed"] as const) {
    const inputs = healthy();
    inputs.canary.runtime = probe(status);
    inputs.canary.doc_db = probe(status);
    inputs.canary.storage = probe(status);
    const assessment = assessHealth(inputs);
    assert.equal(assessment.state, "down", status);
    assert.equal(assessment.components.doc_db.state, "unknown", status);
    assert.equal(assessment.components.storage.state, "unknown", status);
  }
});

test("runtime failing is DOWN even while Signy is down too", () => {
  const inputs = signyDown(healthy());
  inputs.canary.runtime = probe("timeout");
  assert.equal(assessHealth(inputs).state, "down");
});

test("a canary the console cannot reach or is refused by, with Signy down, is UNKNOWN", () => {
  for (const status of ["unauthorized", "unreachable"] as const) {
    const inputs = signyDown(healthy());
    inputs.canary.runtime = probe(status);
    inputs.canary.doc_db = probe(status);
    inputs.canary.storage = probe(status);
    assert.equal(assessHealth(inputs).state, "unknown", status);
  }
});

test("stale worker telemetry is not HEALTHY", () => {
  const inputs = healthy();
  inputs.workerTelemetryAgeSeconds = 181;
  inputs.instances[0]!.telemetry_age_seconds = 181;
  const assessment = assessHealth(inputs);
  assert.equal(assessment.state, "degraded");
  assert.equal(assessment.components.telemetry.state, "degraded");
  assert.equal(assessment.components.worker.state, "unknown");
  assert.equal(assessment.components.capacity.state, "unknown");
});

test("telemetry exactly at the stale limit is still fresh", () => {
  const inputs = healthy();
  inputs.workerTelemetryAgeSeconds = 180;
  inputs.instances[0]!.telemetry_age_seconds = 180;
  assert.equal(assessHealth(inputs).state, "healthy");
});

test("no worker telemetry at all is not HEALTHY", () => {
  const inputs = healthy();
  inputs.workerTelemetryAgeSeconds = null;
  inputs.instances = [];
  assert.equal(assessHealth(inputs).state, "degraded");
});

test("queue_full and closed rejections degrade capacity", () => {
  for (const rejections of [
    { queue_full: 3, project_admission_full: 0, closed: 0 },
    { queue_full: 0, project_admission_full: 0, closed: 1 },
  ]) {
    const inputs = healthy();
    inputs.recentDispatchRejections = rejections;
    const assessment = assessHealth(inputs);
    assert.equal(assessment.state, "degraded");
    assert.equal(assessment.components.capacity.state, "degraded");
  }
});

test("project_admission_full alone is one project's limit, not platform capacity", () => {
  const inputs = healthy();
  inputs.recentDispatchRejections = { queue_full: 0, project_admission_full: 40, closed: 0 };
  const assessment = assessHealth(inputs);
  assert.equal(assessment.state, "healthy");
  assert.equal(assessment.components.capacity.state, "healthy");
});

test("a blue-green overlap with one ready instance is healthy", () => {
  const inputs = healthy();
  inputs.instances.push({
    service_instance_id: "instance-old",
    telemetry_age_seconds: 40,
    manifest_loaded: true,
    draining: true,
    in_flight_requests: 0,
    websocket_connections: 3,
  });
  assert.equal(assessHealth(inputs).components.worker.state, "healthy");
});

test("every fresh instance draining degrades the worker", () => {
  const inputs = healthy();
  inputs.instances[0]!.draining = true;
  const assessment = assessHealth(inputs);
  assert.equal(assessment.components.worker.state, "degraded");
  assert.equal(assessment.state, "degraded");
});

test("Signy's R2 remote unhealthy or lost collecty segments degrade telemetry", () => {
  const remote = healthy();
  remote.signyRemoteHealthy = false;
  assert.equal(assessHealth(remote).components.telemetry.state, "degraded");
  const lost = healthy();
  lost.collectyLostSegmentsRecent = 2;
  assert.equal(assessHealth(lost).components.telemetry.state, "degraded");
});

test("a dependency probe the ops credential is refused on is unknown, not down", () => {
  const inputs = healthy();
  inputs.canary.doc_db = probe("unauthorized");
  const assessment = assessHealth(inputs);
  assert.equal(assessment.components.doc_db.state, "unknown");
  assert.equal(assessment.state, "degraded");
});
