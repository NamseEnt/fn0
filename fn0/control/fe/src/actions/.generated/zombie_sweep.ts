// Auto-generated from src/actions/zombie_sweep.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    scannedInstancesCount: z.number(),
    terminatedInstancesCount: z.number(),
    cleanedDnsRecordsCount: z.number(),
    orphanDnsRecordsCleanedCount: z.number(),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function zombieSweep(input: z.infer<typeof InputSchema>) {
  return callAction("zombie_sweep", input, OutputSchema);
}
