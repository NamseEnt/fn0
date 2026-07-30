use crate::common::auth;
use crate::docs::*;
use fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE;
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

    let started = time_wasi::Instant::now();
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
        let static_cache_active = snapshot.active_code_version == Some(req.body.code_version)
            && snapshot.static_cache_state.as_deref() == Some(STATIC_CACHE_STATE_ACTIVE);

        if active_compiled && static_cache_active {
            return Output::Done {
                active_version,
                pending_version: snapshot.pending,
                pending_compiled,
                compiled_versions: snapshot.compiled_versions,
            };
        }

        if started.elapsed() >= POLL_TIMEOUT {
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
    active_code_version: Option<u64>,
    static_cache_state: Option<String>,
}

async fn load_state(
    db: &doc_db::Database,
    project_id: &str,
    code_version: u64,
) -> anyhow::Result<Snapshot> {
    let (compiled_doc, version_doc, manifest_doc) = (
        CompiledBundleDocGet {
            project_id,
            code_version,
        },
        Fn0WasmtimeVersionDocGet {},
        WorkerManifestDocGet {},
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
    let (active_code_version, static_cache_state) = manifest_doc
        .and_then(|manifest| manifest.project_manifests.get(project_id).cloned())
        .map(|entry| (Some(entry.code_version), Some(entry.static_cache_state)))
        .unwrap_or((None, None));

    Ok(Snapshot {
        active,
        pending,
        compiled_versions,
        active_code_version,
        static_cache_state,
    })
}
