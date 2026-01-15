import { Suspense, useState, useEffect } from "react";
import { Link } from "@/components/ui/link";
import { useMe } from "@/hooks/.generated/useMe";
import { paths } from "@/paths.generated";

function NewsHeaderContent() {
  const me = useMe({});
  const [authUrl, setAuthUrl] = useState<string | undefined>(undefined);

  useEffect(() => {
    if (me.t === "NotLoggedIn") {
      const clientId = import.meta.env.PUBLIC_GITHUB_CLIENT_ID;
      const redirectUri = `${window.location.origin}/api/auth/callback/github`;
      const url = `https://github.com/login/oauth/authorize?client_id=${clientId}&redirect_uri=${encodeURIComponent(redirectUri)}&scope=read:user%20user:email&state=${me.oauthNonce}`;
      setAuthUrl(url);
    }
  }, [me]);

  return (
    <header className="border-b bg-background">
      <nav className="container mx-auto px-4 py-3">
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-4 flex-wrap">
            <a href="/" className="text-lg font-mono font-bold">
              <span className="text-emerald-500">&gt;</span>
              <span className="text-sky-500"> ls</span>
              <span className="text-amber-500"> news</span>
              <span className="text-purple-500">/*</span>
            </a>
            <div className="flex items-center gap-3 text-sm">
              <Link href="/best">Best</Link>
              <Link href="/showls">Show ls</Link>
              <Link href="/write">新規投稿</Link>
            </div>
          </div>

          <div className="flex items-center gap-4">
            {me.t === "LoggedIn" ? (
              <>
                <div className="flex items-center gap-2">
                  {me.user.avatarUrl && (
                    <img
                      src={me.user.avatarUrl}
                      alt={me.user.username}
                      className="w-6 h-6 rounded-full"
                    />
                  )}
                  <span className="text-sm">{me.user.username}</span>
                </div>
                <a
                  href={paths["/signout"]()}
                  className="text-sm hover:underline"
                >
                  ログアウト
                </a>
              </>
            ) : authUrl ? (
              <a href={authUrl} className="text-sm hover:underline">
                GitHubでログイン
              </a>
            ) : null}
          </div>
        </div>
      </nav>
    </header>
  );
}

function NewsHeaderFallback() {
  return (
    <header className="border-b bg-background">
      <nav className="container mx-auto px-4 py-3">
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-4 flex-wrap">
            <a href="/" className="text-lg font-mono font-bold">
              <span className="text-emerald-500">&gt;</span>
              <span className="text-sky-500"> ls</span>
              <span className="text-amber-500"> news</span>
              <span className="text-purple-500">/*</span>
            </a>
            <div className="flex items-center gap-3 text-sm">
              <Link href="/best">Best</Link>
              <Link href="/showls">Show ls</Link>
              <Link href="/write">新規投稿</Link>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm text-muted-foreground">Loading...</span>
          </div>
        </div>
      </nav>
    </header>
  );
}

export function NewsHeader() {
  return (
    <Suspense fallback={<NewsHeaderFallback />}>
      <NewsHeaderContent />
    </Suspense>
  );
}
