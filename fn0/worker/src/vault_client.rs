use anyhow::{Result, anyhow};
use base64::Engine;
use oci_rust_sdk::auth::{RequestSigner, SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct VaultClient {
    crypto_endpoint: String,
    key_ocid: String,
    signer: Arc<RequestSigner>,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct DecryptRequest<'a> {
    ciphertext: &'a str,
    #[serde(rename = "keyId")]
    key_id: &'a str,
}

#[derive(Deserialize)]
struct DecryptResponse {
    plaintext: String,
}

impl VaultClient {
    pub fn from_env() -> Result<Self> {
        let crypto_endpoint = std::env::var("FN0_VAULT_CRYPTO_ENDPOINT")
            .map_err(|_| anyhow!("FN0_VAULT_CRYPTO_ENDPOINT is required"))?;
        let key_ocid = std::env::var("FN0_VAULT_KEY_OCID")
            .map_err(|_| anyhow!("FN0_VAULT_KEY_OCID is required"))?;
        let tenancy = std::env::var("FN0_VAULT_OCI_TENANCY_ID")
            .map_err(|_| anyhow!("FN0_VAULT_OCI_TENANCY_ID is required"))?;
        let user = std::env::var("FN0_VAULT_OCI_USER_ID")
            .map_err(|_| anyhow!("FN0_VAULT_OCI_USER_ID is required"))?;
        let fingerprint = std::env::var("FN0_VAULT_OCI_FINGERPRINT")
            .map_err(|_| anyhow!("FN0_VAULT_OCI_FINGERPRINT is required"))?;
        let private_key_b64 = std::env::var("FN0_VAULT_OCI_PRIVATE_KEY_BASE64")
            .map_err(|_| anyhow!("FN0_VAULT_OCI_PRIVATE_KEY_BASE64 is required"))?;
        let private_key_pem = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(private_key_b64.as_bytes())
                .map_err(|e| anyhow!("vault private key base64: {e}"))?,
        )
        .map_err(|e| anyhow!("vault private key utf8: {e}"))?;

        let provider: Arc<SimpleAuthProvider> = Arc::new(
            SimpleAuthProvider::builder(SimpleAuthProviderRequiredFields {
                tenancy,
                user,
                fingerprint,
                private_key: private_key_pem,
            })
            .build(),
        );
        let signer = Arc::new(
            RequestSigner::new(provider as Arc<dyn oci_rust_sdk::auth::AuthProvider>)
                .map_err(|e| anyhow!("vault signer init: {e:?}"))?,
        );

        Ok(Self {
            crypto_endpoint,
            key_ocid,
            signer,
            client: reqwest::Client::new(),
        })
    }

    pub async fn decrypt(&self, ciphertext_b64: &str) -> Result<Vec<u8>> {
        let body = serde_json::to_vec(&DecryptRequest {
            ciphertext: ciphertext_b64,
            key_id: &self.key_ocid,
        })?;

        let url_str = format!(
            "{}/20180608/decrypt",
            self.crypto_endpoint.trim_end_matches('/')
        );
        let url = url::Url::parse(&url_str).map_err(|e| anyhow!("vault url parse: {e}"))?;

        let mut headers = reqwest::header::HeaderMap::new();
        self.signer
            .sign_request("POST", &url, &mut headers, Some(&body))
            .map_err(|e| anyhow!("vault sign: {e:?}"))?;

        let resp = self
            .client
            .post(url)
            .headers(headers)
            .body(body)
            .send()
            .await?;
        let resp = resp.error_for_status()?;
        let parsed: DecryptResponse = resp.json().await?;
        base64::engine::general_purpose::STANDARD
            .decode(parsed.plaintext.as_bytes())
            .map_err(|e| anyhow!("vault decrypt response base64: {e}"))
    }
}
