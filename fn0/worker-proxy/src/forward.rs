use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tracing::*;

pub async fn serve(listener: TcpListener, target_rx: watch::Receiver<Option<SocketAddr>>) {
    info!("public forwarder listening");
    loop {
        match listener.accept().await {
            Ok((conn, peer)) => {
                match *target_rx.borrow() {
                    Some(addr) => {
                        tokio::spawn(forward(conn, addr, peer));
                    }
                    None => {
                        debug!(%peer, "no target set; dropping connection");
                        drop(conn);
                    }
                }
            }
            Err(err) => {
                warn!(?err, "accept failed; backing off briefly");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

async fn forward(client: TcpStream, target: SocketAddr, peer: SocketAddr) {
    let upstream = match TcpStream::connect(target).await {
        Ok(s) => s,
        Err(err) => {
            warn!(?err, %target, %peer, "failed to dial upstream");
            return;
        }
    };
    let (mut client_read, mut client_write) = client.into_split();
    let (mut upstream_read, mut upstream_write) = upstream.into_split();
    let client_to_upstream = async move {
        let _ = tokio::io::copy(&mut client_read, &mut upstream_write).await;
        let _ = upstream_write.shutdown().await;
    };
    let upstream_to_client = async move {
        let _ = tokio::io::copy(&mut upstream_read, &mut client_write).await;
        let _ = client_write.shutdown().await;
    };
    tokio::join!(client_to_upstream, upstream_to_client);
}
