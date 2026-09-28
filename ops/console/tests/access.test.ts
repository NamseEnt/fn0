import assert from "node:assert/strict";
import { test } from "node:test";
import { forgetSigningKeys } from "../src/access.ts";
import { handleRequest } from "../src/index.ts";
import {
  CONFIG,
  FOREIGN_SIGNING_KEY,
  FakeUpstream,
  NOW_MS,
  accessToken,
} from "./fake_upstream.ts";

async function requestWith(
  upstream: FakeUpstream,
  token: string | null,
  path = "/api/overview",
): Promise<{ status: number; reason: unknown }> {
  const headers: Record<string, string> = {};
  if (token !== null) {
    headers["Cf-Access-Jwt-Assertion"] = token;
  }
  const response = await handleRequest(new Request(`https://ops.test${path}`, { headers }), () =>
    upstream.dependencies(),
  );
  const body = (await response.json()) as { reason?: unknown };
  return { status: response.status, reason: body.reason };
}

function dataRequests(upstream: FakeUpstream) {
  return upstream.requests.filter((request) => request.url.origin !== CONFIG.accessPolicy.issuer);
}

const refusals: [string, () => Promise<string | null>, string][] = [
  ["no token", async () => null, "missing"],
  ["not a JWT", async () => "not-a-jwt", "malformed"],
  ["a foreign signing key", () => accessToken({}, { signingKey: FOREIGN_SIGNING_KEY.privateKey }), "bad_signature"],
  ["an unknown key id", () => accessToken({}, { keyId: "somebody-else" }), "unknown_key"],
  ["another application's audience", () => accessToken({ aud: ["other-app"] }), "wrong_audience"],
  ["another team's issuer", () => accessToken({ iss: "https://other-team.test" }), "wrong_issuer"],
  ["an expired token", () => accessToken({ exp: NOW_MS / 1000 - 120 }), "expired"],
  ["a token not valid yet", () => accessToken({ nbf: NOW_MS / 1000 + 600 }), "not_yet_valid"],
  ["another person", () => accessToken({ email: "intruder@example.com" }), "wrong_operator"],
  ["no email claim", () => accessToken({ email: undefined }), "wrong_operator"],
];

for (const [name, token, reason] of refusals) {
  test(`a request with ${name} is refused before any data is read`, async () => {
    forgetSigningKeys();
    const upstream = new FakeUpstream();
    for (const path of ["/api/overview", "/api/live", "/", "/app.js", "/anything"]) {
      const response = await requestWith(upstream, await token(), path);
      assert.equal(response.status, 403, path);
      assert.equal(response.reason, reason, path);
    }
    assert.equal(dataRequests(upstream).length, 0);
  });
}

test("the operator's token passes, whatever the email's letter case", async () => {
  forgetSigningKeys();
  const upstream = new FakeUpstream();
  const response = await requestWith(
    upstream,
    await accessToken({ email: CONFIG.accessPolicy.operatorEmail.toUpperCase(), aud: "ops-console-audience" }),
  );
  assert.equal(response.status, 200);
});

test("signing keys that cannot be fetched refuse the request", async () => {
  forgetSigningKeys();
  const upstream = new FakeUpstream();
  upstream.signingKeysAnswer = { status: 500, body: "" };
  const response = await requestWith(upstream, await accessToken());
  assert.equal(response.status, 403);
  assert.equal(response.reason, "signing_keys_unavailable");
});

test("signing keys are fetched once and reused", async () => {
  forgetSigningKeys();
  const upstream = new FakeUpstream();
  const token = await accessToken();
  await requestWith(upstream, token, "/api/errors");
  await requestWith(upstream, token, "/api/errors");
  const keyFetches = upstream.requests.filter(
    (request) => request.url.origin === CONFIG.accessPolicy.issuer,
  );
  assert.equal(keyFetches.length, 1);
});
