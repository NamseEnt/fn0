import type { AstroIntegration } from "astro";
import { getAdapter } from "./adapter.js";
import { createVitePlugins } from "./vite/plugins.js";
import { BuildConfig, buildServer } from "./build.js";

export interface SuiseiOptions {}

export default function suisei(_options: SuiseiOptions = {}): AstroIntegration {
  let buildConfig: BuildConfig;

  return {
    name: "suisei",
    hooks: {
      "astro:config:setup": ({ updateConfig }) => {
        const vitePlugins = createVitePlugins();

        updateConfig({
          vite: {
            plugins: vitePlugins,
            ssr: {
              noExternal: ["hono", "@bytecodealliance/jco-std"],
            },
          },
        });
      },

      "astro:config:done": ({ config, setAdapter }) => {
        buildConfig = {
          client: config.build.client,
          server: config.build.server,
        };

        setAdapter(getAdapter());

        if (config.output === "static") {
          throw new Error(
            '⚠️  suisei adapter requires SSR mode. Set output: "server" in astro.config.mjs'
          );
        }
      },

      "astro:build:done": async () => {
        await buildServer(buildConfig);
      },
    },
  };
}
