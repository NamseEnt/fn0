import express from "express";
import { createServer as createViteServer } from "vite";
import { createServer } from "http";

async function startDevServer() {
  const app = express();
  app.use(express.json({ limit: "50mb" }));

  const vite = await createViteServer({
    server: { middlewareMode: true },
    appType: "custom",
  });

  app.use(vite.middlewares);

  app.post("/__ssr_render", async (req, res) => {
    try {
      const { url, props, forteBaseUrl } = req.body;
      const serverModule = await vite.ssrLoadModule("/src/server.tsx");
      const rawHtml = await serverModule.render(url, props, forteBaseUrl);
      const html = await vite.transformIndexHtml(url, rawHtml.html);

      res.set("Content-Type", "text/html");
      res.send(html);
    } catch (e) {
      vite.ssrFixStacktrace(e);
      console.error(e);
      res.status(500).send(`SSR Error: ${e.message}`);
    }
  });

  const server = createServer(app);

  server.listen(0, "127.0.0.1", () => {
    const addr = server.address();
    console.log(JSON.stringify({ port: addr.port }));
  });

  process.stdin.resume();
  process.stdin.on("close", () => process.exit(0));
}

startDevServer();
