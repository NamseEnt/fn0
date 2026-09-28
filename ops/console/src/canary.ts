import { accessHeaders } from "./config.ts";
import type { ProbeResult } from "./schema.ts";
import type { Dependencies } from "./runtime.ts";

export type CanaryProbeName = "runtime" | "dodb" | "dodb-write" | "storage";

const KNOWN_FAILURES = new Set([
  "missing",
  "mismatch",
  "unavailable",
  "write_failed",
  "read_failed",
]);

export async function probeCanary(
  dependencies: Dependencies,
  probe: CanaryProbeName,
): Promise<ProbeResult> {
  const { config, fetch, nowMs, deadlines } = dependencies;
  const startedMs = nowMs();
  let response: Response;
  let body: string;
  try {
    response = await fetch(`${config.canaryUrl}/api/${probe}`, {
      headers: accessHeaders(config.canaryAccess),
      redirect: "manual",
      signal: AbortSignal.timeout(deadlines.canaryMs),
    });
    body = await response.text();
  } catch (error) {
    const timedOut =
      error instanceof Error && (error.name === "TimeoutError" || error.name === "AbortError");
    return {
      status: timedOut ? "timeout" : "unreachable",
      latency_ms: null,
      http_status: null,
      failure: null,
    };
  }
  const latency_ms = Math.max(0, nowMs() - startedMs);
  const http_status = response.status;
  if (http_status === 401 || http_status === 403) {
    return { status: "unauthorized", latency_ms, http_status, failure: null };
  }
  const answer = parseCanaryAnswer(body);
  if (http_status === 200 && answer?.ok === true) {
    return { status: "ok", latency_ms, http_status, failure: null };
  }
  if (
    http_status === 503 &&
    answer?.ok === false &&
    typeof answer.failure === "string" &&
    KNOWN_FAILURES.has(answer.failure)
  ) {
    return { status: "failed", latency_ms, http_status, failure: answer.failure };
  }
  return { status: "unexpected", latency_ms, http_status, failure: null };
}

function parseCanaryAnswer(body: string): { ok?: unknown; failure?: unknown } | null {
  try {
    const parsed: unknown = JSON.parse(body);
    return typeof parsed === "object" && parsed !== null ? parsed : null;
  } catch {
    return null;
  }
}
