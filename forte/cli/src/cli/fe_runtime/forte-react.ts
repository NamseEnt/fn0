import { z } from "zod";

export type HeadDescriptor =
  | { title: string }
  | { name: string; content: string }
  | { property: string; content: string }
  | ({ tagName: "link" | "meta" } & Record<string, string>);

function headDescriptorKey(descriptor: HeadDescriptor): string | null {
  if ("tagName" in descriptor) return null;
  if ("title" in descriptor) return "title";
  if ("name" in descriptor) return `name:${descriptor.name}`;
  return `property:${(descriptor as { property: string }).property}`;
}

export function mergeHeadDescriptors(
  base: HeadDescriptor[],
  overrides: HeadDescriptor[]
): HeadDescriptor[] {
  const overrideKeys = new Set(
    overrides.map(headDescriptorKey).filter((key) => key !== null)
  );
  const kept = base.filter((descriptor) => {
    const key = headDescriptorKey(descriptor);
    return key === null || !overrideKeys.has(key);
  });
  return [...kept, ...overrides];
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function renderHeadDescriptors(descriptors: HeadDescriptor[]): string {
  return descriptors
    .map((descriptor) => {
      if ("tagName" in descriptor) {
        const { tagName, ...attrs } = descriptor;
        const attrHtml = Object.entries(attrs)
          .map(([key, value]) => ` ${key}="${escapeHtml(value)}"`)
          .join("");
        return `<${tagName}${attrHtml}/>`;
      }
      if ("title" in descriptor) {
        return `<title>${escapeHtml(descriptor.title)}</title>`;
      }
      if ("name" in descriptor) {
        return `<meta name="${escapeHtml(descriptor.name)}" content="${escapeHtml(descriptor.content)}"/>`;
      }
      return `<meta property="${escapeHtml(descriptor.property)}" content="${escapeHtml(descriptor.content)}"/>`;
    })
    .join("\n");
}

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

// A host without a loopback HTTP server (Cloudflare Workers) sets this to an
// origin it intercepts itself, so the SSR pass never leaves the process.
function selfInvokeBase(): string {
  const override = (globalThis as any).__FORTE_SELF_INVOKE_BASE__;
  if (typeof override === "string") {
    return override;
  }
  return `http://localhost:${process.env["FORTE_PORT"]}`;
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

  if (cached?.promise) {
    throw cached.promise;
  }

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  let url: string;
  let init: RequestInit;
  if (isSSR) {
    if (requestCookie) {
      headers["Cookie"] = requestCookie;
    }
    url = `${selfInvokeBase()}/__self_invoke/${hookName}`;
    init = { method: "POST", headers, body: JSON.stringify(input) };
  } else {
    headers["X-Forte-Prefetch-Miss"] = "1";
    url = `/__self_invoke/${hookName}`;
    init = {
      method: "POST",
      headers,
      body: JSON.stringify(input),
      credentials: "include",
    };
  }
  const promise = fetch(url, init)
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
