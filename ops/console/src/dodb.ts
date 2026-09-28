import { bucketsFromLabels, latencyFromBuckets } from "./accounting.ts";
import type { DodbObservability, DodbOperation } from "./schema.ts";
import type { Dependencies } from "./runtime.ts";
import { SignyClient, type InstantRow, type SignyResult } from "./signy.ts";

const OPERATIONS: readonly DodbOperation[] = ["get", "put", "delete", "query", "scan", "transact"];
const METRIC_RANGE = "300s";
const METRIC_WINDOW_SECONDS = 300;
const SERVICE_INSTANCE_ID = "service_instance_id";
const HOST_NAME = "host_name";

function sum(rows: readonly InstantRow[], select: (row: InstantRow) => boolean = () => true): number | null {
  const selected = rows.filter(select);
  return selected.length === 0 ? null : selected.reduce((total, row) => total + row.value, 0);
}

function selectedHostRows(rows: readonly InstantRow[], host: string | null): InstantRow[] {
  return host === null ? [] : rows.filter((row) => row.labels[HOST_NAME] === host);
}

function ageSeconds(nowMs: number, rows: readonly InstantRow[]): number | null {
  if (rows.length === 0) return null;
  const newestTimestamp = Math.max(...rows.map((row) => row.timestampMs));
  return Math.max(0, (nowMs - newestTimestamp) / 1000);
}

function emptyOperations(): Record<DodbOperation, number | null> {
  return { get: null, put: null, delete: null, query: null, scan: null, transact: null };
}

function rowsOrEmpty<T>(result: SignyResult<T[]>): T[] {
  return result.ok ? result.value : [];
}

function noData(telemetry: DodbObservability["telemetry"]): DodbObservability {
  return {
    telemetry,
    service_instance_id: null,
    telemetry_age_seconds: null,
    operations_per_minute: emptyOperations(),
    request_latency_seconds: { p50: null, p95: null, p99: null },
    request_bytes_per_minute: null,
    response_bytes_per_minute: null,
    active_connections: null,
    active_streams: null,
    protocol_errors: null,
    transport_errors: null,
    application_errors: null,
    overloaded_responses: null,
    database_file_bytes: null,
    wal_file_bytes: null,
    persisted_shards: null,
    open_shards: null,
    filesystem: null,
    cpu_busy_percent: null,
    memory: null,
    disk_read_bytes_per_minute: null,
    disk_write_bytes_per_minute: null,
    network_received_bytes_per_minute: null,
    network_sent_bytes_per_minute: null,
  };
}

function querySum(
  signy: SignyClient,
  metric: string,
  func?: "increase",
  by: readonly string[] = [SERVICE_INSTANCE_ID],
): Promise<SignyResult<InstantRow[]>> {
  return signy.instant({
    metric,
    ...(func ? { func, range: METRIC_RANGE } : {}),
    agg: "sum",
    by,
    lookback: "10m",
  });
}

