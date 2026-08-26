// Auto-generated from src/actions/websocket_singleton_status.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const StatusSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Heartbeat"),
  }),
    z.object({
    t: z.literal("Disconnected"),
  })
  ]);

const InputSchema = z.object({
    projectId: z.string(),
    singletonId: z.string(),
    claimToken: z.string(),
    connectionId: z.string(),
    status: StatusSchema,
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  }),
    z.object({
    t: z.literal("Ignored"),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
  })
  ]);

export function websocketSingletonStatus(input: z.infer<typeof InputSchema>) {
  return callAction("websocket_singleton_status", input, OutputSchema);
}
