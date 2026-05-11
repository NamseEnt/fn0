use crate::common::auth;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub code_version: u64,
}

#[derive(Serialize)]
pub enum Output {
    Done {
        active_version: String,
        pending_version: Option<String>,
        pending_compiled: bool,
        compiled_versions: Vec<String>,
    },
    Pending {
        active_version: String,
        pending_version: Option<String>,
        pending_compiled: bool,
        compiled_versions: Vec<String>,
    },
    NoActiveVersion,
    NotLoggedIn,
    NotFound,
    InternalError,
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
            tracing::error!("deploy_status ProjectDocGet: {e}");
            return Output::InternalError;
        }
    };
    if project.owner_github_id != user.github_id {
        return Output::NotFound;
    }

    let started = time_wasi::Instant::now().await;
    loop {
        let snapshot = match load_state(&db, &req.body.project_id, req.body.code_version).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("deploy_status load_state: {e}");
                return Output::InternalError;
            }
        };

        let Some(active_version) = snapshot.active.clone() else {
            return Output::NoActiveVersion;
        };

        let active_compiled = snapshot.compiled_versions.contains(&active_version);
        let pending_compiled = snapshot
            .pending
            .as_ref()
            .map(|p| snapshot.compiled_versions.contains(p))
            .unwrap_or(false);

        if active_compiled {
            return Output::Done {
                active_version,
                pending_version: snapshot.pending,
                pending_compiled,
                compiled_versions: snapshot.compiled_versions,
            };
        }

        if started.elapsed().await >= POLL_TIMEOUT {
            return Output::Pending {
                active_version,
                pending_version: snapshot.pending,
                pending_compiled,
                compiled_versions: snapshot.compiled_versions,
            };
        }

        time_wasi::sleep(POLL_INTERVAL).await;
    }
}

struct Snapshot {
    active: Option<String>,
    pending: Option<String>,
    compiled_versions: Vec<String>,
}

async fn load_state(
    db: &doc_db::Database,
    project_id: &str,
    code_version: u64,
) -> anyhow::Result<Snapshot> {
    let (compiled_doc, version_doc) = (
        CompiledBundleDocGet {
            project_id,
            code_version,
        },
        Fn0WasmtimeVersionDocGet {},
    )
        .send_with(db)
        .await?;

    let compiled_versions = compiled_doc
        .map(|d| d.fn0_wasmtime_versions)
        .unwrap_or_default();

    let (active, pending) = match version_doc {
        Some(v) => (Some(v.active), v.pending),
        None => (None, None),
    };

    Ok(Snapshot {
        active,
        pending,
        compiled_versions,
    })
}
