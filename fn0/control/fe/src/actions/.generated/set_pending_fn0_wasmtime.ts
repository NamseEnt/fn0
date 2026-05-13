// Auto-generated from src/actions/set_pending_fn0_wasmtime.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    version: z.string(),
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

export function setPendingFn0Wasmtime(input: z.infer<typeof InputSchema>) {
  return callAction("set_pending_fn0_wasmtime", input, OutputSchema);
}
