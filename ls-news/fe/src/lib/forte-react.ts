import { z } from "zod";

type CacheEntry<T> = {
  promise?: Promise<T>;
  result?: T;
  error?: Error;
};

const isSSR = typeof window === "undefined";
let ssrBaseUrl = "http://localhost:3000";

export function setSSRBaseUrl(url: string) {
  ssrBaseUrl = url;
}

const hookCache = new Map<string, CacheEntry<unknown>>();

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

  const promise = fetch(`${ssrBaseUrl}/__forte_hook/${hookName}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(input),
  })
    .then((res) => {
      if (!res.ok) {
        throw new Error(`Hook '${hookName}' failed: ${res.status}`);
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
