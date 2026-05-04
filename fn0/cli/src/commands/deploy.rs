use crate::config::Config;
use crate::utils::credentials;
use color_eyre::{Result, eyre::eyre};
use std::path::{Path, PathBuf};

pub async fn execute() -> Result<()> {
    let config_path = PathBuf::from("fn0.toml");
    let mut config = Config::load(&config_path)
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;

    let project_name = config
        .name
        .clone()
        .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))?;

    let creds = credentials::require()?;

    let wasm_path = PathBuf::from("dist/component.wasm");
    let dist_dir = Path::new("dist");
    let bundle_path = dist_dir.join("bundle.raw.tar");

    let env_yaml = fn0_deploy::read_env_yaml(Path::new(".")).map_err(|e| eyre!(e))?;
    fn0_deploy::create_raw_bundle_wasm(&wasm_path, env_yaml.as_deref(), &bundle_path)
        .map_err(|e| eyre!(e))?;

    let mut project_id = config.project_id.clone();
    fn0_deploy::deploy(
        &creds.control_url,
        &creds.token,
        &project_name,
        &mut project_id,
        &bundle_path,
    )
    .await
    .map_err(|e| eyre!(e))?;

    if config.project_id != project_id {
        config.project_id = project_id;
        config.save(&config_path)?;
        println!("Saved project_id to fn0.toml");
    }

    Ok(())
}
