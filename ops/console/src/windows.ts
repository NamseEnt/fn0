export const WINDOW_NAMES = ["15m", "1h", "6h", "24h", "7d"] as const;
export type WindowName = (typeof WINDOW_NAMES)[number];
export const DEFAULT_WINDOW: WindowName = "1h";

export interface TimeWindow {
  name: WindowName;
  seconds: number;
  stepSeconds: number;
}

// The worker exports every 60 s, so no step is finer than that: a 15 s grid
// over 60 s samples is three zeros and a spike, not a finer picture.
const WINDOWS: Record<WindowName, TimeWindow> = {
  "15m": { name: "15m", seconds: 15 * 60, stepSeconds: 60 },
  "1h": { name: "1h", seconds: 60 * 60, stepSeconds: 60 },
  "6h": { name: "6h", seconds: 6 * 60 * 60, stepSeconds: 5 * 60 },
  "24h": { name: "24h", seconds: 24 * 60 * 60, stepSeconds: 15 * 60 },
  "7d": { name: "7d", seconds: 7 * 24 * 60 * 60, stepSeconds: 60 * 60 },
};

export function parseWindow(value: string | null): TimeWindow | null {
  const name = value ?? DEFAULT_WINDOW;
  return (WINDOW_NAMES as readonly string[]).includes(name)
    ? WINDOWS[name as WindowName]
    : null;
}

export function pointCount(window: TimeWindow): number {
  return window.seconds / window.stepSeconds;
}

/** Evaluation instants in unix seconds, oldest first, ending at the last
 * completed step boundary. */
export function stepGrid(window: TimeWindow, nowMs: number): number[] {
  const lastBoundary =
    Math.floor(nowMs / 1000 / window.stepSeconds) * window.stepSeconds;
  const count = pointCount(window);
  return Array.from(
    { length: count },
    (_, index) => lastBoundary - (count - 1 - index) * window.stepSeconds,
  );
}
