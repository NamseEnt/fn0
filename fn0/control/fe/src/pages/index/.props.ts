// Auto-generated from src/pages/index/mod.rs

import { z } from "zod";

export const PropsSchema = z.object({
    forteSnippetHtml: z.string(),
  });

export type Props = z.infer<typeof PropsSchema>;
