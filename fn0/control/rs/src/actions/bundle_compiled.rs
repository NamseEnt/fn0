use crate::common::admin;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub last_modified: String,
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
    let last_modified = req.body.last_modified.clone();
    let version = req.body.fn0_wasmtime_version.clone();

    let result = doc_db::turso()
        .trx(|trx| {
            let project_id = project_id.clone();
            let last_modified = last_modified.clone();
            let version = version.clone();
            async move {
                let existing = trx
                    .get(CompiledBundleDocGet {
                        project_id: &project_id,
                        last_modified: &last_modified,
                    })
                    .await?;

                match existing {
                    Some(mut handle) => {
                        if !handle.fn0_wasmtime_versions.contains(&version) {
                            handle.fn0_wasmtime_versions.push(version);
                        }
                    }
                    None => {
                        trx.create(CompiledBundleDoc {
                            project_id,
                            last_modified,
                            fn0_wasmtime_versions: vec![version],
                        })?;
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
