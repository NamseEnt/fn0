# Forte Frontend Runtime

The Forte frontend is a React/TypeScript application built with Vite. The framework generates a minimal runtime layer (`fe/.forte/`) that wires up SSR, hydration, and the hook/action communication layer. This document explains what that layer does and what parts are visible to application code.

## `@forte/react`

All Forte frontend helpers are available via the `@forte/react` module alias, which resolves to `fe/.forte/forte-react.ts`. Import from this alias — never from the file path directly, as the file is generated and git-ignored.

### `clearHookCache`

Evicts cached hook results so the next render re-fetches from the server:

```ts
import { clearHookCache } from "@forte/react";

// Evict a single hook by name
clearHookCache("user_session");

// Evict all hooks
clearHookCache();
```

Call this after a mutation (e.g., after a successful action) so the UI reflects the updated state. See [actions.md](actions.md#invalidating-hook-cache) for usage in context.

### `callAction`

Low-level function for calling a server action without the generated typed client:

```ts
import { callAction } from "@forte/react";

// Without schema validation
const result = await callAction("user_login", { email, password });

// With a Zod schema for response validation
import { z } from "zod";
const OutputSchema = z.discriminatedUnion("t", [
  z.object({ t: z.literal("Ok"), token: z.string() }),
  z.object({ t: z.literal("InvalidCredentials") }),
]);
const result = await callAction("user_login", { email, password }, OutputSchema);
```

Use the generated typed client from `fe/src/actions/.generated/<name>.ts` in normal application code — it uses `callAction` internally and adds type safety. Reach for `callAction` only when the generated client is unavailable (e.g., calling an action whose types haven't been generated yet, or in a non-Forte frontend).

Input fields follow camelCase → snake_case conversion on the server (see [sdk.md](sdk.md#forte_json--serialization-format)).

`callAction` throws if the server returns a non-2xx status.

### `useForteHook`

Internal function called by generated hooks in `fe/src/hooks/.generated/`. Do not call it directly — use the generated `use<HookName>` hook instead.

## `__FORTE_BASE_URL__`

A Vite `define` constant injected at build time. Contains the base URL for static assets:

- **`forte dev`**: `"/"`
- **`forte deploy`**: the CDN URL for this deploy (`https://<project_id>.static.fn0.dev/<code_version>/`)

Available in any frontend TypeScript file:

```ts
declare const __FORTE_BASE_URL__: string;

const assetUrl = `${__FORTE_BASE_URL__}images/logo.png`;
```

Vite replaces `__FORTE_BASE_URL__` at build time with the literal string, so no runtime lookup is needed. The `declare const` is only needed for TypeScript — the value is always present at runtime.

The same URL is also set as Vite's `base` option, so Vite-managed assets (imported with `import logoUrl from "./logo.png"`) already use it automatically. Use `__FORTE_BASE_URL__` only when you need to construct URLs manually (e.g., dynamic asset paths in strings).

## SSR / Hydration Lifecycle

Understanding the lifecycle helps when debugging SSR issues or writing code that behaves differently on server and client.

### Page rendering flow

1. HTTP request arrives for a page route (e.g., `/product/42`).
2. The fn0 Wasmtime runtime executes `backend.wasm` and routes to the Rust page handler.
3. The handler computes and serializes `Props` as JSON, then sets the `x-fn0-next: js` response header.
4. fn0 delegates to the Ski JS runtime, which calls `server.js` (the SSR bundle).
5. The SSR bundle:
   - Calls any hooks needed to render the page (via `/__self_invoke/<name>`).
   - Runs `renderToReadableStream` with the page's React component and Props.
   - Streams the HTML to the client, appending:
     - `window.__FORTE_PROPS__` — the serialized Props.
     - `window.__FORTE_HOOK_CACHE__` — all hook results fetched during SSR.
   - Includes `<script src="client.js">` to boot the client.
   - Sets `Cache-Control: no-store` on the response so CDNs and browser caches do not store the rendered HTML.
6. The browser loads `client.js`, which calls `hydrateRoot` with the same Props from `__FORTE_PROPS__`.

Hook results embedded in `__FORTE_HOOK_CACHE__` are read synchronously on the client — no network request is made on first render unless there is a cache miss. After a `clearHookCache()` call, the next render fetches fresh data.

### Page component props

The React component receives the Props object with path parameters merged in:

```tsx
// fe/src/pages/product/[id]/page.tsx
import type { Props } from "./.props";

// Forte passes params alongside the Props fields
export default function ProductPage(props: Props & { params: { id: string } }) {
    if (props.t !== "Ok") return null;
    return <div>{props.product_name} (id: {props.params.id})</div>;
}
```

The `params` field is a plain object keyed by dynamic segment names. It is not part of the generated `.props.ts` type — add it to your component's prop type manually if you need it.

### Server vs. client detection

```ts
const isSSR = typeof window === "undefined";
```

This is the standard pattern used by `forte-react.ts` itself. During SSR (`isSSR === true`), `window` is not defined and DOM APIs are unavailable.

### Cookies in SSR

During SSR, request cookies are forwarded to hook calls automatically — the SSR bundle reads the `Cookie` header from the incoming request and adds it to every `/__self_invoke/<name>` request. Cookies set by hook handlers during SSR are collected and sent as `Set-Cookie` response headers in the page response.

Application code does not need to handle this manually.

## Vite Configuration

The generated `fe/.forte/vite.config.ts` handles the full Forte build. To customize it without editing a generated file, create `fe/forte.config.ts` — its default export is merged with `mergeConfig` on top of the generated config:

```ts
// fe/forte.config.ts
import type { UserConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default {
  plugins: [tailwindcss()],
} satisfies UserConfig;
```

See [project-structure.md](project-structure.md#feforteconfigt-optional) for details.
