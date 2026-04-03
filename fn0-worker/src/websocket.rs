use adapt_cache::s3::S3AdaptCache;
use color_eyre::eyre::Result;
use fn0::{CodeKind, Fn0};
use futures::{SinkExt, StreamExt};
use host_hq_protocol::{HostToHq, WsHostToHq, WsHqToHost};
use std::net::SocketAddr;
use std::string::FromUtf8Error;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

type JsCache = S3AdaptCache<String, FromUtf8Error>;

pub async fn run_websocket_server(
    port: u16,
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    fn0: Arc<Fn0<JsCache>>,
) -> Result<()> {
    let ws_secret = std::env::var("WS_SECRET").ok();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "WebSocket server listening");

    while let Ok((stream, remote)) = listener.accept().await {
        let deployment_id = deployment_id.clone();
        let instance_count = instance_count.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        let fn0 = fn0.clone();
        let ws_secret = ws_secret.clone();

        tokio::spawn(async move {
            match tokio_tungstenite::accept_async(stream).await {
                Ok(mut ws_stream) => {
                    if let Some(secret) = &ws_secret {
                        match ws_stream.next().await {
                            Some(Ok(Message::Text(token))) if token.as_str() == secret.as_str() => {}
                            _ => {
                                tracing::warn!(%remote, "WebSocket auth failed");
                                let _ = ws_stream.close(None).await;
                                return;
                            }
                        }
                    }
                    tracing::info!(%remote, "HQ connected via WebSocket");
                    handle_connection(
                        ws_stream,
                        deployment_id,
                        instance_count,
                        graceful_shutdown,
                        fn0,
                    )
                    .await;
                }
                Err(err) => {
                    tracing::warn!(%err, "Failed to accept WebSocket connection");
                }
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    fn0: Arc<Fn0<JsCache>>,
) {
    let (mut sink, mut stream) = ws_stream.split();

    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(Message::Binary(data)) => data,
            Ok(Message::Close(_)) | Err(_) => break,
            _ => continue,
        };

        let Ok(ws_msg) = WsHqToHost::from_bytes(&msg) else {
            tracing::warn!("Failed to parse WebSocket message");
            continue;
        };

        match ws_msg {
            WsHqToHost::Datagram(datagram) => match datagram {
                host_hq_protocol::HqToHostDatagram::AdvertiseLatestDeploymentId { .. } => {
                    let current = deployment_id.load(Ordering::Relaxed);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let pong = WsHostToHq::Datagram(HostToHq::NotifyHostStatus {
                        timestamp,
                        deployment_id: current,
                        instances: instance_count.load(Ordering::Relaxed),
                    });

                    if let Ok(bytes) = pong.to_bytes() {
                        let _ = sink.send(Message::Binary(bytes.to_vec().into())).await;
                    }
                }
            },
            WsHqToHost::Reliable(reliable) => match reliable {
                host_hq_protocol::HqToHostReliable::DeploymentUpdates {
                    deployment_id: new_deployment_id,
                    codes,
                } => {
                    tracing::info!(
                        new_deployment_id,
                        codes = codes.len(),
                        "Received deployment update via WebSocket"
                    );

                    for code in &codes {
                        match code {
                            host_hq_protocol::CodeDeployment::Deploy { subdomain, .. } => {
                                fn0.register_code(subdomain, CodeKind::Wasm);
                            }
                            host_hq_protocol::CodeDeployment::Undeploy { subdomain } => {
                                fn0.unregister_code(subdomain);
                            }
                        }
                    }

                    deployment_id.store(new_deployment_id, Ordering::Relaxed);
                }
                host_hq_protocol::HqToHostReliable::GracefulShutdown => {
                    tracing::info!("Received graceful shutdown via WebSocket");
                    graceful_shutdown.store(true, Ordering::Relaxed);
                }
            },
        }
    }
}
