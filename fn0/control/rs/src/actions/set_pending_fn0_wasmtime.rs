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
    NoActiveVersion,
    AlreadyActive,
    Error { message: String },
}

enum Cancel {
    NoActive,
    AlreadyActive,
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
                let mut handle = match trx.get(Fn0WasmtimeVersionDocGet {}).await? {
                    Some(h) => h,
                    None => return trx.cancel(Cancel::NoActive),
                };
                if handle.active == version {
                    return trx.cancel(Cancel::AlreadyActive);
                }
                handle.pending = Some(version);
                trx.commit(())
            }
        })
        .await;

    match result {
        doc_db::TrxResult::Committed(()) => Output::Ok,
        doc_db::TrxResult::Cancelled(Cancel::NoActive) => Output::NoActiveVersion,
        doc_db::TrxResult::Cancelled(Cancel::AlreadyActive) => Output::AlreadyActive,
        doc_db::TrxResult::Conflict(_) => Output::Error {
            message: "conflict".to_string(),
        },
        doc_db::TrxResult::Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
