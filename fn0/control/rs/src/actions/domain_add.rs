use crate::common::auth;
use crate::common::byoc::ProjectStorage;
use crate::common::{cert_manifest, origin_cert};
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub domain: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        /// The one thing the user has to do: a proxied A record for `domain`
        /// pointing here.
        origin_ip: String,
        /// `false` for projects still on the platform's Cloudflare account,
        /// where a SaaS custom hostname is registered instead.
        needs_dns_record: bool,
    },
    NotLoggedIn,
    NotFound,
    InvalidDomain {
        message: String,
    },
    CertificateFailed {
        message: String,
    },
    DomainTaken {
        existing_project_id: String,
    },
    AlreadyHasDomain {
        current_domain: String,
    },
    InternalError,
}

enum Cancel {
    NotFound,
    DomainTaken { existing_project_id: String },
    AlreadyHasDomain { current_domain: String },
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

    let result = doc_db::turso()
        .trx(|trx| {
            let project_id = project_id.clone();
            let domain = domain.clone();
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
                                custom_domain: Some(domain.clone()),
                                static_cache_state: fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
                                    .to_string(),
                                pending_code_version: None,
                                storage: None,
                            },
                        );
                        trx.create(WorkerManifestDoc {
                            manifest_version: 1,
                            project_manifests: entries,
                        })?;
                        return trx.commit::<_, _>(());
                    }
                };

                for (other_project_id, other_manifest) in handle.project_manifests.iter() {
                    if other_project_id == &project_id {
                        continue;
                    }
                    if other_manifest.custom_domain.as_deref() == Some(domain.as_str()) {
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
                        custom_domain: None,
                        static_cache_state: fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
                            .to_string(),
                        pending_code_version: None,
                        storage: None,
                    });
                if let Some(current) = entry.custom_domain.clone()
                    && current != domain
                {
                    return trx.cancel(Cancel::AlreadyHasDomain {
                        current_domain: current,
                    });
                }
                entry.custom_domain = Some(domain.clone());
                handle.manifest_version += 1;
                trx.commit::<_, _>(())
            }
        })
        .await;

    match result {
        doc_db::TrxResult::Committed(()) => {}
        doc_db::TrxResult::Cancelled(Cancel::NotFound) => return Output::NotFound,
        doc_db::TrxResult::Cancelled(Cancel::DomainTaken {
            existing_project_id,
        }) => {
            return Output::DomainTaken {
                existing_project_id,
            };
        }
        doc_db::TrxResult::Cancelled(Cancel::AlreadyHasDomain { current_domain }) => {
            return Output::AlreadyHasDomain { current_domain };
        }
        doc_db::TrxResult::Conflict(d) => {
            tracing::error!("domain_add trx conflict: {d:?}");
            return Output::InternalError;
        }
        doc_db::TrxResult::Err(e) => {
            tracing::error!("domain_add trx err: {e}");
            return Output::InternalError;
        }
    }

    let db = doc_db::turso();
    let storage = match ProjectStorage::resolve(&db, &project_id).await {
        Ok(storage) => storage,
        Err(e) => {
            tracing::error!("domain_add ProjectStorage::resolve: {e}");
            return Output::InternalError;
        }
    };

    // A project on the platform's account still goes through Cloudflare for
    // SaaS: there is no user zone to issue an origin certificate on, and the
    // request never reaches this fleet with their hostname in SNI.
    if !storage.is_byoc() {
        if let Err(e) =
            crate::enqueue::cloudflare_register(crate::queue_task::cloudflare_register::Input {
                domain: domain.clone(),
            })
            .await
        {
            tracing::error!("domain_add enqueue cloudflare_register: {e}");
            return Output::InternalError;
        }
        return Output::Ok {
            origin_ip: String::new(),
            needs_dns_record: false,
        };
    }

    let Ok(origin_ip) = std::env::var("FN0_WORKER_ORIGIN_IP") else {
        tracing::error!("domain_add: FN0_WORKER_ORIGIN_IP not set");
        return Output::InternalError;
    };

    let now = forte_sdk::now();
    let issued = match origin_cert::issue(&storage, &domain, now).await {
        Ok(issued) => issued,
        Err(e) => {
            tracing::error!("domain_add origin certificate: {e}");
            return Output::CertificateFailed {
                message: format!("{e}"),
            };
        }
    };
    let key_ciphertext =
        match crate::common::vault::encrypt(issued.private_key_pem.as_bytes()).await {
            Ok(ciphertext) => ciphertext,
            Err(e) => {
                tracing::error!("domain_add key encrypt: {e}");
                return Output::InternalError;
            }
        };

    if let Err(e) = cert_manifest::put(
        &db,
        &domain,
        WorkerHostnameCert {
            project_id: project_id.clone(),
            cert_pem: issued.certificate_pem,
            key_ciphertext,
            not_after_epoch_seconds: issued.not_after_epoch_seconds,
        },
    )
    .await
    {
        tracing::error!("domain_add cert manifest: {e}");
        return Output::InternalError;
    }

    Output::Ok {
        origin_ip,
        needs_dns_record: true,
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
