use crate::common::auth;
use crate::common::byoc::ProjectStorage;
use crate::common::cloudflare_saas::{CloudflareSaasClient, HostnameStatus};
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
}

#[derive(Serialize)]
pub enum Output {
    NotConfigured,
    Configured {
        domain: String,
        cloudflare_status: CloudflareStatus,
    },
    /// The project runs on its owner's Cloudflare account: their edge holds the
    /// visitor-facing certificate, and fn0 only holds the origin certificate.
    SelfHosted {
        domain: String,
        origin_certificate_ready: bool,
        origin_certificate_expires_epoch_seconds: Option<i64>,
        origin_ip: String,
    },
    NotLoggedIn,
    NotFound,
    InternalError,
}

#[derive(Serialize)]
pub enum CloudflareStatus {
    Active,
    Pending,
    Missing,
    Other { value: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let db = doc_db::turso();
    let project = match (ProjectDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return Output::NotFound,
        Err(e) => {
            tracing::error!("domain_status ProjectDocGet: {e}");
            return Output::InternalError;
        }
    };
    if project.owner_github_id != user.github_id {
        return Output::NotFound;
    }

    let manifest = match (WorkerManifestDocGet {}).send_with(&db).await {
        Ok(Some(m)) => m,
        Ok(None) => return Output::NotConfigured,
        Err(e) => {
            tracing::error!("domain_status WorkerManifestDocGet: {e}");
            return Output::InternalError;
        }
    };

    let Some(entry) = manifest.project_manifests.get(&req.body.project_id) else {
        return Output::NotConfigured;
    };
    let Some(domain) = entry.custom_domain.clone() else {
        return Output::NotConfigured;
    };

    match ProjectStorage::resolve_connected(&db, &req.body.project_id).await {
        Ok(Some(_)) => {
            let cert = match (WorkerCertManifestDocGet {}).send_with(&db).await {
                Ok(manifest) => manifest.and_then(|manifest| manifest.certs.get(&domain).cloned()),
                Err(e) => {
                    tracing::error!("domain_status WorkerCertManifestDocGet: {e}");
                    return Output::InternalError;
                }
            };
            return Output::SelfHosted {
                domain,
                origin_certificate_ready: cert.is_some(),
                origin_certificate_expires_epoch_seconds: cert
                    .as_ref()
                    .map(|cert| cert.not_after_epoch_seconds),
                origin_ip: std::env::var("FN0_WORKER_ORIGIN_IP").unwrap_or_default(),
            };
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("domain_status ProjectStorage::resolve_connected: {e}");
            return Output::InternalError;
        }
    }

    let client = match CloudflareSaasClient::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("domain_status CloudflareSaasClient::from_env: {e}");
            return Output::InternalError;
        }
    };
    let cloudflare_status = match client.find_by_name(&domain).await {
        Ok(Some(h)) => match h.status {
            HostnameStatus::Active => CloudflareStatus::Active,
            HostnameStatus::Pending => CloudflareStatus::Pending,
            HostnameStatus::Other(s) => CloudflareStatus::Other { value: s },
            other => CloudflareStatus::Other {
                value: format!("{other:?}"),
            },
        },
        Ok(None) => CloudflareStatus::Missing,
        Err(e) => {
            tracing::error!("domain_status cloudflare lookup: {e}");
            return Output::InternalError;
        }
    };

    Output::Configured {
        domain,
        cloudflare_status,
    }
}
