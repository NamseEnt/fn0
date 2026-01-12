// Auto-generated from rs/src/hooks/me.rs

import { z } from "zod";
import { useForteHook } from "@/lib/forte-react";

const UserSchema = z.object({
    id: z.string(),
    username: z.string(),
    avatarUrl: z.string(),
  });

const InputSchema = z.object({
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("LoggedIn"),
    user: UserSchema,
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
    oauthNonce: z.string(),
  })
  ]);

export function useMe(input: z.infer<typeof InputSchema>) {
  return useForteHook("me", input, OutputSchema);
}
