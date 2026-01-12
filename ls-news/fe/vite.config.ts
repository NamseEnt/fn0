import { defineConfig, Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

function exitOnStdinClose(): Plugin {
  return {
    name: "exit-on-stdin-close",
    configureServer() {
      process.stdin.resume();
      process.stdin.on("close", () => {
        process.exit(0);
      });
    },
  };
}

export default defineConfig(({ isSsrBuild }) => ({
  plugins: [react(), tailwindcss(), exitOnStdinClose()],
  envDir: path.resolve(__dirname, ".."),
  envPrefix: "PUBLIC_",
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  optimizeDeps: {
    include: ["react", "react-dom"],
  },
  build: {
    rollupOptions: {
      input: isSsrBuild ? "src/server.tsx" : "src/client.tsx",
      output: {
        entryFileNames: isSsrBuild ? "server.js" : "client.js",
      },
    },
  },
  ssr: {
    external: ["react", "react-dom"],
  },
}));
