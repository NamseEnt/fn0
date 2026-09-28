import type { Dependencies } from "./runtime.ts";
import type { ErrorsResponse } from "./schema.ts";
import { SignyClient } from "./signy.ts";
import type { TimeWindow } from "./windows.ts";

export const ERROR_LIMIT = 50;

export async function recentErrors(
  dependencies: Dependencies,
  window: TimeWindow,
): Promise<ErrorsResponse> {
  const nowMs = dependencies.nowMs();
  const result = await new SignyClient(dependencies).logs({
    startSeconds: Math.floor(nowMs / 1000) - window.seconds,
    attributes: ["severity_text=ERROR"],
    limit: ERROR_LIMIT,
  });
  return {
    window: window.name,
    generated_at: new Date(nowMs).toISOString(),
    telemetry: result.ok ? "ok" : "unavailable",
    limit: ERROR_LIMIT,
    errors: result.ok
      ? result.value.map((row) => ({
          timestamp: new Date(row.timestampMs).toISOString(),
          service: row.attributes.service_name ?? null,
          message: row.line,
          error_type: row.attributes.error_type ?? null,
          component: row.attributes.scope_name ?? null,
          attributes: row.attributes,
        }))
      : [],
  };
}
