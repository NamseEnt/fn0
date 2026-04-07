use color_eyre::{eyre::eyre, Result};
use std::path::Path;

pub async fn execute() -> Result<()> {
    let config = crate::config::Config::load("fn0.toml")
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;

    let project_name = config
        .name
        .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))?;

    fn0_deploy::deploy(&project_name, Path::new("dist/component.wasm"))
        .await
        .map_err(|e| eyre!(e))?;

    Ok(())
}
