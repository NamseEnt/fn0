// Auto-generated from src/actions/domain_remove.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    removedDomain: z.string(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("NoDomain"),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function domainRemove(input: z.infer<typeof InputSchema>) {
  return callAction("domain_remove", input, OutputSchema);
}
