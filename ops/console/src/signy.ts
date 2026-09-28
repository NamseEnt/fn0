import { accessHeaders } from "./config.ts";
import type { ProbeResult } from "./schema.ts";
import type { Dependencies } from "./runtime.ts";

/** Everything the console may ask Signy, as fixed fields rather than a query
 * string, so no browser input can become part of a Signy request. */
export interface MetricSelector {
  metric: string;
  attributes?: readonly string[];
  func?: "increase" | "rate";
  range?: string;
  agg?: "sum" | "max" | "min" | "avg" | "count";
  by?: readonly string[];
  lookback?: string;
}

export interface InstantRow {
  labels: Record<string, string>;
  timestampMs: number;
  value: number;
}

export interface RangeRow {
  labels: Record<string, string>;
  samples: [timestampMs: number, value: number][];
}

export interface LogRow {
  timestampMs: number;
  line: string;
  attributes: Record<string, string>;
}

export type SignyFailure =
  | { kind: "timeout" }
  | { kind: "unreachable" }
  | { kind: "http"; status: number }
  | { kind: "malformed" };

export type SignyResult<T> = { ok: true; value: T } | { ok: false; failure: SignyFailure };

const QUERY_PREFIX = "/signy/api/v1";

function nanosecondsToMs(nanoseconds: unknown): number | null {
  if (typeof nanoseconds !== "string" || !/^\d+$/.test(nanoseconds)) {
    return null;
  }
  return Number(BigInt(nanoseconds) / 1_000_000n);
}

function selectorParams(selector: MetricSelector): [string, string][] {
  const params: [string, string][] = [["metric", selector.metric]];
  for (const attribute of selector.attributes ?? []) {
    params.push(["attr", attribute]);
  }
  if (selector.func) params.push(["func", selector.func]);
  if (selector.range) params.push(["range", selector.range]);
  if (selector.agg) params.push(["agg", selector.agg]);
  for (const key of selector.by ?? []) {
    params.push(["by", key]);
  }
  if (selector.lookback) params.push(["lookback", selector.lookback]);
  return params;
}

function isTimeoutError(error: unknown): boolean {
  return (
    error instanceof Error &&
    (error.name === "TimeoutError" || error.name === "AbortError")
  );
}

export class SignyClient {
  private readonly dependencies: Dependencies;

  constructor(dependencies: Dependencies) {
    this.dependencies = dependencies;
  }

  private async get(path: string, params: [string, string][]): Promise<SignyResult<string>> {
    const { config, fetch, deadlines } = this.dependencies;
    const url = `${config.signyUrl}${path}${
      params.length > 0 ? `?${new URLSearchParams(params).toString()}` : ""
    }`;
    try {
      const response = await fetch(url, {
        headers: {
          ...accessHeaders(config.signyAccess),
          "X-Tenant-Id": config.platformTelemetryTenant,
        },
        redirect: "manual",
        signal: AbortSignal.timeout(deadlines.signyMs),
      });
      const body = await response.text();
      if (response.status !== 200) {
        return { ok: false, failure: { kind: "http", status: response.status } };
      }
      return { ok: true, value: body };
    } catch (error) {
      return {
        ok: false,
        failure: isTimeoutError(error) ? { kind: "timeout" } : { kind: "unreachable" },
      };
    }
  }

  private async getLines(
    path: string,
    params: [string, string][],
  ): Promise<SignyResult<Record<string, unknown>[]>> {
    const result = await this.get(`${QUERY_PREFIX}${path}`, params);
    if (!result.ok) {
      return result;
    }
    try {
      const lines = result.value
        .split("\n")
        .filter((line) => line.trim() !== "")
        .map((line) => JSON.parse(line) as unknown);
      if (!lines.every((line) => typeof line === "object" && line !== null)) {
        return { ok: false, failure: { kind: "malformed" } };
      }
      return { ok: true, value: lines as Record<string, unknown>[] };
    } catch {
      return { ok: false, failure: { kind: "malformed" } };
    }
  }

