//! TLS certificates chosen by SNI.
//!
//! The worker used to serve exactly one certificate, the platform's own
//! `*.fn0.dev` origin certificate, which is all a request routed through
//! Cloudflare for SaaS ever asks for. A user pointing their own proxied zone at
//! this origin makes their edge ask for their hostname instead, and Full
//! (strict) requires a certificate that actually covers it. SNI override is
//! Enterprise-only, so there is no way to answer that with one certificate.
//!
//! Certificates arrive through the cert manifest, are issued by each project
//! owner's own Cloudflare Origin CA, and are therefore trusted only for the
//! Cloudflare edge to origin leg. Private keys travel as KMS ciphertext and are
//! decrypted when a manifest version is applied, never during a handshake:
//! `ResolvesServerCert::resolve` is synchronous and blocking it on a network
//! round trip would stall the acceptor.

use crate::vault_client::VaultClient;
use fn0_shared_schema::WorkerHostnameCert;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

pub struct SniCertResolver {
    /// Served when SNI names a host we hold no certificate for, which is every
    /// `*.fn0.dev` request and anything that reaches the origin IP directly.
    fallback: Arc<CertifiedKey>,
    by_hostname: RwLock<Arc<HashMap<String, Arc<CertifiedKey>>>>,
    cert_version: AtomicU64,
    vault: Arc<VaultClient>,
}

impl SniCertResolver {
    pub fn new(fallback: CertifiedKey, vault: Arc<VaultClient>) -> Self {
        Self {
            fallback: Arc::new(fallback),
            by_hostname: RwLock::new(Arc::new(HashMap::new())),
            cert_version: AtomicU64::new(0),
            vault,
        }
    }

    pub fn current_version(&self) -> u64 {
        self.cert_version.load(Ordering::Acquire)
    }

    /// Replaces the whole set. A certificate that fails to build is dropped
    /// with a log rather than failing the batch: one project's broken
    /// certificate must not stop every other project's from rotating.
    pub async fn apply(&self, cert_version: u64, certs: &HashMap<String, WorkerHostnameCert>) {
        let mut built = HashMap::with_capacity(certs.len());
        for (hostname, cert) in certs {
            match self.build(cert).await {
                Ok(certified) => {
                    built.insert(hostname.clone(), Arc::new(certified));
                }
                Err(error) => tracing::error!(
                    %error,
                    %hostname,
                    project_id = %cert.project_id,
                    "hostname certificate could not be loaded"
                ),
            }
        }
        let loaded = built.len();
        *self.by_hostname.write().expect("cert map lock") = Arc::new(built);
        self.cert_version.store(cert_version, Ordering::Release);
        tracing::info!(cert_version, loaded, "hostname certificates applied");
    }

    async fn build(&self, cert: &WorkerHostnameCert) -> anyhow::Result<CertifiedKey> {
        let key_pem = String::from_utf8(self.vault.decrypt(&cert.key_ciphertext).await?)
            .map_err(|error| anyhow::anyhow!("decrypted private key is not utf8: {error}"))?;
        certified_key(&cert.cert_pem, &key_pem)
    }
}

pub fn certified_key(cert_pem: &str, key_pem: &str) -> anyhow::Result<CertifiedKey> {
    let chain: Vec<_> = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<Result<_, _>>()
        .map_err(|error| anyhow::anyhow!("certificate pem: {error}"))?;
    if chain.is_empty() {
        anyhow::bail!("no certificate found in pem");
    }
    let key = rustls_pemfile::private_key(&mut key_pem.as_bytes())
        .map_err(|error| anyhow::anyhow!("private key pem: {error}"))?
        .ok_or_else(|| anyhow::anyhow!("no private key found in pem"))?;
    let signing_key = rustls::crypto::aws_lc_rs::sign::any_supported_type(&key)
        .map_err(|error| anyhow::anyhow!("unsupported private key type: {error}"))?;
    Ok(CertifiedKey::new(chain, signing_key))
}

impl std::fmt::Debug for SniCertResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SniCertResolver")
            .field("cert_version", &self.current_version())
            .finish()
    }
}

impl ResolvesServerCert for SniCertResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        // No SNI still gets the platform certificate: that is what the previous
        // single-certificate acceptor did, and refusing the handshake here
        // would break every client that reaches the origin without one.
        let Some(hostname) = client_hello.server_name() else {
            return Some(self.fallback.clone());
        };
        let by_hostname = self.by_hostname.read().expect("cert map lock");
        Some(
            by_hostname
                .get(hostname)
                .cloned()
                .unwrap_or_else(|| self.fallback.clone()),
        )
    }
}
