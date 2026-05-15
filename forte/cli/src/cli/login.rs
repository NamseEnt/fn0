use anyhow::{Result, anyhow};
use fn0_deploy::credentials::{self, Credentials};

pub async fn run(token_arg: Option<String>) -> Result<()> {
    let control_url = credentials::default_control_url();

    let token = match token_arg {
        Some(t) => t,
        None => fn0_deploy::cli_login::login_pkce(&control_url).await?,
    };

    let trimmed = token.trim().to_string();
    if trimmed.is_empty() {
        return Err(anyhow!("empty token"));
    }
    if !trimmed.starts_with("fn0_") {
        return Err(anyhow!(
            "token does not look like an fn0 token (expected `fn0_…`)"
        ));
    }

    credentials::save(&Credentials {
        token: trimmed,
        control_url: control_url.clone(),
    })?;
    let path = credentials::path()?;
    println!("Saved credentials to {}", path.display());
    Ok(())
}
