import assert from "node:assert/strict";
import { test } from "node:test";
import { handleRequest } from "../src/index.ts";
import type { LiveResponse, OverviewResponse, SeriesResponse } from "../src/schema.ts";
import { MissingBindingError } from "../src/config.ts";
import { forgetSigningKeys } from "../src/access.ts";
import {
  CONFIG,
  FakeUpstream,
  NOW_MS,
  SECRET_VALUES,
  accessToken,
  counter,
} from "./fake_upstream.ts";

const VALID_TOKEN = await accessToken();

async function get(upstream: FakeUpstream, path: string, method = "GET") {
  const response = await handleRequest(
    new Request(`https://ops.test${path}`, {
      method,
      headers: { "Cf-Access-Jwt-Assertion": VALID_TOKEN },
    }),
    () => upstream.dependencies(),
  );
  const text = await response.text();
  return { status: response.status, text, body: JSON.parse(text) as unknown };
}

async function getText(upstream: FakeUpstream, path: string) {
  const response = await handleRequest(
    new Request(`https://ops.test${path}`, {
      headers: { "Cf-Access-Jwt-Assertion": VALID_TOKEN },
    }),
    () => upstream.dependencies(),
  );
  return { response, text: await response.text() };
}

test("an invalid, repeated or extra parameter is a 400", async () => {
  const upstream = new FakeUpstream();
  for (const path of [
    "/api/overview?window=30m",
    "/api/series?window=1H",
    "/api/overview?window=1h&window=6h",
    "/api/overview?window=1h&metric=fn0.failures",
    "/api/errors?window=1h&attr=severity_text%3DINFO",
    "/api/live?window=1h",
    "/api/series?tenant=5t1hmzd4",
  ]) {
    const response = await get(upstream, path);
    assert.equal(response.status, 400, path);
  }
  assert.equal(
    upstream.requests.filter((request) => request.url.origin !== CONFIG.accessPolicy.issuer).length,
    0,
    "a refused request reached no data upstream",
  );
});

test("unknown paths are 404 and other methods 405", async () => {
  const upstream = new FakeUpstream();
  assert.equal((await get(upstream, "/api/query")).status, 404);
  assert.equal((await get(upstream, "/signy/api/v1/logs")).status, 404);
  assert.equal((await get(upstream, "/api/live", "POST")).status, 405);
});

test("the authenticated worker serves the console document and browser bundle", async () => {
  const upstream = new FakeUpstream();
  const html = await getText(upstream, "/");
  assert.equal(html.response.status, 200);
  assert.match(html.text, /fn0 Operations/);
  assert.match(html.text, /src="\/app\.js"/);
  const script = await getText(upstream, "/app.js");
  assert.equal(script.response.status, 200);
  assert.match(script.text, /\/api\/live/);
  assert.match(script.text, /visibilitychange/);
  assert.doesNotThrow(() => new Function(script.text));
  assert.match(script.response.headers.get("content-type") ?? "", /javascript/);
  assert.equal(script.response.headers.get("cache-control"), "no-store");
  assert.match(script.response.headers.get("content-security-policy") ?? "", /script-src 'self'/);
});

test("a valid window answers a bounded, aligned series", async () => {
  for (const window of ["15m", "1h", "6h", "24h", "7d"]) {
    const upstream = new FakeUpstream();
    const response = await get(upstream, `/api/series?window=${window}`);
    assert.equal(response.status, 200);
    const series = response.body as SeriesResponse;
    const points = series.timestamps_ms.length;
    assert.ok(points > 0 && points <= 168, window);
    for (const column of [
      ...Object.values(series.invocations_per_minute),
      ...Object.values(series.dispatch_rejections),
      ...Object.values(series.latency_seconds),
      series.deadline_exceeded,
    ]) {
      assert.equal(column.length, points, window);
    }
    assert.ok(series.timestamps_ms[points - 1]! <= NOW_MS);
  }
});

test("series places Signy samples on the grid and computes per-step quantiles", async () => {
  const upstream = new FakeUpstream();
  const stepMs = 60_000;
  const lastMs = Math.floor(NOW_MS / stepMs) * stepMs;
  const ns = (ms: number) => `${BigInt(ms) * 1_000_000n}`;
  upstream.range["fn0.platform.request.duration_count"] = [
    JSON.stringify({ labels: { outcome: "ok" }, samples: [[ns(lastMs), 120]] }),
  ];
  upstream.range["fn0.platform.request.duration_bucket"] = [
    ["0.1", 60],
    ["0.5", 120],
    ["+Inf", 120],
  ].map(([le, count]) => JSON.stringify({ labels: { le }, samples: [[ns(lastMs), count]] }));
  const series = (await get(upstream, "/api/series?window=1h")).body as SeriesResponse;
  const last = series.timestamps_ms.length - 1;
  assert.equal(series.invocations_per_minute.ok[last], 120);
  assert.equal(series.invocations_per_minute.ok[last - 1], null, "a step Signy omitted stays null");
  assert.equal(series.invocations_per_minute.failed[last], 0, "an outcome never recorded is 0");
  assert.equal(series.latency_seconds.p50[last], 0.1);
  assert.equal(series.latency_seconds.p95[last - 1], null);
});

