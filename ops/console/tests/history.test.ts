import assert from "node:assert/strict";
import { test } from "node:test";
import { dodbSeries } from "../src/dodb-series.ts";
import { alignGaugeRows, alignedRange, perMinute } from "../src/history.ts";
import type { RangeRow } from "../src/signy.ts";
import { telemetrySeries } from "../src/telemetry-series.ts";
import { workerSeries } from "../src/worker-series.ts";
import { parseWindow } from "../src/windows.ts";
import { FakeUpstream, NOW_MS } from "./fake_upstream.ts";

const timestampAt = (offsetSeconds = 0) => BigInt(NOW_MS + offsetSeconds * 1000) * 1_000_000n;
const row = (value: number, labels: Record<string, string> = {}, offsetSeconds = 0): string =>
  JSON.stringify({ labels, samples: [[String(timestampAt(offsetSeconds)), value]] });

test("range alignment distinguishes absent counters from missing samples and gauge absence", () => {
  const grid = [100, 160, 220];
  const stepSeconds = 60;
  assert.deepEqual(alignedRange(grid, [], stepSeconds), [0, 0, 0]);
  assert.deepEqual(alignGaugeRows(grid, [], stepSeconds), [null, null, null]);
  const sample: RangeRow = { labels: {}, samples: [[160_000, 4]] };
  assert.deepEqual(alignedRange(grid, [sample], stepSeconds), [null, 4, null]);
  assert.deepEqual(perMinute([10, 25, null], 300), [2, 5, null]);
});

test("DODB operations map each operation and normalize each selected step to per-minute", async () => {
  for (const [windowName, increment, expected] of [
    ["15m", 12, 12],
    ["1h", 12, 12],
    ["6h", 25, 5],
  ] as const) {
    const upstream = new FakeUpstream();
    upstream.range["dodb.server.requests"] = [
      row(increment, { operation: "get", service_instance_id: "fn0-dodb" }),
      row(increment * 2, { operation: "put", service_instance_id: "fn0-dodb" }),
      row(increment * 100, { operation: "get", service_instance_id: "other-instance" }),
    ];
    const result = await dodbSeries(upstream.dependencies(), parseWindow(windowName)!);
    const last = result.timestamps_ms.length - 1;
    assert.equal(result.operations.telemetry, "ok");
    assert.equal(result.operations.per_minute.get[last], expected);
    assert.equal(result.operations.per_minute.put[last], expected * 2);
    assert.equal(result.operations.per_minute.delete[last], 0);
    assert.equal(result.operations.per_minute.get[last - 1], null);
    const request = upstream.requests.find((entry) => entry.url.searchParams.get("metric") === "dodb.server.requests");
    assert.equal(request?.url.searchParams.get("func"), "increase");
    assert.equal(request?.url.searchParams.get("range"), `${result.step_seconds}s`);
    assert.equal(request?.url.searchParams.getAll("by").includes("operation"), true);
  }
});

