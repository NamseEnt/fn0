use crate::env_crypto;
use crate::vault_client::VaultClient;
use anyhow::{Result, anyhow};
use base64::Engine;

const DEK_KEY: &str = "__dek";

pub async fn load(bytes: &[u8], vault: &VaultClient) -> Result<Vec<(String, String)>> {
    let mapping: serde_yaml::Mapping =
        serde_yaml::from_slice(bytes).map_err(|e| anyhow!("env.yaml parse: {e}"))?;

    let dek_value = mapping
        .get(DEK_KEY)
        .ok_or_else(|| anyhow!("env.yaml missing __dek"))?;
    let dek_encrypted = dek_value
        .as_mapping()
        .and_then(|m| m.get("encrypted"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("env.yaml __dek.encrypted missing or not a string"))?;

    let dek = vault
        .decrypt(dek_encrypted)
        .await
        .map_err(|e| anyhow!("vault decrypt of __dek failed: {e}"))?;

    let mut out = Vec::with_capacity(mapping.len());
    for (key_v, value_v) in &mapping {
        let key = key_v
            .as_str()
            .ok_or_else(|| anyhow!("env.yaml key is not a string"))?;
        if key == DEK_KEY {
            continue;
        }
        let value = match value_v {
            serde_yaml::Value::String(s) => s.clone(),
            serde_yaml::Value::Mapping(m) => {
                let secret_b64 = m
                    .get("secret")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("env.yaml entry {key} missing 'secret' string"))?;
                let blob = base64::engine::general_purpose::STANDARD
                    .decode(secret_b64.as_bytes())
                    .map_err(|e| anyhow!("env.yaml entry {key} base64: {e}"))?;
                let plaintext = env_crypto::decrypt_with_dek(&dek, &blob)
                    .map_err(|e| anyhow!("env.yaml entry {key} decrypt: {e}"))?;
                String::from_utf8(plaintext)
                    .map_err(|e| anyhow!("env.yaml entry {key} utf8: {e}"))?
            }
            _ => {
                return Err(anyhow!(
                    "env.yaml entry {key} must be a scalar string or a mapping with 'secret'"
                ));
            }
        };
        out.push((key.to_string(), value));
    }

    Ok(out)
}
