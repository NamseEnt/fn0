// Auto-generated from src/actions/usage_metering.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    projectsCount: z.number(),
    operationsDocsCount: z.number(),
    snapshotDocsCount: z.number(),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function usageMetering(input: z.infer<typeof InputSchema>) {
  return callAction("usage_metering", input, OutputSchema);
}
