mod deployment;
mod scale_config;

pub use deployment::*;
use libsql::{Builder, Database, Result};
pub use scale_config::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct DocDb {
    db: Arc<Database>,
}

impl DocDb {
    pub async fn new(url: String, token: String) -> Result<Self> {
        let db = Builder::new_remote(url, token).build().await?;
        Ok(Self { db: Arc::new(db) })
    }
}
