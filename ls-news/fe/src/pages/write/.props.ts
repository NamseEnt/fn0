// Auto-generated from rs/src/pages/write.rs

import { z } from "zod";

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
    oauthNonce: z.string(),
  })
  ]);

export type Props = z.infer<typeof PropsSchema>;
