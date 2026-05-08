use crate::common::auth;
use crate::common::project_name::is_valid_project_name;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub new_name: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
    NotLoggedIn,
    NotFound,
    InvalidName,
    InternalError,
}

enum Cancel {
    NotFound,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let new_name = req.body.new_name.trim().to_string();
    if !is_valid_project_name(&new_name) {
        return Output::InvalidName;
    }

    let project_id = req.body.project_id.clone();
    let github_id = user.github_id;

    let result = doc_db::turso()
        .trx(|trx| {
            let project_id = project_id.clone();
            let new_name = new_name.clone();
            async move {
                let mut project = match trx
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
                if project.name == new_name {
                    return trx.commit::<_, _>(());
                }
                project.name = new_name.clone();

                let mut user_handle = trx
                    .get(UserDocGet { github_id })
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("authenticated user has no UserDoc"))?;
                for entry in user_handle.projects.iter_mut() {
                    if entry.project_id == project_id {
                        entry.name = new_name.clone();
                        break;
                    }
                }
                trx.commit::<_, _>(())
            }
        })
        .await;

    match result {
        doc_db::TrxResult::Committed(()) => Output::Ok,
        doc_db::TrxResult::Cancelled(Cancel::NotFound) => Output::NotFound,
        doc_db::TrxResult::Conflict(d) => {
            tracing::error!("rename_project trx conflict: {d:?}");
            Output::InternalError
        }
        doc_db::TrxResult::Err(e) => {
            tracing::error!("rename_project trx err: {e}");
            Output::InternalError
        }
    }
}
