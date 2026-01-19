// Auto-generated from rs/src/pages/index/mod.rs

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

export const UserSchema = z.object({
    id: z.string(),
    username: z.string(),
    avatarUrl: z.string(),
  });

export type User = z.infer<typeof UserSchema>;

export const RowSchema = z.object({
    post: PostDocSchema,
    deletedAt: z.coerce.date().optional(),
    author: UserSchema,
  });

export type Row = z.infer<typeof RowSchema>;

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    rows: z.array(RowSchema),
  }),
    z.object({
    t: z.literal("DbErr"),
    message: z.string(),
  })
  ]);

export type Props = z.infer<typeof PropsSchema>;
