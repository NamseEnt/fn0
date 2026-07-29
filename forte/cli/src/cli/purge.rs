use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct ForteConfig {
    project_id: Option<String>,
}

fn load_project_id(project_dir: &std::path::Path) -> Result<String> {
    let config_path = project_dir.join("Forte.toml");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|_| anyhow!("Forte.toml not found. Are you in a Forte project directory?"))?;
    let config: ForteConfig =
        toml::from_str(&content).map_err(|e| anyhow!("Failed to parse Forte.toml: {}", e))?;
    config.project_id.ok_or_else(|| {
        anyhow!("'project_id' field missing in Forte.toml. Run `forte deploy` first.")
    })
}

pub async fn run(keys: Vec<String>, project_dir: PathBuf) -> Result<()> {
    let project_id = load_project_id(&project_dir)?;
    let urls = fn0_deploy::public_purge(&project_id, &keys).await?;
    for url in &urls {
        println!("{url}");
    }
    println!("queued {} invalidation(s)", urls.len());
    Ok(())
}
