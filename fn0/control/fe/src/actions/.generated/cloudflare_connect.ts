// Auto-generated from src/actions/cloudflare_connect.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    accountId: z.string(),
    zoneId: z.string(),
    zoneName: z.string(),
    staticHostname: z.string(),
    objectBucket: z.string(),
    assetBucket: z.string(),
    pageBucket: z.string(),
    dataplaneAccessKeyId: z.string(),
    dataplaneSecret: z.string(),
    purgeToken: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  }),
    z.object({
    t: z.literal("CredentialRejected"),
    reason: z.string(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("InternalError"),
    reason: z.string(),
  })
  ]);

export function cloudflareConnect(input: z.infer<typeof InputSchema>) {
  return callAction("cloudflare_connect", input, OutputSchema);
}
