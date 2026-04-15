use color_eyre::eyre::{Result, eyre};
use host_hq_protocol::{HqToHostDatagram, HqToHostReliable, HostToHq};
use quinn::{ClientConfig, Connection, Endpoint};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

#[derive(Clone)]
pub struct HostConnection {
    inner: Arc<HostConnectionInner>,
}

struct HostConnectionInner {
    connection: Connection,
}

impl HostConnection {
    pub async fn connect_quic(addr: SocketAddr, ca_cert_pem: &str) -> Result<Self> {
        let local = if addr.is_ipv4() {
            LOCAL_IPV4
        } else {
            LOCAL_IPV6
        };
        let endpoint = Endpoint::client(local)?;
        let client_config = configure_client(ca_cert_pem)?;
        let connection = endpoint
            .connect_with(client_config, addr, "host.fn0")?
            .await?;

        Ok(Self {
            inner: Arc::new(HostConnectionInner { connection }),
        })
    }

    pub async fn send_datagram(&self, datagram: HqToHostDatagram) -> Result<()> {
        let bytes = datagram.to_bytes()?;
        if bytes.len() > 1200 {
            return Err(eyre!("Datagram is too large"));
        }
        self.inner.connection.send_datagram(bytes)?;
        Ok(())
    }

    pub async fn send_reliable(&self, message: HqToHostReliable) -> Result<()> {
        let bytes = message.to_bytes()?;
        let mut send = self.inner.connection.open_uni().await?;
        send.write_all(&bytes).await?;
        send.finish()?;
        Ok(())
    }

    pub async fn read_message(&self) -> Result<HostToHq> {
        let bytes = self.inner.connection.read_datagram().await?;
        let msg = HostToHq::from_bytes(bytes)?;
        Ok(msg)
    }

    pub fn close(&self) {
        self.inner.connection.close(0_u8.into(), &[]);
    }
}

const LOCAL_IPV4: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
const LOCAL_IPV6: SocketAddr =
    SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0);

fn configure_client(ca_cert_pem: &str) -> Result<ClientConfig> {
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut ca_cert_pem.as_bytes()) {
        root_store.add(cert?)?;
    }
    if root_store.is_empty() {
        return Err(eyre!("No valid CA certificates found in PEM"));
    }
    Ok(ClientConfig::with_root_certificates(Arc::new(root_store))?)
}
