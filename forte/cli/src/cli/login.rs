use anyhow::{Result, anyhow};
use fn0_deploy::credentials::{self, Credentials};
use std::process::Command;

pub fn run(token_arg: Option<String>) -> Result<()> {
    let control_url = credentials::default_control_url();

    let token = match token_arg {
        Some(t) => t,
        None => paste_flow(&control_url)?,
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

fn paste_flow(control_url: &str) -> Result<String> {
    let tokens_url = format!("{}/tokens", control_url.trim_end_matches('/'));
    println!("Opening {tokens_url}");
    if let Err(err) = open_browser(&tokens_url) {
        eprintln!("(could not auto-open browser: {err}; open the URL manually)");
    }
    println!();
    println!("Paste the token below (input is hidden), then press Enter:");

    let token = inquire::Password::new("token:")
        .with_display_mode(inquire::PasswordDisplayMode::Masked)
        .without_confirmation()
        .prompt()?;
    Ok(token)
}

fn open_browser(url: &str) -> Result<()> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "xdg-open"
    };
    let mut command = Command::new(cmd);
    if cfg!(target_os = "windows") {
        command.args(["/C", "start", "", url]);
    } else {
        command.arg(url);
    }
    let status = command.spawn()?.wait()?;
    if !status.success() {
        return Err(anyhow!("browser open command exited with {status}"));
    }
    Ok(())
}
