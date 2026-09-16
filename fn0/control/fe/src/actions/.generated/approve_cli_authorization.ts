// Auto-generated from src/actions/approve_cli_authorization.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    responseMode: z.string().optional(),
    redirectUri: z.string().optional(),
    codeChallenge: z.string(),
    codeChallengeMethod: z.string(),
    state: z.string().optional(),
    label: z.string(),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    code: z.string(),
    redirectTo: z.string().optional(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("InvalidRequest"),
    message: z.string(),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  })
  ]);

export function approveCliAuthorization(input: z.infer<typeof InputSchema>) {
  return callAction("approve_cli_authorization", input, OutputSchema);
}
