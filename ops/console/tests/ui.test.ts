import assert from "node:assert/strict";
import { test } from "node:test";
import { APP_HTML, APP_SCRIPT } from "../src/ui.ts";

class MockNode {
  attributes: Record<string, string> = {};
  children: MockNode[] = [];
  style: Record<string, string> = {};
  textContent = "";
  className = "";
  readonly tagName: string;

  constructor(tagName: string) {
    this.tagName = tagName;
  }

  setAttribute(name: string, value: string): void {
    this.attributes[name] = value;
  }

  append(...nodes: MockNode[]): void {
    this.children.push(...nodes);
  }

  replaceChildren(...nodes: MockNode[]): void {
    this.children = nodes;
  }
}

function chartFunctions() {
  const start = APP_SCRIPT.indexOf("function axisValue(");
  const end = APP_SCRIPT.indexOf("\nfunction renderOverview", start);
  assert.ok(start >= 0 && end > start);
  const chartSource = APP_SCRIPT.slice(start, end);
  const document = {
    createElementNS: (_namespace: string, tagName: string) => new MockNode(tagName),
    createElement: (tagName: string) => new MockNode(tagName),
    createTextNode: (value: string) => {
      const node = new MockNode("text-node");
      node.textContent = value;
      return node;
    },
  };
  return new Function(
    "document",
    "number",
    "formatBytes",
    "percent",
    "duration",
    `${chartSource}; return { axisValue, drawChart };`,
  )(
    document,
    (value: number) => `number:${value}`,
    (value: number) => `bytes:${value}`,
    (value: number) => `percent:${value}`,
    (value: number) => `duration:${value}`,
  ) as {
    axisValue(value: number, format: string): string;
    drawChart(target: MockNode, columns: (number | null)[][], labels: string[], colors: string[]): void;
  };
}

test("history chart containers preserve the current graphs and adapt to mobile width", () => {
  for (const id of [
    "request-chart",
    "latency-chart",
    "rejection-chart",
    "dodb-operations-chart",
    "dodb-latency-chart",
    "dodb-storage-chart",
    "host-cpu-chart",
    "host-memory-chart",
    "host-io-chart",
    "worker-history-chart",
    "telemetry-queue-chart",
    "telemetry-segments-chart",
  ]) {
    assert.match(APP_HTML, new RegExp(`id="${id}"`));
  }
  assert.match(APP_HTML, /name="viewport"/);
  assert.match(APP_HTML, /\.chart\{width:100%;height:150px/);
  assert.match(APP_SCRIPT, /setAttribute\("viewBox"/);
});

test("chart axis supports number, bytes, percent and duration formatters", () => {
  const { axisValue } = chartFunctions();
  assert.equal(axisValue(12, "number"), "number:12");
  assert.equal(axisValue(2048, "bytes"), "bytes:2048");
  assert.equal(axisValue(30, "percent"), "percent:0.3");
  assert.equal(axisValue(0.25, "duration"), "duration:0.25");
});

test("chart rendering splits null gaps and renders multiple series legends", () => {
  const { drawChart } = chartFunctions();
  const target = new MockNode("target");
  assert.doesNotThrow(() => drawChart(
    target,
    [[1, null, 3], [4, 5, 6]],
    ["counter", "gauge"],
    ["#315fbd", "#e1a136"],
  ));
  assert.equal(target.children.length, 2);
  const svg = target.children[0]!;
  assert.equal(svg.attributes.viewBox, "0 0 720 150");
  assert.equal(svg.children.filter((child) => child.tagName === "path").length, 3);
  assert.equal(target.children[1]!.className, "chart-legend");
  assert.equal(target.children[1]!.children.length, 2);
});
