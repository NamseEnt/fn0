//! Polls the certificate manifest and hands new certificates to the SNI
//! resolver.
//!
//! Separate from the project manifest poller and slower: custom hostnames
//! change on the timescale of a person editing DNS, while the project manifest
//! carries every deploy. A certificate that appears ten seconds late costs the
//! user one retry; a certificate polled every second would put a PEM per
//! hostname on the wire sixty times a minute per worker.

use crate::cert_resolver::SniCertResolver;
use crate::vault_client::VaultClient;
use doc_db::{Database, DbRequest};
use fn0_shared_schema::WorkerCertManifestDocGet;
use std::sync::Arc;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_secs(10);

pub async fn run(db: Database, resolver: Arc<SniCertResolver>, vault: Arc<VaultClient>) {
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let manifest = match (WorkerCertManifestDocGet {}).send_with(&db).await {
            Ok(Some(manifest)) => manifest,
            Ok(None) => continue,
            Err(error) => {
                tracing::warn!(%error, "cert manifest fetch failed");
                continue;
            }
        };

        if manifest.cert_version == resolver.current_version() {
            continue;
        }
        resolver
            .apply(&vault, manifest.cert_version, &manifest.certs)
            .await;
    }
}
