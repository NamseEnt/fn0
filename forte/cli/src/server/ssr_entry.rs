pub fn generate_ssr_entry_code(server_module_path: &str) -> String {
    format!(
        r#"import "{server_module_path}";

async function runSsrHandler() {{
  try {{
    const [url, method, headers, rid] = __ski_getRequestParts();

    const body = rid !== null ? __ski_readableStreamForRid(rid) : null;
    const request = new Request(url, {{ method, headers, body }});

    if (typeof handler !== "function") {{
      throw new Error("User code must define a global 'handler' function.");
    }}

    const response = await handler(request);
    const responseBody = response.body;

    let responseRid = null;
    if (responseBody) {{
      const denoRid = responseBody[Symbol.for("Deno.core.resourceId")];
      if (denoRid !== undefined) {{
        responseRid = denoRid;
      }} else {{
        responseRid = __ski_resourceForReadableStream(responseBody);
      }}
    }}

    await __ski_respond(
      response.status,
      Array.from(response.headers.entries()),
      responseRid
    );
  }} catch (e) {{
    const stack = e.stack || String(e);
    console.error("[ssr-entry] Error:", e.message);
    console.error(stack);
    await __ski_respond(
      500,
      [["content-type", "text/plain"]],
      null
    );
  }}
}}

await runSsrHandler();
"#
    )
}
