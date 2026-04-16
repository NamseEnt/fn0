use libsql::Row;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::*;

const BUSY_RETRY_BACKOFFS: [Duration; 5] = [
    Duration::from_millis(20),
    Duration::from_millis(50),
    Duration::from_millis(100),
    Duration::from_millis(200),
    Duration::from_millis(500),
];

fn is_busy(err: &libsql::Error) -> bool {
    let msg = err.to_string();
    msg.contains("SQLITE_BUSY") || msg.contains("database is locked")
}

async fn with_busy_retry<F, Fut, T>(mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0usize;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if is_busy(&e) && attempt < BUSY_RETRY_BACKOFFS.len() => {
                tokio::time::sleep(BUSY_RETRY_BACKOFFS[attempt]).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostStatus {
    pub host_id: String,
    pub site_name: String,
    pub addr: String,
    pub dns_addr: Option<String>,
    #[serde(default)]
    pub generation: Option<u64>,
    #[serde(default)]
    pub instances: Option<u64>,
    #[serde(default)]
    pub healthy: bool,
    #[serde(default)]
    pub consecutive_failures: u32,
    #[serde(default)]
    pub last_verified_at: Option<String>,
    #[serde(default)]
    pub graceful: bool,
    #[serde(default)]
    pub graceful_purpose: Option<GracefulPurpose>,
    #[serde(default)]
    pub graceful_since_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GracefulPurpose {
    Terminate,
}

fn pk(host_id: &str) -> String {
    format!("host-status:{host_id}")
}

impl DocDb {
    pub async fn upsert_host_status_if_absent(
        &self,
        host_id: &str,
        site_name: &str,
        addr: &str,
        dns_addr: Option<&str>,
    ) -> Result<()> {
        with_busy_retry(|| async {
            let new = HostStatus {
                host_id: host_id.to_string(),
                site_name: site_name.to_string(),
                addr: addr.to_string(),
                dns_addr: dns_addr.map(|s| s.to_string()),
                generation: None,
                instances: None,
                healthy: false,
                consecutive_failures: 0,
                last_verified_at: None,
                graceful: false,
                graceful_purpose: None,
                graceful_since_at: None,
            };
            let conn = self.db.connect()?;
            conn.execute(
                "INSERT OR IGNORE INTO docs (pk, sk, value) VALUES (?, 0, ?)",
                libsql::params![pk(host_id), serde_json::to_string(&new).unwrap()],
            )
            .await?;
            Ok(())
        })
        .await
    }

    pub async fn get_host_status(&self, host_id: &str) -> Result<Option<HostStatus>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? AND sk = 0",
                libsql::params![pk(host_id)],
            )
            .await?;
        Ok(rows.next().await?.map(|row| row.into()))
    }

    pub async fn list_host_statuses(&self, site_name: &str) -> Result<Vec<HostStatus>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk LIKE 'host-status:%' AND sk = 0",
                libsql::params!(),
            )
            .await?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await? {
            let s: HostStatus = row.into();
            if s.site_name == site_name {
                out.push(s);
            }
        }
        Ok(out)
    }

    pub async fn delete_host_status(&self, host_id: &str) -> Result<()> {
        with_busy_retry(|| async {
            let conn = self.db.connect()?;
            conn.execute(
                "DELETE FROM docs WHERE pk = ? AND sk = 0",
                libsql::params![pk(host_id)],
            )
            .await?;
            Ok(())
        })
        .await
    }

    pub async fn mark_host_verified(
        &self,
        host_id: &str,
        generation: u64,
        instances: u64,
        now_rfc3339: &str,
    ) -> Result<()> {
        with_busy_retry(|| async {
            let existing = self.get_host_status(host_id).await?;
            let Some(mut s) = existing else { return Ok(()) };
            s.generation = Some(generation);
            s.instances = Some(instances);
            s.healthy = true;
            s.consecutive_failures = 0;
            s.last_verified_at = Some(now_rfc3339.to_string());
            let conn = self.db.connect()?;
            conn.execute(
                "REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)",
                libsql::params![pk(host_id), serde_json::to_string(&s).unwrap()],
            )
            .await?;
            Ok(())
        })
        .await
    }

    pub async fn mark_host_failed(&self, host_id: &str) -> Result<u32> {
        with_busy_retry(|| async {
            let Some(mut s) = self.get_host_status(host_id).await? else {
                return Ok(0);
            };
            s.healthy = false;
            s.consecutive_failures = s.consecutive_failures.saturating_add(1);
            let failures = s.consecutive_failures;
            let conn = self.db.connect()?;
            conn.execute(
                "REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)",
                libsql::params![pk(host_id), serde_json::to_string(&s).unwrap()],
            )
            .await?;
            Ok(failures)
        })
        .await
    }

    pub async fn set_host_graceful(
        &self,
        host_id: &str,
        purpose: GracefulPurpose,
        now_rfc3339: &str,
    ) -> Result<bool> {
        with_busy_retry(|| async {
            let Some(mut s) = self.get_host_status(host_id).await? else {
                return Ok(false);
            };
            if s.graceful {
                return Ok(false);
            }
            s.graceful = true;
            s.graceful_purpose = Some(purpose);
            s.graceful_since_at = Some(now_rfc3339.to_string());
            let conn = self.db.connect()?;
            conn.execute(
                "REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)",
                libsql::params![pk(host_id), serde_json::to_string(&s).unwrap()],
            )
            .await?;
            Ok(true)
        })
        .await
    }
}

impl From<Row> for HostStatus {
    fn from(row: Row) -> Self {
        let json: String = row.get(0).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
