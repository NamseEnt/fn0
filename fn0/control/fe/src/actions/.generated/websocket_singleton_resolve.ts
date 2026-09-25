// Auto-generated from src/actions/websocket_singleton_resolve.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    singletonId: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Connected"),
    connectionId: z.string(),
    leaseExpiresAtMillis: z.number(),
    resolvedAtMillis: z.number(),
  }),
    z.object({
    t: z.literal("Unavailable"),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
  })
  ]);

export function websocketSingletonResolve(input: z.infer<typeof InputSchema>) {
  return callAction("websocket_singleton_resolve", input, OutputSchema);
}
