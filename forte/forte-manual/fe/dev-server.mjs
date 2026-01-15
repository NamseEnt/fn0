import { createServer as createViteServer } from 'vite';
import express from 'express';

const app = express();

app.use(express.json());

const vite = await createViteServer({
  server: { middlewareMode: true },
  appType: 'custom',
});

app.use(vite.middlewares);

app.post('/__ssr_render', async (req, res) => {
  try {
    const { url, props } = req.body;

    await vite.ssrLoadModule('/src/server.ts');

    const handler = globalThis.handler;
    if (!handler) {
      throw new Error('handler not found on globalThis');
    }

    const request = new Request(`http://localhost${url}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(props),
    });

    const response = await handler(request);
    const html = await response.text();

    res.status(response.status).set('content-type', 'text/html; charset=utf-8').send(html);
  } catch (e) {
    vite.ssrFixStacktrace(e);
    console.error(e);
    res.status(500).send(e.message);
  }
});

const PORT = 9000;
app.listen(PORT, () => {
  console.log(JSON.stringify({ port: PORT }));
});
