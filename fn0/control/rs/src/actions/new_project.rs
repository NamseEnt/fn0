use crate::common::auth;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub name: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok { project_id: String },
    NotLoggedIn,
    Error { message: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::current_user(req.jar).await else {
        return Output::NotLoggedIn;
    };

    let name = req.body.name.trim().to_string();
    if name.is_empty() {
        return Output::Error {
            message: "name cannot be empty".to_string(),
        };
    }

    let project_id = match generate_project_id().await {
        Ok(id) => id,
        Err(e) => return Output::Error { message: e },
    };
    let now = forte_sdk::now();
    let github_id = user.github_id;

    let project_id_for_trx = project_id.clone();
    let name_for_trx = name.clone();

    let result = doc_db::turso()
        .trx(|trx| {
            let project_id = project_id_for_trx.clone();
            let name = name_for_trx.clone();
            async move {
                let mut user_handle = trx
                    .get(UserDocGet { github_id })
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("user not found"))?;
                user_handle.projects.push(ProjectIndexEntry {
                    project_id: project_id.clone(),
                    name: name.clone(),
                });
                trx.create(ProjectDoc {
                    project_id,
                    owner_github_id: github_id,
                    name,
                    created_at: now,
                })?;
                trx.commit::<_, ()>(())
            }
        })
        .await;

    match result {
        doc_db::TrxResult::Committed(()) => Output::Ok { project_id },
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(_) => Output::Error {
            message: "conflict".to_string(),
        },
        doc_db::TrxResult::Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}

const PROJECT_ID_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const PROJECT_ID_LEN: usize = 8;

async fn generate_project_id() -> Result<String, String> {
    let bytes = rand::get_random_bytes(PROJECT_ID_LEN).await;
    if bytes.len() < PROJECT_ID_LEN {
        return Err("rng returned wrong length".to_string());
    }
    let alpha_len = PROJECT_ID_ALPHABET.len() as u8;
    let mut out = String::with_capacity(PROJECT_ID_LEN);
    for b in bytes.iter().take(PROJECT_ID_LEN) {
        out.push(PROJECT_ID_ALPHABET[(b % alpha_len) as usize] as char);
    }
    Ok(out)
}
