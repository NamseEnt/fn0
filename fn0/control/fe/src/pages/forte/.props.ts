// Auto-generated from src/pages/forte/mod.rs

import { z } from "zod";

export const PropsSchema = z.object({
    rustSnippetHtml: z.string(),
    tsxSnippetHtml: z.string(),
  });

export type Props = z.infer<typeof PropsSchema>;
