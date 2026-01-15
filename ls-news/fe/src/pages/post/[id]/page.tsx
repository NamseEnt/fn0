import type { Props, Post, Comment, User } from "./.props";
import { Layout } from "@/components/Layout";
import { Link } from "@/components/ui/link";
import { IconButton } from "@/components/ui/icon-button";
import { ThumbsUp, ThumbsDown } from "lucide-react";
import Markdown from "react-markdown";

export default function PostPage(props: Props) {
  if (props.t === "NotFound") {
    return (
      <Layout>
        <div className="text-center py-20">
          <h1 className="text-2xl font-bold mb-4">Post not found</h1>
          <Link href="/">Go back to home</Link>
        </div>
      </Layout>
    );
  }

  if (props.t === "DbErr") {
    return (
      <Layout>
        <div className="text-center text-red-500">Db Error: {props.message}</div>
      </Layout>
    );
  }

  const { post, comments, users } = props;
  const author = users[post.authorId];

  return (
    <Layout>
      <PostHeader
        post={post}
        author={author}
        commentsCount={comments.length}
      />

      <div className="mb-8">
        <PostContent content={post.content} />
        <PostInteractions likes={post.likes} />
      </div>

      <CommentList comments={comments} users={users} />
    </Layout>
  );
}

function PostHeader({
  post,
  author,
  commentsCount,
}: {
  post: Post;
  author: User | undefined;
  commentsCount: number;
}) {
  const timeAgo = getRelativeTimeText(post.createdAt);

  return (
    <div className="mb-8">
      <div className="mb-2 flex items-start justify-between gap-4">
        <div className="flex-1">
          <h1 className="text-3xl font-bold">{post.title}</h1>
          <Link href={post.url} size="sm" muted popup>
            {post.url}
          </Link>
        </div>
      </div>

      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        {author && (
          <>
            <img
              src={author.avatarUrl}
              alt={author.username}
              className="w-6 h-6 rounded-full"
            />
            <Link href={`https://github.com/${author.username}`} popup>
              {author.username}
            </Link>
            {" | "}
          </>
        )}
        <span>{timeAgo}</span>
        {" | "}
        <span>{commentsCount} comments</span>
      </div>
    </div>
  );
}

function PostContent({ content }: { content: string }) {
  return (
    <div className="prose prose-lg prose-slate max-w-none mb-4 p-6 bg-muted/30 rounded-lg leading-relaxed">
      <Markdown>{content}</Markdown>
    </div>
  );
}

function PostInteractions({ likes }: { likes: number }) {
  return (
    <div className="flex items-center gap-4 px-4">
      <IconButton aria-label="Like" className="flex items-center gap-2 px-2">
        <ThumbsUp size={20} />
        <span className="text-sm font-medium">{likes}</span>
      </IconButton>

      <IconButton aria-label="Dislike">
        <ThumbsDown size={20} />
      </IconButton>
    </div>
  );
}

function CommentList({
  comments,
  users,
}: {
  comments: Comment[];
  users: Record<string, User>;
}) {
  const commentsMap = new Map<string, Comment>();
  comments.forEach((comment) => {
    commentsMap.set(comment.id, comment);
  });

  function calculateIndent(comment: Comment): number {
    let indent = 0;
    let currentComment: Comment | undefined = comment;
    while (currentComment?.parentCommentId) {
      currentComment = commentsMap.get(currentComment.parentCommentId);
      indent++;
    }
    return indent;
  }

  return (
    <div>
      {comments.map((comment) => (
        <CommentItem
          key={comment.id}
          comment={comment}
          author={users[comment.authorId]}
          indent={calculateIndent(comment)}
        />
      ))}
    </div>
  );
}

function CommentItem({
  comment,
  author,
  indent,
}: {
  comment: Comment;
  author: User | undefined;
  indent: number;
}) {
  const timeAgo = getRelativeTimeText(comment.createdAt);

  return (
    <div
      id={`comment-${comment.id}`}
      className="border-l-2 border-muted pl-4 py-4 mb-2"
      style={{ marginLeft: `${indent * 24}px` }}
    >
      <div className="flex items-center gap-2 text-sm text-muted-foreground mb-2">
        {author && (
          <>
            <img
              src={author.avatarUrl}
              alt={author.username}
              className="w-6 h-6 rounded-full"
            />
            <Link href={`https://github.com/${author.username}`} popup>
              {author.username}
            </Link>
          </>
        )}
        <span>{timeAgo}</span>
      </div>

      <div className="prose prose-base max-w-none leading-relaxed mb-3">
        <Markdown>{comment.content}</Markdown>
      </div>

      <div className="flex items-center gap-4">
        <IconButton aria-label="Like" className="flex items-center gap-2 px-2">
          <ThumbsUp size={16} />
          <span className="text-sm">{comment.likes}</span>
        </IconButton>

        <IconButton aria-label="Dislike">
          <ThumbsDown size={16} />
        </IconButton>
      </div>
    </div>
  );
}

function getRelativeTimeText(date: Date): string {
  const formatter = new Intl.RelativeTimeFormat("en");
  const ranges: Partial<Record<Intl.RelativeTimeFormatUnit, number>> = {
    years: 3600 * 24 * 365,
    months: 3600 * 24 * 30,
    weeks: 3600 * 24 * 7,
    days: 3600 * 24,
    hours: 3600,
    minutes: 60,
    seconds: 1,
  };
  const secondsElapsed = (date.getTime() - Date.now()) / 1000;
  for (const [key, value] of Object.entries(ranges)) {
    if (value < Math.abs(secondsElapsed)) {
      const delta = secondsElapsed / value;
      return formatter.format(
        Math.round(delta),
        key as Intl.RelativeTimeFormatUnit
      );
    }
  }
  return "just now";
}
