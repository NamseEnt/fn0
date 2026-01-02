use anyhow::{Context, Result};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::server::{self, ServerConfig};

#[derive(Debug)]
pub struct DevOptions {
    pub project_dir: PathBuf,
    /// None means auto-select starting from 3000
    pub port: Option<u16>,
}

impl Default for DevOptions {
    fn default() -> Self {
        Self {
            project_dir: PathBuf::from("."),
            port: None,
        }
    }
}

fn is_port_available(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpListener::bind(addr).is_ok()
}

fn find_available_port(start: u16) -> Option<u16> {
    (start..=65535).find(|&port| is_port_available(port))
}

fn run_codegen(project_dir: &Path) -> Result<()> {
    let rs_dir = project_dir.join("rs");

    // TODO: Use cargo binstall (github releases) instead of hardcoded path
    let forte_rs_to_ts_dir = "/home/namse/fn0/forte-rs-to-ts";

    println!("[codegen] Running forte-rs-to-ts...");
    let status = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg(&rs_dir)
        .current_dir(forte_rs_to_ts_dir)
        .status()
        .context("Failed to run forte-rs-to-ts. Is it installed?")?;

    if !status.success() {
        anyhow::bail!("forte-rs-to-ts failed with status: {}", status);
    }

    // TODO: Generate frontend routes (routes.generated.ts)

    Ok(())
}

fn build_backend(project_dir: &Path) -> Result<()> {
    let rs_dir = project_dir.join("rs");

    println!("[build] Building backend...");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&rs_dir)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed with status: {}", status);
    }

    Ok(())
}

fn build_frontend(project_dir: &Path) -> Result<()> {
    let fe_dir = project_dir.join("fe");

    println!("[build] Building frontend...");
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(&fe_dir)
        .status()
        .context("Failed to run npm run build")?;

    if !status.success() {
        anyhow::bail!("npm run build failed with status: {}", status);
    }

    Ok(())
}

pub async fn run(options: DevOptions) -> Result<()> {
    let project_dir = options.project_dir.canonicalize()?;

    let port = match options.port {
        Some(p) => {
            if !is_port_available(p) {
                eprintln!("Error: Port {} is already in use", p);
                std::process::exit(1);
            }
            p
        }
        None => {
            let p = find_available_port(3000)
                .ok_or_else(|| anyhow::anyhow!("No available port found starting from 3000"))?;
            if p != 3000 {
                println!("Port 3000 in use, using port {} instead", p);
            }
            p
        }
    };

    println!("Starting Forte dev server...");
    println!("Project directory: {}", project_dir.display());

    run_codegen(&project_dir)?;
    build_backend(&project_dir)?;
    build_frontend(&project_dir)?;

    // TODO: Watch mode

    let backend_path = project_dir
        .join("rs/target/wasm32-wasip2/release/backend.wasm")
        .to_string_lossy()
        .to_string();

    let frontend_path = project_dir
        .join("fe/dist/server.js")
        .to_string_lossy()
        .to_string();

    let config = ServerConfig {
        port,
        backend_path,
        frontend_path,
    };

    server::run(config).await
}
