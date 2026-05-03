use crate::common::{auth, vault};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub encrypted_dek: String,
    pub value: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok { ciphertext: String },
    Unauthorized,
    Error { message: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if auth::bearer_user(req.headers).await.is_none() {
        return Output::Unauthorized;
    }

    let dek = match vault::decrypt_dek(&req.body.encrypted_dek).await {
        Ok(d) => d,
        Err(e) => {
            return Output::Error {
                message: format!("vault decrypt: {e}"),
            };
        }
    };
    if dek.len() != 32 {
        return Output::Error {
            message: format!("DEK must be 32 bytes, got {}", dek.len()),
        };
    }
    let key: [u8; 32] = dek[..].try_into().expect("len-checked above");

    let nonce_bytes = rand::get_random_bytes(12).await;
    let Ok(nonce_arr): Result<[u8; 12], _> = nonce_bytes.as_slice().try_into() else {
        return Output::Error {
            message: "rng returned wrong length".to_string(),
        };
    };
    let nonce = Nonce::from_slice(&nonce_arr);

    let cipher = Aes256Gcm::new((&key).into());
    let ciphertext = match cipher.encrypt(nonce, req.body.value.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            return Output::Error {
                message: format!("aes-gcm encrypt: {e}"),
            };
        }
    };

    let mut blob = Vec::with_capacity(12 + ciphertext.len());
    blob.extend_from_slice(&nonce_arr);
    blob.extend_from_slice(&ciphertext);
    Output::Ok {
        ciphertext: STANDARD.encode(blob),
    }
}
