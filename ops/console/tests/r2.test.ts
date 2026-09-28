import assert from "node:assert/strict";
import { test } from "node:test";
import { r2Analytics } from "../src/r2.ts";
import { FakeUpstream } from "./fake_upstream.ts";

test("R2 analytics returns storage, operation outcomes, bandwidth and source age by bucket", async () => {
  const upstream = new FakeUpstream();
  const sampleTime = new Date(upstream.dependencies().nowMs() - 5 * 60_000).toISOString();
  upstream.graphqlAnswer = {
    status: 200,
    body: JSON.stringify({
      data: {
        viewer: {
          accounts: [{
            storage: [
              {
                max: { payloadSize: 12000, metadataSize: 700, objectCount: 12 },
                dimensions: { bucketName: "fn0-signy-test", datetime: sampleTime },
              },
            ],
            operations: [
              { sum: { requests: 20 }, dimensions: { bucketName: "fn0-signy-test", actionStatus: "success", datetime: sampleTime } },
              { sum: { requests: 3 }, dimensions: { bucketName: "fn0-signy-test", actionStatus: "userError", datetime: sampleTime } },
              { sum: { requests: 1 }, dimensions: { bucketName: "fn0-signy-test", actionStatus: "internalError", datetime: sampleTime } },
            ],
            bandwidth: [
              { sum: { bytesUpload: 4096, bytesDownload: 8192 }, dimensions: { bucketName: "fn0-signy-test", datetimeFiveMinutes: sampleTime } },
            ],
          }],
        },
      },
    }),
  };

  const result = await r2Analytics(upstream.dependencies());

  assert.equal(result.status, "ok");
  assert.equal(result.window_seconds, 24 * 60 * 60);
  assert.deepEqual(result.buckets, [{
    bucket_name: "fn0-signy-test",
    payload_bytes: 12000,
    metadata_bytes: 700,
    object_count: 12,
    operation_count: 24,
    success_operations: 20,
    user_error_operations: 3,
    internal_error_operations: 1,
    upload_bytes: 4096,
    download_bytes: 8192,
    analytics_freshness_seconds: 300,
  }]);
  const request = upstream.requests.find((item) => item.url.origin === "https://api.cloudflare.com");
  assert.ok(request);
  assert.equal(request.headers.authorization, "Bearer analytics-api-token-XYZ");
  assert.equal(request.url.pathname, "/client/v4/graphql");
  const body = JSON.parse(request.body ?? "{}");
  assert.equal(body.variables.accountTag, "account-id-test");
  assert.doesNotMatch(body.query, /objectName/);
});

test("R2 analytics stays outside platform health when its account token is not configured", async () => {
  const upstream = new FakeUpstream();
  const dependencies = upstream.dependencies();
  dependencies.config = { ...dependencies.config, cloudflareAnalyticsApiToken: null };
  const result = await r2Analytics(dependencies);
  assert.equal(result.status, "unavailable");
  assert.deepEqual(result.buckets, []);
  assert.equal(upstream.requests.length, 0);
});

test("R2 GraphQL errors answer unavailable without leaking upstream details", async () => {
  const upstream = new FakeUpstream();
  upstream.graphqlAnswer = {
    status: 200,
    body: JSON.stringify({ errors: [{ message: "token must not be exposed" }] }),
  };
  const result = await r2Analytics(upstream.dependencies());
  assert.equal(result.status, "unavailable");
  assert.deepEqual(result.buckets, []);
  assert.doesNotMatch(JSON.stringify(result), /token must not be exposed/);
});
