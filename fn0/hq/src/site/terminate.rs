use crate::doc_db::DocDb;
use crate::host_id::HostId;
use crate::host_provider::{HostProvide, oci_compute_vm::OciComputeVmHostProvider};
use crate::ssh_pool::SshPool;
use tracing::*;

pub async fn terminate_host(
    host_provider: &OciComputeVmHostProvider,
    doc_db: &DocDb,
    ssh_pool: &SshPool,
    site_name: &str,
    host_id: &str,
    addr: &str,
) {
    let id = HostId::new(host_id.to_string());
    match host_provider.terminate(&id).await {
        Ok(()) => {
            info!(%host_id, "Terminated host");
        }
        Err(err) => {
            warn!(%err, %host_id, "Failed to terminate host");
            return;
        }
    }
    ssh_pool.invalidate(addr);
    if let Err(err) = doc_db.delete_host_status(site_name, host_id).await {
        warn!(%err, %host_id, "Failed to delete host-status after terminate");
    }
}
