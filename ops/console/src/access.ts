import type { FetchLike } from "./runtime.ts";

/**
 * Second line of defence behind Cloudflare Access: every request must carry
 * the `Cf-Access-Jwt-Assertion` Access issued for this application and the
 * configured operator. Access at the edge already turns everyone else away
 * (and runs the login flow), so a request failing here means the edge was
 * bypassed or misconfigured, and it gets a bare 403.
 *
 * https://developers.cloudflare.com/cloudflare-one/access-controls/applications/http-apps/authorization-cookie/validating-json/
 */
export const ACCESS_JWT_HEADER = "Cf-Access-Jwt-Assertion";
const CLOCK_SKEW_SECONDS = 60;
const SIGNING_KEYS_TTL_MS = 60 * 60 * 1000;

export interface AccessPolicy {
  /** `https://<team>.cloudflareaccess.com`, exactly as Access puts it in `iss`. */
  issuer: string;
  audience: string;
  operatorEmail: string;
}

export type AccessRefusal =
  | "missing"
  | "malformed"
  | "unknown_key"
  | "bad_signature"
  | "wrong_issuer"
  | "wrong_audience"
  | "expired"
  | "not_yet_valid"
  | "wrong_operator"
  | "signing_keys_unavailable";

interface SigningKeyCache {
  issuer: string;
  fetchedAtMs: number;
  keys: Map<string, CryptoKey>;
}

let signingKeyCache: SigningKeyCache | null = null;

export function forgetSigningKeys(): void {
  signingKeyCache = null;
}

function base64UrlDecode(segment: string): Uint8Array | null {
  if (!/^[A-Za-z0-9_-]*$/.test(segment)) {
    return null;
  }
  const base64 = segment.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64 + "=".repeat((4 - (base64.length % 4)) % 4);
  try {
    return Uint8Array.from(atob(padded), (character) => character.charCodeAt(0));
  } catch {
    return null;
  }
}

function decodeJson(segment: string): Record<string, unknown> | null {
  const bytes = base64UrlDecode(segment);
  if (bytes === null) {
    return null;
  }
  try {
    const value: unknown = JSON.parse(new TextDecoder().decode(bytes));
    return typeof value === "object" && value !== null && !Array.isArray(value)
      ? (value as Record<string, unknown>)
      : null;
  } catch {
    return null;
  }
}

async function loadSigningKeys(
  policy: AccessPolicy,
  fetch: FetchLike,
  nowMs: number,
  refresh: boolean,
): Promise<Map<string, CryptoKey> | null> {
  if (
    !refresh &&
    signingKeyCache !== null &&
    signingKeyCache.issuer === policy.issuer &&
    nowMs - signingKeyCache.fetchedAtMs < SIGNING_KEYS_TTL_MS
  ) {
    return signingKeyCache.keys;
  }
  try {
    const response = await fetch(`${policy.issuer}/cdn-cgi/access/certs`, {
      signal: AbortSignal.timeout(5_000),
    });
    if (response.status !== 200) {
      return null;
    }
    const body = (await response.json()) as { keys?: unknown };
    if (!Array.isArray(body.keys)) {
      return null;
    }
    const keys = new Map<string, CryptoKey>();
    for (const jwk of body.keys as (JsonWebKey & { kid?: string })[]) {
      if (typeof jwk.kid !== "string" || jwk.kty !== "RSA") {
        continue;
      }
      keys.set(
        jwk.kid,
        await crypto.subtle.importKey(
          "jwk",
          { kty: jwk.kty, n: jwk.n, e: jwk.e },
          { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
          false,
          ["verify"],
        ),
      );
    }
    signingKeyCache = { issuer: policy.issuer, fetchedAtMs: nowMs, keys };
    return keys;
  } catch {
    return null;
  }
}

export async function verifyAccessRequest(
  request: Request,
  policy: AccessPolicy,
  fetch: FetchLike,
  nowMs: number,
): Promise<AccessRefusal | null> {
  const token = request.headers.get(ACCESS_JWT_HEADER);
  if (token === null || token === "") {
    return "missing";
  }
  const segments = token.split(".");
  if (segments.length !== 3) {
    return "malformed";
  }
  const [encodedHeader, encodedPayload, encodedSignature] = segments as [string, string, string];
  const header = decodeJson(encodedHeader);
  const payload = decodeJson(encodedPayload);
  const signature = base64UrlDecode(encodedSignature);
  if (header === null || payload === null || signature === null) {
    return "malformed";
  }
  if (header.alg !== "RS256" || typeof header.kid !== "string") {
    return "malformed";
  }

  let keys = await loadSigningKeys(policy, fetch, nowMs, false);
  if (keys !== null && !keys.has(header.kid)) {
    keys = await loadSigningKeys(policy, fetch, nowMs, true);
  }
  if (keys === null) {
    return "signing_keys_unavailable";
  }
  const key = keys.get(header.kid);
  if (key === undefined) {
    return "unknown_key";
  }
  const signedBytes = new TextEncoder().encode(`${encodedHeader}.${encodedPayload}`);
  const signatureValid = await crypto.subtle.verify(
    "RSASSA-PKCS1-v1_5",
    key,
    signature,
    signedBytes,
  );
  if (!signatureValid) {
    return "bad_signature";
  }

  if (payload.iss !== policy.issuer) {
    return "wrong_issuer";
  }
  const audiences = Array.isArray(payload.aud) ? payload.aud : [payload.aud];
  if (!audiences.includes(policy.audience)) {
    return "wrong_audience";
  }
  const nowSeconds = nowMs / 1000;
  if (typeof payload.exp !== "number" || payload.exp + CLOCK_SKEW_SECONDS < nowSeconds) {
    return "expired";
  }
  if (typeof payload.nbf === "number" && payload.nbf - CLOCK_SKEW_SECONDS > nowSeconds) {
    return "not_yet_valid";
  }
  if (
    typeof payload.email !== "string" ||
    payload.email.toLowerCase() !== policy.operatorEmail.toLowerCase()
  ) {
    return "wrong_operator";
  }
  return null;
}
