// Auto-generated from src/actions/project_logs.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const AttributePairSchema = z.object({
    key: z.string(),
    value: z.string(),
  });

const HistogramBucketOutSchema = z.object({
    bucketStart: z.string(),
    bucketEnd: z.string(),
    count: z.number(),
  });

const LogRowOutSchema = z.object({
    timestamp: z.string(),
    line: z.string(),
    attributes: z.array(AttributePairSchema),
  });

const InputSchema = z.object({
    projectId: z.string(),
    start: z.string(),
    end: z.string().optional(),
    stream: z.string().optional(),
    contains: z.string().optional(),
    regex: z.string().optional(),
    limit: z.number(),
    before: z.string().optional(),
    includeHistogram: z.boolean(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    rows: z.array(LogRowOutSchema),
    histogram: z.array(HistogramBucketOutSchema).optional(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function projectLogs(input: z.infer<typeof InputSchema>) {
  return callAction("project_logs", input, OutputSchema);
}
