pub mod s3;
mod update_health_records;

use crate::{
    WorkerId,
    worker_infra::{WorkerHealthResponseMap, WorkerInstanceState},
};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, future::Future, net::IpAddr, pin::Pin};
pub use update_health_records::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecord {
    pub state: HealthState,
    pub state_transited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthState {
    Starting,
    Healthy { ip: IpAddr },
    RetryingCheck { retrials: usize },
    MarkedForTermination,
    GracefulShuttingDown,
    TerminatedConfirm,
    InvisibleOnInfra,
}

pub type HealthRecords = BTreeMap<WorkerId, HealthRecord>;

pub trait HealthRecorder: Send + Sync {
    fn read_all<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<HealthRecords>> + 'a + Send>>;
    fn write_all<'a>(
        &'a self,
        records: HealthRecords,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a + Send>>;
}

pub fn get_workers_to_terminate(health_records: &HealthRecords) -> Vec<WorkerId> {
    health_records
        .iter()
        .filter_map(|(id, record)| {
            if let HealthState::MarkedForTermination = record.state {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn get_healthy_ips(health_records: &HealthRecords) -> Vec<IpAddr> {
    health_records
        .iter()
        .filter_map(|(_, record)| {
            if let HealthState::Healthy { ip } = record.state {
                Some(ip)
            } else {
                None
            }
        })
        .collect()
}
