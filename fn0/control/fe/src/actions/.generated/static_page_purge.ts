// Auto-generated from src/actions/static_page_purge.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    paths: z.array(z.string()),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    paths: z.array(z.string()),
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
    t: z.literal("NoDomain"),
  }),
    z.object({
    t: z.literal("TooManyPaths"),
    max: z.number(),
  }),
    z.object({
    t: z.literal("UnusablePath"),
    path: z.string(),
    reason: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
    reason: z.string(),
  })
  ]);

export function staticPagePurge(input: z.infer<typeof InputSchema>) {
  return callAction("static_page_purge", input, OutputSchema);
}
