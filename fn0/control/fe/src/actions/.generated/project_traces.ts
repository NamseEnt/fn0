// Auto-generated from src/actions/project_traces.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const TraceSummaryOutSchema = z.object({
    traceId: z.string(),
    rootService: z.string(),
    rootName: z.string(),
    start: z.string(),
    end: z.string(),
    duration: z.string(),
    spanCount: z.number(),
  });

const InputSchema = z.object({
    projectId: z.string(),
    start: z.string(),
    end: z.string().optional(),
    status: z.string().optional(),
    minDuration: z.string().optional(),
    nameContains: z.string().optional(),
    nameRegex: z.string().optional(),
    limit: z.number(),
    beforeStart: z.string().optional(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    traces: z.array(TraceSummaryOutSchema),
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

export function projectTraces(input: z.infer<typeof InputSchema>) {
  return callAction("project_traces", input, OutputSchema);
}
