//! Origin certificates for user-owned custom hostnames.
//!
//! A user points a proxied A record at the worker fleet, so their Cloudflare
//! edge terminates TLS for the visitor and then connects to us as origin. Under
//! Full (strict) that connection needs a certificate covering *their* hostname,
//! and Origin Rules cannot rewrite SNI outside Enterprise, so the platform's
//! `*.fn0.dev` certificate cannot answer for it.
//!
//! The key pair is generated here and never leaves the platform; only a CSR
//! goes to Cloudflare. The user grants certificate issuance on their own zone
//! and never handles a private key. The result is trusted by the Cloudflare
//! edge alone, for the one leg the worker terminates.

use crate::common::byoc::ProjectStorage;
use forte_sdk::*;

/// Origin CA's longest offered validity. Renewal is a live-traffic concern on
/// a hostname the platform does not control the DNS for, so the fewer times it
/// has to happen the better.
const VALIDITY_DAYS: u32 = 5475;
const SECONDS_PER_DAY: i64 = 86_400;

/// rcgen generates an ECDSA P-256 key by default, which Origin CA signs under
/// `origin-ecc`. Asking for `origin-rsa` with an ECDSA CSR is rejected.
const REQUEST_TYPE: &str = "origin-ecc";

pub struct IssuedCertificate {
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub not_after_epoch_seconds: i64,
}

pub async fn issue(
    storage: &ProjectStorage,
    hostname: &str,
    now: DateTime,
) -> anyhow::Result<IssuedCertificate> {
    let key_pair = rcgen::KeyPair::generate()
        .map_err(|error| anyhow::anyhow!("origin key generation: {error}"))?;
    let params = rcgen::CertificateParams::new(vec![hostname.to_string()])
        .map_err(|error| anyhow::anyhow!("origin csr params: {error}"))?;
    let signing_request = params
        .serialize_request(&key_pair)
        .map_err(|error| anyhow::anyhow!("origin csr: {error}"))?;
    let csr_pem = signing_request
        .pem()
        .map_err(|error| anyhow::anyhow!("origin csr pem: {error}"))?;

    let certificate = storage
        .cloudflare()?
        .create_origin_ca_certificate(
            &csr_pem,
            &[hostname.to_string()],
            REQUEST_TYPE,
            VALIDITY_DAYS,
        )
        .await?;

    Ok(IssuedCertificate {
        certificate_pem: certificate.certificate_pem,
        private_key_pem: key_pair.serialize_pem(),
        // Derived rather than parsed out of the response: Cloudflare answers
        // with a Go-formatted timestamp, and the validity we asked for is the
        // same fact without a format to get wrong.
        not_after_epoch_seconds: now.timestamp() + i64::from(VALIDITY_DAYS) * SECONDS_PER_DAY,
    })
}
