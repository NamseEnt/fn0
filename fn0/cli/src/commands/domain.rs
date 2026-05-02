use color_eyre::{Result, eyre::eyre};

fn project_name() -> Result<String> {
    let config = crate::config::Config::load("fn0.toml")
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;
    config
        .name
        .clone()
        .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))
}

pub async fn add(domain: &str) -> Result<()> {
    let name = project_name()?;
    fn0_deploy::domain_add(&name, domain)
        .await
        .map_err(|e| eyre!(e))
}

pub async fn remove() -> Result<()> {
    let name = project_name()?;
    fn0_deploy::domain_remove(&name).await.map_err(|e| eyre!(e))
}

pub async fn status() -> Result<()> {
    let name = project_name()?;
    fn0_deploy::domain_status(&name).await.map_err(|e| eyre!(e))
}
