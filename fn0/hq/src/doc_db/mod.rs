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
use color_eyre::eyre::Result;
pub use worker_target::*;

#[derive(Clone)]
pub struct DocDb {
    pub forte: forte_db::Database,
}

impl DocDb {
    pub async fn new(url: String, token: String) -> Result<Self> {
        let forte = forte_db::turso_with_config(url, token);
        Ok(Self { forte })
    }

    #[cfg(test)]
    pub async fn new_test_db() -> Result<Self> {
        Ok(Self {
            forte: forte_db::memory(),
        })
    }
}
