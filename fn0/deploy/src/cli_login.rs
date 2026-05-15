use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::process::Command;
use std::time::Duration;

const PKCE_TIMEOUT_SECS: u64 = 300;

pub async fn login_pkce(control_url: &str) -> Result<String> {
    let listener = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        0,
    )))?;
    let local_addr = listener.local_addr()?;
    let port = local_addr.port();
    listener.set_nonblocking(false)?;

    let mut verifier_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut verifier_bytes);
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    let mut state_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut state_bytes);
    let state = URL_SAFE_NO_PAD.encode(state_bytes);

    let label = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "cli".to_string());

    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let trimmed_control = control_url.trim_end_matches('/');
    let authorize_url = format!(
        "{trimmed_control}/oauth/cli/authorize?redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}&label={}",
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&code_challenge),
        urlencoding::encode(&state),
        urlencoding::encode(&label),
    );

    println!("Opening {authorize_url}");
    if let Err(err) = open_browser(&authorize_url) {
        eprintln!("(could not auto-open browser: {err}; open the URL manually)");
    }
    println!();
    println!("Waiting for browser approval (timeout {PKCE_TIMEOUT_SECS}s)…");

    let callback = tokio::task::spawn_blocking(move || -> Result<CallbackParams> {
        listener.set_nonblocking(false)?;
        let (mut stream, _peer) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(PKCE_TIMEOUT_SECS)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;
        let params = read_callback_params(&mut stream)?;
        write_callback_response(&mut stream, &params)?;
        Ok(params)
    });

    let callback = tokio::time::timeout(
        Duration::from_secs(PKCE_TIMEOUT_SECS),
        callback,
    )
    .await
    .map_err(|_| anyhow!("timed out waiting for browser callback"))?
    .map_err(|e| anyhow!("callback task panicked: {e}"))??;

    if callback.state != state {
        return Err(anyhow!("state mismatch in browser callback (possible CSRF)"));
    }

    let exchange_url = format!("{trimmed_control}/__forte_action/oauth_cli_exchange");
    let exchange_body = ExchangeInput {
        code: callback.code,
        code_verifier,
        redirect_uri,
    };
    let resp = reqwest::Client::new()
        .post(&exchange_url)
        .json(&exchange_body)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "exchange failed with status {}",
            resp.status()
        ));
    }
    let parsed: ExchangeOutput = resp.json().await?;
    match parsed {
        ExchangeOutput::Ok { token } => Ok(token),
        ExchangeOutput::InvalidGrant { message } => {
            Err(anyhow!("invalid_grant: {message}"))
        }
        ExchangeOutput::Error { message } => Err(anyhow!("exchange error: {message}")),
    }
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

struct CallbackParams {
    code: String,
    state: String,
}

fn read_callback_params(stream: &mut TcpStream) -> Result<CallbackParams> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Err(anyhow!("empty callback request"));
    }
    let request_text = std::str::from_utf8(&buf[..n])
        .map_err(|_| anyhow!("non-utf8 callback request"))?;
    let request_line = request_text
        .lines()
        .next()
        .ok_or_else(|| anyhow!("missing request line"))?;
    let path_and_query = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("malformed request line"))?;
    let query = path_and_query
        .split_once('?')
        .map(|x| x.1)
        .ok_or_else(|| anyhow!("missing query string in callback"))?;
    let mut params: HashMap<String, String> = HashMap::new();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let decoded_value = urlencoding::decode(v)
                .map_err(|e| anyhow!("invalid url-encoded value: {e}"))?
                .into_owned();
            params.insert(k.to_string(), decoded_value);
        }
    }
    let code = params
        .remove("code")
        .ok_or_else(|| anyhow!("callback missing `code` parameter"))?;
    let state = params
        .remove("state")
        .ok_or_else(|| anyhow!("callback missing `state` parameter"))?;
    Ok(CallbackParams { code, state })
}

fn write_callback_response(stream: &mut TcpStream, _params: &CallbackParams) -> Result<()> {
    let body = "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>fn0 CLI login</title></head><body style=\"font-family:system-ui;max-width:420px;margin:3rem auto\"><h1>Authorized.</h1><p>You can close this tab and return to the terminal.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body,
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

#[derive(Serialize)]
struct ExchangeInput {
    code: String,
    code_verifier: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
#[serde(tag = "t")]
enum ExchangeOutput {
    Ok { token: String },
    InvalidGrant { message: String },
    Error { message: String },
}
