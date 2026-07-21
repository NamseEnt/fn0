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
    Ok { removed_domain: String },
    NotLoggedIn,
    NotFound,
    NoDomain,
    InternalError,
}

enum Cancel {
    NotFound,
    NoDomain,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let project_id = req.body.project_id.clone();
    let github_id = user.github_id;

    let result = doc_db::turso()
        .trx(|trx| {
            let project_id = project_id.clone();
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
                    None => return trx.cancel(Cancel::NoDomain),
                };
                let entry = match handle.project_manifests.get_mut(&project_id) {
                    Some(e) => e,
                    None => return trx.cancel(Cancel::NoDomain),
                };
                let removed = match entry.custom_domain.take() {
                    Some(d) => d,
                    None => return trx.cancel(Cancel::NoDomain),
                };
                handle.manifest_version += 1;
                trx.commit::<_, _>(removed)
            }
        })
        .await;

    let removed_domain = match result {
        doc_db::TrxResult::Committed(d) => d,
        doc_db::TrxResult::Cancelled(Cancel::NotFound) => return Output::NotFound,
        doc_db::TrxResult::Cancelled(Cancel::NoDomain) => return Output::NoDomain,
        doc_db::TrxResult::Conflict(d) => {
            tracing::error!("domain_remove trx conflict: {d:?}");
            return Output::InternalError;
        }
        doc_db::TrxResult::Err(e) => {
            tracing::error!("domain_remove trx err: {e}");
            return Output::InternalError;
        }
    };

    if let Err(e) =
        crate::enqueue::cloudflare_unregister(crate::queue_task::cloudflare_unregister::Input {
            domain: removed_domain.clone(),
        })
        .await
    {
        tracing::error!("domain_remove enqueue cloudflare_unregister: {e}");
        return Output::InternalError;
    }

    Output::Ok { removed_domain }
}
