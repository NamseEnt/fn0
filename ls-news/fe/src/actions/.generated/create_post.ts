// Auto-generated from rs/src/actions/create_post.rs

import { z } from "zod";
import { callAction } from "@/lib/forte-react";

const PostTypeSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Normal"),
  }),
    z.object({
    t: z.literal("ShowLs"),
  })
  ]);

const InputSchema = z.object({
    title: z.string(),
    url: z.string(),
    content: z.string(),
    postType: PostTypeSchema,
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    id: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  })
  ]);

export function createPost(input: z.infer<typeof InputSchema>) {
  return callAction("create_post", input, OutputSchema);
}
