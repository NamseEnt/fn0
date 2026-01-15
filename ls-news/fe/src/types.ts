export const PostType = {
  Normal: "normal",
  ShowLs: "showls",
} as const;

export type PostType = (typeof PostType)[keyof typeof PostType];
