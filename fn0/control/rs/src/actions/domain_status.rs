use crate::common::auth;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
}

#[derive(Serialize)]
pub enum Output {
    /// The project has no registered domain, so the worker serves nothing for
    /// it.
    NoDomain,
    /// The project runs on its owner's Cloudflare account: their edge holds the
    /// visitor-facing certificate, and fn0 only holds the origin certificate.
    SelfHosted {
        domain: String,
        origin_certificate_ready: bool,
        origin_certificate_expires_epoch_seconds: Option<i64>,
        origin_hostname: String,
    },
    NotLoggedIn,
    NotFound,
    InternalError,
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
        Ok(None) => return Output::NoDomain,
        Err(e) => {
            tracing::error!("domain_status WorkerManifestDocGet: {e}");
            return Output::InternalError;
        }
    };

    // The manifest entry is the only writer of a project's domain, and an
    // entry without one predates the rename: such a project answers nowhere.
    let Some(entry) = manifest.project_manifests.get(&req.body.project_id) else {
        return Output::NoDomain;
    };
    if entry.domain.is_empty() {
        return Output::NoDomain;
    }
    let domain = entry.domain.clone();

    let cert = match (WorkerCertManifestDocGet {}).send_with(&db).await {
        Ok(manifest) => manifest.and_then(|manifest| manifest.certs.get(&domain).cloned()),
        Err(e) => {
            tracing::error!("domain_status WorkerCertManifestDocGet: {e}");
            return Output::InternalError;
        }
    };
    Output::SelfHosted {
        domain,
        origin_certificate_ready: cert.is_some(),
        origin_certificate_expires_epoch_seconds: cert
            .as_ref()
            .map(|cert| cert.not_after_epoch_seconds),
        origin_hostname: std::env::var("FN0_WORKER_ORIGIN_HOSTNAME").unwrap_or_default(),
    }
}
