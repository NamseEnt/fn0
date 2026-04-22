use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::Engine;
use color_eyre::eyre::{Result, eyre};

pub fn decode_key_base64(key_base64: &str) -> Result<[u8; 32]> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(key_base64)
        .map_err(|e| eyre!("FN0_ENV_KEY_BASE64 is not valid base64: {e}"))?;
    let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| {
        eyre!(
            "FN0_ENV_KEY_BASE64 must decode to 32 bytes, got {}",
            v.len()
        )
    })?;
    Ok(arr)
}

pub fn decrypt(key: &[u8; 32], blob: &[u8]) -> Result<Vec<u8>> {
    if blob.len() < 12 {
        return Err(eyre!("env ciphertext too short"));
    }
    let (nonce_bytes, ciphertext) = blob.split_at(12);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| eyre!("env decryption failed: {e}"))
}

pub fn parse_env_file(content: &str) -> Vec<(String, String)> {
    let mut vars = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            vars.push((key.trim().to_string(), value.trim().to_string()));
        }
    }
    vars
}
