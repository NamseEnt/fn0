// Auto-generated from src/actions/presign_quota.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    evaluatedCount: z.number(),
    blockedCount: z.number(),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function presignQuota(input: z.infer<typeof InputSchema>) {
  return callAction("presign_quota", input, OutputSchema);
}
