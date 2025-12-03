pub mod s3;

use crate::WorkerId;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, pin::Pin};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecord {
    pub health_check_retrials: usize,
    pub graceful_shutdown_start_at: Option<u64>,
}

pub type HealthRecords = BTreeMap<WorkerId, HealthRecord>;

pub trait HealthRecorder {
    fn read_all<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<HealthRecords>> + 'a + Send>>;
    fn write_all<'a>(
        &'a self,
        records: HealthRecords,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a + Send>>;
}
