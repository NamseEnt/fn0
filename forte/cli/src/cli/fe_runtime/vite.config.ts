import { defineConfig, mergeConfig, type UserConfig, type UserConfigFn } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";

const dirname = path.dirname(fileURLToPath(import.meta.url));
const feDir = path.resolve(dirname, "..");

let userConfig: UserConfig | UserConfigFn = {};
const userConfigPath = path.resolve(feDir, "forte.config.ts");
if (fs.existsSync(userConfigPath)) {
  const mod = await import(userConfigPath);
  userConfig = mod.default ?? {};
}

export default defineConfig(async (env) => {
  const base: UserConfig = {
    root: feDir,
    plugins: [react()],
    resolve: {
      alias: {
        "@forte/react": path.resolve(dirname, "./forte-react.ts"),
      },
    },
    build: {
      outDir: env.isSsrBuild
        ? path.resolve(feDir, "dist/ssr")
        : path.resolve(feDir, "dist"),
      emptyOutDir: true,
      rollupOptions: {
        input: env.isSsrBuild
          ? path.resolve(dirname, "./server.tsx")
          : path.resolve(dirname, "./client.tsx"),
        output: env.isSsrBuild
          ? {
              entryFileNames: "server.js",
              format: "iife",
              inlineDynamicImports: true,
            }
          : {
              entryFileNames: "client.js",
            },
      },
    },
    ssr: env.isSsrBuild
      ? { target: "webworker", noExternal: true }
      : undefined,
  };
  const resolved =
    typeof userConfig === "function" ? await userConfig(env) : userConfig;
  return mergeConfig(base, resolved);
});
