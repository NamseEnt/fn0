use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Child, Command, Stdio};

const SQLD_VERSION: &str = "v0.24.32";

fn sqld_archive_url() -> Result<String> {
    let triple = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        (os, arch) => anyhow::bail!("Unsupported platform: {os}-{arch}"),
    };
    Ok(format!(
        "https://github.com/tursodatabase/libsql/releases/download/libsql-server-{SQLD_VERSION}/libsql-server-{triple}.tar.xz"
    ))
}

pub struct SqldProcess {
    child: Child,
}

impl SqldProcess {
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for SqldProcess {
    fn drop(&mut self) {
        self.kill();
    }
}

pub async fn start(project_dir: &Path, port: u16) -> Result<SqldProcess> {
    let url = sqld_archive_url()?;
    let binary =
        crate::tools::ensure_github_tool("sqld", SQLD_VERSION, &url, "sqld").await?;

    let data_dir = project_dir.join(".forte").join("data");
    std::fs::create_dir_all(&data_dir)?;

    let child = Command::new(&binary)
        .arg("--db-path")
        .arg(&data_dir)
        .arg("--http-listen-addr")
        .arg(format!("127.0.0.1:{}", port))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start sqld")?;

    wait_for_ready(port).await?;

    Ok(SqldProcess { child })
}

async fn wait_for_ready(port: u16) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/health", port);

    for _ in 0..50 {
        if client.get(&url).send().await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    anyhow::bail!("sqld did not become ready within 5 seconds")
}
