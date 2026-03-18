use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const SQLD_VERSION: &str = "v0.24.32";

fn sqld_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".forte").join("bin"))
}

fn sqld_binary_path() -> Result<PathBuf> {
    Ok(sqld_dir()?.join(format!("sqld-{}", SQLD_VERSION)))
}

fn archive_name() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("libsql-server-aarch64-apple-darwin.tar.xz"),
        ("macos", "x86_64") => Ok("libsql-server-x86_64-apple-darwin.tar.xz"),
        ("linux", "x86_64") => Ok("libsql-server-x86_64-unknown-linux-gnu.tar.xz"),
        ("linux", "aarch64") => Ok("libsql-server-aarch64-unknown-linux-gnu.tar.xz"),
        (os, arch) => anyhow::bail!("Unsupported platform: {os}-{arch}"),
    }
}

async fn ensure_binary() -> Result<PathBuf> {
    let path = sqld_binary_path()?;
    if path.exists() {
        return Ok(path);
    }

    let dir = sqld_dir()?;
    std::fs::create_dir_all(&dir)?;

    let archive = archive_name()?;
    let url = format!(
        "https://github.com/tursodatabase/libsql/releases/download/libsql-server-{}/{}",
        SQLD_VERSION, archive
    );

    println!("Downloading sqld {}...", SQLD_VERSION);

    let response = reqwest::get(&url).await?.error_for_status()?;
    let bytes = response.bytes().await?;

    let temp_archive = dir.join("temp.tar.xz");
    std::fs::write(&temp_archive, &bytes)?;

    let temp_extract = dir.join("temp_extract");
    std::fs::create_dir_all(&temp_extract)?;

    let status = Command::new("tar")
        .arg("xf")
        .arg(&temp_archive)
        .arg("-C")
        .arg(&temp_extract)
        .status()
        .context("Failed to run tar. Is tar installed?")?;

    std::fs::remove_file(&temp_archive)?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&temp_extract);
        anyhow::bail!("Failed to extract sqld archive");
    }

    let extracted = find_binary(&temp_extract)?;
    std::fs::rename(&extracted, &path)?;
    let _ = std::fs::remove_dir_all(&temp_extract);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("sqld {} ready.", SQLD_VERSION);
    Ok(path)
}

fn find_binary(dir: &Path) -> Result<PathBuf> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_binary(&path) {
                return Ok(found);
            }
        } else if path.file_name().is_some_and(|n| n == "sqld") {
            return Ok(path);
        }
    }
    anyhow::bail!("sqld binary not found in archive")
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
    let binary = ensure_binary().await?;

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
