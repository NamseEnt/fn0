pub mod oci_container;

use crate::*;
use std::net::IpAddr;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Host {
    pub id: HostId,
    pub ip: IpAddr,
}

pub trait HostProvide: Send + Sync {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>>;
    async fn terminate(&self, host_id: &HostId) -> color_eyre::Result<()>;
    #[allow(unused)]
    async fn launch_instance(&self) -> color_eyre::Result<()>;
}

#[derive(Clone)]
pub enum HostProvider {
    OciContainerInstance(oci_container::OciContainerInstanceHostProvider),
}

impl HostProvide for HostProvider {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>> {
        match self {
            HostProvider::OciContainerInstance(provider) => provider.list_hosts().await,
        }
    }

    async fn terminate(&self, host_id: &HostId) -> color_eyre::Result<()> {
        match self {
            HostProvider::OciContainerInstance(provider) => provider.terminate(host_id).await,
        }
    }

    async fn launch_instance(&self) -> color_eyre::Result<()> {
        match self {
            HostProvider::OciContainerInstance(provider) => provider.launch_instance().await,
        }
    }
}
