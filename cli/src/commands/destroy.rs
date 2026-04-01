use color_eyre::{eyre::eyre, Result};

use super::deploy::{HQ_URL, get_github_token};

pub async fn execute() -> Result<()> {
    let config = crate::config::Config::load("fn0.toml")
        .map_err(|_| eyre!("fn0.toml not found. Run 'fn0 init' first."))?;

    let project_name = config
        .name
        .ok_or_else(|| eyre!("'name' field missing in fn0.toml"))?;

    let github_token = get_github_token().await?;

    let client = reqwest::Client::new();

    println!("Destroying deployment...");
    let resp = client
        .post(format!("{}/deploy/destroy", HQ_URL))
        .json(&serde_json::json!({
            "github_token": github_token,
            "project_name": project_name,
        }))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| eyre!("Destroy failed: {}", e))?;

    let body: serde_json::Value = resp.json().await?;
    if let Some(subdomain) = body.get("subdomain").and_then(|s| s.as_str()) {
        println!("Destroyed: {}.fn0.dev", subdomain);
    }

    println!("Destroy complete!");

    Ok(())
}
