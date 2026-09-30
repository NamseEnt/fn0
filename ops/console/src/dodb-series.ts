import { bucketsFromLabels, latencyFromBuckets } from "./accounting.ts";
import { alignGaugeRows, alignedRange, emptyColumn, mapLimit, perMinute } from "./history.ts";
import type { DodbHistoryResponse, DodbOperation } from "./schema.ts";
import type { Dependencies } from "./runtime.ts";
import { SignyClient } from "./signy.ts";
import { type TimeWindow, stepGrid } from "./windows.ts";

const OPERATIONS: readonly DodbOperation[] = ["get", "put", "delete", "query", "scan", "transact"];
const SERVICE_INSTANCE_ID = "fn0-dodb";
const HOST_NAME = "fn0-dodb";
type DodbRangeResult = Awaited<ReturnType<SignyClient["range"]>>;
type DodbRangeResults = [
  operations: DodbRangeResult,
  buckets: DodbRangeResult,
  database: DodbRangeResult,
  wal: DodbRangeResult,
  cpu: DodbRangeResult,
  memory: DodbRangeResult,
  disk: DodbRangeResult,
  network: DodbRangeResult,
];

function emptyOperations(length: number): Record<DodbOperation, (number | null)[]> {
  return Object.fromEntries(OPERATIONS.map((operation) => [operation, emptyColumn(length)])) as Record<DodbOperation, (number | null)[]>;
}

export async function dodbSeries(dependencies: Dependencies, window: TimeWindow): Promise<DodbHistoryResponse> {
  const signy = new SignyClient(dependencies);
  const grid = stepGrid(window, dependencies.nowMs());
  const gridRange = {
    startSeconds: grid[0]!,
    endSeconds: grid[grid.length - 1]!,
    stepSeconds: window.stepSeconds,
  };
  const range = `${window.stepSeconds}s`;
  const queryTasks = [
    () => signy.range({ metric: "dodb.server.requests", func: "increase", range, agg: "sum", by: ["operation", "service_instance_id"] }, gridRange),
    () => signy.range({ metric: "dodb.server.request.duration_bucket", func: "increase", range, agg: "sum", by: ["le", "service_instance_id"] }, gridRange),
    () => signy.range({ metric: "dodb.storage.database.file.bytes", agg: "sum", by: ["service_instance_id"] }, gridRange),
    () => signy.range({ metric: "dodb.storage.wal.file.bytes", agg: "sum", by: ["service_instance_id"] }, gridRange),
    () => signy.range({ metric: "system.cpu.time", func: "increase", range, agg: "sum", by: ["host_name", "cpu_mode"] }, gridRange),
    () => signy.range({ metric: "system.memory.usage", agg: "sum", by: ["host_name", "state"] }, gridRange),
    () => signy.range({ metric: "system.disk.io", func: "increase", range, agg: "sum", by: ["host_name", "direction"] }, gridRange),
    () => signy.range({ metric: "system.network.io", func: "increase", range, agg: "sum", by: ["host_name", "direction"] }, gridRange),
  ];
  const queryResults = await mapLimit(
    queryTasks,
    2,
    (query) => query(),
  );
  const [operations, buckets, database, wal, cpu, memory, disk, network] = queryResults as DodbRangeResults;
  const count = grid.length;
  const empty = () => emptyColumn(count);
  const operationsPerMinute = emptyOperations(count);
  if (operations.ok) {
    for (const operation of OPERATIONS) {
      operationsPerMinute[operation] = perMinute(
        alignedRange(grid, operations.value, window.stepSeconds, { operation, service_instance_id: SERVICE_INSTANCE_ID }),
        window.stepSeconds,
      );
    }
  }

  const latency = { p50: empty(), p95: empty(), p99: empty() };
  if (buckets.ok) {
    const bucketRows = buckets.value.filter((row) => row.labels.service_instance_id === SERVICE_INSTANCE_ID);
    const columns = bucketRows.map((row) => ({
      labels: row.labels,
      values: alignedRange(grid, [row], window.stepSeconds),
    }));
    grid.forEach((_, point) => {
      if (columns.every((column) => column.values[point] === null)) return;
      const pointBuckets = bucketsFromLabels(columns.flatMap((column) =>
        column.values[point] === null ? [] : [{ labels: column.labels, value: column.values[point]! }],
      ));
      const quantiles = latencyFromBuckets(pointBuckets);
      latency.p50[point] = quantiles.p50;
      latency.p95[point] = quantiles.p95;
      latency.p99[point] = quantiles.p99;
    });
  }

  const gauge = (result: typeof database): (number | null)[] => result.ok
    ? alignGaugeRows(grid, result.value, window.stepSeconds, { service_instance_id: SERVICE_INSTANCE_ID })
    : empty();
  const hostGauge = (result: typeof memory, state: string): (number | null)[] => result.ok
    ? alignGaugeRows(grid, result.value, window.stepSeconds, { host_name: HOST_NAME, state })
    : empty();
  const hostCounter = (result: typeof cpu, label: string, value: string): (number | null)[] => result.ok
    ? perMinute(alignedRange(grid, result.value, window.stepSeconds, { host_name: HOST_NAME, [label]: value }), window.stepSeconds)
    : empty();

  const cpuBusy = empty();
  if (cpu.ok) {
    const total = alignedRange(grid, cpu.value, window.stepSeconds, { host_name: HOST_NAME });
    const idle = alignedRange(grid, cpu.value, window.stepSeconds, { host_name: HOST_NAME, cpu_mode: "idle" }, "null");
    grid.forEach((_, point) => {
      const totalValue = total[point];
      const idleValue = idle[point];
      if (totalValue === null || totalValue === undefined || idleValue === null || idleValue === undefined || totalValue <= 0) return;
      cpuBusy[point] = Math.max(0, Math.min(100, (totalValue - idleValue) / totalValue * 100));
    });
  }

  const operationsTelemetry = operations.ok ? "ok" : "unavailable";
  const latencyTelemetry = buckets.ok ? "ok" : "unavailable";
  const storageTelemetry = database.ok && wal.ok ? "ok" : "unavailable";
  const cpuTelemetry = cpu.ok ? "ok" : "unavailable";
  const memoryTelemetry = memory.ok ? "ok" : "unavailable";
  const diskTelemetry = disk.ok ? "ok" : "unavailable";
  const networkTelemetry = network.ok ? "ok" : "unavailable";
  return {
    window: window.name,
    step_seconds: window.stepSeconds,
    generated_at: new Date(dependencies.nowMs()).toISOString(),
    timestamps_ms: grid.map((seconds) => seconds * 1000),
    operations: { telemetry: operationsTelemetry, per_minute: operationsPerMinute },
    latency: { telemetry: latencyTelemetry, seconds: latency },
    storage: {
      telemetry: storageTelemetry,
      database_file_bytes: gauge(database),
      wal_file_bytes: gauge(wal),
    },
    host: {
      cpu: { telemetry: cpuTelemetry, busy_percent: cpu.ok ? cpuBusy : empty() },
      memory: { telemetry: memoryTelemetry, used_bytes: hostGauge(memory, "used"), available_bytes: hostGauge(memory, "free") },
      disk: {
        telemetry: diskTelemetry,
        read_bytes_per_minute: hostCounter(disk, "direction", "read"),
        write_bytes_per_minute: hostCounter(disk, "direction", "write"),
      },
      network: {
        telemetry: networkTelemetry,
        received_bytes_per_minute: hostCounter(network, "direction", "receive"),
        sent_bytes_per_minute: hostCounter(network, "direction", "transmit"),
      },
    },
  };
}
