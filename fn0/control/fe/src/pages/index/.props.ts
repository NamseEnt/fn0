// Auto-generated from src/pages/index/mod.rs

import { z } from "zod";

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    message: z.string(),
  })
  ]);

export type Props = z.infer<typeof PropsSchema>;
