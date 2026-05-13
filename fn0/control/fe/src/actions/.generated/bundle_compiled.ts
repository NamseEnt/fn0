// Auto-generated from src/actions/bundle_compiled.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    projectId: z.string(),
    codeVersion: z.number(),
    fn0WasmtimeVersion: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function bundleCompiled(input: z.infer<typeof InputSchema>) {
  return callAction("bundle_compiled", input, OutputSchema);
}
