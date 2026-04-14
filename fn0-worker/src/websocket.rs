use crate::WorkerFn0;
use crate::bundle::BundleFetcher;
use axum::Router;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use axum::routing::any;
use color_eyre::eyre::Result;
use futures::{SinkExt, StreamExt};
use host_hq_protocol::{HostToHq, WsHostToHq, WsHqToHost};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

struct AppState {
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    bundle_fetcher: Arc<BundleFetcher>,
    ws_secret: Option<String>,
}

pub async fn run_websocket_server(
    port: u16,
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    _fn0: Arc<WorkerFn0>,
    bundle_fetcher: Arc<BundleFetcher>,
) -> Result<()> {
    let ws_secret = std::env::var("WS_SECRET").ok();

    let state = Arc::new(AppState {
        deployment_id,
        instance_count,
        graceful_shutdown,
        bundle_fetcher,
        ws_secret,
    });

    let app = Router::new().route("/", any(ws_handler)).with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "WebSocket server listening");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

async fn handle_connection(mut socket: WebSocket, state: Arc<AppState>) {
    if let Some(secret) = &state.ws_secret {
        match socket.recv().await {
            Some(Ok(Message::Text(token))) if token.as_str() == secret.as_str() => {}
            _ => {
                tracing::warn!("WebSocket auth failed");
                let _ = socket.close().await;
                return;
            }
        }
    }

    tracing::info!("HQ connected via WebSocket");

    let (mut sink, mut stream) = socket.split();

    while let Some(msg) = stream.next().await {
        let data = match msg {
            Ok(Message::Binary(data)) => data,
            Ok(Message::Close(_)) | Err(_) => break,
            _ => continue,
        };

        let Ok(ws_msg) = WsHqToHost::from_bytes(&data) else {
            tracing::warn!("Failed to parse WebSocket message");
            continue;
        };

        match ws_msg {
            WsHqToHost::Datagram(datagram) => match datagram {
                host_hq_protocol::HqToHostDatagram::AdvertiseLatestDeploymentId { .. } => {
                    let current = state.deployment_id.load(Ordering::Relaxed);
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let pong = WsHostToHq::Datagram(HostToHq::NotifyHostStatus {
                        timestamp,
                        deployment_id: current,
                        instances: state.instance_count.load(Ordering::Relaxed),
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

                    let mut all_applied = true;

                    for code in &codes {
                        match code {
                            host_hq_protocol::CodeDeployment::Deploy {
                                subdomain,
                                code_id,
                                code_version,
                            } => {
                                if let Err(err) = state
                                    .bundle_fetcher
                                    .fetch_and_register(subdomain, *code_id, *code_version)
                                    .await
                                {
                                    tracing::error!(%err, %subdomain, "Failed to fetch bundle");
                                    all_applied = false;
                                }
                            }
                            host_hq_protocol::CodeDeployment::Undeploy { subdomain } => {
                                state.bundle_fetcher.unregister(subdomain).await;
                            }
                        }
                    }

                    if all_applied {
                        state
                            .deployment_id
                            .store(new_deployment_id, Ordering::Relaxed);
                    } else {
                        tracing::warn!(
                            new_deployment_id,
                            "Deployment update was only partially applied; will retry on next HQ ping"
                        );
                    }
                }
                host_hq_protocol::HqToHostReliable::GracefulShutdown => {
                    tracing::info!("Received graceful shutdown via WebSocket");
                    state.graceful_shutdown.store(true, Ordering::Relaxed);
                }
            },
        }
    }
}
