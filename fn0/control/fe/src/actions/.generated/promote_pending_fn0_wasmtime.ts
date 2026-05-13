// Auto-generated from src/actions/promote_pending_fn0_wasmtime.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    oldActive: z.string(),
    newActive: z.string(),
  }),
    z.object({
    t: z.literal("NoPending"),
  }),
    z.object({
    t: z.literal("NoActiveVersion"),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function promotePendingFn0Wasmtime(input: z.infer<typeof InputSchema>) {
  return callAction("promote_pending_fn0_wasmtime", input, OutputSchema);
}
