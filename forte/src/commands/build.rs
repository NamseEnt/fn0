use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config;
use crate::watcher;

pub fn execute() -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    println!("Building Forte project for production...");
    println!("Project root: {}\n", current_dir.display());

    // Verify we're in a Forte project
    let forte_toml = current_dir.join("Forte.toml");
    if !forte_toml.exists() {
        anyhow::bail!(
            "Forte.toml not found. Are you in a Forte project root?\nRun 'forte init <project-name>' to create a new project."
        );
    }

    // Load config
    let forte_config = config::ForteConfig::load(&current_dir)?;

    // Load and validate environment variables for production
    let env_vars = config::load_env_vars(&current_dir, "production")?;
    config::validate_env(&forte_config, &env_vars)?;

    // Generate code
    println!("Generating code...");
    let routes = watcher::scan_routes(&current_dir)?;
    for route in &routes {
        if let Err(e) = watcher::process_props_file(route) {
            eprintln!("Error processing {}: {}", route.props_path.display(), e);
        }
    }
    crate::codegen::generate_backend_code(&current_dir, &routes, &forte_config)?;
    crate::codegen::generate_frontend_code(&current_dir, &routes)?;

    // Build backend in release mode
    println!("\nBuilding backend (release mode)...");
    let backend_dir = current_dir.join("backend");
    let build_status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&backend_dir)
        .status()
        .context("Failed to run cargo build --release")?;

    if !build_status.success() {
        anyhow::bail!("Backend build failed");
    }

    // Install frontend dependencies if needed
    let frontend_dir = current_dir.join("frontend");
    let node_modules = frontend_dir.join("node_modules");
    if !node_modules.exists() {
        println!("\nInstalling frontend dependencies...");
        let npm_status = Command::new("npm")
            .arg("install")
            .current_dir(&frontend_dir)
            .status()
            .context("Failed to run npm install")?;

        if !npm_status.success() {
            anyhow::bail!("npm install failed");
        }
    }

    // Build frontend with Vite
    println!("\nBuilding frontend (production mode)...");
    let vite_status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(&frontend_dir)
        .status()
        .context("Failed to run npm run build")?;

    if !vite_status.success() {
        anyhow::bail!("Frontend build failed");
    }

    // Create dist directory and copy artifacts
    let dist_dir = current_dir.join("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir).context("Failed to remove old dist directory")?;
    }
    fs::create_dir_all(&dist_dir).context("Failed to create dist directory")?;

    // Copy backend binary
    println!("\nCopying artifacts to dist/...");
    let backend_binary_name = backend_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("backend");

    let backend_binary_src = backend_dir
        .join("target/release")
        .join(backend_binary_name);

    let backend_binary_dst = dist_dir.join(backend_binary_name);

    fs::copy(&backend_binary_src, &backend_binary_dst)
        .with_context(|| format!("Failed to copy backend binary from {} to {}",
            backend_binary_src.display(), backend_binary_dst.display()))?;

    // Copy frontend dist
    let frontend_dist_src = frontend_dir.join("dist");
    let frontend_dist_dst = dist_dir.join("public");

    copy_dir_all(&frontend_dist_src, &frontend_dist_dst)
        .context("Failed to copy frontend dist")?;

    // Copy .env.production if it exists
    let env_prod_src = current_dir.join(".env.production");
    if env_prod_src.exists() {
        let env_prod_dst = dist_dir.join(".env.production");
        fs::copy(&env_prod_src, &env_prod_dst)
            .context("Failed to copy .env.production")?;
        println!("  ✓ Copied .env.production");
    }

    println!("\n✓ Build complete!");
    println!("\nProduction artifacts:");
    println!("  Backend binary: dist/{}", backend_binary_name);
    println!("  Frontend files: dist/public/");
    if env_prod_src.exists() {
        println!("  Environment:    dist/.env.production");
    }
    println!("\nTo run in production:");
    println!("  cd dist && ./{}", backend_binary_name);

    Ok(())
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).context("Failed to create destination directory")?;

    for entry in fs::read_dir(src).context("Failed to read source directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let ty = entry.file_type().context("Failed to get file type")?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("Failed to copy {} to {}",
                    src_path.display(), dst_path.display()))?;
        }
    }

    Ok(())
}
