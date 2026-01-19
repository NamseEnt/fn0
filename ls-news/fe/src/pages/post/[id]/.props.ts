// Auto-generated from rs/src/pages/post/[id]/mod.rs

import { z } from "zod";

export const PostDocSchema = z.object({
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

export type PostDoc = z.infer<typeof PostDocSchema>;

export const CommentDocSchema = z.object({
    postId: z.string(),
    id: z.string(),
    content: z.string(),
    authorId: z.string(),
    parentCommentId: z.string().optional(),
    likes: z.number(),
    dislikes: z.number(),
    createdAt: z.coerce.date(),
    updatedAt: z.coerce.date(),
  });

export type CommentDoc = z.infer<typeof CommentDocSchema>;

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
    post: PostDocSchema,
    comments: z.array(CommentDocSchema),
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
