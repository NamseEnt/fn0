use super::*;
use doc_db::DocDb;

#[derive(Clone)]
pub struct DedicatedHostProvider {
    doc_db: DocDb,
}

impl DedicatedHostProvider {
    pub fn new(doc_db: DocDb) -> Self {
        Self { doc_db }
    }
}

impl HostProvide for DedicatedHostProvider {
    async fn list_hosts(&self) -> color_eyre::Result<Vec<Host>> {
        let records = self.doc_db.list_dedicated_hosts().await
            .map_err(|e| color_eyre::eyre::eyre!("Failed to list dedicated hosts: {}", e))?;

        Ok(records
            .into_iter()
            .map(|(id, record)| {
                let transport = match record.transport {
                    doc_db::DwsTransport::Quic => HostTransport::Quic,
                    doc_db::DwsTransport::WebSocket => HostTransport::WebSocket,
                };
                Host {
                    id: HostId::new(id),
                    addr: record.addr,
                    port: record.port,
                    transport,
                    dns_addr: record.http_host,
                }
            })
            .collect())
    }

    async fn terminate(&self, _host_id: &HostId) -> color_eyre::Result<()> {
        Ok(())
    }

    async fn scale_to(&self, _n: usize) -> color_eyre::Result<()> {
        Ok(())
    }
}
