use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use inquire::Password;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::process::Command;

const RESPONSE_MODE: &str = "code";

pub async fn login_pkce(control_url: &str) -> Result<String> {
    let mut verifier_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut verifier_bytes);
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    let label = hostname::get()
        .ok()
        .and_then(|host| host.into_string().ok())
        .unwrap_or_else(|| "cli".to_string());

    let trimmed_control = control_url.trim_end_matches('/');
    let authorize_url = build_authorize_url(trimmed_control, &code_challenge, &label);

    println!("Open this URL in any browser:\n\n{authorize_url}");
    if should_open_browser()
        && let Err(error) = open_browser(&authorize_url)
    {
        eprintln!("Could not open a browser automatically: {error}");
    }
    println!();
    println!(
        "After approval, copy the one-time authorization code here. It expires in five minutes."
    );
    let code = Password::new("Authorization code")
        .without_confirmation()
        .prompt()?;
    let code = code.trim();
    if code.is_empty() {
        return Err(anyhow!("authorization code cannot be empty"));
    }

    exchange_code(trimmed_control, code, &code_verifier).await
}

fn build_authorize_url(control_url: &str, code_challenge: &str, label: &str) -> String {
    format!(
        "{control_url}/oauth/cli/authorize?response_mode={RESPONSE_MODE}&code_challenge={}&code_challenge_method=S256&label={}",
        urlencoding::encode(code_challenge),
        urlencoding::encode(label),
    )
}

async fn exchange_code(control_url: &str, code: &str, code_verifier: &str) -> Result<String> {
    let exchange_url = format!(
        "{}/__forte_action/oauth_cli_exchange",
        control_url.trim_end_matches('/')
    );
    let exchange_body = ExchangeInput {
        code: code.to_string(),
        code_verifier: code_verifier.to_string(),
    };
    let resp = reqwest::Client::new()
        .post(&exchange_url)
        .json(&exchange_body)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!("exchange failed with status {}", resp.status()));
    }
    let parsed: ExchangeOutput = resp.json().await?;
    match parsed {
        ExchangeOutput::Ok { token } => Ok(token),
        ExchangeOutput::InvalidGrant { message } => Err(anyhow!("invalid_grant: {message}")),
        ExchangeOutput::Error { message } => Err(anyhow!("exchange error: {message}")),
    }
}

fn should_open_browser() -> bool {
    if std::env::var_os("SSH_CONNECTION").is_some()
        || std::env::var_os("SSH_TTY").is_some()
        || std::env::var_os("CI").is_some()
    {
        return false;
    }
    if cfg!(target_os = "linux") {
        return std::env::var_os("DISPLAY").is_some()
            || std::env::var_os("WAYLAND_DISPLAY").is_some();
    }
    true
}

fn open_browser(url: &str) -> Result<()> {
    let mut command = if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(url);
        command
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", url]);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };
    let status = command.status()?;
    if !status.success() {
        return Err(anyhow!("browser open command exited with {status}"));
    }
    Ok(())
}

#[derive(Serialize)]
struct ExchangeInput {
    code: String,
    code_verifier: String,
}

#[derive(Deserialize)]
#[serde(tag = "t")]
enum ExchangeOutput {
    Ok { token: String },
    InvalidGrant { message: String },
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::{build_authorize_url, exchange_code};
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn authorize_url_uses_manual_code_mode_without_a_loopback_callback() {
        let url = build_authorize_url("https://fn0.dev", "challenge", "remote-cli");
        assert!(url.contains("response_mode=code"));
        assert!(url.contains("code_challenge=challenge"));
        assert!(!url.contains("redirect_uri="));
        assert!(!url.contains("state="));
    }

    #[tokio::test]
    async fn exchange_sends_only_code_and_verifier() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/__forte_action/oauth_cli_exchange"))
            .and(body_json(json!({
                "code": "authorization-code",
                "code_verifier": "code-verifier"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "t": "Ok",
                "token": "fn0_test"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let token = exchange_code(&server.uri(), "authorization-code", "code-verifier")
            .await
            .unwrap();
        assert_eq!(token, "fn0_test");
        server.verify().await;
    }
}
