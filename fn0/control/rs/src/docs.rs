use forte_sdk::*;
use serde::{Deserialize, Serialize};

pub use doc_db::DbRequest;

#[derive(Serialize, Deserialize, Clone)]
pub struct CliTokenEntry {
    pub id: String,
    pub label: String,
    pub created_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebSessionEntry {
    pub token: String,
    pub created_at: DateTime,
}

#[forte_doc]
pub struct UserDoc {
    #[pk]
    pub github_id: i64,
    pub github_login: String,
    pub created_at: DateTime,
    pub cli_tokens: Vec<CliTokenEntry>,
    pub web_sessions: Vec<WebSessionEntry>,
}
