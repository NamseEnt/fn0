use crate::common::auth;
use crate::common::byoc::worker_storage_for;
use crate::common::cert_manifest;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub domain: String,
    /// Origin certificate for `domain`, signed through the owner's own
    /// Cloudflare Origin CA by the CLI. fn0 cannot sign one: that needs a token
    /// it deliberately never holds.
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub not_after_epoch_seconds: i64,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        /// The one thing the user has to do: a proxied CNAME for `domain`
        /// pointing here.
        origin_hostname: String,
        /// The domain this replaced, when the project already had a different
        /// one. Its certificate has been dropped.
        replaced_domain: Option<String>,
    },
    NotLoggedIn,
    NotFound,
    InvalidDomain {
        message: String,
    },
    DomainTaken {
        existing_project_id: String,
    },
    InternalError,
}

enum Cancel {
    NotFound,
    DomainTaken { existing_project_id: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let domain = req.body.domain.trim().to_ascii_lowercase();
    if let Err(message) = validate_domain(&domain) {
        return Output::InvalidDomain { message };
    }

    let project_id = req.body.project_id.clone();
    let github_id = user.github_id;

    // The manifest entry is written only for a connected project: the worker
    // needs the storage target the moment it starts routing a domain, and this
    // read is what refuses a project that connected nothing.
    let db = doc_db::turso();
    let storage = match (ProjectCloudflareConfigDocGet {
        project_id: &project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(config)) => worker_storage_for(&config),
        Ok(None) => {
            tracing::warn!(
                project_id,
                "domain_set refused: project has no connected Cloudflare account"
            );
            return Output::InternalError;
        }
        Err(e) => {
            tracing::error!("domain_set ProjectCloudflareConfigDocGet: {e}");
            return Output::InternalError;
        }
    };

    let result = doc_db::turso()
        .trx(|trx| {
            let project_id = project_id.clone();
            let domain = domain.clone();
            let storage = storage.clone();
            async move {
                let project = match trx
                    .get(ProjectDocGet {
                        project_id: &project_id,
                    })
                    .await?
                {
                    Some(p) => p,
                    None => return trx.cancel(Cancel::NotFound),
                };
                if project.owner_github_id != github_id {
                    return trx.cancel(Cancel::NotFound);
                }

                let mut handle = match trx.get(WorkerManifestDocGet {}).await? {
                    Some(h) => h,
                    None => {
                        let mut entries = HashMap::new();
                        entries.insert(
                            project_id.clone(),
                            WorkerProjectManifest {
                                code_version: 0,
                                domain: domain.clone(),
                                static_cache_state: fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
                                    .to_string(),
                                pending_code_version: None,
                                storage: Some(storage.clone()),
                            },
                        );
                        trx.create(WorkerManifestDoc {
                            manifest_version: 1,
                            project_manifests: entries,
                        })?;
                        return trx.commit::<_, _>(None);
                    }
                };

                for (other_project_id, other_manifest) in handle.project_manifests.iter() {
                    if other_project_id == &project_id {
                        continue;
                    }
                    if other_manifest.domain == domain {
                        return trx.cancel(Cancel::DomainTaken {
                            existing_project_id: other_project_id.clone(),
                        });
                    }
                }

                let entry = handle
                    .project_manifests
                    .entry(project_id.clone())
                    .or_insert(WorkerProjectManifest {
                        code_version: 0,
                        domain: domain.clone(),
                        static_cache_state: fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
                            .to_string(),
                        pending_code_version: None,
                        storage: Some(storage.clone()),
                    });
                let replaced = std::mem::replace(&mut entry.domain, domain.clone());
                if entry.storage.is_none() {
                    entry.storage = Some(storage.clone());
                }
                handle.manifest_version += 1;
                trx.commit::<_, _>(Some(replaced).filter(|previous| previous != &domain))
            }
        })
        .await;

    let replaced_domain = match result {
        doc_db::TrxResult::Committed(replaced) => replaced,
        doc_db::TrxResult::Cancelled(Cancel::NotFound) => return Output::NotFound,
        doc_db::TrxResult::Cancelled(Cancel::DomainTaken {
            existing_project_id,
        }) => {
            return Output::DomainTaken {
                existing_project_id,
            };
        }
        doc_db::TrxResult::Conflict(d) => {
            tracing::error!("domain_set trx conflict: {d:?}");
            return Output::InternalError;
        }
        doc_db::TrxResult::Err(e) => {
            tracing::error!("domain_set trx err: {e}");
            return Output::InternalError;
        }
    };

    let Ok(origin_hostname) = std::env::var("FN0_WORKER_ORIGIN_HOSTNAME") else {
        tracing::error!("domain_set: FN0_WORKER_ORIGIN_HOSTNAME not set");
        return Output::InternalError;
    };

    let key_ciphertext =
        match crate::common::vault::encrypt(req.body.private_key_pem.as_bytes()).await {
            Ok(ciphertext) => ciphertext,
            Err(e) => {
                tracing::error!("domain_set key encrypt: {e}");
                return Output::InternalError;
            }
        };

    if let Err(e) = cert_manifest::put(
        &db,
        &domain,
        WorkerHostnameCert {
            project_id: project_id.clone(),
            cert_pem: req.body.certificate_pem.clone(),
            key_ciphertext,
            not_after_epoch_seconds: req.body.not_after_epoch_seconds,
        },
    )
    .await
    {
        tracing::error!("domain_set cert manifest: {e}");
        return Output::InternalError;
    }

    // The replaced certificate is dropped but not revoked: revocation is the
    // user's call on their own zone, and a certificate no worker serves is
    // already unreachable.
    if let Some(replaced) = replaced_domain.as_deref()
        && let Err(e) = cert_manifest::remove(&db, replaced).await
    {
        tracing::error!("domain_set cert manifest remove {replaced}: {e}");
        return Output::InternalError;
    }

    Output::Ok {
        origin_hostname,
        replaced_domain,
    }
}

fn validate_domain(d: &str) -> Result<(), String> {
    if d.is_empty() || d.len() > 253 {
        return Err("invalid length".to_string());
    }
    if !d.contains('.') {
        return Err("must contain a dot".to_string());
    }
    for label in d.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err("invalid label length".to_string());
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err("invalid label characters".to_string());
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err("label cannot start or end with hyphen".to_string());
        }
    }
    Ok(())
}
