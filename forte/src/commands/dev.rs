use anyhow::{Context, Result};
use std::env;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::config;
use crate::watcher;

pub fn execute() -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    println!("Starting Forte development server...");
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

    // Load and validate environment variables
    let env_vars = config::load_env_vars(&current_dir, "development")?;
    config::validate_env(&forte_config, &env_vars)?;

    // Initial code generation
    println!("Generating initial code...");
    let routes = watcher::scan_routes(&current_dir)?;
    for route in &routes {
        if let Err(e) = watcher::process_props_file(route) {
            eprintln!("Error processing {}: {}", route.props_path.display(), e);
        }
    }
    crate::codegen::generate_backend_code(&current_dir, &routes, &forte_config)?;
    crate::codegen::generate_frontend_code(&current_dir, &routes)?;

    // Build backend
    println!("\nBuilding backend...");
    let backend_dir = current_dir.join("backend");
    let build_status = Command::new("cargo")
        .arg("build")
        .current_dir(&backend_dir)
        .status()
        .context("Failed to run cargo build")?;

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

    // Start backend server
    println!("\nStarting backend server...");
    let backend_process = Command::new("cargo")
        .arg("run")
        .current_dir(&backend_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start backend server")?;

    // Wait a moment for backend to start
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Start frontend server
    println!("Starting frontend server...");
    let frontend_process = Command::new("npm")
        .arg("run")
        .arg("dev")
        .current_dir(&frontend_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start frontend server")?;

    let backend_handle = Arc::new(Mutex::new(Some(backend_process)));
    let frontend_handle = Arc::new(Mutex::new(Some(frontend_process)));
    let backend_handle_clone = backend_handle.clone();
    let frontend_handle_clone = frontend_handle.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("\n\nShutting down...");
        if let Some(mut child) = backend_handle_clone.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(mut child) = frontend_handle_clone.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        std::process::exit(0);
    })
    .context("Failed to set Ctrl+C handler")?;

    // Watch for file changes
    println!();
    watcher::watch_routes(&current_dir, &forte_config)?;

    // Clean up
    if let Some(mut child) = backend_handle.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    if let Some(mut child) = frontend_handle.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    Ok(())
}