test("overview reads the platform tenant and accounts rejections apart", async () => {
  const upstream = new FakeUpstream();
  upstream.instant["fn0.platform.request.duration_count"] = [
    counter(97, { outcome: "ok" }),
    counter(3, { outcome: "server_error" }),
  ];
  upstream.instant["fn0.platform.request.duration_bucket"] = [
    counter(50, { le: "0.05" }),
    counter(100, { le: "0.1" }),
    counter(100, { le: "+Inf" }),
  ];
  upstream.instant["fn0.failures"] = [counter(2)];
  upstream.instant["fn0.worker.dispatch.rejections"] = [
    counter(5, { reason: "project_admission_full" }),
    counter(1, { reason: "queue_full" }),
  ];
  upstream.instant["fn0.platform.cpu_timeouts"] = [counter(4)];
  const response = await get(upstream, "/api/overview");
  const overview = response.body as OverviewResponse;
  assert.equal(overview.window, "1h");
  assert.equal(overview.telemetry, "ok");
  assert.equal(overview.invocations?.total, 100);
  assert.equal(overview.deadline_exceeded, 2);
  assert.deepEqual(overview.dispatch_rejections, {
    queue_full: 1,
    project_admission_full: 5,
    closed: 0,
  });
  assert.equal(overview.ratios.guest_server_error, 0.03);
  assert.equal(overview.ratios.unanswered, (0 + 2 + 6) / (100 + 2 + 6));
  assert.equal(overview.latency_seconds?.p50, 0.05);
  assert.equal(overview.instance_cpu_budget_exceeded, 4);
  const failuresRequest = upstream.requests.find(
    (request) => request.url.searchParams.get("metric") === "fn0.failures",
  );
  assert.deepEqual(failuresRequest?.url.searchParams.getAll("attr"), [
    "component=executor",
    "error_type=deadline_exceeded",
  ]);
  const bucketRequest = upstream.requests.find(
    (request) => request.url.searchParams.get("metric") === "fn0.platform.request.duration_bucket",
  );
  assert.equal(bucketRequest?.url.searchParams.get("func"), "increase");
  assert.equal(bucketRequest?.url.searchParams.get("agg"), "sum");
  assert.deepEqual(bucketRequest?.url.searchParams.getAll("by"), ["le"]);
  assert.equal(bucketRequest?.url.searchParams.get("range"), "3600s");
});

test("Signy failing marks telemetry unavailable and never marks fn0 down", async () => {
  for (const signyDown of [{ status: 502, body: "bad gateway" }, "timeout", "unreachable"] as const) {
    const upstream = new FakeUpstream();
    upstream.signyDown = signyDown;
    const liveResponse = (await get(upstream, "/api/live")).body as LiveResponse;
    assert.equal(liveResponse.signy.query, "unavailable");
    assert.equal(liveResponse.components.runtime.state, "healthy");
    assert.equal(liveResponse.components.telemetry.state, "down");
    assert.equal(liveResponse.health.state, "degraded");

    for (const path of ["/api/overview", "/api/series", "/api/errors"]) {
      const response = await get(upstream, path);
      assert.equal(response.status, 200, path);
      assert.equal((response.body as { telemetry: string }).telemetry, "unavailable", path);
    }
    const overview = (await get(upstream, "/api/overview")).body as OverviewResponse;
    assert.equal(overview.invocations, null, "no fabricated numbers");
  }
});

test("a canary timeout is reported as a timeout and makes fn0 DOWN", async () => {
  const upstream = new FakeUpstream();
  upstream.canary.runtime = "timeout";
  const liveResponse = (await get(upstream, "/api/live")).body as LiveResponse;
  assert.equal(liveResponse.canary.runtime.status, "timeout");
  assert.equal(liveResponse.components.runtime.state, "down");
  assert.equal(liveResponse.health.state, "down");
});

test("a canary dependency failure is DEGRADED with the canary's reason", async () => {
  const upstream = new FakeUpstream();
  upstream.canary.dodb = { status: 503, body: '{"ok":false,"failure":"mismatch"}' };
  const liveResponse = (await get(upstream, "/api/live")).body as LiveResponse;
  assert.equal(liveResponse.canary.doc_db.status, "failed");
  assert.equal(liveResponse.canary.doc_db.failure, "mismatch");
  assert.equal(liveResponse.health.state, "degraded");
});

test("Access refusing the ops credential is UNKNOWN, not DOWN", async () => {
  const upstream = new FakeUpstream();
  for (const probe of ["runtime", "dodb", "storage"]) {
    upstream.canary[probe] = { status: 401, body: "" };
  }
  const liveResponse = (await get(upstream, "/api/live")).body as LiveResponse;
  assert.equal(liveResponse.canary.runtime.status, "unauthorized");
  assert.equal(liveResponse.health.state, "unknown");
});

