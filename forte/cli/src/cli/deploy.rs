use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct ForteConfig {
    name: Option<String>,
}

pub async fn run(project_dir: PathBuf) -> Result<()> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;

    let project_name = config
        .name
        .ok_or_else(|| anyhow!("'name' field missing in Forte.toml"))?;

    let wasm_path = project_dir.join("dist/backend.wasm");

    fn0_deploy::deploy(&project_name, &wasm_path).await?;

    Ok(())
}
