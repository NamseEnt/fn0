// Auto-generated from src/actions/deploy_status.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    codeVersion: z.number(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Done"),
    activeVersion: z.string(),
    pendingVersion: z.string().optional(),
    pendingCompiled: z.boolean(),
    compiledVersions: z.array(z.string()),
  }),
    z.object({
    t: z.literal("Pending"),
    activeVersion: z.string(),
    pendingVersion: z.string().optional(),
    pendingCompiled: z.boolean(),
    compiledVersions: z.array(z.string()),
  }),
    z.object({
    t: z.literal("NoActiveVersion"),
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

export function deployStatus(input: z.infer<typeof InputSchema>) {
  return callAction("deploy_status", input, OutputSchema);
}
