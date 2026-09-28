import assert from "node:assert/strict";
import { test } from "node:test";
import { dodbObservability } from "../src/dodb.ts";
import { FakeUpstream, counter } from "./fake_upstream.ts";

const instance = "dodb-instance-one";
const host = (value: number, labels: Record<string, string>) =>
  counter(value, { ["service_instance_id"]: instance, ...labels });

test("dodb telemetry separates operation rates, request histogram and persisted storage", async () => {
  const upstream = new FakeUpstream();
  upstream.instant["dodb.server.requests"] = [
    host(10, { operation: "get" }),
    host(2, { operation: "put" }),
  ];
  upstream.instant["dodb.server.request.duration_bucket"] = [
    host(8, { le: "0.005" }),
    host(12, { le: "0.025" }),
    host(12, { le: "+Inf" }),
  ];
  for (const metric of [
    "dodb.server.request.bytes",
    "dodb.server.response.bytes",
    "dodb.server.connections.active",
    "dodb.server.streams.active",
    "dodb.server.protocol.errors",
    "dodb.server.transport.errors",
    "dodb.server.application.errors",
    "dodb.server.overloaded.responses",
    "dodb.storage.database.file.bytes",
    "dodb.storage.wal.file.bytes",
    "dodb.storage.shards.persisted",
    "dodb.storage.shards.open",
  ]) {
    upstream.instant[metric] = [host(3, {})];
  }
  upstream.instant["dodb.server.connections.active"] = [host(0, {})];
  upstream.instant["dodb.server.request.bytes"] = [host(500, {})];
  upstream.instant["dodb.server.response.bytes"] = [host(900, {})];
  upstream.instant["dodb.storage.database.file.bytes"] = [host(20_000, {})];
  upstream.instant["dodb.storage.wal.file.bytes"] = [host(3_000, {})];
  upstream.instant["dodb.storage.shards.persisted"] = [host(4, {})];
  upstream.instant["dodb.storage.shards.open"] = [host(2, {})];
  upstream.instant["system.filesystem.usage"] = [
    counter(100, { host_name: instance, mountpoint: "/", state: "used" }),
    counter(900, { host_name: instance, mountpoint: "/", state: "free" }),
    counter(200, { host_name: instance, mountpoint: "/var/lib/dodb", state: "used" }),
    counter(800, { host_name: instance, mountpoint: "/var/lib/dodb", state: "free" }),
    counter(20, { host_name: instance, mountpoint: "/boot", state: "used" }),
    counter(80, { host_name: instance, mountpoint: "/boot", state: "free" }),
  ];
  upstream.instant["system.memory.usage"] = [
    counter(400, { host_name: instance, state: "used" }),
    counter(600, { host_name: instance, state: "free" }),
  ];
  upstream.instant["system.cpu.time"] = [
    counter(300, { host_name: instance, cpu_mode: "user" }),
    counter(700, { host_name: instance, cpu_mode: "idle" }),
  ];
  upstream.instant["system.disk.io"] = [
    counter(3000, { host_name: instance, direction: "read" }),
    counter(6000, { host_name: instance, direction: "write" }),
  ];
  upstream.instant["system.network.io"] = [
    counter(9000, { host_name: instance, direction: "receive" }),
    counter(12000, { host_name: instance, direction: "transmit" }),
  ];

  const result = await dodbObservability(upstream.dependencies());

  assert.equal(result.telemetry, "ok");
  assert.equal(result.service_instance_id, instance);
  assert.equal(result.telemetry_age_seconds, 0);
  assert.equal(result.operations_per_minute.get, 2);
  assert.equal(result.operations_per_minute.put, 0.4);
  assert.equal(result.operations_per_minute.scan, null);
  assert.ok(result.request_latency_seconds.p50 !== null);
  assert.ok(result.request_latency_seconds.p95 !== null);
  assert.ok(result.request_latency_seconds.p99 !== null);
  assert.equal(result.database_file_bytes, 20_000);
  assert.equal(result.wal_file_bytes, 3_000);
  assert.equal(result.persisted_shards, 4);
  assert.equal(result.open_shards, 2);
  assert.equal(result.disk_read_bytes_per_minute, 600);
  assert.equal(result.disk_write_bytes_per_minute, 1200);
  assert.equal(result.network_received_bytes_per_minute, 1800);
  assert.equal(result.network_sent_bytes_per_minute, 2400);
  assert.deepEqual(result.filesystem, {
    mountpoint: "/var/lib/dodb",
    total_bytes: 1_000,
    used_bytes: 200,
    available_bytes: 800,
  });
  assert.deepEqual(result.memory, { used_bytes: 400, available_bytes: 600 });
  assert.equal(result.cpu_busy_percent, 30);
});
