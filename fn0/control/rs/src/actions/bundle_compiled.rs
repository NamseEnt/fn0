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
                        match trx.get(WorkerManifestDocGet {}).await? {
                            Some(mut manifest) => {
                                let entry = manifest
                                    .project_manifests
                                    .entry(project_id.clone())
                                    .or_insert(WorkerProjectManifest {
                                        code_version: 0,
                                        custom_domain: None,
                                    });
                                if code_version > entry.code_version {
                                    entry.code_version = code_version;
                                    manifest.manifest_version += 1;
                                }
                            }
                            None => {
                                let mut entries = HashMap::new();
                                entries.insert(
                                    project_id,
                                    WorkerProjectManifest {
                                        code_version,
                                        custom_domain: None,
                                    },
                                );
                                trx.create(WorkerManifestDoc {
                                    manifest_version: 1,
                                    project_manifests: entries,
                                })?;
                            }
                        }
                    }
                }
                trx.commit::<_, ()>(())
            }
        })
        .await;

    match result {
        doc_db::TrxResult::Committed(()) => Output::Ok,
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(_) => Output::Error {
            message: "conflict".to_string(),
        },
        doc_db::TrxResult::Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
