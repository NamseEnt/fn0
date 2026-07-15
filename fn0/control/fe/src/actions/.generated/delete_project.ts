// Auto-generated from src/actions/delete_project.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
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

export function deleteProject(input: z.infer<typeof InputSchema>) {
  return callAction("delete_project", input, OutputSchema);
}
