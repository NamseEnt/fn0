// Auto-generated from rs/src/pages/post/[id]/mod.rs

import { z } from "zod";

export const PostSchema = z.object({
    id: z.string(),
    title: z.string(),
    url: z.string(),
    content: z.string(),
    authorId: z.string(),
    likes: z.number(),
    dislikes: z.number(),
    createdAt: z.coerce.date(),
    updatedAt: z.coerce.date(),
  });

export type Post = z.infer<typeof PostSchema>;

export const CommentSchema = z.object({
    id: z.string(),
    content: z.string(),
    postId: z.string(),
    authorId: z.string(),
    parentCommentId: z.string().optional(),
    likes: z.number(),
    dislikes: z.number(),
    createdAt: z.coerce.date(),
    updatedAt: z.coerce.date(),
  });

export type Comment = z.infer<typeof CommentSchema>;

export const UserDocSchema = z.object({
    id: z.string(),
    username: z.string(),
    avatarUrl: z.string(),
    createdAt: z.coerce.date(),
    updatedAt: z.coerce.date(),
  });

export type UserDoc = z.infer<typeof UserDocSchema>;

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    post: PostSchema,
    comments: z.array(CommentSchema),
    users: z.record(z.string(), UserDocSchema),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("DbErr"),
    message: z.string(),
  })
  ]);

export type Props = z.infer<typeof PropsSchema>;
