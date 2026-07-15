// Auto-generated from src/pages/docs/[page]/mod.rs

import { z } from "zod";

export const NavItemSchema = z.object({
    route: z.string(),
    title: z.string(),
  });

export type NavItem = z.infer<typeof NavItemSchema>;

export const NavSectionSchema = z.object({
    title: z.string(),
    items: z.array(NavItemSchema),
  });

export type NavSection = z.infer<typeof NavSectionSchema>;

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    title: z.string(),
    html: z.string(),
    nav: z.array(NavSectionSchema),
  }),
    z.object({
    t: z.literal("NotFound"),
    nav: z.array(NavSectionSchema),
  })
  ]);

export type Props = z.infer<typeof PropsSchema>;
