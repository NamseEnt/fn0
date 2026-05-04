use crate::common::auth;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub last_modified: String,
}

#[derive(Serialize)]
pub enum Output {
    Done {
        compiled_versions: Vec<String>,
        target_versions: Vec<String>,
    },
    Pending {
        compiled_versions: Vec<String>,
        target_versions: Vec<String>,
        missing: Vec<String>,
    },
    NotLoggedIn,
    NotFound,
    Forbidden,
    Error {
        message: String,
    },
}

const POLL_INTERVAL: time_wasi::Duration = time_wasi::Duration::from_secs(2);
const POLL_TIMEOUT: time_wasi::Duration = time_wasi::Duration::from_secs(25);

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
            return Output::Error {
                message: e.to_string(),
            };
        }
    };
    if project.owner_github_id != user.github_id {
        return Output::Forbidden;
    }

    let started = time_wasi::Instant::now().await;
    loop {
        let (compiled, target) = match load_state(&db, &req.body.project_id, &req.body.last_modified).await {
            Ok(v) => v,
            Err(e) => {
                return Output::Error {
                    message: e.to_string(),
                };
            }
        };

        let missing: Vec<String> = target
            .iter()
            .filter(|v| !compiled.contains(v))
            .cloned()
            .collect();

        if missing.is_empty() && !target.is_empty() {
            return Output::Done {
                compiled_versions: compiled,
                target_versions: target,
            };
        }

        if started.elapsed().await >= POLL_TIMEOUT {
            return Output::Pending {
                compiled_versions: compiled,
                target_versions: target,
                missing,
            };
        }

        time_wasi::sleep(POLL_INTERVAL).await;
    }
}

async fn load_state(
    db: &doc_db::Database,
    project_id: &str,
    last_modified: &str,
) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let (compiled_doc, version_doc) = (
        CompiledBundleDocGet {
            project_id,
            last_modified,
        },
        Fn0WasmtimeVersionDocGet {},
    )
        .send_with(db)
        .await?;

    let compiled = compiled_doc
        .map(|d| d.fn0_wasmtime_versions)
        .unwrap_or_default();

    let mut target = Vec::new();
    if let Some(v) = version_doc {
        target.push(v.active);
        if let Some(p) = v.pending {
            target.push(p);
        }
    }
    Ok((compiled, target))
}
