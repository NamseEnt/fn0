use crate::utils::credentials::{self, Credentials};
use color_eyre::Result;
use color_eyre::eyre::eyre;

pub async fn execute(token_arg: Option<String>) -> Result<()> {
    let control_url = credentials::default_control_url();

    let token = match token_arg {
        Some(t) => t,
        None => fn0_deploy::cli_login::login_pkce(&control_url)
            .await
            .map_err(|e| eyre!("{e:#}"))?,
    };

    let trimmed = token.trim().to_string();
    if trimmed.is_empty() {
        return Err(eyre!("empty token"));
    }
    if !trimmed.starts_with("fn0_") {
        return Err(eyre!(
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
