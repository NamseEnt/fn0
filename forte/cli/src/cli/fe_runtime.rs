use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn is_new_mode(project_dir: &Path) -> bool {
    project_dir.join("fe/src/app.tsx").exists()
}

pub fn forte_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("fe/.forte")
}

pub fn ssr_entry(project_dir: &Path) -> PathBuf {
    if is_new_mode(project_dir) {
        forte_dir(project_dir).join("server.tsx")
    } else {
        project_dir.join("fe/src/server.tsx")
    }
}

pub fn vite_config(project_dir: &Path) -> Option<PathBuf> {
    if is_new_mode(project_dir) {
        Some(forte_dir(project_dir).join("vite.config.ts"))
    } else {
        None
    }
}

pub fn routes_generated(project_dir: &Path) -> PathBuf {
    if is_new_mode(project_dir) {
        forte_dir(project_dir).join("routes.generated.ts")
    } else {
        project_dir.join("fe/src/routes.generated.ts")
    }
}

pub fn page_import_prefix(project_dir: &Path) -> &'static str {
    if is_new_mode(project_dir) {
        "../src"
    } else {
        "."
    }
}

pub fn ssr_load_module_path(project_dir: &Path) -> &'static str {
    if is_new_mode(project_dir) {
        "/.forte/server.tsx"
    } else {
        "/src/server.tsx"
    }
}

pub fn ensure(project_dir: &Path) -> Result<()> {
    if !is_new_mode(project_dir) {
        return Ok(());
    }
    let dir = forte_dir(project_dir);
    fs::create_dir_all(&dir)?;
    write_if_changed(&dir.join("server.tsx"), SERVER_TSX)?;
    write_if_changed(&dir.join("client.tsx"), CLIENT_TSX)?;
    write_if_changed(&dir.join("vite.config.ts"), VITE_CONFIG_TS)?;
    write_if_changed(&dir.join("forte-react.ts"), FORTE_REACT_TS)?;
    Ok(())
}

fn write_if_changed(path: &Path, content: &str) -> Result<()> {
    if let Ok(existing) = fs::read_to_string(path)
        && existing == content
    {
        return Ok(());
    }
    fs::write(path, content)?;
    Ok(())
}

const SERVER_TSX: &str = include_str!("fe_runtime/server.tsx");
const CLIENT_TSX: &str = include_str!("fe_runtime/client.tsx");
const VITE_CONFIG_TS: &str = include_str!("fe_runtime/vite.config.ts");
const FORTE_REACT_TS: &str = include_str!("fe_runtime/forte-react.ts");
