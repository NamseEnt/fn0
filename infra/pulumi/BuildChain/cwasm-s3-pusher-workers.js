addEventListener('fetch', (event) => {
  event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
  const secretHeader = request.headers.get('X-Worker-Secret');
  if (!secretHeader || secretHeader !== WORKER_SECRET_KEY) {
    return new Response('Unauthorized', { status: 401 });
  }

  const { sourceUrl, targetUrls } = await request.json();

  const response = await fetch(sourceUrl);
  if (!response.ok) {
    return new Response(`Failed to download: ${response.status}`, { status: 502 });
  }

  const arrayBuffer = await response.arrayBuffer();

  const results = await Promise.allSettled(
    targetUrls.map(async (url) => {
      const uploadResponse = await fetch(url, {
        method: 'PUT',
        body: arrayBuffer,
      });
      if (!uploadResponse.ok) {
        throw new Error(`Failed to upload: ${uploadResponse.status}`);
      }
    })
  );

  const failures = results.filter((r) => r.status === 'rejected');
  if (failures.length > 0) {
    return new Response(`Failed: ${failures.map((f) => f.reason).join(', ')}`, { status: 502 });
  }

  return new Response('OK', { status: 200 });
}
