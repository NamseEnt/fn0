pub mod oci;

use crate::watchdog::HostId;
use chrono::{DateTime, Utc};
use std::{future::Future, net::IpAddr, pin::Pin};

#[derive(Debug, Clone)]
pub struct HostInfo {
    pub id: HostId,
    pub instance_created: DateTime<Utc>,
    pub ip: Option<IpAddr>,
    pub instance_state: HostInstanceState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HostInstanceState {
    Starting,
    Running,
    Terminating,
}

pub type HostInfos = Vec<HostInfo>;

pub trait HostInfra: Send + Sync {
    fn get_host_infos<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<HostInfos>> + 'a + Send>>;

    fn stream_host_infos<'a>(
        &'a self,
        tx: tokio::sync::mpsc::Sender<HostInfo>,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>>;

    fn terminate<'a>(
        &'a self,
        host_id: &'a HostId,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>>;

    fn launch_instances<'a>(
        &'a self,
        count: usize,
    ) -> Pin<Box<dyn Future<Output = color_eyre::Result<()>> + 'a + Send>>;
}

#[derive(Debug, Clone)]
pub struct HostHealthResponse {
    pub kind: HostHealthKind,
    pub ip: IpAddr,
}

#[derive(Debug, Clone)]
pub enum HostHealthKind {
    Good,
    GracefulShuttingDown,
}
