// Auto-generated from src/actions/egress_grant.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    requestedBytes: z.number(),
    minimumBytes: z.number(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Granted"),
    month: z.string(),
    grantedBytes: z.number(),
  }),
    z.object({
    t: z.literal("QuotaExhausted"),
    month: z.string(),
  }),
    z.object({
    t: z.literal("QuotaNotConfigured"),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
  })
  ]);

export function egressGrant(input: z.infer<typeof InputSchema>) {
  return callAction("egress_grant", input, OutputSchema);
}
