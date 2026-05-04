use crate::common::admin;
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub enum Output {
    Ok {
        old_active: String,
        new_active: String,
    },
    NoPending,
    NoActiveVersion,
    Unauthorized,
    Error {
        message: String,
    },
}

enum Cancel {
    NoPending,
    NoActive,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if !admin::verify(req.headers) {
        return Output::Unauthorized;
    }

    let result = doc_db::turso()
        .trx(|trx| async move {
            let mut handle = match trx.get(Fn0WasmtimeVersionDocGet {}).await? {
                Some(h) => h,
                None => return trx.cancel(Cancel::NoActive),
            };
            let Some(pending) = handle.pending.clone() else {
                return trx.cancel(Cancel::NoPending);
            };
            let old_active = handle.active.clone();
            handle.active = pending.clone();
            handle.pending = None;
            trx.commit((old_active, pending))
        })
        .await;

    match result {
        doc_db::TrxResult::Committed((old_active, new_active)) => Output::Ok {
            old_active,
            new_active,
        },
        doc_db::TrxResult::Cancelled(Cancel::NoPending) => Output::NoPending,
        doc_db::TrxResult::Cancelled(Cancel::NoActive) => Output::NoActiveVersion,
        doc_db::TrxResult::Conflict(_) => Output::Error {
            message: "conflict".to_string(),
        },
        doc_db::TrxResult::Err(e) => Output::Error {
            message: e.to_string(),
        },
    }
}
