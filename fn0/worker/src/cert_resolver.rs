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
}

impl SniCertResolver {
    pub fn new(fallback: CertifiedKey) -> Self {
        Self {
            fallback: Arc::new(fallback),
            by_hostname: RwLock::new(Arc::new(HashMap::new())),
            cert_version: AtomicU64::new(0),
        }
    }

    pub fn current_version(&self) -> u64 {
        self.cert_version.load(Ordering::Acquire)
    }

    /// Replaces the whole set. A certificate that fails to build is dropped
    /// with a log rather than failing the batch: one project's broken
    /// certificate must not stop every other project's from rotating.
    pub async fn apply(
        &self,
        vault: &VaultClient,
        cert_version: u64,
        certs: &HashMap<String, WorkerHostnameCert>,
    ) {
        let mut built = HashMap::with_capacity(certs.len());
        for (hostname, cert) in certs {
            match build(vault, cert).await {
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
}

async fn build(vault: &VaultClient, cert: &WorkerHostnameCert) -> anyhow::Result<CertifiedKey> {
    let key_pem = String::from_utf8(vault.decrypt(&cert.key_ciphertext).await?)
        .map_err(|error| anyhow::anyhow!("decrypted private key is not utf8: {error}"))?;
    certified_key(&cert.cert_pem, &key_pem)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rustls::pki_types::ServerName;
    use std::sync::Arc;
    use tokio::io::AsyncWriteExt;

    fn self_signed(hostname: &str) -> (String, String) {
        let certified = rcgen::generate_simple_self_signed(vec![hostname.to_string()])
            .expect("self signed certificate");
        (certified.cert.pem(), certified.signing_key.serialize_pem())
    }

    /// Drives a real handshake, because the thing under test is what rustls
    /// calls during one and a hand-built `ClientHello` would not prove it.
    async fn served_certificate(resolver: Arc<SniCertResolver>, sni: &str) -> Vec<u8> {
        served_for(resolver, ServerName::try_from(sni.to_string()).unwrap()).await
    }

    async fn served_for(resolver: Arc<SniCertResolver>, name: ServerName<'static>) -> Vec<u8> {
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_cert_resolver(resolver);
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((socket, _)) = listener.accept().await
                && let Ok(mut stream) = acceptor.accept(socket).await
            {
                let _ = stream.write_all(b"ok").await;
                let _ = stream.shutdown().await;
            }
        });

        // Accepts whatever it is handed: the assertion is which certificate
        // arrived, not whether a test CA trusts it.
        #[derive(Debug)]
        struct AcceptAny;
        impl rustls::client::danger::ServerCertVerifier for AcceptAny {
            fn verify_server_cert(
                &self,
                _end_entity: &rustls::pki_types::CertificateDer<'_>,
                _intermediates: &[rustls::pki_types::CertificateDer<'_>],
                _server_name: &ServerName<'_>,
                _ocsp: &[u8],
                _now: rustls::pki_types::UnixTime,
            ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
                Ok(rustls::client::danger::ServerCertVerified::assertion())
            }
            fn verify_tls12_signature(
                &self,
                _m: &[u8],
                _c: &rustls::pki_types::CertificateDer<'_>,
                _d: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }
            fn verify_tls13_signature(
                &self,
                _m: &[u8],
                _c: &rustls::pki_types::CertificateDer<'_>,
                _d: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }
            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                rustls::crypto::aws_lc_rs::default_provider()
                    .signature_verification_algorithms
                    .supported_schemes()
            }
        }

        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAny))
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
        let socket = tokio::net::TcpStream::connect(addr).await.unwrap();
        let stream = connector.connect(name, socket).await.expect("handshake");
        let (_, connection) = stream.get_ref();
        let chain = connection.peer_certificates().expect("peer certificate");
        chain[0].as_ref().to_vec()
    }

    fn resolver_with_one_hostname() -> (Arc<SniCertResolver>, Vec<u8>, Vec<u8>) {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let (fallback_pem, fallback_key) = self_signed("fallback.fn0.dev");
        let (custom_pem, custom_key) = self_signed("app.example.test");
        let fallback = certified_key(&fallback_pem, &fallback_key).unwrap();
        let custom = certified_key(&custom_pem, &custom_key).unwrap();
        let fallback_der = fallback.cert[0].as_ref().to_vec();
        let custom_der = custom.cert[0].as_ref().to_vec();

        let resolver = SniCertResolver::new(fallback);
        let mut map = HashMap::new();
        map.insert("app.example.test".to_string(), Arc::new(custom));
        *resolver.by_hostname.write().unwrap() = Arc::new(map);
        (Arc::new(resolver), fallback_der, custom_der)
    }

    #[tokio::test]
    async fn a_known_hostname_gets_its_own_certificate() {
        let (resolver, _fallback, custom) = resolver_with_one_hostname();
        assert_eq!(
            served_certificate(resolver, "app.example.test").await,
            custom
        );
    }

    #[tokio::test]
    async fn every_other_hostname_gets_the_platform_certificate() {
        let (resolver, fallback, _custom) = resolver_with_one_hostname();
        assert_eq!(
            served_certificate(resolver.clone(), "someproject.fn0.dev").await,
            fallback
        );
        assert_eq!(
            served_certificate(resolver, "never.heard.of.it").await,
            fallback
        );
    }

    /// rustls omits SNI for an IP address, which is what reaching the origin
    /// directly looks like. The previous single-certificate acceptor answered
    /// those, so this one has to as well.
    #[tokio::test]
    async fn a_connection_without_sni_gets_the_platform_certificate() {
        let (resolver, fallback, _custom) = resolver_with_one_hostname();
        let name = ServerName::IpAddress(std::net::Ipv4Addr::new(127, 0, 0, 1).into());
        assert_eq!(served_for(resolver, name).await, fallback);
    }

    #[tokio::test]
    async fn a_resolver_holding_no_hostnames_still_serves_the_platform_certificate() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let (pem, key) = self_signed("fallback.fn0.dev");
        let fallback = certified_key(&pem, &key).unwrap();
        let der = fallback.cert[0].as_ref().to_vec();
        let resolver = Arc::new(SniCertResolver::new(fallback));
        assert_eq!(served_certificate(resolver, "anything.fn0.dev").await, der);
    }
}
