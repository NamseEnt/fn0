use crate::common::admin;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok {
        scanned_instances_count: u64,
        terminated_instances_count: u64,
        cleaned_dns_records_count: u64,
    },
    Unauthorized,
    Error {
        message: String,
    },
}

pub const ZOMBIE_TIMEOUT_SECS: i64 = 5 * 60;

pub struct SweepStats {
    pub scanned_instances: u64,
    pub terminated_instances: u64,
    pub cleaned_dns_records: u64,
}

pub async fn run_sweep() -> anyhow::Result<SweepStats> {
    tracing::info!(
        timeout_secs = ZOMBIE_TIMEOUT_SECS,
        "zombie_sweep stub: TODO list OCI compute instances, ping each, terminate ones whose \
         last normal signal is older than the timeout, then delete matching Cloudflare DNS A records"
    );
    Ok(SweepStats {
        scanned_instances: 0,
        terminated_instances: 0,
        cleaned_dns_records: 0,
    })
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }
    match run_sweep().await {
        Ok(stats) => Output::Ok {
            scanned_instances_count: stats.scanned_instances,
            terminated_instances_count: stats.terminated_instances,
            cleaned_dns_records_count: stats.cleaned_dns_records,
        },
        Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
