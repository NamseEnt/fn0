pub mod oci;

use crate::WorkerId;
use std::{future::Future, net::IpAddr, pin::Pin};

#[derive(Clone)]
pub struct WorkerInfo {
    pub id: WorkerId,
    pub ip: Option<IpAddr>,
}

pub type WorkerInfos = Vec<WorkerInfo>;

pub trait WorkerInfra: Send + Sync {
    fn get_worker_infos<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<WorkerInfos>> + 'a + Send>>;

    fn terminate<'a>(
        &'a self,
        worker_id: &'a WorkerId,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a + Send>>;
}
