// Auto-generated from src/actions/bundle_gc.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    deletedVersionsCount: z.number(),
    deletedOrphansCount: z.number(),
    deletedStaticPrefixesCount: z.number(),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function bundleGc(input: z.infer<typeof InputSchema>) {
  return callAction("bundle_gc", input, OutputSchema);
}