test("DODB gauges retain gaps while CPU, disk and network use the live host identity", async () => {
  const upstream = new FakeUpstream();
  upstream.range["dodb.storage.database.file.bytes"] = [row(9000, { service_instance_id: "fn0-dodb" })];
  upstream.range["dodb.storage.wal.file.bytes"] = [row(3000, { service_instance_id: "fn0-dodb" })];
  upstream.range["system.cpu.time"] = [
    row(70, { host_name: "fn0-dodb", cpu_mode: "idle" }),
    row(30, { host_name: "fn0-dodb", cpu_mode: "user" }),
    row(900, { host_name: "another-host", cpu_mode: "idle" }),
  ];
  upstream.range["system.memory.usage"] = [
    row(400, { host_name: "fn0-dodb", state: "used" }),
    row(600, { host_name: "fn0-dodb", state: "free" }),
  ];
  upstream.range["system.disk.io"] = [
    row(3000, { host_name: "fn0-dodb", direction: "read" }),
    row(6000, { host_name: "fn0-dodb", direction: "write" }),
  ];
  upstream.range["system.network.io"] = [
    row(9000, { host_name: "fn0-dodb", direction: "receive" }),
    row(12000, { host_name: "fn0-dodb", direction: "transmit" }),
  ];

  const result = await dodbSeries(upstream.dependencies(), parseWindow("6h")!);
  const last = result.timestamps_ms.length - 1;
  assert.equal(result.storage.database_file_bytes[last], 9000);
  assert.equal(result.storage.wal_file_bytes[last], 3000);
  assert.equal(result.storage.database_file_bytes[last - 1], null);
  assert.equal(result.host.cpu.busy_percent[last], 30);
  assert.equal(result.host.cpu.busy_percent[last - 1], null);
  assert.equal(result.host.memory.used_bytes[last], 400);
  assert.equal(result.host.memory.available_bytes[last], 600);
  assert.equal(result.host.disk.read_bytes_per_minute[last], 600);
  assert.equal(result.host.disk.write_bytes_per_minute[last], 1200);
  assert.equal(result.host.network.received_bytes_per_minute[last], 1800);
  assert.equal(result.host.network.sent_bytes_per_minute[last], 2400);
  const cpuRequest = upstream.requests.find((entry) => entry.url.searchParams.get("metric") === "system.cpu.time");
  assert.deepEqual(cpuRequest?.url.searchParams.getAll("by"), ["host_name", "cpu_mode"]);
});

test("DODB host query failure stays local to that host graph", async () => {
  const upstream = new FakeUpstream();
  upstream.rangeFailures["system.cpu.time"] = { status: 502, body: "unavailable" };
  upstream.range["dodb.server.requests"] = [row(12, { operation: "get", service_instance_id: "fn0-dodb" })];
  const result = await dodbSeries(upstream.dependencies(), parseWindow("1h")!);
  assert.equal(result.operations.telemetry, "ok");
  assert.equal(result.host.cpu.telemetry, "unavailable");
  assert.equal(result.host.memory.telemetry, "ok");
  assert.ok(result.host.cpu.busy_percent.every((value) => value === null));
});

test("worker history sums blue-green instances while preserving gauge gaps", async () => {
  const upstream = new FakeUpstream();
  upstream.range["fn0.worker.requests.in_flight"] = [
    row(2, { service_instance_id: "worker-a" }),
    row(3, { service_instance_id: "worker-b" }),
  ];
  upstream.range["fn0.worker.websocket.connections"] = [
    row(4, { service_instance_id: "worker-a" }),
    row(6, { service_instance_id: "worker-b" }),
  ];
  const result = await workerSeries(upstream.dependencies(), parseWindow("1h")!);
  const last = result.timestamps_ms.length - 1;
  assert.equal(result.telemetry, "ok");
  assert.equal(result.in_flight_requests[last], 5);
  assert.equal(result.websocket_connections[last], 10);
  assert.equal(result.in_flight_requests[last - 1], null);
  assert.equal(upstream.requests.filter((entry) => entry.url.pathname.endsWith("/metrics/query")).length, 2);
});

test("Collecty history treats queue as a gauge and absent counters as zero", async () => {
  const upstream = new FakeUpstream();
  upstream.range.collecty_queue_bytes = [row(7000)];
  upstream.range.collecty_queue_dropped_segments_total = [row(5)];
  const result = await telemetrySeries(upstream.dependencies(), parseWindow("6h")!);
  const last = result.timestamps_ms.length - 1;
  assert.equal(result.queue.bytes[last], 7000);
  assert.equal(result.queue.bytes[last - 1], null);
  assert.equal(result.segments.dropped[last], 5);
  assert.equal(result.segments.refused[last], 0);
  assert.equal(result.segments.refused[last - 1], 0);
  const request = upstream.requests.find((entry) => entry.url.searchParams.get("metric") === "collecty_queue_dropped_segments_total");
  assert.equal(request?.url.searchParams.get("func"), "increase");
  assert.equal(request?.url.searchParams.get("range"), "300s");
});
