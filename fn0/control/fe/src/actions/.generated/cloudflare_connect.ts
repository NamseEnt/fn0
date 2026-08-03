// Auto-generated from src/actions/cloudflare_connect.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    accountId: z.string(),
    zoneId: z.string(),
    zoneName: z.string(),
    frontendAssetHostname: z.string(),
    publicObjectStorageHostname: z.string(),
    privateObjectStorageBucket: z.string(),
    publicObjectStorageBucket: z.string(),
    frontendAssetBucket: z.string(),
    workerAccessKeyId: z.string(),
    workerSecret: z.string(),
    frontendAssetAccessKeyId: z.string(),
    frontendAssetSecret: z.string(),
    purgeToken: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  }),
    z.object({
    t: z.literal("AlreadyConnected"),
    accountId: z.string(),
    zoneName: z.string(),
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
