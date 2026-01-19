import { useState, useEffect } from "react";
import type { Props } from "./.props";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MarkdownEditor } from "@/components/MarkdownEditor";
import { createPost } from "@/actions/.generated";
import { Layout } from "@/components/Layout";

function buildGithubAuthUrl(oauthNonce: string): string {
  if (typeof window === "undefined") {
    return "#";
  }
  const clientId = import.meta.env.PUBLIC_GITHUB_CLIENT_ID;
  const redirectUri = `${window.location.origin}/api/auth/callback/github`;
  return `https://github.com/login/oauth/authorize?client_id=${clientId}&redirect_uri=${encodeURIComponent(
    redirectUri
  )}&scope=read:user%20user:email&state=${oauthNonce}`;
}

export default function WritePage(props: Props) {
  if (props.t === "NotLoggedIn") {
    return <RedirectToLogin oauthNonce={props.oauthNonce} />;
  }

  return (
    <Layout>
      <WriteForm />
    </Layout>
  );
}

function RedirectToLogin({ oauthNonce }: { oauthNonce: string }) {
  useEffect(() => {
    window.location.href = buildGithubAuthUrl(oauthNonce);
  }, [oauthNonce]);

  return <div>ログイン中...</div>;
}

type PostType = "Normal" | "ShowLs";

function WriteForm() {
  const [title, setTitle] = useState("");
  const [url, setUrl] = useState("");
  const [content, setContent] = useState("");
  const [postType, setPostType] = useState<PostType>("Normal");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!title || !url || !content) {
      return;
    }

    setIsSubmitting(true);

    try {
      const result = await createPost({
        title,
        url,
        content,
        postType: { t: postType },
      });

      if (result.t === "Ok") {
        window.location.href = `/post/${result.id}`;
        return;
      }

      if (result.t === "InternalError") {
        alert("投稿に失敗しました");
        return;
      }

      if (result.t === "NotLoggedIn") {
        alert("ログインが必要です。");
        return;
      }
    } catch (error) {
      console.error("Error creating post:", error);
      alert("投稿に失敗しました");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">新規投稿</h1>
      <form onSubmit={handleSubmit} className="space-y-6">
        <div className="space-y-2">
          <Label htmlFor="title">タイトル</Label>
          <Input
            id="title"
            type="text"
            placeholder="タイトルを入力してください"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
          />
        </div>

        <div className="space-y-2">
          <Label>投稿タイプ</Label>
          <div className="flex gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="postType"
                value="Normal"
                checked={postType === "Normal"}
                onChange={(e) => setPostType(e.target.value as PostType)}
                className="cursor-pointer"
              />
              <span>Normal</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="postType"
                value="ShowLs"
                checked={postType === "ShowLs"}
                onChange={(e) => setPostType(e.target.value as PostType)}
                className="cursor-pointer"
              />
              <span>Show ls</span>
            </label>
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="url">URL</Label>
          <Input
            id="url"
            type="url"
            placeholder="https://example.com"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            required
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="content">内容</Label>
          <MarkdownEditor
            value={content}
            onChange={setContent}
            minHeight={400}
          />
        </div>

        <Button type="submit" className="w-full" disabled={isSubmitting}>
          {isSubmitting ? "投稿中..." : "投稿"}
        </Button>
      </form>
    </div>
  );
}
