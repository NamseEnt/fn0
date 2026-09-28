import assert from "node:assert/strict";
import { test } from "node:test";
import { invocationCounts, ratios } from "../src/accounting.ts";

test("invocation counts sum the known outcomes and ignore anything else", () => {
  const counts = invocationCounts(
    new Map([
      ["ok", 90],
      ["client_error", 5],
      ["server_error", 3],
      ["failed", 2],
      ["mystery", 1000],
    ]),
  );
  assert.deepEqual(counts, { total: 100, ok: 90, client_error: 5, server_error: 3, failed: 2 });
});

test("unanswered adds 504 and every 503 rejection to both sides", () => {
  const counts = invocationCounts(new Map([["ok", 90], ["server_error", 8], ["failed", 2]]));
  const result = ratios(counts, 5, { queue_full: 3, project_admission_full: 1, closed: 1 });
  assert.equal(result.guest_server_error, 8 / 100);
  assert.equal(result.unanswered, (2 + 5 + 3 + 1 + 1) / (100 + 5 + 5));
});

test("no traffic answers null ratios, not zero", () => {
  const empty = invocationCounts(new Map());
  assert.deepEqual(ratios(empty, 0, { queue_full: 0, project_admission_full: 0, closed: 0 }), {
    guest_server_error: null,
    unanswered: null,
  });
});

test("rejections alone still count as unanswered", () => {
  const empty = invocationCounts(new Map());
  const result = ratios(empty, 0, { queue_full: 4, project_admission_full: 0, closed: 0 });
  assert.equal(result.guest_server_error, null);
  assert.equal(result.unanswered, 1);
});
