// Auto-generated from src/actions/new_project.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    name: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    projectId: z.string(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("InvalidName"),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function newProject(input: z.infer<typeof InputSchema>) {
  return callAction("new_project", input, OutputSchema);
}
