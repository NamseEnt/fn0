use crate::shutdown::Shutdown;
use crate::worker_container_pool::UpstreamRoute;
use rand::Rng;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tracing::*;

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:443";

pub async fn run(shutdown: Shutdown, upstream_rx: watch::Receiver<Vec<UpstreamRoute>>) {
    let listen_addr: SocketAddr = std::env::var("FN0_AGENT_LISTEN_ADDR")
        .unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string())
        .parse()
        .expect("FN0_AGENT_LISTEN_ADDR must be host:port");

    let listener = match TcpListener::bind(listen_addr).await {
        Ok(l) => {
            info!(%listen_addr, "inbound proxy listening");
            l
        }
        Err(err) => {
            error!(?err, %listen_addr, "failed to bind inbound listener; aborting proxy");
            return;
        }
    };

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            accepted = listener.accept() => {
                match accepted {
                    Ok((conn, peer)) => {
                        let routes = upstream_rx.borrow().clone();
                        let Some(route) = pick_upstream(&routes) else {
                            debug!(%peer, "no active upstream; dropping connection");
                            drop(conn);
                            continue;
                        };
                        tokio::spawn(forward(conn, route, peer));
                    }
                    Err(err) => {
                        warn!(?err, "accept failed; backing off briefly");
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
            }
        }
    }
    info!("inbound proxy stopped");
}

fn pick_upstream(routes: &[UpstreamRoute]) -> Option<UpstreamRoute> {
    let total: u32 = routes.iter().map(|r| r.weight).sum();
    if total == 0 {
        return None;
    }
    let pick = rand::thread_rng().gen_range(0..total);
    let mut acc = 0u32;
    for r in routes {
        acc = acc.saturating_add(r.weight);
        if pick < acc {
            return Some(r.clone());
        }
    }
    routes.last().cloned()
}

async fn forward(client: TcpStream, route: UpstreamRoute, peer: SocketAddr) {
    let upstream = match TcpStream::connect(route.local_addr).await {
        Ok(s) => s,
        Err(err) => {
            warn!(
                ?err,
                upstream = %route.local_addr,
                container_name = %route.container_name,
                %peer,
                "failed to dial upstream"
            );
            return;
        }
    };

    let (mut a_r, mut a_w) = client.into_split();
    let (mut b_r, mut b_w) = upstream.into_split();
    let from_client = async move {
        let _ = tokio::io::copy(&mut a_r, &mut b_w).await;
        let _ = b_w.shutdown().await;
    };
    let from_upstream = async move {
        let _ = tokio::io::copy(&mut b_r, &mut a_w).await;
        let _ = a_w.shutdown().await;
    };
    tokio::join!(from_client, from_upstream);
}
