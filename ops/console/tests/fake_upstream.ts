import type { ConsoleConfig } from "../src/config.ts";
import type { Dependencies } from "../src/runtime.ts";

export const NOW_MS = Date.UTC(2026, 8, 28, 3, 0, 0);

export const CONFIG: ConsoleConfig = {
  signyUrl: "https://signy.test",
  signyAccess: { clientId: "signy-client-id-XYZ", clientSecret: "signy-client-secret-XYZ" },
  platformTelemetryTenant: "fn0",
  canaryUrl: "https://canary.test",
  canaryAccess: { clientId: "canary-client-id-XYZ", clientSecret: "canary-client-secret-XYZ" },
  accessPolicy: {
    issuer: "https://team.test",
    audience: "ops-console-audience",
    operatorEmail: "operator@example.com",
  },
};

const SIGNING_KEY_ID = "signing-key-1";

async function generateSigningKey(): Promise<CryptoKeyPair> {
  return (await crypto.subtle.generateKey(
    {
      name: "RSASSA-PKCS1-v1_5",
      modulusLength: 2048,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true,
    ["sign", "verify"],
  )) as CryptoKeyPair;
}

export const ACCESS_SIGNING_KEY = await generateSigningKey();
export const FOREIGN_SIGNING_KEY = await generateSigningKey();

function base64Url(bytes: Uint8Array | string): string {
  const raw = typeof bytes === "string" ? new TextEncoder().encode(bytes) : bytes;
  return Buffer.from(raw).toString("base64url");
}

export async function accessToken(
  claims: Record<string, unknown> = {},
  options: { signingKey?: CryptoKey; keyId?: string } = {},
): Promise<string> {
  const header = base64Url(
    JSON.stringify({ alg: "RS256", kid: options.keyId ?? SIGNING_KEY_ID, typ: "JWT" }),
  );
  const payload = base64Url(
    JSON.stringify({
      iss: CONFIG.accessPolicy.issuer,
      aud: [CONFIG.accessPolicy.audience],
      email: CONFIG.accessPolicy.operatorEmail,
      iat: NOW_MS / 1000 - 60,
      nbf: NOW_MS / 1000 - 60,
      exp: NOW_MS / 1000 + 3600,
      type: "app",
      ...claims,
    }),
  );
  const signature = await crypto.subtle.sign(
    "RSASSA-PKCS1-v1_5",
    options.signingKey ?? ACCESS_SIGNING_KEY.privateKey,
    new TextEncoder().encode(`${header}.${payload}`),
  );
  return `${header}.${payload}.${base64Url(new Uint8Array(signature))}`;
}

const signingJwk = {
  ...(await crypto.subtle.exportKey("jwk", ACCESS_SIGNING_KEY.publicKey)),
  kid: SIGNING_KEY_ID,
};

export const SECRET_VALUES = [
  CONFIG.signyAccess.clientId,
  CONFIG.signyAccess.clientSecret,
  CONFIG.canaryAccess.clientId,
  CONFIG.canaryAccess.clientSecret,
];

export type Answer =
  | { status: number; body: string }
  | "timeout"
  | "unreachable";

export interface RecordedRequest {
  url: URL;
  headers: Record<string, string>;
}

const nanoseconds = (ms: number) => `${BigInt(ms) * 1_000_000n}`;

export function gauge(value: number, ageSeconds: number, labels: Record<string, string> = {}) {
  return JSON.stringify({
    labels: { service_instance_id: "instance-a", service_name: "fn0-worker", ...labels },
    timestamp: nanoseconds(NOW_MS - ageSeconds * 1000),
    value,
  });
}

export function counter(value: number, labels: Record<string, string> = {}) {
  return JSON.stringify({ labels, timestamp: nanoseconds(NOW_MS), value });
}

export class FakeUpstream {
  canary: Record<string, Answer> = {
    runtime: { status: 200, body: '{"ok":true}' },
    dodb: { status: 200, body: '{"ok":true}' },
    storage: { status: 200, body: '{"ok":true}' },
  };
  signyDown: Answer | null = null;
  workerTelemetryAgeSeconds = 30;
  instant: Record<string, string[]> = {};
  range: Record<string, string[]> = {};
  logs: string[] = [];
  signingKeysAnswer: Answer | null = null;
  requests: RecordedRequest[] = [];

  dependencies(): Dependencies {
    return {
      config: CONFIG,
      fetch: (input, init) => this.fetch(input, init),
      nowMs: () => NOW_MS,
      deadlines: { canaryMs: 50, signyMs: 50 },
    };
  }

  private async fetch(input: string, init?: RequestInit): Promise<Response> {
    const url = new URL(input);
    const headers = Object.fromEntries(new Headers(init?.headers).entries());
    this.requests.push({ url, headers });
    const answer = this.answerFor(url);
    if (answer === "timeout") {
      // Node does not keep the process alive for AbortSignal.timeout, so a
      // request that only waits on it would end the test run early.
      return new Promise((_, reject) => {
        const keepAlive = setInterval(() => {}, 1_000);
        init?.signal?.addEventListener("abort", () => {
          clearInterval(keepAlive);
          reject(init.signal?.reason);
        });
      });
    }
    if (answer === "unreachable") {
      throw new TypeError("fetch failed");
    }
    return new Response(answer.body, { status: answer.status });
  }

  private answerFor(url: URL): Answer {
    if (url.origin === CONFIG.accessPolicy.issuer) {
      return (
        this.signingKeysAnswer ??
        (url.pathname === "/cdn-cgi/access/certs"
          ? { status: 200, body: JSON.stringify({ keys: [signingJwk] }) }
          : { status: 404, body: "" })
      );
    }
    if (url.origin === CONFIG.canaryUrl) {
      return this.canary[url.pathname.replace("/api/", "")] ?? { status: 404, body: "" };
    }
    if (url.origin !== CONFIG.signyUrl) {
      return { status: 599, body: "unexpected host" };
    }
    if (this.signyDown !== null) {
      return this.signyDown;
    }
    if (url.pathname === "/ready") {
      return { status: 200, body: "ready" };
    }
    if (url.pathname === "/metrics") {
      return { status: 200, body: "signy_remote_healthy 1\nsigny_ingest_errors_total 0\n" };
    }
    const metric = url.searchParams.get("metric") ?? "";
    if (url.pathname === "/signy/api/v1/metrics/instant") {
      return { status: 200, body: (this.instant[metric] ?? this.defaultInstant(metric)).join("\n") };
    }
    if (url.pathname === "/signy/api/v1/metrics/query") {
      return { status: 200, body: (this.range[metric] ?? []).join("\n") };
    }
    if (url.pathname === "/signy/api/v1/logs") {
      return { status: 200, body: this.logs.join("\n") };
    }
    return { status: 404, body: "" };
  }

  private defaultInstant(metric: string): string[] {
    const age = this.workerTelemetryAgeSeconds;
    switch (metric) {
      case "fn0.worker.manifest_loaded":
        return [gauge(1, age)];
      case "fn0.worker.draining":
        return [gauge(0, age)];
      case "fn0.worker.requests.in_flight":
        return [gauge(2, age)];
      case "fn0.worker.websocket.connections":
        return [gauge(0, age)];
      case "collecty_queue_bytes":
        return [counter(0)];
      default:
        return [];
    }
  }
}