export async function dodbObservability(dependencies: Dependencies): Promise<DodbObservability> {
  const signy = new SignyClient(dependencies);
  const metricNames = {
    requests: "dodb.server.requests",
    buckets: "dodb.server.request.duration_bucket",
    requestBytes: "dodb.server.request.bytes",
    responseBytes: "dodb.server.response.bytes",
    activeConnections: "dodb.server.connections.active",
    activeStreams: "dodb.server.streams.active",
    protocolErrors: "dodb.server.protocol.errors",
    transportErrors: "dodb.server.transport.errors",
    applicationErrors: "dodb.server.application.errors",
    overloads: "dodb.server.overloaded.responses",
    databaseBytes: "dodb.storage.database.file.bytes",
    walBytes: "dodb.storage.wal.file.bytes",
    persistedShards: "dodb.storage.shards.persisted",
    openShards: "dodb.storage.shards.open",
    filesystem: "system.filesystem.usage",
    memory: "system.memory.usage",
    cpu: "system.cpu.time",
    disk: "system.disk.io",
    network: "system.network.io",
  };
  const [
    requests,
    buckets,
    requestBytes,
    responseBytes,
    activeConnections,
    activeStreams,
    protocolErrors,
    transportErrors,
    applicationErrors,
    overloads,
    databaseBytes,
    walBytes,
    persistedShards,
    openShards,
    filesystem,
    memory,
    cpu,
    disk,
    network,
  ] = await Promise.all([
    querySum(signy, metricNames.requests, "increase", ["operation", SERVICE_INSTANCE_ID]),
    querySum(signy, metricNames.buckets, "increase", ["le", SERVICE_INSTANCE_ID]),
    querySum(signy, metricNames.requestBytes, "increase"),
    querySum(signy, metricNames.responseBytes, "increase"),
    querySum(signy, metricNames.activeConnections),
    querySum(signy, metricNames.activeStreams),
    querySum(signy, metricNames.protocolErrors, "increase"),
    querySum(signy, metricNames.transportErrors, "increase"),
    querySum(signy, metricNames.applicationErrors, "increase"),
    querySum(signy, metricNames.overloads, "increase"),
    querySum(signy, metricNames.databaseBytes),
    querySum(signy, metricNames.walBytes),
    querySum(signy, metricNames.persistedShards),
    querySum(signy, metricNames.openShards),
    signy.instant({ metric: metricNames.filesystem, agg: "sum", by: [HOST_NAME, "mountpoint", "state"], lookback: "10m" }),
    signy.instant({ metric: metricNames.memory, agg: "sum", by: [HOST_NAME, "state"], lookback: "10m" }),
    signy.instant({ metric: metricNames.cpu, func: "increase", range: METRIC_RANGE, agg: "sum", by: [HOST_NAME, "cpu_mode"], lookback: "10m" }),
    signy.instant({ metric: metricNames.disk, func: "increase", range: METRIC_RANGE, agg: "sum", by: [HOST_NAME, "direction"], lookback: "10m" }),
    signy.instant({ metric: metricNames.network, func: "increase", range: METRIC_RANGE, agg: "sum", by: [HOST_NAME, "direction"], lookback: "10m" }),
  ]);

  const required = [
    requests,
    buckets,
    requestBytes,
    responseBytes,
    activeConnections,
    activeStreams,
    protocolErrors,
    transportErrors,
    applicationErrors,
    overloads,
    databaseBytes,
    walBytes,
    persistedShards,
    openShards,
  ];
  if (required.some((result) => !result.ok)) {
    return noData("unavailable");
  }

  const requestRows = rowsOrEmpty(requests);
  const bucketRows = rowsOrEmpty(buckets);
  const requestByteRows = rowsOrEmpty(requestBytes);
  const responseByteRows = rowsOrEmpty(responseBytes);
  const activeConnectionRows = rowsOrEmpty(activeConnections);
  const activeStreamRows = rowsOrEmpty(activeStreams);
  const protocolErrorRows = rowsOrEmpty(protocolErrors);
  const transportErrorRows = rowsOrEmpty(transportErrors);
  const applicationErrorRows = rowsOrEmpty(applicationErrors);
  const overloadRows = rowsOrEmpty(overloads);
  const databaseByteRows = rowsOrEmpty(databaseBytes);
  const walByteRows = rowsOrEmpty(walBytes);
  const persistedShardRows = rowsOrEmpty(persistedShards);
  const openShardRows = rowsOrEmpty(openShards);
  const allAppRows = required.flatMap(rowsOrEmpty);
  const instanceRows = activeConnectionRows.length > 0 ? activeConnectionRows : requestRows;
  const serviceInstanceId = instanceRows[0]?.labels[SERVICE_INSTANCE_ID] ?? null;
  if (serviceInstanceId === null) {
    return noData("ok");
  }
  const appRows = allAppRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId);
  const operationRates = emptyOperations();
  for (const operation of OPERATIONS) {
    operationRates[operation] = sum(
      requestRows,
      (row) =>
        row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId &&
        row.labels.operation === operation,
    );
    if (operationRates[operation] !== null) {
      operationRates[operation] = operationRates[operation]! / METRIC_WINDOW_SECONDS * 60;
    }
  }
  const instanceBuckets = bucketRows.filter(
    (row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId,
  );
  const hostRows = (result: SignyResult<InstantRow[]>) =>
    result.ok ? selectedHostRows(result.value, serviceInstanceId) : [];
  const filesystemRows = hostRows(filesystem);
  const mountpoints = [...new Set(filesystemRows.map((row) => row.labels.mountpoint).filter((value) => value !== undefined))];
  const dataDirectoryMountpoint = mountpoints
    .filter((mountpoint) =>
      mountpoint === "/var/lib/dodb" ||
      mountpoint === "/" ||
      "/var/lib/dodb".startsWith(`${mountpoint.replace(/\/$/, "")}/`),
    )
    .sort((left, right) => right.length - left.length)[0];
  const fsUsed = dataDirectoryMountpoint === undefined
    ? null
    : sum(filesystemRows, (row) => row.labels.mountpoint === dataDirectoryMountpoint && row.labels.state === "used");
  const fsAvailable = dataDirectoryMountpoint === undefined
    ? null
    : sum(filesystemRows, (row) => row.labels.mountpoint === dataDirectoryMountpoint && row.labels.state === "free");
  const memoryRows = hostRows(memory);
  const memoryUsed = sum(memoryRows, (row) => row.labels.state === "used");
  const memoryAvailable = sum(memoryRows, (row) => row.labels.state === "free");
  const cpuRows = hostRows(cpu);
  const cpuTotal = sum(cpuRows);
  const cpuIdle = sum(cpuRows, (row) => row.labels.cpu_mode === "idle");
  const diskRows = hostRows(disk);
  const networkRows = hostRows(network);
  const ratesPerMinute = (rows: readonly InstantRow[], direction: string) => {
    const total = sum(rows, (row) => row.labels.direction === direction);
    return total === null ? null : total / METRIC_WINDOW_SECONDS * 60;
  };
  const nowMs = dependencies.nowMs();
  const ageRows = [
    ...appRows,
    ...filesystemRows,
    ...memoryRows,
    ...cpuRows,
    ...diskRows,
    ...networkRows,
  ];
  const allMetricsAvailable =
    filesystem.ok && memory.ok && cpu.ok && disk.ok && network.ok;

  return {
    ...noData(allMetricsAvailable ? "ok" : "unavailable"),
    service_instance_id: serviceInstanceId,
    telemetry_age_seconds: ageSeconds(nowMs, ageRows),
    operations_per_minute: operationRates,
    request_latency_seconds: latencyFromBuckets(bucketsFromLabels(instanceBuckets)),
    request_bytes_per_minute: (sum(requestByteRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)) ?? 0) / METRIC_WINDOW_SECONDS * 60,
    response_bytes_per_minute: (sum(responseByteRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)) ?? 0) / METRIC_WINDOW_SECONDS * 60,
    active_connections: sum(activeConnectionRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    active_streams: sum(activeStreamRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    protocol_errors: sum(protocolErrorRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    transport_errors: sum(transportErrorRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    application_errors: sum(applicationErrorRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    overloaded_responses: sum(overloadRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    database_file_bytes: sum(databaseByteRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    wal_file_bytes: sum(walByteRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    persisted_shards: sum(persistedShardRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    open_shards: sum(openShardRows.filter((row) => row.labels[SERVICE_INSTANCE_ID] === serviceInstanceId)),
    filesystem:
      dataDirectoryMountpoint !== undefined && fsUsed !== null && fsAvailable !== null
        ? {
            mountpoint: dataDirectoryMountpoint,
            total_bytes: fsUsed + fsAvailable,
            used_bytes: fsUsed,
            available_bytes: fsAvailable,
          }
        : null,
    cpu_busy_percent:
      cpuTotal !== null && cpuTotal > 0 && cpuIdle !== null
        ? ((cpuTotal - cpuIdle) / cpuTotal) * 100
        : null,
    memory:
      memoryUsed !== null && memoryAvailable !== null
        ? { used_bytes: memoryUsed, available_bytes: memoryAvailable }
        : null,
    disk_read_bytes_per_minute: ratesPerMinute(diskRows, "read"),
    disk_write_bytes_per_minute: ratesPerMinute(diskRows, "write"),
    network_received_bytes_per_minute: ratesPerMinute(networkRows, "receive"),
    network_sent_bytes_per_minute: ratesPerMinute(networkRows, "transmit"),
  };
}
