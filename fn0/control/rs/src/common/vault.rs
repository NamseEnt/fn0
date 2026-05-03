use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct DecryptRequest<'a> {
    ciphertext: &'a str,
}

#[derive(Deserialize)]
struct DecryptResponse {
    plaintext: String,
}

#[derive(Deserialize)]
struct GenerateDekResponse {
    ciphertext: String,
}

fn vault_url() -> anyhow::Result<String> {
    std::env::var("FN0_VAULT_URL")
        .map_err(|_| anyhow::anyhow!("FN0_VAULT_URL not set"))
}

pub async fn generate_data_encryption_key() -> anyhow::Result<String> {
    let url = format!(
        "{}/generate-dek",
        vault_url()?.trim_end_matches('/')
    );
    let resp = http::Client::new()
        .send(
            http::Request::builder()
                .uri(url)
                .method("POST")
                .header("Content-Type", "application/json")
                .body(b"{}".to_vec())?,
        )
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("vault generate-dek failed: {}", resp.status());
    }
    let parsed: GenerateDekResponse = resp.into_body().json().await?;
    Ok(parsed.ciphertext)
}

pub async fn decrypt_dek(ciphertext_b64: &str) -> anyhow::Result<Vec<u8>> {
    let url = format!("{}/decrypt", vault_url()?.trim_end_matches('/'));
    let body = serde_json::to_vec(&DecryptRequest {
        ciphertext: ciphertext_b64,
    })?;

    let resp = http::Client::new()
        .send(
            http::Request::builder()
                .uri(url)
                .method("POST")
                .header("Content-Type", "application/json")
                .body(body)?,
        )
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("vault decrypt failed: {}", resp.status());
    }
    let parsed: DecryptResponse = resp.into_body().json().await?;
    let plaintext = STANDARD
        .decode(parsed.plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("vault decrypt response base64: {e}"))?;
    Ok(plaintext)
}
