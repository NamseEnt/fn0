use color_eyre::eyre::{Result, eyre};
use host_hq_protocol::{HqToHostDatagram, HqToHostReliable};
use quinn::{ClientConfig, Connection, Endpoint, ReadDatagram};
use rustls::pki_types::CertificateDer;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

#[derive(Clone)]
pub struct HostConnection {
    connection: Connection,
}

impl HostConnection {
    pub async fn connect(addr: SocketAddr, cert: &str) -> Result<Self> {
        let local = if addr.is_ipv4() {
            LOCAL_IPV4
        } else {
            LOCAL_IPV6
        };
        let endpoint = Endpoint::client(local)?;
        let client_config = configure_client(cert)?;
        let connection = endpoint
            .connect_with(client_config, addr, "host.fn0")?
            .await?;

        Ok(Self { connection })
    }
    pub fn send_datagram(&self, datagram: HqToHostDatagram) -> Result<()> {
        let bytes = datagram.to_bytes()?;
        if bytes.len() > 1200 {
            return Err(eyre!("Datagram is too large"));
        }
        self.connection.send_datagram(bytes)?;
        Ok(())
    }
    pub async fn send_reliable(&self, message: HqToHostReliable) -> Result<()> {
        let bytes = message.to_bytes()?;
        let mut send = self.connection.open_uni().await?;
        send.write_all(&bytes).await?;
        send.finish()?;
        Ok(())
    }
    pub fn read_unreliable_small_message(&self) -> ReadDatagram<'_> {
        self.connection.read_datagram()
    }
    pub fn close(&self) {
        self.connection.close(0_u8.into(), &[]);
    }
}

const LOCAL_IPV4: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
const LOCAL_IPV6: SocketAddr =
    SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0);

fn configure_client(server_cert: &str) -> Result<ClientConfig> {
    let mut certs = rustls::RootCertStore::empty();
    certs.add(CertificateDer::from(server_cert.as_bytes()))?;

    Ok(ClientConfig::with_root_certificates(Arc::new(certs))?)
}
