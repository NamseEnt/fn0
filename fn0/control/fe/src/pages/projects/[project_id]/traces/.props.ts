// Auto-generated from src/pages/projects/[project_id]/traces/mod.rs

import { z } from "zod";

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    projectId: z.string(),
    name: z.string(),
  }),
    z.object({
    t: z.literal("NotFound"),
  })
  ]);

export type Props = z.infer<typeof PropsSchema>;
