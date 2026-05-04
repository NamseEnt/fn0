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

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectIndexEntry {
    pub project_id: String,
    pub name: String,
}

#[forte_doc]
pub struct UserDoc {
    #[pk]
    pub github_id: i64,
    pub github_login: String,
    pub created_at: DateTime,
    pub cli_tokens: Vec<CliTokenEntry>,
    pub web_sessions: Vec<WebSessionEntry>,
    pub projects: Vec<ProjectIndexEntry>,
}

#[forte_doc]
pub struct ProjectDoc {
    #[pk]
    pub project_id: String,
    pub owner_github_id: i64,
    pub name: String,
    pub created_at: DateTime,
}

#[forte_doc]
pub struct CompiledBundleDoc {
    #[pk]
    pub project_id: String,
    #[pk]
    pub last_modified: String,
    pub fn0_wasmtime_versions: Vec<String>,
}

#[forte_doc]
pub struct Fn0WasmtimeVersionDoc {
    pub active: String,
    pub pending: Option<String>,
}
