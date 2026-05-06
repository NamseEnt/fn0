use crate::common::auth;
use crate::common::queue;
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
    Ok,
    NotLoggedIn,
    Forbidden,
    NotFound,
    InvalidDomain { message: String },
    DomainTaken { existing_project_id: String },
    AlreadyHasDomain { current_domain: String },
    Error { message: String },
}

enum Cancel {
    NotFound,
    Forbidden,
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
                    return trx.cancel(Cancel::Forbidden);
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
        doc_db::TrxResult::Cancelled(Cancel::Forbidden) => return Output::Forbidden,
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
        doc_db::TrxResult::Conflict(_) => {
            return Output::Error {
                message: "manifest trx conflict".to_string(),
            };
        }
        doc_db::TrxResult::Err(e) => {
            return Output::Error {
                message: e.to_string(),
            };
        }
    }

    if let Err(e) = queue::enqueue(
        "fn0-control",
        "cloudflare_register",
        serde_json::json!({ "domain": domain }),
    )
    .await
    {
        return Output::Error {
            message: format!("enqueue cloudflare_register: {e}"),
        };
    }

    Output::Ok
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
