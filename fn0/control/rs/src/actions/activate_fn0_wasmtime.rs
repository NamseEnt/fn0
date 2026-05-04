use crate::common::admin;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub version: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok { old_active: Option<String> },
    Unauthorized,
    Error { message: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }

    let version = req.body.version.clone();

    let result = doc_db::turso()
        .trx(|trx| {
            let version = version.clone();
            async move {
                let old_active = match trx.get(Fn0WasmtimeVersionDocGet {}).await? {
                    Some(mut handle) => {
                        let prev = if handle.active != version {
                            Some(handle.active.clone())
                        } else {
                            None
                        };
                        handle.active = version;
                        handle.pending = None;
                        prev
                    }
                    None => {
                        trx.create(Fn0WasmtimeVersionDoc {
                            active: version,
                            pending: None,
                        })?;
                        None
                    }
                };
                trx.commit::<_, ()>(old_active)
            }
        })
        .await;

    match result {
        doc_db::TrxResult::Committed(old_active) => Output::Ok { old_active },
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(_) => Output::Error {
            message: "conflict".to_string(),
        },
        doc_db::TrxResult::Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
