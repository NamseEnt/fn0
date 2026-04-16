use color_eyre::{eyre::eyre, Result};
use std::path::{Path, PathBuf};

pub async fn execute() -> Result<()> {
    let config = crate::config::Config::load("fn0.toml")
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;

    let project_name = config
        .name
        .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))?;

    let wasm_path = PathBuf::from("dist/component.wasm");
    let dist_dir = Path::new("dist");
    let bundle_path = dist_dir.join("bundle.raw.tar");

    fn0_deploy::create_raw_bundle_wasm(&wasm_path, &bundle_path)
        .map_err(|e| eyre!(e))?;

    fn0_deploy::deploy(&project_name, &bundle_path, None)
        .await
        .map_err(|e| eyre!(e))?;

    Ok(())
}
