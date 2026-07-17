use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use uuid::Uuid;

pub fn generate_vite_server_script(
    socket_path: &Path,
    config_file: Option<&Path>,
    ssr_module_path: &str,
) -> String {
    let socket_path_str = socket_path.display();
    let config_file_js = match config_file {
        Some(p) => format!("\"{}\"", p.display()),
        None => "undefined".to_string(),
    };
    format!(
        r#"import {{ createServer }} from "vite";
import {{ createServer as createHttpServer }} from "http";
import {{ unlinkSync, existsSync }} from "fs";

const SOCKET_PATH = "{socket_path_str}";
const CONFIG_FILE = {config_file_js};
const SSR_MODULE_PATH = "{ssr_module_path}";

function cleanup() {{
    try {{
        if (existsSync(SOCKET_PATH)) {{
            unlinkSync(SOCKET_PATH);
        }}
    }} catch (e) {{
        // ignore
    }}
}}

process.on("exit", cleanup);
process.on("SIGINT", () => {{ cleanup(); process.exit(0); }});
process.on("SIGTERM", () => {{ cleanup(); process.exit(0); }});

process.stdin.resume();
process.stdin.on("end", () => {{
    cleanup();
    process.exit(0);
}});
process.stdin.on("close", () => {{
    cleanup();
    process.exit(0);
}});

async function streamToString(stream) {{
    const reader = stream.getReader();
    const chunks = [];
    while (true) {{
        const {{ done, value }} = await reader.read();
        if (done) break;
        chunks.push(value);
    }}
    const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {{
        result.set(chunk, offset);
        offset += chunk.length;
    }}
    return result;
}}

async function main() {{
    cleanup();

    const viteOptions = {{
        server: {{
            middlewareMode: true,
        }},
        appType: "custom",
    }};
    if (CONFIG_FILE) {{
        viteOptions.configFile = CONFIG_FILE;
    }}
    const vite = await createServer(viteOptions);


    const server = createHttpServer(async (req, res) => {{
        if (req.method === "POST" && req.url === "/__ssr_render") {{
            let body = "";
            req.on("data", (chunk) => {{
                body += chunk;
            }});
            req.on("end", async () => {{
                try {{
                    const {{ url, props, cookie }} = JSON.parse(body);
                    await vite.ssrLoadModule(SSR_MODULE_PATH);
                    const renderStream = globalThis.renderStream;
                    const {{ stream, cookies }} = await renderStream(url, props, cookie);

                    const htmlBuffer = Buffer.from(await streamToString(stream));
                    res.useChunkedEncodingByDefault = false;
                    res.setHeader("Content-Type", "text/html");
                    res.setHeader("Content-Length", htmlBuffer.length);
                    if (cookies && cookies.length > 0) {{
                        res.setHeader("Set-Cookie", cookies);
                    }}
                    res.writeHead(200);
                    res.end(htmlBuffer);
                }} catch (e) {{
                    vite.ssrFixStacktrace(e);
                    console.error("[vite-ssr] Error:", e.stack || e.message);
                    res.writeHead(500, {{ "Content-Type": "text/plain" }});
                    res.end(e.stack || e.message);
                }}
            }});
            return;
        }}

        vite.middlewares(req, res);
    }});

    server.listen(SOCKET_PATH);
}}

main().catch((e) => {{
    console.error("[vite-ssr] Failed to start:", e);
    process.exit(1);
}});
"#
    )
}

pub struct Vite {
    pub child: Child,
    pub socket_path: PathBuf,
}

pub fn spawn_vite(
    fe_dir: &Path,
    forte_port: u16,
    config_file: Option<&Path>,
    ssr_module_path: &str,
) -> Result<Vite> {
    let dev_dir = fe_dir.join(".forte/dev");
    std::fs::create_dir_all(&dev_dir)?;

    let session_id = Uuid::new_v4();
    // The socket lives in the OS temp dir, not next to the project: sun_path is
    // capped at 104 bytes on macOS, so a socket inside a nested project dir
    // silently binds to a truncated path and fails with EADDRINUSE.
    let socket_path = std::env::temp_dir().join(format!(
        "forte-vite-{}.sock",
        &session_id.simple().to_string()[..8]
    ));

    let script_path = dev_dir.join(format!("vite-ssr-server-{}.mjs", session_id));
    let script_content = generate_vite_server_script(&socket_path, config_file, ssr_module_path);
    std::fs::write(&script_path, &script_content)?;

    let child = Command::new("node")
        .arg(&script_path)
        .current_dir(fe_dir)
        .env("FORTE_PORT", forte_port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(Vite { child, socket_path })
}

pub async fn wait_for_vite_ready(socket_path: &Path) -> Result<()> {
    for _ in 0..50 {
        if socket_path.exists() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            return Ok(());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    anyhow::bail!("Vite server did not start within 5 seconds")
}
