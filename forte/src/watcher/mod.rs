use anyhow::{Context, Result};
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use walkdir::WalkDir;

use crate::generator::generate_typescript;
use crate::parser::parse_props_file;

/// Route information extracted from file path
#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub props_path: PathBuf,      // backend/src/routes/product/_id_/props.rs
    pub frontend_dir: PathBuf,     // frontend/src/app/product/[id]
    pub gen_ts_path: PathBuf,      // frontend/src/app/product/[id]/props.gen.ts
    pub has_action_input: bool,    // Whether this route has ActionInput struct (for POST actions)
    pub has_error_page: bool,      // Whether this route has an error.tsx file
}

/// Scan the backend/src/routes directory and find all props.rs files
pub fn scan_routes(project_root: &Path) -> Result<Vec<RouteInfo>> {
    let routes_dir = project_root.join("backend/src/routes");

    if !routes_dir.exists() {
        anyhow::bail!("Routes directory not found: {}", routes_dir.display());
    }

    let mut routes = Vec::new();

    for entry in WalkDir::new(&routes_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some("props.rs") {
            let route_info = build_route_info(project_root, path)?;
            routes.push(route_info);
        }
    }

    Ok(routes)
}

/// Build route info from a props.rs path
fn build_route_info(project_root: &Path, props_path: &Path) -> Result<RouteInfo> {
    let routes_dir = project_root.join("backend/src/routes");

    // Get relative path from routes dir
    let rel_path = props_path
        .strip_prefix(&routes_dir)
        .context("Failed to strip routes prefix")?;

    // Remove "props.rs" from the end
    let route_path = rel_path.parent().context("No parent directory")?;

    // Convert backend path to frontend path
    // backend: product/_id_  -> frontend: product/[id]
    let frontend_path = convert_backend_to_frontend_path(route_path)?;

    let frontend_dir = project_root
        .join("frontend/src/app")
        .join(&frontend_path);

    let gen_ts_path = frontend_dir.join("props.gen.ts");

    // Parse props.rs to check if it has ActionInput
    let has_action_input = match parse_props_file(props_path.to_str().unwrap()) {
        Ok(parsed) => parsed.structs.iter().any(|s| s.is_action_input),
        Err(_) => false,  // If parsing fails, assume no action input
    };

    // Check if error.tsx exists in the frontend directory
    let error_tsx_path = frontend_dir.join("error.tsx");
    let has_error_page = error_tsx_path.exists();

    Ok(RouteInfo {
        props_path: props_path.to_path_buf(),
        frontend_dir,
        gen_ts_path,
        has_action_input,
        has_error_page,
    })
}

/// Convert backend route path to frontend path
/// Examples:
///   index -> index
///   product/_id_ -> product/[id]
///   user/_userId_/post/_postId_ -> user/[userId]/post/[postId]
fn convert_backend_to_frontend_path(backend_path: &Path) -> Result<PathBuf> {
    let mut frontend_path = PathBuf::new();

    for component in backend_path.components() {
        if let std::path::Component::Normal(os_str) = component {
            let s = os_str.to_str().context("Invalid UTF-8 in path")?;

            // Convert _paramName_ to [paramName]
            if s.starts_with('_') && s.ends_with('_') && s.len() > 2 {
                let param_name = &s[1..s.len() - 1];
                frontend_path.push(format!("[{}]", param_name));
            } else {
                frontend_path.push(s);
            }
        }
    }

    Ok(frontend_path)
}

/// Process a single props.rs file: parse and generate TypeScript
pub fn process_props_file(route_info: &RouteInfo) -> Result<()> {
    println!("Processing: {}", route_info.props_path.display());

    // Parse Rust file
    let parsed = parse_props_file(
        route_info.props_path.to_str().context("Invalid path")?
    )?;

    // Generate TypeScript
    let ts_code = generate_typescript(&parsed);

    // Ensure frontend directory exists
    std::fs::create_dir_all(&route_info.frontend_dir)
        .context("Failed to create frontend directory")?;

    // Write TypeScript file
    std::fs::write(&route_info.gen_ts_path, ts_code)
        .context("Failed to write TypeScript file")?;

    println!("  ✓ Generated: {}", route_info.gen_ts_path.display());

    Ok(())
}

/// Watch for file changes and trigger code generation
pub fn watch_routes(project_root: &Path, config: &crate::config::ForteConfig) -> Result<()> {
    let project_root = project_root.to_path_buf();
    let config = config.clone();

    // Initial scan and generation
    println!("Scanning routes...");
    let routes = scan_routes(&project_root)?;
    println!("Found {} route(s)", routes.len());

    for route in &routes {
        if let Err(e) = process_props_file(route) {
            eprintln!("Error processing {}: {}", route.props_path.display(), e);
        }
    }

    // Generate backend code
    println!("\nGenerating backend code...");
    if let Err(e) = crate::codegen::generate_backend_code(&project_root, &routes, &config) {
        eprintln!("Error generating backend code: {}", e);
    }

    // Generate frontend code
    println!("Generating frontend code...");
    if let Err(e) = crate::codegen::generate_frontend_code(&project_root, &routes) {
        eprintln!("Error generating frontend code: {}", e);
    }

    println!("\nWatching for changes... (Press Ctrl+C to stop)");

    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        None,
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    for event in events {
                        for path in &event.paths {
                            if path.file_name().and_then(|n| n.to_str()) == Some("props.rs") {
                                let _ = tx.send(path.clone());
                            }
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        eprintln!("Watch error: {:?}", error);
                    }
                }
            }
        },
    )?;

    let routes_dir = project_root.join("backend/src/routes");
    debouncer.watcher().watch(
        &routes_dir,
        RecursiveMode::Recursive,
    )?;

    // Keep the watcher alive and process events
    loop {
        match rx.recv() {
            Ok(changed_path) => {
                println!("\nFile changed: {}", changed_path.display());

                // Re-scan to get updated route info
                match scan_routes(&project_root) {
                    Ok(routes) => {
                        // Process the changed file
                        for route in &routes {
                            if route.props_path == changed_path {
                                if let Err(e) = process_props_file(route) {
                                    eprintln!("Error: {}", e);
                                }
                                break;
                            }
                        }

                        // Regenerate backend code
                        if let Err(e) = crate::codegen::generate_backend_code(&project_root, &routes, &config) {
                            eprintln!("Error generating backend code: {}", e);
                        }

                        // Regenerate frontend code
                        if let Err(e) = crate::codegen::generate_frontend_code(&project_root, &routes) {
                            eprintln!("Error generating frontend code: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error scanning routes: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_conversion() {
        let backend = Path::new("product/_id_");
        let frontend = convert_backend_to_frontend_path(backend).unwrap();
        assert_eq!(frontend, PathBuf::from("product/[id]"));
    }

    #[test]
    fn test_nested_path_conversion() {
        let backend = Path::new("user/_userId_/post/_postId_");
        let frontend = convert_backend_to_frontend_path(backend).unwrap();
        assert_eq!(frontend, PathBuf::from("user/[userId]/post/[postId]"));
    }
}
