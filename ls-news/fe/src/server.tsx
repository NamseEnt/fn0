import { renderToReadableStream } from "react-dom/server";
import { routes } from "./routes.generated";
import { serializeHookCache, clearHookCache } from "./lib/forte-react";

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

const isDev = import.meta.env.DEV;

async function streamToString(stream: ReadableStream<Uint8Array>): Promise<string> {
  const reader = stream.getReader();
  const chunks: Uint8Array[] = [];

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
  }

  const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }

  return new TextDecoder().decode(result);
}

export async function render(url: string, rawProps: any): Promise<string> {
  const urlObj = new URL(url, "http://localhost");
  const matched = matchRoute(urlObj.pathname);

  if (!matched) {
    return "Not Found";
  }

  // Clear hook cache for fresh request
  clearHookCache();

  const [pageModule, schemaModule] = await Promise.all([
    matched.route.component(),
    matched.route.schema(),
  ]);
  const props = schemaModule.PropsSchema.parse(rawProps);
  const allProps = { ...props, params: matched.params };
  const propsJson = escapeJsonForScript(JSON.stringify(allProps));

  const hmrScript = `<script type="module" src="/__hmr-client.js"></script>`;
  const reactRefreshScript = `<script type="module" src="/__react-refresh.js"></script>`;
  const clientScript = `/src/client.tsx`;

  const stream = await renderToReadableStream(pageModule.default(allProps));
  const html = await streamToString(stream);

  // Serialize hook cache for client hydration
  const hookCacheJson = serializeHookCache();

  return `<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>ls-news</title>
    <link rel="stylesheet" href="/src/styles/globals.css" />
    ${hmrScript}
    ${reactRefreshScript}
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
  const bodyText = await request.text();
  let rawProps;
  try {
    rawProps = JSON.parse(bodyText);
  } catch (e) {
    console.error(`[SSR] JSON parse error. URL: ${request.url}, Body: "${bodyText.slice(0, 500)}"`);
    throw e;
  }
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

    const hmrScript = isDev
      ? `<script type="module" src="/__hmr-client.js"></script>`
      : "";
    const reactRefreshScript = isDev
      ? `<script type="module" src="/__react-refresh.js"></script>`
      : "";
    const clientScript = isDev ? `/src/client.tsx` : `/public/client.js`;

    const cssLink = isDev
      ? `<link rel="stylesheet" href="/src/styles/globals.css" />`
      : `<link rel="stylesheet" href="/public/globals.css" />`;

    const stream = await renderToReadableStream(pageModule.default(allProps));
    const html = await streamToString(stream);

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
    ${hmrScript}
    ${reactRefreshScript}
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
