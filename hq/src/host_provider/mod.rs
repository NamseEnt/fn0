pub mod dedicated;
pub mod oci_compute_vm;
pub mod oci_container;

use crate::*;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Host {
    pub id: HostId,
    pub addr: String,
    pub port: u16,
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
    OciComputeVm(oci_compute_vm::OciComputeVmHostProvider),
    Dedicated(dedicated::DedicatedHostProvider),
}

impl HostProvide for HostProvider {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>> {
        match self {
            HostProvider::OciContainerInstance(p) => p.list_hosts().await,
            HostProvider::OciComputeVm(p) => p.list_hosts().await,
            HostProvider::Dedicated(p) => p.list_hosts().await,
        }
    }

    async fn terminate(&self, host_id: &HostId) -> color_eyre::Result<()> {
        match self {
            HostProvider::OciContainerInstance(p) => p.terminate(host_id).await,
            HostProvider::OciComputeVm(p) => p.terminate(host_id).await,
            HostProvider::Dedicated(p) => p.terminate(host_id).await,
        }
    }

    async fn launch_instance(&self) -> color_eyre::Result<()> {
        match self {
            HostProvider::OciContainerInstance(p) => p.launch_instance().await,
            HostProvider::OciComputeVm(p) => p.launch_instance().await,
            HostProvider::Dedicated(p) => p.launch_instance().await,
        }
    }
}
