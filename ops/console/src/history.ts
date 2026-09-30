import type { RangeRow } from "./signy.ts";

export type HistoryColumn = (number | null)[];

export function alignedRange(
  timestampsSeconds: readonly number[],
  rows: readonly RangeRow[],
  stepSeconds: number,
  labels: Record<string, string> = {},
  absent: "zero" | "null" = "zero",
): HistoryColumn {
  const selected = rows.filter((row) =>
    Object.entries(labels).every(([key, value]) => row.labels[key] === value),
  );
  if (selected.length === 0) return timestampsSeconds.map(() => absent === "zero" ? 0 : null);
  const valuesByStep = new Map<number, number>();
  for (const row of selected) {
    for (const [timestampMs, value] of row.samples) {
      const step = Math.round(timestampMs / 1000 / stepSeconds);
      valuesByStep.set(step, (valuesByStep.get(step) ?? 0) + value);
    }
  }
  return timestampsSeconds.map((timestamp) => valuesByStep.get(Math.round(timestamp / stepSeconds)) ?? null);
}

export function alignGaugeRows(
  timestampsSeconds: readonly number[],
  rows: readonly RangeRow[],
  stepSeconds: number,
  labels: Record<string, string> = {},
): HistoryColumn {
  return alignedRange(timestampsSeconds, rows, stepSeconds, labels, "null");
}

export function perMinute(column: HistoryColumn, stepSeconds: number): HistoryColumn {
  return column.map((value) => value === null ? null : value * 60 / stepSeconds);
}

export function emptyColumn(length: number): HistoryColumn {
  return Array.from({ length }, () => null);
}
