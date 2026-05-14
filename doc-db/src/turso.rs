mod database;
mod transaction;

use bytes::Bytes;

const UPSERT_DOC_SQL: &str = "INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0) ON CONFLICT(pk, sk) DO UPDATE SET data = excluded.data, version = docs.version + 1";

#[derive(Clone)]
pub(crate) struct TursoDatabase {
    http_url: String,
    auth_token: String,
}

#[derive(Clone)]
pub struct StoredDoc {
    pub(crate) data: Bytes,
    pub(crate) version: i64,
}

pub(crate) struct TursoTransaction {
    db: TursoDatabase,
    baton: Option<String>,
}
