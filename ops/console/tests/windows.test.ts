import assert from "node:assert/strict";
import { test } from "node:test";
import { WINDOW_NAMES, parseWindow, pointCount, stepGrid } from "../src/windows.ts";

test("only whitelisted windows parse, and the default is 1h", () => {
  assert.equal(parseWindow(null)?.name, "1h");
  for (const name of WINDOW_NAMES) {
    assert.equal(parseWindow(name)?.name, name);
  }
  for (const invalid of ["30m", "", "1H", "7d ", "3600", "-1h"]) {
    assert.equal(parseWindow(invalid), null, invalid);
  }
});

test("every window has a bounded number of points and no step under the export cadence", () => {
  for (const name of WINDOW_NAMES) {
    const window = parseWindow(name)!;
    assert.ok(window.stepSeconds >= 60, name);
    assert.ok(Number.isInteger(pointCount(window)), name);
    assert.ok(pointCount(window) <= 168, name);
  }
});

test("the grid ends at the last completed step and is evenly spaced", () => {
  const window = parseWindow("6h")!;
  const nowMs = Date.UTC(2026, 8, 28, 3, 7, 42);
  const grid = stepGrid(window, nowMs);
  assert.equal(grid.length, pointCount(window));
  assert.equal(grid[grid.length - 1], Date.UTC(2026, 8, 28, 3, 5, 0) / 1000);
  for (let index = 1; index < grid.length; index += 1) {
    assert.equal(grid[index]! - grid[index - 1]!, window.stepSeconds);
  }
});
