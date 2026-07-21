use color_eyre::{Result, eyre::eyre};

fn project_id() -> Result<String> {
    let config = crate::config::Config::load("fn0.toml")
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;
    config.project_id.clone().ok_or_else(|| {
        eyre!("no project_id in fn0.toml; run `fn0 deploy` first to create the project.")
    })
}

pub async fn add(domain: &str) -> Result<()> {
    let id = project_id()?;
    fn0_deploy::domain_add(&id, domain)
        .await
        .map_err(|e| eyre!(e))
}

pub async fn remove() -> Result<()> {
    let id = project_id()?;
    fn0_deploy::domain_remove(&id).await.map_err(|e| eyre!(e))
}

pub async fn status() -> Result<()> {
    let id = project_id()?;
    fn0_deploy::domain_status(&id).await.map_err(|e| eyre!(e))
}
