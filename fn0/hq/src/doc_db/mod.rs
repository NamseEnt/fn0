mod build_registry;
mod deploy_job;
mod deployment;
mod host_status;
mod job_queue;
mod project;
mod scale_config;
mod worker_target;

pub use deploy_job::*;
pub use deployment::*;
pub use host_status::*;
use libsql::{Builder, Database, Result};
use std::sync::Arc;
pub use worker_target::*;

#[derive(Clone)]
pub struct DocDb {
    db: Arc<Database>,
}

impl DocDb {
    pub async fn new(url: String, token: String) -> Result<Self> {
        let db = Builder::new_remote(url, token).build().await?;
        Ok(Self { db: Arc::new(db) })
    }

    #[cfg(test)]
    pub async fn new_test_db() -> Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("docdb_test_{}_{}.db", std::process::id(), id));
        let _ = std::fs::remove_file(&path);
        let db = Builder::new_local(path.to_str().unwrap()).build().await?;
        let conn = db.connect()?;
        conn.execute(
            "CREATE TABLE docs (pk TEXT NOT NULL, sk INTEGER NOT NULL, value BLOB NOT NULL, PRIMARY KEY (pk, sk))",
            libsql::params!(),
        )
        .await?;
        Ok(Self { db: Arc::new(db) })
    }
}
