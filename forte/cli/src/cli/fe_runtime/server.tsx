import { renderToReadableStream } from "react-dom/server.browser";
import { renderToString } from "react-dom/server";
import { routes } from "./routes.generated";
import {
  serializeHookCache,
  clearHookCache,
  getCollectedCookies,
  clearCollectedCookies,
  setRequestCookie,
} from "./forte-react";
import { Head } from "../src/app";

const isDev = (import.meta as any).env?.DEV ?? false;

const clientScriptSrc = isDev ? "/.forte/client.tsx" : "/public/client.js";

const viteDevBlock = isDev
  ? `<script type="module" src="/@vite/client"></script>
<script type="module">
import RefreshRuntime from "/@react-refresh";
RefreshRuntime.injectIntoGlobalHook(window);
window.$RefreshReg$ = () => {};
window.$RefreshSig$ = () => (type) => type;
window.__vite_plugin_react_preamble_installed__ = true;
</script>`
  : "";

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
    if (match) return { route, params };
  }
  return null;
}

function escapeJsonForScript(json: string): string {
  return json.replace(/</g, "\\u003c").replace(/>/g, "\\u003e");
}

async function renderStream(
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

  const PageComponent = pageModule.default;
  const reactStream = await renderToReadableStream(
    <PageComponent {...(allProps as any)} />
  );
  await reactStream.allReady;

  const headHtml = renderToString(<Head />);
  const hookCacheJson = serializeHookCache();
  const cookies = getCollectedCookies();

  const htmlHead = `<!DOCTYPE html>
<html>
<head>
${headHtml}
${viteDevBlock}
</head>
<body>
<div id="root">`;

  const htmlTail = `</div>
<script>window.__FORTE_PROPS__ = ${propsJson};</script>
<script>window.__FORTE_HOOK_CACHE__ = ${hookCacheJson};</script>
<script type="module" src="${clientScriptSrc}"></script>
</body>
</html>`;

  const { readable, writable } = new TransformStream<Uint8Array, Uint8Array>();
  (async () => {
    const writer = writable.getWriter();
    await writer.write(encoder.encode(htmlHead));
    const reader = reactStream.getReader();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      await writer.write(value);
    }
    await writer.write(encoder.encode(htmlTail));
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
      `[SSR] JSON parse error. URL: ${request.url}, Body: "${bodyText.slice(0, 500)}"`
    );
    throw e;
  }
  const cookie = request.headers.get("cookie");
  const { stream, cookies } = await renderStream(request.url, rawProps, cookie);
  const headers = new Headers({ "Content-Type": "text/html" });
  for (const c of cookies) headers.append("Set-Cookie", c);
  return new Response(stream, { headers });
};

(globalThis as any).renderStream = renderStream;
