use std::time::Duration;

pub async fn random_sleep(ms: u64) {
    tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % ms)).await;
}
