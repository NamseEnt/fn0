use crate::config::Config;
use color_eyre::{Result, eyre::eyre};
use std::path::PathBuf;

pub async fn execute(new_name: &str) -> Result<()> {
    let new_name = new_name.trim();
    if let Err(e) = fn0_deploy::validate_project_name(new_name) {
        return Err(eyre!(e));
    }

    let config_path = PathBuf::from("fn0.toml");
    let mut config = Config::load(&config_path)
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;

    let project_id = config.project_id.clone().ok_or_else(|| {
        eyre!("no project_id in fn0.toml; run `fn0 deploy` first to create the project.")
    })?;

    fn0_deploy::rename_project(&project_id, new_name)
        .await
        .map_err(|e| eyre!(e))?;

    config.name = Some(new_name.to_string());
    config.save(&config_path)?;
    println!("Renamed project '{project_id}' to '{new_name}' (fn0.toml updated)");
    Ok(())
}