  async instant(selector: MetricSelector): Promise<SignyResult<InstantRow[]>> {
    const result = await this.getLines("/metrics/instant", selectorParams(selector));
    if (!result.ok) {
      return result;
    }
    const rows: InstantRow[] = [];
    for (const line of result.value) {
      const timestampMs = nanosecondsToMs(line.timestamp);
      if (timestampMs === null || typeof line.value !== "number" || !isLabels(line.labels)) {
        return { ok: false, failure: { kind: "malformed" } };
      }
      rows.push({ labels: line.labels, timestampMs, value: line.value });
    }
    return { ok: true, value: rows };
  }

  async range(
    selector: MetricSelector,
    grid: { startSeconds: number; endSeconds: number; stepSeconds: number },
  ): Promise<SignyResult<RangeRow[]>> {
    const result = await this.getLines("/metrics/query", [
      ...selectorParams(selector),
      ["start", String(grid.startSeconds)],
      ["end", String(grid.endSeconds)],
      ["step", `${grid.stepSeconds}s`],
    ]);
    if (!result.ok) {
      return result;
    }
    const rows: RangeRow[] = [];
    for (const line of result.value) {
      if (!isLabels(line.labels) || !Array.isArray(line.samples)) {
        return { ok: false, failure: { kind: "malformed" } };
      }
      const samples: [number, number][] = [];
      for (const sample of line.samples as unknown[]) {
        if (!Array.isArray(sample)) {
          return { ok: false, failure: { kind: "malformed" } };
        }
        const timestampMs = nanosecondsToMs(sample[0]);
        if (timestampMs === null || typeof sample[1] !== "number") {
          return { ok: false, failure: { kind: "malformed" } };
        }
        samples.push([timestampMs, sample[1]]);
      }
      rows.push({ labels: line.labels, samples });
    }
    return { ok: true, value: rows };
  }

  async logs(query: {
    startSeconds: number;
    attributes: readonly string[];
    limit: number;
  }): Promise<SignyResult<LogRow[]>> {
    const params: [string, string][] = [
      ["start", String(query.startSeconds)],
      ["limit", String(query.limit)],
      ["direction", "backward"],
    ];
    for (const attribute of query.attributes) {
      params.push(["attr", attribute]);
    }
    const result = await this.getLines("/logs", params);
    if (!result.ok) {
      return result;
    }
    const rows: LogRow[] = [];
    for (const line of result.value) {
      const timestampMs = nanosecondsToMs(line.timestamp);
      if (timestampMs === null || typeof line.line !== "string" || !isLabels(line.attributes)) {
        return { ok: false, failure: { kind: "malformed" } };
      }
      rows.push({ timestampMs, line: line.line, attributes: line.attributes });
    }
    return { ok: true, value: rows };
  }

  async ready(): Promise<ProbeResult> {
    const startedMs = this.dependencies.nowMs();
    const result = await this.get("/ready", []);
    const latency_ms = Math.max(0, this.dependencies.nowMs() - startedMs);
    if (result.ok) {
      return { status: "ok", latency_ms, http_status: 200, failure: null };
    }
    switch (result.failure.kind) {
      case "timeout":
        return { status: "timeout", latency_ms: null, http_status: null, failure: null };
      case "unreachable":
        return { status: "unreachable", latency_ms: null, http_status: null, failure: null };
      case "http": {
        const status = result.failure.status;
        return {
          status: status === 401 || status === 403 ? "unauthorized" : "unexpected",
          latency_ms,
          http_status: status,
          failure: null,
        };
      }
      case "malformed":
        return { status: "unexpected", latency_ms, http_status: 200, failure: null };
    }
  }

  /** Signy's own Prometheus exposition. Its health gauges live only here, not
   * in any tenant. */
  async exposition(): Promise<SignyResult<Map<string, number>>> {
    const result = await this.get("/metrics", []);
    if (!result.ok) {
      return result;
    }
    const values = new Map<string, number>();
    for (const line of result.value.split("\n")) {
      const match = /^([a-zA-Z_:][a-zA-Z0-9_:]*) (\S+)$/.exec(line.trim());
      if (match) {
        const value = Number(match[2]);
        if (Number.isFinite(value)) {
          values.set(match[1]!, value);
        }
      }
    }
    return { ok: true, value: values };
  }
}

function isLabels(value: unknown): value is Record<string, string> {
  return (
    typeof value === "object" &&
    value !== null &&
    Object.values(value).every((entry) => typeof entry === "string")
  );
}