test("stale worker telemetry is not healthy", async () => {
  const upstream = new FakeUpstream();
  upstream.workerTelemetryAgeSeconds = 300;
  const liveResponse = (await get(upstream, "/api/live")).body as LiveResponse;
  assert.equal(liveResponse.telemetry.worker_telemetry_age_seconds, 300);
  assert.equal(liveResponse.components.telemetry.state, "degraded");
  assert.equal(liveResponse.components.worker.state, "unknown");
  assert.equal(liveResponse.health.state, "degraded");
});

test("everything answering as it should is HEALTHY", async () => {
  const upstream = new FakeUpstream();
  const liveResponse = (await get(upstream, "/api/live")).body as LiveResponse;
  assert.equal(liveResponse.health.state, "healthy", liveResponse.health.reasons.join("; "));
  assert.equal(liveResponse.worker.instances[0]?.service_instance_id, "instance-a");
  assert.equal(liveResponse.signy.remote_healthy, true);
});

test("each upstream gets only its own credential, and Signy only the platform tenant", async () => {
  const upstream = new FakeUpstream();
  await get(upstream, "/api/live");
  await get(upstream, "/api/overview?window=24h");
  await get(upstream, "/api/series?window=7d");
  await get(upstream, "/api/errors?window=15m");
  assert.ok(upstream.requests.length > 0);
  for (const request of upstream.requests) {
    if (request.url.origin === CONFIG.accessPolicy.issuer) {
      assert.equal(request.url.pathname, "/cdn-cgi/access/certs");
      assert.equal(request.headers["cf-access-client-id"], undefined);
      continue;
    }
    if (request.url.origin === CONFIG.canaryUrl) {
      assert.equal(request.headers["cf-access-client-id"], CONFIG.canaryAccess.clientId);
      assert.equal(request.headers["x-tenant-id"], undefined);
    } else {
      assert.equal(request.url.origin, CONFIG.signyUrl);
      assert.equal(request.headers["cf-access-client-id"], CONFIG.signyAccess.clientId);
      assert.equal(request.headers["x-tenant-id"], "fn0");
    }
  }
});

test("no response ever carries a credential", async () => {
  const scenarios: ((upstream: FakeUpstream) => void)[] = [
    () => {},
    (upstream) => {
      upstream.signyDown = { status: 500, body: CONFIG.signyAccess.clientSecret };
    },
    (upstream) => {
      upstream.canary.runtime = { status: 502, body: CONFIG.canaryAccess.clientSecret };
    },
    (upstream) => {
      upstream.logs = [
        JSON.stringify({
          timestamp: "1790500000000000000",
          line: "manifest poll failed",
          attributes: { severity_text: "ERROR", service_name: "fn0-worker", error_type: "timeout" },
        }),
      ];
    },
  ];
  for (const scenario of scenarios) {
    const upstream = new FakeUpstream();
    scenario(upstream);
    for (const path of ["/api/live", "/api/overview", "/api/series", "/api/errors"]) {
      const { text } = await get(upstream, path);
      for (const secret of SECRET_VALUES) {
        assert.ok(!text.includes(secret), `${path} leaked a credential`);
      }
    }
  }
});

test("errors come back as bounded, normalized rows", async () => {
  const upstream = new FakeUpstream();
  upstream.logs = [
    JSON.stringify({
      timestamp: "1790500000000000000",
      line: "the request ran past its execution deadline",
      attributes: {
        severity_text: "ERROR",
        service_name: "fn0-worker",
        error_type: "deadline_exceeded",
        scope_name: "fn0_worker",
      },
    }),
  ];
  const response = (await get(upstream, "/api/errors")).body as {
    limit: number;
    errors: { timestamp: string; service: string; error_type: string; component: string }[];
  };
  assert.equal(response.errors[0]?.timestamp, new Date(1_790_500_000_000).toISOString());
  assert.equal(response.errors[0]?.error_type, "deadline_exceeded");
  assert.equal(response.errors[0]?.component, "fn0_worker");
  const logsRequest = upstream.requests.find((request) => request.url.pathname.endsWith("/logs"));
  assert.equal(logsRequest?.url.searchParams.get("limit"), String(response.limit));
  assert.deepEqual(logsRequest?.url.searchParams.getAll("attr"), ["severity_text=ERROR"]);
});

test("a missing binding is a 500 that names the binding and nothing else", async () => {
  forgetSigningKeys();
  const response = await handleRequest(new Request("https://ops.test/api/live"), () => {
    throw new MissingBindingError("CANARY_ACCESS_CLIENT_SECRET");
  });
  assert.equal(response.status, 500);
  assert.match(await response.text(), /CANARY_ACCESS_CLIENT_SECRET is not bound/);
});
