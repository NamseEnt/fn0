use crate::common::admin;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub code_version: u64,
    pub fn0_wasmtime_version: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
    Unauthorized,
    Error { message: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }

    let project_id = req.body.project_id.clone();
    let code_version = req.body.code_version;
    let fn0_wasmtime_version = req.body.fn0_wasmtime_version.clone();
    let now = forte_sdk::now();

    let result = doc_db::turso()
        .trx(|trx| {
            let project_id = project_id.clone();
            let fn0_wasmtime_version = fn0_wasmtime_version.clone();
            async move {
                let mut should_enqueue = false;
                // doc-db rejects a second read of the same key in one transaction.
                let mut manifest = trx.get(WorkerManifestDocGet {}).await?;
                let existing = trx
                    .get(CompiledBundleDocGet {
                        project_id: &project_id,
                        code_version,
                    })
                    .await?;

                let newly_compiled = match existing {
                    Some(mut handle) => {
                        if handle.fn0_wasmtime_versions.contains(&fn0_wasmtime_version) {
                            false
                        } else {
                            handle
                                .fn0_wasmtime_versions
                                .push(fn0_wasmtime_version.clone());
                            true
                        }
                    }
                    None => {
                        trx.create(CompiledBundleDoc {
                            project_id: project_id.clone(),
                            code_version,
                            created_at: now,
                            fn0_wasmtime_versions: vec![fn0_wasmtime_version.clone()],
                        })?;
                        true
                    }
                };

                if newly_compiled {
                    let active_fn0_wasmtime_version = trx
                        .get(Fn0WasmtimeVersionDocGet {})
                        .await?
                        .map(|v| v.active.clone());
                    if active_fn0_wasmtime_version.as_deref() == Some(fn0_wasmtime_version.as_str())
                    {
                        match &mut manifest {
                            Some(manifest) => {
                                let entry = manifest
                                    .project_manifests
                                    .entry(project_id.clone())
                                    .or_insert(WorkerProjectManifest {
                                        code_version: 0,
                                        custom_domain: None,
                                        static_cache_state:
                                            fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE.to_string(),
                                        pending_code_version: None,
                                    });
                                if (code_version > entry.code_version
                                    || (code_version == entry.code_version
                                        && entry.pending_code_version.is_none()))
                                    && entry.pending_code_version
                                        .is_none_or(|pending| code_version > pending)
                                {
                                    if entry.code_version == 0 {
                                        entry.code_version = code_version;
                                    }
                                    entry.static_cache_state =
                                        fn0_shared_schema::STATIC_CACHE_STATE_PRE_PURGE.to_string();
                                    entry.pending_code_version = Some(code_version);
                                    manifest.manifest_version += 1;
                                    should_enqueue = true;
                                }
                            }
                            None => {
                                let mut entries = HashMap::new();
                                entries.insert(
                                    project_id.clone(),
                                    WorkerProjectManifest {
                                        code_version,
                                        custom_domain: None,
                                        static_cache_state:
                                            fn0_shared_schema::STATIC_CACHE_STATE_PRE_PURGE.to_string(),
                                        pending_code_version: Some(code_version),
                                    },
                                );
                                trx.create(WorkerManifestDoc {
                                    manifest_version: 1,
                                    project_manifests: entries,
                                })?;
                                should_enqueue = true;
                            }
                        }
                    }
                }
                if !should_enqueue
                    && let Some(manifest) = &manifest
                    && let Some(entry) = manifest.project_manifests.get(&project_id)
                    && entry.pending_code_version == Some(code_version)
                    && entry.static_cache_state
                        != fn0_shared_schema::STATIC_CACHE_STATE_ACTIVE
                {
                    should_enqueue = true;
                }
                trx.commit::<_, ()>(should_enqueue)
            }
        })
        .await;

    match result {
        doc_db::TrxResult::Committed(true) => {
            if let Err(error) = crate::enqueue::static_cache_purge(
                crate::queue_task::static_cache_purge::Input {
                    project_id,
                    code_version,
                },
            )
            .await
            {
                return Output::Error {
                    message: error.to_string(),
                };
            }
            Output::Ok
        }
        doc_db::TrxResult::Committed(false) => Output::Ok,
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(_) => Output::Error {
            message: "conflict".to_string(),
        },
        doc_db::TrxResult::Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
