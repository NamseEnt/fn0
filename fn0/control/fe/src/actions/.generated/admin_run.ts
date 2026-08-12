// Auto-generated from src/actions/admin_run.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    task: z.string(),
    input: z.json(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    status: z.number(),
    contentType: z.string().optional(),
    body: z.string(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("Forbidden"),
  }),
    z.object({
    t: z.literal("NotDeployed"),
  }),
    z.object({
    t: z.literal("UpstreamError"),
    status: z.number(),
    contentType: z.string().optional(),
    body: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
    reason: z.string(),
  })
  ]);

export function adminRun(input: z.infer<typeof InputSchema>) {
  return callAction("admin_run", input, OutputSchema);
}
