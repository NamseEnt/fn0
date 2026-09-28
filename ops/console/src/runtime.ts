import type { ConsoleConfig } from "./config.ts";

export type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

export interface Deadlines {
  canaryMs: number;
  signyMs: number;
}

export interface Dependencies {
  config: ConsoleConfig;
  fetch: FetchLike;
  nowMs: () => number;
  deadlines: Deadlines;
}

export const DEFAULT_DEADLINES: Deadlines = { canaryMs: 5_000, signyMs: 8_000 };
