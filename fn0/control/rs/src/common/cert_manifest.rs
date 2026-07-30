//! The certificate manifest workers poll to learn which hostnames they can
//! terminate TLS for.
//!
//! Separate from `WorkerManifestDoc` because that document is read in full by
//! every worker every second, and a PEM per hostname does not belong on that
//! path.

use crate::docs::*;
use forte_sdk::*;

pub async fn put(
    db: &doc_db::Database,
    hostname: &str,
    cert: WorkerHostnameCert,
) -> anyhow::Result<()> {
    upsert(db, hostname.to_string(), Some(cert)).await
}

pub async fn remove(db: &doc_db::Database, hostname: &str) -> anyhow::Result<()> {
    upsert(db, hostname.to_string(), None).await
}

async fn upsert(
    db: &doc_db::Database,
    hostname: String,
    cert: Option<WorkerHostnameCert>,
) -> anyhow::Result<()> {
    let result = db
        .trx(|trx| {
            let hostname = hostname.clone();
            let cert = cert.clone();
            async move {
                let mut manifest = match trx.get(WorkerCertManifestDocGet {}).await? {
                    Some(manifest) => manifest,
                    None => trx.create(WorkerCertManifestDoc {
                        cert_version: 0,
                        certs: std::collections::HashMap::new(),
                    })?,
                };
                let changed = match cert {
                    Some(cert) => {
                        manifest.certs.insert(hostname, cert);
                        true
                    }
                    None => manifest.certs.remove(&hostname).is_some(),
                };
                if changed {
                    manifest.cert_version += 1;
                }
                trx.commit::<_, std::convert::Infallible>(())
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(cancel) => match cancel {},
        doc_db::TrxResult::Conflict(detail) => {
            anyhow::bail!("cert manifest conflict: {detail:?}")
        }
        doc_db::TrxResult::Err(error) => Err(error),
    }
}
