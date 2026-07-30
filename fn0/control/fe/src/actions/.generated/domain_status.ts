// Auto-generated from src/actions/domain_status.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const CloudflareStatusSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Active"),
  }),
    z.object({
    t: z.literal("Pending"),
  }),
    z.object({
    t: z.literal("Missing"),
  }),
    z.object({
    t: z.literal("Other"),
    value: z.string(),
  })
  ]);

const InputSchema = z.object({
    projectId: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("NotConfigured"),
  }),
    z.object({
    t: z.literal("Configured"),
    domain: z.string(),
    cloudflareStatus: CloudflareStatusSchema,
  }),
    z.object({
    t: z.literal("SelfHosted"),
    domain: z.string(),
    originCertificateReady: z.boolean(),
    originCertificateExpiresEpochSeconds: z.number().optional(),
    originIp: z.string(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function domainStatus(input: z.infer<typeof InputSchema>) {
  return callAction("domain_status", input, OutputSchema);
}
