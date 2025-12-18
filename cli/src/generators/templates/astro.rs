pub fn generate_config() -> String {
    r#"import { defineConfig } from 'astro/config';
import fn0 from './src/lib/fn0-adapter.ts';

export default defineConfig({
  output: 'server',
  adapter: fn0(),
});
"#
    .to_string()
}

pub fn generate_adapter() -> String {
    r#"import type { AstroIntegration } from 'astro';

export default function fn0Adapter(): AstroIntegration {
  return {
    name: 'fn0-astro-adapter',
    hooks: {
      'astro:config:done': ({ setAdapter }) => {
        setAdapter({
          name: 'fn0-astro-adapter',
          serverEntrypoint: './src/lib/server-entry.ts',
          exports: ['incomingHandler'],
          supportedAstroFeatures: {
            serverOutput: 'stable',
            staticOutput: 'stable',
          },
        });
      },
    },
  };
}
"#
    .to_string()
}

pub fn generate_server_entry() -> String {
    r#"import { App } from 'astro/app';
import type { SSRManifest } from 'astro';

let manifest: SSRManifest;
let app: App;

export function setManifest(ssrManifest: SSRManifest) {
  manifest = ssrManifest;
  app = new App(manifest);
}

export async function incomingHandler(request: Request): Promise<Response> {
  if (!app) {
    return new Response('Server not initialized', { status: 500 });
  }

  if (app.match(request)) {
    const response = await app.render(request);
    return response;
  }

  return new Response('Not Found', { status: 404 });
}
"#
    .to_string()
}

pub fn generate_index_page() -> String {
    r#"---
const currentTime = new Date().toISOString();
---

<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width" />
    <title>Fn0 + Astro</title>
    <style>
      body {
        font-family: system-ui, sans-serif;
        max-width: 800px;
        margin: 0 auto;
        padding: 2rem;
      }
      h1 {
        color: #0066cc;
      }
    </style>
  </head>
  <body>
    <h1>Hello from Fn0 + Astro!</h1>
    <p>This page is server-side rendered at: <strong>{currentTime}</strong></p>
    <p>Try refreshing the page to see the time update!</p>
  </body>
</html>
"#
    .to_string()
}

pub fn generate_env_d_ts() -> String {
    r#"/// <reference types="astro/client" />
"#
    .to_string()
}
