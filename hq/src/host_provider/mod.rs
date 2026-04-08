pub mod oci_compute_vm;
pub mod oci_scaler;

use crate::*;

#[derive(Debug, Clone)]
pub struct Host {
    pub id: HostId,
    pub addr: String,
    pub port: u16,
    pub transport: HostTransport,
    pub dns_addr: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub enum HostTransport {
    #[default]
    Quic,
    WebSocket,
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.addr == other.addr && self.port == other.port
    }
}
impl Eq for Host {}
impl std::hash::Hash for Host {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.addr.hash(state);
        self.port.hash(state);
    }
}
impl PartialOrd for Host {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Host {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then(self.addr.cmp(&other.addr))
            .then(self.port.cmp(&other.port))
    }
}

pub trait HostProvide: Send + Sync {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>>;
    async fn terminate(&self, host_id: &HostId) -> color_eyre::Result<()>;
    async fn scale_to(&self, n: usize) -> color_eyre::Result<()>;
}
