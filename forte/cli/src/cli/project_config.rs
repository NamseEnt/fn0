use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct ForteConfig {
    project_id: Option<String>,
}

pub fn read_project_id(project_dir: &Path) -> Result<String> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;
    config
        .project_id
        .ok_or_else(|| anyhow!("'project_id' field missing in Forte.toml. Run `forte deploy` first to register the project."))
}
