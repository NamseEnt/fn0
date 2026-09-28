use crate::common::known_values::{DOC_DB_WRITE_PARTITION_KEY, ProbeFailure, probe_response};
use anyhow::Result;
use forte_sdk::{ForteRequest, ForteResponse};
use std::fmt::Write;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CLEANUP_MAX_KEYS: usize = 32;
const CLEANUP_SCAN_LIMIT: usize = 257;
const MAX_LIVE_KEYS: usize = 256;
const STALE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

fn sort_key(timestamp_millis: u128, nonce: &str) -> String {
    format!("{timestamp_millis:013}-{nonce}")
}

fn key_timestamp_millis(sort_key: &str) -> Option<u128> {
    sort_key.split_once('-')?.0.parse().ok()
}

async fn cleanup_stale_keys(database: &doc_db::Database, now_millis: u128) {
    let stale_before = now_millis.saturating_sub(STALE_AFTER.as_millis());
    let rows = match database
        .query::<&str, &str>(DOC_DB_WRITE_PARTITION_KEY, None, CLEANUP_SCAN_LIMIT)
        .await
    {
        Ok(rows) => rows,
        Err(error) => {
            eprintln!("dodb write canary cleanup query failed: {error:?}");
            return;
        }
    };
    let mut removed = 0;
    for (key, _) in rows.iter().take(CLEANUP_MAX_KEYS) {
        if key_timestamp_millis(key).is_some_and(|timestamp| timestamp < stale_before) {
            match database.delete(DOC_DB_WRITE_PARTITION_KEY, key).await {
                Ok(()) => removed += 1,
                Err(error) => eprintln!("dodb write canary cleanup delete failed: {error:?}"),
            }
        }
    }
    if removed > 0 {
        eprintln!("dodb write canary removed {removed} stale probe keys");
    }
    if rows.len() >= MAX_LIVE_KEYS {
        eprintln!("dodb write canary probe-key capacity reached");
    }
}

pub type Props = ForteResponse;

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    let random_bytes = forte_sdk::rand::get_random_bytes(16);
    let mut nonce = String::with_capacity(random_bytes.len() * 2);
    for random_byte in random_bytes {
        write!(&mut nonce, "{random_byte:02x}")?;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let timestamp_millis = now.as_millis();
    let key = sort_key(timestamp_millis, &nonce);
    let database = doc_db::database();
    cleanup_stale_keys(&database, timestamp_millis).await;
    let row_count = match database
        .query::<&str, &str>(DOC_DB_WRITE_PARTITION_KEY, None, MAX_LIVE_KEYS)
        .await
    {
        Ok(rows) => rows.len(),
        Err(error) => {
            eprintln!("dodb write canary capacity query failed: {error:?}");
            return probe_response(Err(ProbeFailure::Unavailable));
        }
    };
    if row_count >= MAX_LIVE_KEYS {
        return probe_response(Err(ProbeFailure::Unavailable));
    }
    if let Err(error) = database
        .put(DOC_DB_WRITE_PARTITION_KEY, &key, nonce.as_bytes())
        .await
    {
        eprintln!("dodb write canary put failed: {error:?}");
        return probe_response(Err(ProbeFailure::WriteFailed));
    }
    let stored_value = match database.get(DOC_DB_WRITE_PARTITION_KEY, &key).await {
        Ok(value) => value,
        Err(error) => {
            eprintln!("dodb write canary get failed: {error:?}");
            if let Err(cleanup_error) = database.delete(DOC_DB_WRITE_PARTITION_KEY, &key).await {
                eprintln!("dodb write canary cleanup delete failed: {cleanup_error:?}");
            }
            return probe_response(Err(ProbeFailure::ReadFailed));
        }
    };
    let result = match stored_value.as_deref() {
        Some(value) if value == nonce.as_bytes() => probe_response(Ok(())),
        _ => probe_response(Err(ProbeFailure::Mismatch)),
    };
    if let Err(error) = database.delete(DOC_DB_WRITE_PARTITION_KEY, &key).await {
        eprintln!("dodb write canary cleanup delete failed: {error:?}");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn concurrent_probe_keys_do_not_overwrite_each_other() {
        let database = Arc::new(Mutex::new(HashMap::<String, Vec<u8>>::new()));
        let probe_count = 64;
        let mut workers = Vec::with_capacity(probe_count);
        for probe_index in 0..probe_count {
            let database = Arc::clone(&database);
            workers.push(thread::spawn(move || {
                let nonce = format!("{probe_index:032x}");
                let key = sort_key(1_800_000_000_000, &nonce);
                let value = nonce.into_bytes();
                database
                    .lock()
                    .expect("canary test database")
                    .insert(key.clone(), value.clone());
                let stored = database
                    .lock()
                    .expect("canary test database")
                    .get(&key)
                    .cloned();
                assert_eq!(stored.as_deref(), Some(value.as_slice()));
            }));
        }
        for worker in workers {
            worker.join().expect("concurrent canary probe");
        }
    }
}
