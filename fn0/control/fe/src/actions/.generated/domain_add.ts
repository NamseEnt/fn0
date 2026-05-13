// Auto-generated from src/actions/domain_add.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    domain: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("InvalidDomain"),
    message: z.string(),
  }),
    z.object({
    t: z.literal("DomainTaken"),
    existingProjectId: z.string(),
  }),
    z.object({
    t: z.literal("AlreadyHasDomain"),
    currentDomain: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function domainAdd(input: z.infer<typeof InputSchema>) {
  return callAction("domain_add", input, OutputSchema);
}
