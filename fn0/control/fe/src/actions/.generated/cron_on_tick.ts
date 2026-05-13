// Auto-generated from src/actions/cron_on_tick.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    scheduledTime: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    firedCount: z.number(),
    scannedProjectsCount: z.number(),
    scannedJobsCount: z.number(),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function cronOnTick(input: z.infer<typeof InputSchema>) {
  return callAction("cron_on_tick", input, OutputSchema);
}
