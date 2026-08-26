// Auto-generated from src/pages/projects/mod.rs

import { z } from "zod";

export const ProjectItemSchema = z.object({
    projectId: z.string(),
    name: z.string(),
  });

export type ProjectItem = z.infer<typeof ProjectItemSchema>;

export const PropsSchema = z.object({
    githubLogin: z.string(),
    projects: z.array(ProjectItemSchema),
  });

export type Props = z.infer<typeof PropsSchema>;
