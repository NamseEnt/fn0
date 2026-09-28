import assert from "node:assert/strict";
import { test } from "node:test";
import { type CumulativeBucket, histogramQuantile, parseUpperBound } from "../src/quantile.ts";

const buckets = (pairs: [number, number][]): CumulativeBucket[] =>
  pairs.map(([upperBound, count]) => ({ upperBound, count }));

const close = (actual: number | null, expected: number) => {
  assert.notEqual(actual, null);
  assert.ok(Math.abs((actual as number) - expected) < 1e-9, `${actual} != ${expected}`);
};

const EVEN = buckets([
  [1, 25],
  [2, 50],
  [3, 75],
  [4, 100],
  [Infinity, 100],
]);

test("interpolates linearly inside the bucket holding the rank", () => {
  close(histogramQuantile(0.5, EVEN), 2);
  close(histogramQuantile(0.95, EVEN), 3.8);
  close(histogramQuantile(0.99, EVEN), 3.96);
  close(histogramQuantile(0.1, EVEN), 0.4);
});

test("the first bucket starts at zero", () => {
  close(histogramQuantile(0.5, buckets([[0.1, 10], [Infinity, 10]])), 0.05);
});

test("a rank in the +Inf bucket answers the highest finite bound", () => {
  assert.equal(histogramQuantile(0.99, buckets([[1, 10], [5, 50], [Infinity, 100]])), 5);
});

test("reproduces the production platform histogram read on 2026-09-27", () => {
  const production = buckets([
    [0.005, 0],
    [0.01, 2],
    [0.025, 4],
    [0.05, 4],
    [0.1, 4],
    [0.25, 5],
    [0.5, 6],
    [1, 6],
    [2.5, 6],
    [5, 6],
    [10, 6],
    [15, 6],
    [30, 6],
    [Infinity, 6],
  ]);
  close(histogramQuantile(0.5, production), 0.0175);
  close(histogramQuantile(0.95, production), 0.425);
});

test("nothing to rank answers null", () => {
  assert.equal(histogramQuantile(0.5, []), null);
  assert.equal(histogramQuantile(0.5, buckets([[1, 0], [Infinity, 0]])), null);
});

test("a quantile outside [0, 1] answers null", () => {
  assert.equal(histogramQuantile(1.5, EVEN), null);
  assert.equal(histogramQuantile(-0.1, EVEN), null);
  assert.equal(histogramQuantile(Number.NaN, EVEN), null);
});

test("input order does not matter", () => {
  close(histogramQuantile(0.95, [...EVEN].reverse()), 3.8);
});

test("counts that dip at a window edge are made monotone first", () => {
  const dipping = buckets([
    [1, 25],
    [2, 24],
    [3, 75],
    [4, 100],
    [Infinity, 100],
  ]);
  close(histogramQuantile(0.25, dipping), 1);
  close(histogramQuantile(0.5, dipping), 2.5);
});

test("empty leading buckets are skipped rather than divided by", () => {
  close(histogramQuantile(0.5, buckets([[1, 0], [2, 0], [4, 10], [Infinity, 10]])), 3);
  close(histogramQuantile(0, buckets([[1, 0], [2, 0], [4, 10], [Infinity, 10]])), 2);
});

test("non-finite counts and bounds are ignored", () => {
  const polluted = [...EVEN, { upperBound: Number.NaN, count: 7 }, { upperBound: 5, count: Number.NaN }];
  close(histogramQuantile(0.5, polluted), 2);
});

test("parses Signy's le labels", () => {
  assert.equal(parseUpperBound("+Inf"), Infinity);
  assert.equal(parseUpperBound("0.025"), 0.025);
  assert.equal(parseUpperBound("30"), 30);
  assert.equal(parseUpperBound("abc"), null);
  assert.equal(parseUpperBound(undefined), null);
});
