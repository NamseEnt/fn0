const ORIGINAL_RE = /^original\/([^\/]+)\.tar$/;
const COMPILED_RE = /^compiled\/([^\/]+)\/([^\/]+)\/(.+)\.tar\.zst$/;
const RELEVANT_ACTIONS = new Set([
  "PutObject",
  "CompleteMultipartUpload",
  "CopyObject",
]);

export default {
  async queue(batch, env) {
    for (const msg of batch.messages) {
      try {
        await handleMessage(msg, env);
      } catch (err) {
        console.error("worker error:", err?.stack || err);
        msg.retry();
      }
    }
  },
};

async function handleMessage(msg, env) {
  const evt = msg.body;
  if (!evt || !RELEVANT_ACTIONS.has(evt.action)) {
    msg.ack();
    return;
  }
  const key = evt.object?.key;
  if (typeof key !== "string") {
    msg.ack();
    return;
  }

  const original = ORIGINAL_RE.exec(key);
  if (original) {
    const projectId = original[1];
    const head = await env.BUCKET.head(key);
    if (!head) {
      msg.ack();
      return;
    }
    const lastModified = head.uploaded.toISOString();
    const ok = await postAction(env, "bundle_uploaded", {
      project_id: projectId,
      last_modified: lastModified,
    });
    if (ok) msg.ack();
    else msg.retry();
    return;
  }

  const compiled = COMPILED_RE.exec(key);
  if (compiled) {
    const ok = await postAction(env, "bundle_compiled", {
      project_id: compiled[2],
      last_modified: compiled[3],
      fn0_wasmtime_version: compiled[1],
    });
    if (ok) msg.ack();
    else msg.retry();
    return;
  }

  msg.ack();
}

async function postAction(env, name, body) {
  const base = env.CONTROL_URL.replace(/\/$/, "");
  const resp = await fetch(`${base}/__forte_action/${name}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${env.CONTROL_ADMIN_TOKEN}`,
    },
    body: JSON.stringify(body),
  });
  if (!resp.ok) {
    const text = await resp.text().catch(() => "");
    console.warn(`control ${name} ${resp.status}: ${text}`);
    return false;
  }
  return true;
}
