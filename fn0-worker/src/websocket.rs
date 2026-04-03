use adapt_cache::s3::S3AdaptCache;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::any;
use axum::Router;
use color_eyre::eyre::Result;
use fn0::{CodeKind, Fn0};
use futures::{SinkExt, StreamExt};
use host_hq_protocol::{HostToHq, WsHostToHq, WsHqToHost};
use std::net::SocketAddr;
use std::string::FromUtf8Error;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

type JsCache = S3AdaptCache<String, FromUtf8Error>;

struct AppState {
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    fn0: Arc<Fn0<JsCache>>,
    ws_secret: Option<String>,
}

pub async fn run_websocket_server(
    port: u16,
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    fn0: Arc<Fn0<JsCache>>,
) -> Result<()> {
    let ws_secret = std::env::var("WS_SECRET").ok();

    let state = Arc::new(AppState {
        deployment_id,
        instance_count,
        graceful_shutdown,
        fn0,
        ws_secret,
    });

    let app = Router::new()
        .route("/", any(ws_handler))
        .with_state(state);

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

                    for code in &codes {
                        match code {
                            host_hq_protocol::CodeDeployment::Deploy { subdomain, .. } => {
                                state.fn0.register_code(subdomain, CodeKind::Wasm);
                            }
                            host_hq_protocol::CodeDeployment::Undeploy { subdomain } => {
                                state.fn0.unregister_code(subdomain);
                            }
                        }
                    }

                    state
                        .deployment_id
                        .store(new_deployment_id, Ordering::Relaxed);
                }
                host_hq_protocol::HqToHostReliable::GracefulShutdown => {
                    tracing::info!("Received graceful shutdown via WebSocket");
                    state.graceful_shutdown.store(true, Ordering::Relaxed);
                }
            },
        }
    }
}
