// Auto-generated from src/actions/secrets_init.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    encryptedDek: z.string(),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function secretsInit(input: z.infer<typeof InputSchema>) {
  return callAction("secrets_init", input, OutputSchema);
}
