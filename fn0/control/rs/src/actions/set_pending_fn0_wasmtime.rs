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
    Ok,
    Unauthorized,
    Error { message: String },
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }

    let version = req.body.version.clone();

    let result = doc_db::database()
        .trx(|trx| {
            let version = version.clone();
            async move {
                match trx.get(Fn0WasmtimeVersionDocGet {}).await? {
                    Some(mut handle) => {
                        if handle.active != version {
                            handle.pending = Some(version);
                        } else if handle.pending.is_some() {
                            handle.pending = None;
                        }
                    }
                    None => {
                        trx.create(Fn0WasmtimeVersionDoc {
                            active: version,
                            pending: None,
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
