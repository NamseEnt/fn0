import { renderToPipeableStream } from "react-dom/server";
import { routes } from "./routes.generated";
import { PassThrough } from "stream";
import { serializeHookCache, clearHookCache, setSSRBaseUrl } from "./lib/forte-react";

function matchRoute(
  pathname: string
): { route: (typeof routes)[0]; params: Record<string, string> } | null {
  for (const route of routes) {
    const routeParts = route.path.split("/");
    const pathParts = pathname.split("/");

    if (routeParts.length !== pathParts.length) continue;

    const params: Record<string, string> = {};
    let match = true;

    for (let i = 0; i < routeParts.length; i++) {
      if (routeParts[i].startsWith(":")) {
        params[routeParts[i].slice(1)] = pathParts[i];
      } else if (routeParts[i] !== pathParts[i]) {
        match = false;
        break;
      }
    }

    if (match) {
      return { route, params };
    }
  }
  return null;
}

function escapeJsonForScript(json: string): string {
  return json.replace(/</g, "\\u003c").replace(/>/g, "\\u003e");
}

const isDev = (import.meta as any).env?.DEV ?? false;

function streamToString(stream: PassThrough): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    stream.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    stream.on("end", () => resolve(Buffer.concat(chunks).toString("utf-8")));
    stream.on("error", reject);
  });
}

export async function render(url: string, rawProps: any, forteBaseUrl: string): Promise<string> {
  const urlObj = new URL(url, "http://localhost");
  const matched = matchRoute(urlObj.pathname);

  if (!matched) {
    return "Not Found";
  }

  // Set base URL for SSR hook requests
  setSSRBaseUrl(forteBaseUrl);

  // Clear hook cache for fresh request
  clearHookCache();

  const [pageModule, schemaModule] = await Promise.all([
    matched.route.component(),
    matched.route.schema(),
  ]);
  const props = schemaModule.PropsSchema.parse(rawProps);
  const allProps = { ...props, params: matched.params };
  const propsJson = escapeJsonForScript(JSON.stringify(allProps));

  const viteScripts = `<script type="module" src="/@vite/client"></script>`;
  const clientScript = `/src/client.tsx`;

  const html = await new Promise<string>((resolve, reject) => {
    const passThrough = new PassThrough();
    const { pipe } = renderToPipeableStream(pageModule.default(allProps), {
      onAllReady() {
        pipe(passThrough);
        streamToString(passThrough).then(resolve).catch(reject);
      },
      onError(error) {
        reject(error);
      },
    });
  });

  // Serialize hook cache for client hydration
  const hookCacheJson = serializeHookCache();

  return `<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>ls-news</title>
    <link rel="stylesheet" href="/src/styles/globals.css" />
    ${viteScripts}
</head>
<body>
    <div id="root">${html}</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};</script>
    <script>window.__FORTE_HOOK_CACHE__ = ${hookCacheJson};</script>
    <script type="module" src="${clientScript}"></script>
</body>
</html>`;
}

(globalThis as any).handler = async function handler(
  request: Request
): Promise<Response> {
  const rawProps = await request.json();
  const url = new URL(request.url);

  const matched = matchRoute(url.pathname);
  if (matched) {
    // Clear hook cache for fresh request
    clearHookCache();

    const [pageModule, schemaModule] = await Promise.all([
      matched.route.component(),
      matched.route.schema(),
    ]);
    const props = schemaModule.PropsSchema.parse(rawProps);
    const allProps = { ...props, params: matched.params };
    const propsJson = escapeJsonForScript(JSON.stringify(allProps));

    const viteScripts = isDev
      ? `<script type="module" src="/@vite/client"></script>`
      : "";
    const clientScript = isDev ? `/src/client.tsx` : `/public/client.js`;

    const cssLink = isDev
      ? `<link rel="stylesheet" href="/src/styles/globals.css" />`
      : `<link rel="stylesheet" href="/public/globals.css" />`;

    const html = await new Promise<string>((resolve, reject) => {
      const passThrough = new PassThrough();
      const { pipe } = renderToPipeableStream(pageModule.default(allProps), {
        onAllReady() {
          pipe(passThrough);
          streamToString(passThrough).then(resolve).catch(reject);
        },
        onError(error) {
          reject(error);
        },
      });
    });

    // Serialize hook cache for client hydration
    const hookCacheJson = serializeHookCache();

    return new Response(
      `<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>ls-news</title>
    ${cssLink}
    ${viteScripts}
</head>
<body>
    <div id="root">${html}</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};</script>
    <script>window.__FORTE_HOOK_CACHE__ = ${hookCacheJson};</script>
    <script type="module" src="${clientScript}"></script>
</body>
</html>`,
      {
        headers: { "Content-Type": "text/html" },
      }
    );
  }

  return new Response("Not Found", { status: 404 });
};
