import { renderToReadableStream } from "react-dom/server.browser";
import { routes } from "./routes.generated";
import {
  serializeHookCache,
  clearHookCache,
  getCollectedCookies,
  clearCollectedCookies,
  setRequestCookie,
} from "./lib/forte-react";

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


export async function renderStream(
  url: string,
  rawProps: any,
  cookie?: string | null
): Promise<{ stream: ReadableStream<Uint8Array>; cookies: string[] }> {
  const urlObj = new URL(url, "http://localhost");
  const matched = matchRoute(urlObj.pathname);

  const encoder = new TextEncoder();

  if (!matched) {
    return {
      stream: new ReadableStream({
        start(controller) {
          controller.enqueue(encoder.encode("Not Found"));
          controller.close();
        },
      }),
      cookies: [],
    };
  }

  clearHookCache();
  clearCollectedCookies();
  setRequestCookie(cookie ?? null);

  const [pageModule, schemaModule] = await Promise.all([
    matched.route.component(),
    matched.route.schema(),
  ]);
  const props = schemaModule.PropsSchema.parse(rawProps);
  const allProps = { ...props, params: matched.params };
  const propsJson = escapeJsonForScript(JSON.stringify(allProps));

  const reactStream = await renderToReadableStream(pageModule.default(allProps));

  await reactStream.allReady;

  const hookCacheJson = serializeHookCache();
  const cookies = getCollectedCookies();

  const headHtml = `<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>ls-news</title>
    <link rel="stylesheet" href="/src/styles/globals.css?direct" />
    <script type="module" src="/@vite/client"></script>
    <script type="module">
      import RefreshRuntime from "/@react-refresh";
      RefreshRuntime.injectIntoGlobalHook(window);
      window.$RefreshReg$ = () => {};
      window.$RefreshSig$ = () => (type) => type;
      window.__vite_plugin_react_preamble_installed__ = true;
    </script>
</head>
<body>
    <div id="root">`;

  const tailHtml = `</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};</script>
    <script>window.__FORTE_HOOK_CACHE__ = ${hookCacheJson};</script>
    <script type="module" src="/src/client.tsx"></script>
</body>
</html>`;

  const { readable, writable } = new TransformStream<Uint8Array, Uint8Array>();

  (async () => {
    const writer = writable.getWriter();

    await writer.write(encoder.encode(headHtml));

    const reader = reactStream.getReader();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      await writer.write(value);
    }

    await writer.write(encoder.encode(tailHtml));
    await writer.close();
  })();

  return { stream: readable, cookies };
}

(globalThis as any).handler = async function handler(
  request: Request
): Promise<Response> {
  const bodyText = await request.text();
  let rawProps;
  try {
    rawProps = JSON.parse(bodyText);
  } catch (e) {
    console.error(
      `[SSR] JSON parse error. URL: ${request.url}, Body: "${bodyText.slice(
        0,
        500
      )}"`
    );
    throw e;
  }
  const url = new URL(request.url);

  const matched = matchRoute(url.pathname);
  if (matched) {
    clearHookCache();
    clearCollectedCookies();

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

    const reactStream = await renderToReadableStream(pageModule.default(allProps));

    await reactStream.allReady;

    const hookCacheJson = serializeHookCache();
    const cookies = getCollectedCookies();

    const headers = new Headers({ "Content-Type": "text/html" });
    for (const cookie of cookies) {
      headers.append("Set-Cookie", cookie);
    }

    const encoder = new TextEncoder();

    const headHtml = `<!DOCTYPE html>
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
    <div id="root">`;

    const tailHtml = `</div>
    <script>window.__FORTE_PROPS__ = ${propsJson};</script>
    <script>window.__FORTE_HOOK_CACHE__ = ${hookCacheJson};</script>
    <script type="module" src="${clientScript}"></script>
</body>
</html>`;

    const { readable, writable } = new TransformStream<Uint8Array, Uint8Array>();

    (async () => {
      const writer = writable.getWriter();

      await writer.write(encoder.encode(headHtml));

      const reader = reactStream.getReader();
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        await writer.write(value);
      }

      await writer.write(encoder.encode(tailHtml));
      await writer.close();
    })();

    return new Response(readable, { headers });
  }

  return new Response("Not Found", { status: 404 });
};
