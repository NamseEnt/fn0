// Auto-generated from src/actions/cloudflare_broker_authorize.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    operation: z.string(),
    accountId: z.string(),
    projectId: z.string().optional(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Authorized"),
    githubId: z.number(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("InvalidRequest"),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function cloudflareBrokerAuthorize(input: z.infer<typeof InputSchema>) {
  return callAction("cloudflare_broker_authorize", input, OutputSchema);
}
