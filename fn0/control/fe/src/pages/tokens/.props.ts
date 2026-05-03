// Auto-generated from src/pages/tokens/mod.rs

import { z } from "zod";

export const PropsSchema = z.object({
    githubId: z.number(),
    githubLogin: z.string(),
  });

export type Props = z.infer<typeof PropsSchema>;
