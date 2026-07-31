// Auto-generated from src/actions/cloudflare_status.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("NotConnected"),
  }),
    z.object({
    t: z.literal("Connected"),
    accountId: z.string(),
    zoneName: z.string(),
    frontendAssetHostname: z.string(),
    publicObjectStorageHostname: z.string(),
    frontendAssetBucket: z.string(),
    publicObjectStorageBucket: z.string(),
    privateObjectStorageBucket: z.string(),
    renderedHtmlCacheBucket: z.string(),
    healthy: z.boolean(),
    problem: z.string().optional(),
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

export function cloudflareStatus(input: z.infer<typeof InputSchema>) {
  return callAction("cloudflare_status", input, OutputSchema);
}
