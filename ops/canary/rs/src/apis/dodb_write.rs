use crate::common::known_values::{
    DOC_DB_WRITE_PARTITION_KEY, DOC_DB_WRITE_SORT_KEY, ProbeFailure, probe_response,
};
use anyhow::Result;
use forte_sdk::{ForteRequest, ForteResponse};
use std::fmt::Write;

pub type Props = ForteResponse;

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    let random_bytes = forte_sdk::rand::get_random_bytes(16);
    let mut nonce = String::with_capacity(random_bytes.len() * 2);
    for random_byte in random_bytes {
        write!(&mut nonce, "{random_byte:02x}")?;
    }
    let database = doc_db::database();
    if let Err(error) = database
        .put(
            DOC_DB_WRITE_PARTITION_KEY,
            DOC_DB_WRITE_SORT_KEY,
            nonce.as_bytes(),
        )
        .await
    {
        eprintln!("dodb write canary put failed: {error:?}");
        return probe_response(Err(ProbeFailure::WriteFailed));
    }
    let stored_value = match database
        .get(DOC_DB_WRITE_PARTITION_KEY, DOC_DB_WRITE_SORT_KEY)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            eprintln!("dodb write canary get failed: {error:?}");
            return probe_response(Err(ProbeFailure::ReadFailed));
        }
    };
    match stored_value.as_deref() {
        Some(value) if value == nonce.as_bytes() => probe_response(Ok(())),
        _ => probe_response(Err(ProbeFailure::Mismatch)),
    }
}
