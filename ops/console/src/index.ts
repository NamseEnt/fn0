import { verifyAccessRequest } from "./access.ts";
import { type Env, MissingBindingError, readConfig } from "./config.ts";
import { recentErrors } from "./errors.ts";
import { dodbSeries } from "./dodb-series.ts";
import { live } from "./live.ts";
import { overview } from "./overview.ts";
import { r2Analytics } from "./r2.ts";
import { DEFAULT_DEADLINES, type Dependencies } from "./runtime.ts";
import { series } from "./series.ts";
import { telemetrySeries } from "./telemetry-series.ts";
import { workerSeries } from "./worker-series.ts";
import { APP_HTML, APP_SCRIPT } from "./ui.ts";
import { type TimeWindow, WINDOW_NAMES, parseWindow } from "./windows.ts";

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "cache-control": "no-store",
      "x-content-type-options": "nosniff",
    },
  });
}

function staticAsset(contentType: string, body: string): Response {
  return new Response(body, {
    headers: {
      "content-type": contentType,
      "cache-control": "no-store",
      "content-security-policy": "default-src 'none'; script-src 'self'; style-src 'unsafe-inline'; connect-src 'self'; img-src data:; base-uri 'none'; frame-ancestors 'none'",
      "x-content-type-options": "nosniff",
      "x-frame-options": "DENY",
      "referrer-policy": "no-referrer",
    },
  });
}

type Route =
  | { windowed: false; handle: (dependencies: Dependencies) => Promise<unknown> }
  | {
      windowed: true;
      handle: (dependencies: Dependencies, window: TimeWindow) => Promise<unknown>;
    };

const ROUTES: Record<string, Route> = {
  "/api/live": { windowed: false, handle: live },
  "/api/r2": { windowed: false, handle: r2Analytics },
  "/api/overview": { windowed: true, handle: overview },
  "/api/series": { windowed: true, handle: series },
  "/api/dodb-series": { windowed: true, handle: dodbSeries },
  "/api/worker-series": { windowed: true, handle: workerSeries },
  "/api/telemetry-series": { windowed: true, handle: telemetrySeries },
  "/api/errors": { windowed: true, handle: recentErrors },
};

/**
 * Every request must first pass the Access JWT check. Every answer is built from fixed queries: a route accepts `window` from the
 * whitelist and nothing else, so no browser input reaches Signy or the canary.
 */
export async function handleRequest(
  request: Request,
  dependencies: () => Dependencies,
): Promise<Response> {
  let resolved: Dependencies;
  try {
    resolved = dependencies();
  } catch (error) {
    if (error instanceof MissingBindingError) {
      return json(500, { error: `console misconfigured: ${error.binding} is not bound` });
    }
    throw error;
  }
  const refusal = await verifyAccessRequest(
    request,
    resolved.config.accessPolicy,
    resolved.fetch,
    resolved.nowMs(),
  );
  if (refusal !== null) {
    return json(403, { error: "forbidden", reason: refusal });
  }

  const url = new URL(request.url);
  if (url.pathname === "/" || url.pathname === "/app.js") {
    if (request.method !== "GET") {
      return json(405, { error: "only GET is served" });
    }
    return url.pathname === "/"
      ? staticAsset("text/html; charset=utf-8", APP_HTML)
      : staticAsset("text/javascript; charset=utf-8", APP_SCRIPT);
  }
  const route = ROUTES[url.pathname];
  if (route === undefined) {
    return json(404, { error: "not found" });
  }
  if (request.method !== "GET") {
    return json(405, { error: "only GET is served" });
  }
  const allowedParameters = route.windowed ? ["window"] : [];
  const unexpected = [...url.searchParams.keys()].filter(
    (key) => !allowedParameters.includes(key),
  );
  if (unexpected.length > 0) {
    return json(400, { error: `unknown parameter: ${unexpected.join(", ")}` });
  }
  if (!route.windowed) {
    return json(200, await route.handle(resolved));
  }
  if (url.searchParams.getAll("window").length > 1) {
    return json(400, { error: "window given more than once" });
  }
  const window = parseWindow(url.searchParams.get("window"));
  if (window === null) {
    return json(400, { error: "invalid window", allowed: WINDOW_NAMES });
  }
  return json(200, await route.handle(resolved, window));
}

export default {
  fetch(request: Request, env: Env): Promise<Response> {
    return handleRequest(request, () => ({
      config: readConfig(env),
      fetch: (input, init) => fetch(input, init),
      nowMs: () => Date.now(),
      deadlines: DEFAULT_DEADLINES,
    }));
  },
};
