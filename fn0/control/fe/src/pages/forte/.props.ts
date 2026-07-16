// Auto-generated from src/pages/forte/mod.rs

import { z } from "zod";

export const PropsSchema = z.object({
    rustSnippetHtml: z.string(),
    propsJsonHtml: z.string(),
    tsxSnippetHtml: z.string(),
    docDbSnippetHtml: z.string(),
    objectStorageSnippetHtml: z.string(),
  });

export type Props = z.infer<typeof PropsSchema>;
