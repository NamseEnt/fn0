use crate::WorkerFn0;
use crate::bundle::BundleFetcher;
use color_eyre::eyre::{Result, eyre};
use host_hq_protocol::{HostToHq, HqToHostDatagram, HqToHostReliable};
use quinn::Endpoint;
use rcgen::{CertificateParams, KeyPair};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::read_pem_env;

pub async fn run_quic_server(
    port: u16,
    deployment_id: Arc<AtomicU64>,
    instance_count: Arc<AtomicU64>,
    graceful_shutdown: Arc<AtomicBool>,
    _fn0: Arc<WorkerFn0>,
    bundle_fetcher: Arc<BundleFetcher>,
) -> Result<()> {
    let ca_cert_pem = match read_pem_env("CA_CERT_PEM") {
        Some(v) => v,
        None => {
            tracing::info!("CA_CERT_PEM not set, QUIC server disabled");
            std::future::pending::<()>().await;
            return Ok(());
        }
    };
    let ca_key_pem = read_pem_env("CA_KEY_PEM")
        .ok_or_else(|| eyre!("CA_KEY_PEM env var not set"))?;

    let ca_key_pair = KeyPair::from_pem(&ca_key_pem)?;
    let ca_params = CertificateParams::from_ca_cert_pem(&ca_cert_pem)?;
    let ca_cert = ca_params.self_signed(&ca_key_pair)?;

    let mut worker_params = CertificateParams::new(vec!["host.fn0".to_string()])?;
    worker_params.is_ca = rcgen::IsCa::NoCa;
    let worker_key_pair = KeyPair::generate()?;
    let worker_cert = worker_params.signed_by(&worker_key_pair, &ca_cert, &ca_key_pair)?;

    let cert_der = CertificateDer::from(worker_cert.der().to_vec());
    let key_der = PrivatePkcs8KeyDer::from(worker_key_pair.serialize_der());

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der.into())?;

    let quinn_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(server_config)?,
    ));

    let addr = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port));
    let endpoint = Endpoint::server(quinn_config, addr)?;
    tracing::info!(%addr, "QUIC server listening");

    while let Some(incoming) = endpoint.accept().await {
        let deployment_id = deployment_id.clone();
        let instance_count = instance_count.clone();
        let graceful_shutdown = graceful_shutdown.clone();
        let bundle_fetcher = bundle_fetcher.clone();

        tokio::spawn(async move {
            match incoming.await {
                Ok(connection) => {
                    tracing::info!(remote = %connection.remote_address(), "HQ connected");
                    handle_connection(connection, deployment_id, instance_count, graceful_shutdown, bundle_fetcher).await;
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
    bundle_fetcher: Arc<BundleFetcher>,
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
        let bundle_fetcher = bundle_fetcher.clone();
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
                                        host_hq_protocol::CodeDeployment::Deploy {
                                            subdomain,
                                            code_id,
                                            code_version,
                                        } => {
                                            if let Err(err) = bundle_fetcher
                                                .fetch_and_register(subdomain, *code_id, *code_version)
                                                .await
                                            {
                                                tracing::error!(%err, %subdomain, "Failed to fetch bundle");
                                            }
                                        }
                                        host_hq_protocol::CodeDeployment::Undeploy { subdomain } => {
                                            bundle_fetcher.unregister(subdomain).await;
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
