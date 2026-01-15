import { core } from "ext:core/mod.js";
import { readableStreamForRid, resourceForReadableStream } from "ext:deno_web/06_streams.js";

export async function runHandler() {
  try {
    const {
      0: url,
      1: method,
      2: headers,
      3: rid,
    } = core.ops.op_get_request_parts();

    const body = rid !== null ? readableStreamForRid(rid) : null;
    const request = new Request(url, { method, headers, body });

    if (typeof handler !== "function") {
      throw new Error("User code must define a global 'handler' function.");
    }
    const response = await handler(request);
    const responseBody = response.body;

    let responseRid = null;
    if (responseBody) {
      const denoRid = responseBody[Symbol.for("Deno.core.resourceId")];
      if (denoRid !== undefined) {
        responseRid = denoRid;
      } else {
        responseRid = resourceForReadableStream(responseBody);
      }
    }

    await core.ops.op_respond(
      response.status,
      Array.from(response.headers.entries()),
      responseRid
    );
  } catch (e) {
    console.error("[ski/run.js] Error:", e.message, e.stack);
    await core.ops.op_respond(
      500,
      [["content-type", "text/plain"]],
      null
    );
  }
}
