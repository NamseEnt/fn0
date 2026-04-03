use color_eyre::eyre::{Result, eyre};
use futures::{SinkExt, StreamExt};
use host_hq_protocol::{HqToHostDatagram, HqToHostReliable, HostToHq, WsHostToHq, WsHqToHost};
use quinn::{ClientConfig, Connection, Endpoint};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

type WsSink = futures::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    Message,
>;
type WsStream = futures::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
>;

#[derive(Clone)]
pub struct HostConnection {
    inner: Arc<HostConnectionInner>,
}

enum HostConnectionInner {
    Quic {
        connection: Connection,
    },
    WebSocket {
        sink: Mutex<WsSink>,
        stream: Mutex<WsStream>,
    },
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
            inner: Arc::new(HostConnectionInner::Quic { connection }),
        })
    }

    pub async fn connect_websocket(url: &str, secret: Option<&str>) -> Result<Self> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url).await
            .map_err(|e| eyre!("WebSocket connect failed: {e}"))?;

        let (mut sink, stream) = ws_stream.split();

        if let Some(secret) = secret {
            sink.send(Message::Text(secret.to_string().into()))
                .await
                .map_err(|e| eyre!("WebSocket auth send failed: {e}"))?;
        }

        Ok(Self {
            inner: Arc::new(HostConnectionInner::WebSocket {
                sink: Mutex::new(sink),
                stream: Mutex::new(stream),
            }),
        })
    }

    pub async fn send_datagram(&self, datagram: HqToHostDatagram) -> Result<()> {
        match self.inner.as_ref() {
            HostConnectionInner::Quic { connection } => {
                let bytes = datagram.to_bytes()?;
                if bytes.len() > 1200 {
                    return Err(eyre!("Datagram is too large"));
                }
                connection.send_datagram(bytes)?;
                Ok(())
            }
            HostConnectionInner::WebSocket { sink, .. } => {
                let msg = WsHqToHost::Datagram(datagram);
                let bytes = msg.to_bytes()?;
                sink.lock()
                    .await
                    .send(Message::Binary(bytes.to_vec().into()))
                    .await
                    .map_err(|e| eyre!("WebSocket send failed: {e}"))?;
                Ok(())
            }
        }
    }

    pub async fn send_reliable(&self, message: HqToHostReliable) -> Result<()> {
        match self.inner.as_ref() {
            HostConnectionInner::Quic { connection } => {
                let bytes = message.to_bytes()?;
                let mut send = connection.open_uni().await?;
                send.write_all(&bytes).await?;
                send.finish()?;
                Ok(())
            }
            HostConnectionInner::WebSocket { sink, .. } => {
                let msg = WsHqToHost::Reliable(message);
                let bytes = msg.to_bytes()?;
                sink.lock()
                    .await
                    .send(Message::Binary(bytes.to_vec().into()))
                    .await
                    .map_err(|e| eyre!("WebSocket send failed: {e}"))?;
                Ok(())
            }
        }
    }

    pub async fn read_message(&self) -> Result<HostToHq> {
        match self.inner.as_ref() {
            HostConnectionInner::Quic { connection } => {
                let bytes = connection.read_datagram().await?;
                let msg = HostToHq::from_bytes(bytes)?;
                Ok(msg)
            }
            HostConnectionInner::WebSocket { stream, .. } => {
                let mut stream = stream.lock().await;
                loop {
                    match stream.next().await {
                        Some(Ok(Message::Binary(data))) => {
                            let msg = WsHostToHq::from_bytes(&data)
                                .map_err(|e| eyre!("Failed to parse WebSocket message: {e}"))?;
                            match msg {
                                WsHostToHq::Datagram(host_msg) => return Ok(host_msg),
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            return Err(eyre!("WebSocket connection closed"));
                        }
                        Some(Err(e)) => {
                            return Err(eyre!("WebSocket read error: {e}"));
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    pub fn close(&self) {
        match self.inner.as_ref() {
            HostConnectionInner::Quic { connection } => {
                connection.close(0_u8.into(), &[]);
            }
            HostConnectionInner::WebSocket { .. } => {
                // WebSocket will be closed when dropped
            }
        }
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
