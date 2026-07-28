// Auto-generated from src/actions/deploy/mod.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const FileEntrySchema = z.object({
    path: z.string(),
    size: z.number(),
  });

const CronJobSchema = z.object({
    function: z.string(),
    everyMinutes: z.number(),
  });

const StaticUploadSchema = z.object({
    path: z.string(),
    presignedUrl: z.string(),
    cacheControl: z.string(),
  });

const InputSchema = z.object({
    projectId: z.string(),
    codeVersion: z.number(),
    bundleSize: z.number(),
    files: z.array(FileEntrySchema),
    supportsStaticAssetCacheControl: z.boolean(),
    jobs: z.array(CronJobSchema),
    cronUpdatedAt: z.coerce.date(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    presignedPutUrl: z.string(),
    objectKey: z.string(),
    staticUploads: z.array(StaticUploadSchema),
  }),
    z.object({
    t: z.literal("QuotaExceeded"),
    reason: z.string(),
  }),
    z.object({
    t: z.literal("BadCodeVersion"),
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
  })
  ]);

export function deploy(input: z.infer<typeof InputSchema>) {
  return callAction("deploy", input, OutputSchema);
}
