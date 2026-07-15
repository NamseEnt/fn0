use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const QUERY_PAGE_LIMIT: usize = 200;

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub struct Entry {
    pub email: String,
    pub tier_interest: String,
    pub created_at: DateTime,
}

#[derive(Serialize)]
pub struct Output {
    pub total: usize,
    pub entries: Vec<Entry>,
}

pub async fn handle(_input: Input) -> anyhow::Result<Output> {
    let db = doc_db::turso();
    let mut entries: Vec<Entry> = Vec::new();
    let mut after: Option<String> = None;

    loop {
        let docs: Vec<WaitlistDoc> = WaitlistDocQuery {
            email: after.clone(),
            limit: Some(QUERY_PAGE_LIMIT),
        }
        .send_with(&db)
        .await?;
        if docs.is_empty() {
            break;
        }
        after = docs.last().map(|d| d.email.clone());
        entries.extend(docs.into_iter().map(|d| Entry {
            email: d.email,
            tier_interest: d.tier_interest,
            created_at: d.created_at,
        }));
    }

    Ok(Output {
        total: entries.len(),
        entries,
    })
}
