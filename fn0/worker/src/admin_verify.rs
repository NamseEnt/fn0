use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub struct AdminTokenClaims {
    pub project_id: String,
    pub task: String,
    pub github_login: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug)]
pub enum VerifyError {
    Malformed,
    BadSignature,
    Expired,
    ProjectIdMismatch,
}

pub fn decode_key_base64(s: &str) -> Result<[u8; 32], String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| format!("not valid base64: {e}"))?;
    bytes
        .try_into()
        .map_err(|v: Vec<u8>| format!("must be 32 bytes, got {}", v.len()))
}

pub fn verify_token(
    token: &str,
    key: &[u8; 32],
    expected_project_id: &str,
    now_unix: i64,
) -> Result<AdminTokenClaims, VerifyError> {
    let (payload_b64, sig_b64) = token.split_once('.').ok_or(VerifyError::Malformed)?;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| VerifyError::Malformed)?;

    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
    mac.update(payload_b64.as_bytes());
    mac.verify_slice(&sig_bytes)
        .map_err(|_| VerifyError::BadSignature)?;

    let payload_json = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| VerifyError::Malformed)?;
    let claims: AdminTokenClaims =
        serde_json::from_slice(&payload_json).map_err(|_| VerifyError::Malformed)?;

    if claims.exp < now_unix {
        return Err(VerifyError::Expired);
    }
    if claims.project_id != expected_project_id {
        return Err(VerifyError::ProjectIdMismatch);
    }
    Ok(claims)
}
