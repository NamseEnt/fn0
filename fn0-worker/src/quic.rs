use adapt_cache::s3::S3AdaptCache;
use color_eyre::eyre::Result;
use fn0::{CodeKind, Fn0};
use host_hq_protocol::{HostToHq, HqToHostDatagram, HqToHostReliable};
use quinn::Endpoint;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::net::SocketAddr;
use std::string::FromUtf8Error;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

type JsCache = S3AdaptCache<String, FromUtf8Error>;

pub async fn run_quic_server(
    port: u16,
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    fn0: Arc<Fn0<JsCache>>,
) -> Result<()> {
    let cert = generate_simple_self_signed(vec!["host.fn0".to_string()])?;
    let cert_der = CertificateDer::from(cert.cert);
    let key_der = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der.into())?;

    let quinn_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(server_config)?,
    ));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let endpoint = Endpoint::server(quinn_config, addr)?;
    tracing::info!(%addr, "QUIC server listening");

    while let Some(incoming) = endpoint.accept().await {
        let deployment_id = deployment_id.clone();
        let instance_count = instance_count.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        let fn0 = fn0.clone();

        tokio::spawn(async move {
            match incoming.await {
                Ok(connection) => {
                    tracing::info!(remote = %connection.remote_address(), "HQ connected");
                    handle_connection(connection, deployment_id, instance_count, graceful_shutdown, fn0).await;
                }
                Err(err) => {
                    tracing::warn!(%err, "Failed to accept QUIC connection");
                }
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    connection: quinn::Connection,
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    fn0: Arc<Fn0<JsCache>>,
) {
    let datagram_handle = tokio::spawn({
        let connection = connection.clone();
        let deployment_id = deployment_id.clone();
        let instance_count = instance_count.clone();
        async move {
            loop {
                match connection.read_datagram().await {
                    Ok(bytes) => {
                        let Ok(msg) = HqToHostDatagram::from_bytes(bytes) else {
                            tracing::warn!("Failed to parse HQ datagram");
                            continue;
                        };

                        match msg {
                            HqToHostDatagram::AdvertiseLatestDeploymentId { .. } => {
                                let current = deployment_id.load(Ordering::Relaxed);
                                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();

                                let pong = HostToHq::NotifyHostStatus {
                                    timestamp,
                                    deployment_id: current,
                                    instances: instance_count.load(Ordering::Relaxed),
                                };
                                if let Ok(bytes) = pong.to_bytes() {
                                    let _ = connection.send_datagram(bytes);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!(%err, "Datagram read error, HQ disconnected");
                        break;
                    }
                }
            }
        }
    });

    let reliable_handle = tokio::spawn({
        let connection = connection.clone();
        let deployment_id = deployment_id.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        async move {
            loop {
                match connection.accept_uni().await {
                    Ok(mut recv) => {
                        let data = match recv.read_to_end(64 * 1024).await {
                            Ok(data) => data,
                            Err(err) => {
                                tracing::warn!(%err, "Failed to read reliable message");
                                continue;
                            }
                        };

                        let Ok(msg) = HqToHostReliable::from_bytes(data.into()) else {
                            tracing::warn!("Failed to parse HQ reliable message");
                            continue;
                        };

                        match msg {
                            HqToHostReliable::DeploymentUpdates {
                                deployment_id: new_deployment_id,
                                codes,
                            } => {
                                tracing::info!(
                                    new_deployment_id,
                                    codes = codes.len(),
                                    "Received deployment update"
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
                            HqToHostReliable::GracefulShutdown => {
                                tracing::info!("Received graceful shutdown from HQ");
                                graceful_shutdown.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!(%err, "Uni stream accept error, HQ disconnected");
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = datagram_handle => {},
        _ = reliable_handle => {},
    }
}
