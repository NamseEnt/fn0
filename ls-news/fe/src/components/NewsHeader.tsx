import { Suspense } from "react";
import { Link } from "@/components/ui/link";
import { useMe } from "@/hooks/.generated/useMe";

function NewsHeaderContent() {
  const me = useMe({});

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
                <button
                  type="button"
                  className="text-sm hover:underline cursor-pointer"
                  onClick={async () => {
                    await fetch("/api/auth/signout", { method: "POST" });
                    window.location.reload();
                  }}
                >
                  ログアウト
                </button>
              </>
            ) : (
              <a href={buildGithubAuthUrl(me.oauthNonce)} className="text-sm hover:underline">
                GitHubでログイン
              </a>
            )}
          </div>
        </div>
      </nav>
    </header>
  );
}

function buildGithubAuthUrl(oauthNonce: string): string {
  if (typeof window === "undefined") {
    return "#";
  }
  const clientId = import.meta.env.PUBLIC_GITHUB_CLIENT_ID;
  const redirectUri = `${window.location.origin}/api/auth/callback/github`;
  return `https://github.com/login/oauth/authorize?client_id=${clientId}&redirect_uri=${encodeURIComponent(redirectUri)}&scope=read:user%20user:email&state=${oauthNonce}`;
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
