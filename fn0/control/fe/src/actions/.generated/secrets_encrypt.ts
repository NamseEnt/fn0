// Auto-generated from src/actions/secrets_encrypt.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    encryptedDek: z.string(),
    value: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    ciphertext: z.string(),
  }),
    z.object({
    t: z.literal("Unauthorized"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function secretsEncrypt(input: z.infer<typeof InputSchema>) {
  return callAction("secrets_encrypt", input, OutputSchema);
}
