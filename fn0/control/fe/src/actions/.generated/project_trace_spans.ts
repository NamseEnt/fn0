// Auto-generated from src/actions/project_trace_spans.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const AttributePairSchema = z.object({
    key: z.string(),
    value: z.string(),
  });

const SpanEventOutSchema = z.object({
    timestamp: z.string(),
    name: z.string(),
    attributes: z.array(AttributePairSchema),
  });

const SpanOutSchema = z.object({
    spanId: z.string(),
    parentSpanId: z.string(),
    name: z.string(),
    kind: z.string(),
    service: z.string(),
    status: z.string(),
    start: z.string(),
    end: z.string(),
    duration: z.string(),
    attributes: z.array(AttributePairSchema),
    events: z.array(SpanEventOutSchema),
  });

const InputSchema = z.object({
    projectId: z.string(),
    traceId: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    spans: z.array(SpanOutSchema),
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

export function projectTraceSpans(input: z.infer<typeof InputSchema>) {
  return callAction("project_trace_spans", input, OutputSchema);
}
