import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig(({ command }) => {
  const isBuild = command === "build";

  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "src"),
      },
    },
    server: {
      hmr: {
        host: "localhost",
        protocol: "ws",
      },
      cors: true,
    },
    ssr: isBuild
      ? {
          target: "webworker",
          noExternal: true,
        }
      : undefined,
  };
});
