use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use uuid::Uuid;

pub fn generate_vite_server_script(socket_path: &Path) -> String {
    let socket_path_str = socket_path.display();
    format!(
        r#"import {{ createServer }} from "vite";
import {{ createServer as createHttpServer }} from "http";
import {{ unlinkSync, existsSync }} from "fs";

const SOCKET_PATH = "{socket_path_str}";

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

async function main() {{
    cleanup();

    const vite = await createServer({{
        server: {{
            middlewareMode: true,
            hmr: false,
        }},
        appType: "custom",
    }});

    const server = createHttpServer(async (req, res) => {{
        if (req.method === "POST" && req.url === "/__ssr_render") {{
            let body = "";
            req.on("data", (chunk) => {{
                body += chunk;
            }});
            req.on("end", async () => {{
                try {{
                    const {{ url, props }} = JSON.parse(body);
                    const {{ render }} = await vite.ssrLoadModule("/src/server.tsx");
                    const html = await render(url, props);
                    res.writeHead(200, {{ "Content-Type": "text/html" }});
                    res.end(html);
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

pub struct ViteServer {
    pub child: Child,
    pub socket_path: PathBuf,
}

pub fn spawn_vite_server(fe_dir: &Path) -> Result<ViteServer> {
    let dev_dir = fe_dir.join(".forte/dev");
    std::fs::create_dir_all(&dev_dir)?;

    let session_id = Uuid::new_v4();
    let socket_path = dev_dir.join(format!("vite-{}.sock", session_id));

    let script_path = dev_dir.join(format!("vite-ssr-server-{}.mjs", session_id));
    let script_content = generate_vite_server_script(&socket_path);
    std::fs::write(&script_path, &script_content)?;

    let child = Command::new("node")
        .arg(&script_path)
        .current_dir(fe_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(ViteServer { child, socket_path })
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
