import { z } from "zod";

type CacheEntry<T> = {
  promise?: Promise<T>;
  result?: T;
  error?: Error;
};

const isSSR = typeof window === "undefined";

const hookCache = new Map<string, CacheEntry<unknown>>();
const collectedCookies: string[] = [];
let requestCookie: string | null = null;

export function setRequestCookie(cookie: string | null): void {
  requestCookie = cookie;
}

if (!isSSR && (window as any).__FORTE_HOOK_CACHE__) {
  const serialized = (window as any).__FORTE_HOOK_CACHE__ as Record<
    string,
    unknown
  >;
  for (const [key, value] of Object.entries(serialized)) {
    hookCache.set(key, { result: value });
  }
}

export function useForteHook<T>(
  hookName: string,
  input: unknown,
  schema: z.ZodSchema<T>
): T {
  const cacheKey = `${hookName}:${JSON.stringify(input)}`;

  const cached = hookCache.get(cacheKey) as CacheEntry<T> | undefined;

  if (cached?.error) {
    throw cached.error;
  }

  if (cached?.result) {
    return cached.result;
  }

  if (!isSSR) {
    throw new Error(`Hook '${hookName}' was not pre-fetched during SSR`);
  }

  if (cached?.promise) {
    throw cached.promise;
  }

  const fortePort = process.env["FORTE_PORT"];
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (requestCookie) {
    headers["Cookie"] = requestCookie;
  }
  const promise = fetch(
    `http://localhost:${fortePort}/__forte_hook/${hookName}`,
    {
      method: "POST",
      headers,
      body: JSON.stringify(input),
    }
  )
    .then((res) => {
      if (!res.ok) {
        throw new Error(`Hook '${hookName}' failed: ${res.status}`);
      }
      const setCookies = res.headers.getSetCookie?.() ?? [];
      for (const cookie of setCookies) {
        collectedCookies.push(cookie);
      }
      return res.json();
    })
    .then((data) => {
      const result = schema.parse(data);
      const entry = hookCache.get(cacheKey) as CacheEntry<T>;
      entry.result = result;
      return result;
    })
    .catch((error) => {
      const entry = hookCache.get(cacheKey) as CacheEntry<T>;
      entry.error = error instanceof Error ? error : new Error(String(error));
      throw entry.error;
    });

  hookCache.set(cacheKey, { promise });
  throw promise;
}

export function serializeHookCache(): string {
  const serialized: Record<string, unknown> = {};
  for (const [key, entry] of hookCache.entries()) {
    if (entry.result) {
      serialized[key] = entry.result;
    }
  }
  return JSON.stringify(serialized)
    .replace(/</g, "\\u003c")
    .replace(/>/g, "\\u003e");
}

export function getCollectedCookies(): string[] {
  return [...collectedCookies];
}

export function clearCollectedCookies(): void {
  collectedCookies.length = 0;
}

export function clearHookCache(hookName?: string) {
  if (hookName) {
    for (const key of hookCache.keys()) {
      if (key.startsWith(`${hookName}:`)) {
        hookCache.delete(key);
      }
    }
  } else {
    hookCache.clear();
  }
}

export async function callAction<T = any>(
  actionName: string,
  input: unknown,
  schema?: z.ZodSchema<T>
): Promise<T> {
  const res = await fetch(`/__forte_action/${actionName}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    body: JSON.stringify(input),
  });

  if (!res.ok) {
    throw new Error(`Action '${actionName}' failed: ${res.status}`);
  }

  const data = await res.json();
  return schema ? schema.parse(data) : data;
}
