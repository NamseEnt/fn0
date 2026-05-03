use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use color_eyre::eyre::{Result, eyre};

pub fn decrypt_with_dek(dek: &[u8], blob: &[u8]) -> Result<Vec<u8>> {
    if dek.len() != 32 {
        return Err(eyre!("DEK must be 32 bytes, got {}", dek.len()));
    }
    if blob.len() < 12 {
        return Err(eyre!("ciphertext too short"));
    }
    let (nonce_bytes, ciphertext) = blob.split_at(12);
    let key: &[u8; 32] = dek.try_into().expect("len-checked above");
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| eyre!("AES-GCM decryption failed: {e}"))
}
