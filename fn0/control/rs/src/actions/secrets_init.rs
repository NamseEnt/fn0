use crate::common::{auth, vault};
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok { encrypted_dek: String },
    Unauthorized,
    Error { message: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if auth::bearer_user(req.headers).await.is_none() {
        return Output::Unauthorized;
    }
    match vault::generate_data_encryption_key().await {
        Ok(ciphertext) => Output::Ok {
            encrypted_dek: ciphertext,
        },
        Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
