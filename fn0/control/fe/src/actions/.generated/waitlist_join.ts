// Auto-generated from src/actions/waitlist_join.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    email: z.string(),
    tier: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  }),
    z.object({
    t: z.literal("InvalidEmail"),
  }),
    z.object({
    t: z.literal("InvalidTier"),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function waitlistJoin(input: z.infer<typeof InputSchema>) {
  return callAction("waitlist_join", input, OutputSchema);
}
