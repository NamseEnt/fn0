// Auto-generated from src/actions/domain_set.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    domain: z.string(),
    certificatePem: z.string(),
    privateKeyPem: z.string(),
    notAfterEpochSeconds: z.number(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    originHostname: z.string(),
    replacedDomain: z.string().optional(),
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
    t: z.literal("InternalError"),
  })
  ]);

export function domainSet(input: z.infer<typeof InputSchema>) {
  return callAction("domain_set", input, OutputSchema);
}
