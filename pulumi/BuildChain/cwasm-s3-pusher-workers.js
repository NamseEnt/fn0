export default {
  async fetch(request, env) {
    const secretHeader = request.headers.get('X-Worker-Secret');
    if (!secretHeader || secretHeader !== env.WORKER_SECRET_KEY) {
      return new Response('Unauthorized', { status: 401 });
    }

    const { sourceUrl, targetUrls } = await request.json();

    const response = await fetch(sourceUrl);
    if (!response.ok) {
      throw new Error(`Failed to download: ${response.status}`);
    }

    const arrayBuffer = await response.arrayBuffer();

    await Promise.all(
      targetUrls.map(async (url) => {
        const uploadResponse = await fetch(url, {
          method: 'PUT',
          body: arrayBuffer,
        });
        if (!uploadResponse.ok) {
          throw new Error(`Failed to upload to ${url}: ${uploadResponse.status}`);
        }
      })
    );

    return new Response('OK', { status: 200 });
  }
}
