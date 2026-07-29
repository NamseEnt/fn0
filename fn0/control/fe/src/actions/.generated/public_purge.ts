// Auto-generated from src/actions/public_purge.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    keys: z.array(z.string()),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    urls: z.array(z.string()),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("Forbidden"),
  }),
    z.object({
    t: z.literal("TooManyKeys"),
    max: z.number(),
  }),
    z.object({
    t: z.literal("InternalError"),
    reason: z.string(),
  })
  ]);

export function publicPurge(input: z.infer<typeof InputSchema>) {
  return callAction("public_purge", input, OutputSchema);
}
