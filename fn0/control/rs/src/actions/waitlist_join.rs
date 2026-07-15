use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const TIERS: &[&str] = &["free", "one_dollar"];
const EMAIL_MAX_LEN: usize = 254;

#[derive(Deserialize)]
pub struct Input {
    pub email: String,
    pub tier: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok,
    InvalidEmail,
    InvalidTier,
    InternalError,
}

fn is_valid_email(email: &str) -> bool {
    if email.len() > EMAIL_MAX_LEN || email.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    domain.split('.').count() >= 2 && domain.split('.').all(|part| !part.is_empty())
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let email = req.body.email.trim().to_lowercase();
    if !is_valid_email(&email) {
        return Output::InvalidEmail;
    }
    let tier = req.body.tier.clone();
    if !TIERS.contains(&tier.as_str()) {
        return Output::InvalidTier;
    }

    let now = forte_sdk::now();
    let result = doc_db::turso()
        .trx(|trx| {
            let email = email.clone();
            let tier = tier.clone();
            async move {
                match trx.get(WaitlistDocGet { email: &email }).await? {
                    Some(mut entry) => {
                        entry.tier_interest = tier;
                    }
                    None => {
                        trx.create(WaitlistDoc {
                            email,
                            tier_interest: tier,
                            created_at: now,
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
        doc_db::TrxResult::Conflict(d) => {
            tracing::error!("waitlist_join trx conflict: {d:?}");
            Output::InternalError
        }
        doc_db::TrxResult::Err(e) => {
            tracing::error!("waitlist_join trx err: {e}");
            Output::InternalError
        }
    }
}
