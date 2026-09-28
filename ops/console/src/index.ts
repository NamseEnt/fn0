import { type Env, MissingBindingError, readConfig } from "./config.ts";
import { recentErrors } from "./errors.ts";
import { live } from "./live.ts";
import { overview } from "./overview.ts";
import { DEFAULT_DEADLINES, type Dependencies } from "./runtime.ts";
import { series } from "./series.ts";
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

type Route =
  | { windowed: false; handle: (dependencies: Dependencies) => Promise<unknown> }
  | {
      windowed: true;
      handle: (dependencies: Dependencies, window: TimeWindow) => Promise<unknown>;
    };

const ROUTES: Record<string, Route> = {
  "/api/live": { windowed: false, handle: live },
  "/api/overview": { windowed: true, handle: overview },
  "/api/series": { windowed: true, handle: series },
  "/api/errors": { windowed: true, handle: recentErrors },
};

/**
 * Every answer is built from fixed queries: a route accepts `window` from the
 * whitelist and nothing else, so no browser input reaches Signy or the canary.
 */
export async function handleRequest(
  request: Request,
  dependencies: () => Dependencies,
): Promise<Response> {
  const url = new URL(request.url);
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
  let resolved: Dependencies;
  try {
    resolved = dependencies();
  } catch (error) {
    if (error instanceof MissingBindingError) {
      return json(500, { error: `console misconfigured: ${error.binding} is not bound` });
    }
    throw error;
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
