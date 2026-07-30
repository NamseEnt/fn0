// Auto-generated from src/actions/project_asset_origin.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    codeVersion: z.number(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    staticBaseUrl: z.string(),
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

export function projectAssetOrigin(input: z.infer<typeof InputSchema>) {
  return callAction("project_asset_origin", input, OutputSchema);
}
