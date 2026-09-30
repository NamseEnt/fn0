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
  assert.match(APP_SCRIPT, /historyDue = Date\.now\(\) \+ 60_000/);
  assert.match(APP_SCRIPT, /setInterval\(\(\) => void updateLive\(\), 12_000\)/);
  assert.match(APP_SCRIPT, /setInterval\(\(\) => void updateHistory\(\), 2_000\)/);
  assert.match(APP_SCRIPT, /windowSelect\.addEventListener\("change", \(\) => \{ historyDue = 0; void updateHistory\(true\); \}\)/);
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

test("browser history loads DODB, Worker, then Telemetry and continues after endpoint failure", async () => {
  const start = APP_SCRIPT.indexOf("async function historyEndpoint(");
  const end = APP_SCRIPT.indexOf("\nasync function updateHistory", start);
  assert.ok(start >= 0 && end > start);
  const sequenceSource = APP_SCRIPT.slice(start, end);
  const calls: string[] = [];
  let activeNewHistoryRequests = 0;
  let maximumNewHistoryConcurrency = 0;
  const getJson = async (path: string) => {
    calls.push(path);
    const isNewHistoryEndpoint = ["/api/dodb-series", "/api/worker-series", "/api/telemetry-series"]
      .some((endpoint) => path.startsWith(endpoint));
    if (isNewHistoryEndpoint) {
      activeNewHistoryRequests += 1;
      maximumNewHistoryConcurrency = Math.max(maximumNewHistoryConcurrency, activeNewHistoryRequests);
    }
    try {
      await new Promise((resolve) => setTimeout(resolve, 0));
      if (path.startsWith("/api/dodb-series")) throw new Error("DODB history unavailable");
      return path;
    } finally {
      if (isNewHistoryEndpoint) activeNewHistoryRequests -= 1;
    }
  };
  const requestHistorySequence = new Function(
    "getJson",
    `${sequenceSource}; return requestHistorySequence;`,
  )(getJson) as (
    query: string,
    onStage: (stage: string, result: Record<string, unknown>) => boolean,
  ) => Promise<void>;
  const stages: string[] = [];
  const results: Record<string, unknown>[] = [];
  await requestHistorySequence("?window=1h", (stage, result) => {
    stages.push(stage);
    results.push(result);
    return true;
  });

  assert.deepEqual(calls, [
    "/api/overview?window=1h",
    "/api/series?window=1h",
    "/api/errors?window=1h",
    "/api/dodb-series?window=1h",
    "/api/worker-series?window=1h",
    "/api/telemetry-series?window=1h",
  ]);
  assert.deepEqual(stages, ["base", "dodb", "worker", "telemetry"]);
  assert.equal((results[1] as { status: string }).status, "rejected");
  assert.equal((results[2] as { status: string }).status, "fulfilled");
  assert.equal((results[3] as { status: string }).status, "fulfilled");
  assert.equal(maximumNewHistoryConcurrency, 1);
});
