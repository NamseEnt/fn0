use super::*;
use crate::BatchOp;
use crate::runtime;
use anyhow::{Result, bail};
use bytes::Bytes;
use libsql_hrana::proto::*;
use std::sync::Arc;

enum RetryKind {
    Schema,
    Busy,
}

const CREATE_DOCS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS docs (pk TEXT, sk TEXT, data BLOB, version INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (pk, sk))";
const ADD_VERSION_COLUMN_SQL: &str =
    "ALTER TABLE docs ADD COLUMN version INTEGER NOT NULL DEFAULT 0";

fn normalize_pipeline_url(url: &str) -> String {
    let (scheme, rest) = match url.split_once("://") {
        Some((s, r)) => (s, r),
        None => ("https", url),
    };
    let scheme = match scheme {
        "libsql" | "" => "https",
        other => other,
    };
    let host = rest.split('/').next().unwrap_or(rest);
    format!("{scheme}://{host}/v2/pipeline")
}

impl TursoDatabase {
    pub(crate) fn new(url: String, auth_token: String) -> Self {
        let http_url = normalize_pipeline_url(&url);
        Self {
            http_url,
            auth_token,
        }
    }
    async fn execute_pipeline(&self, requests: Vec<StreamRequest>) -> Result<PipelineRespBody> {
        self.execute_pipeline_with_baton(None, requests).await
    }

    pub(super) async fn execute_pipeline_with_baton(
        &self,
        baton: Option<String>,
        requests: Vec<StreamRequest>,
    ) -> Result<PipelineRespBody> {
        let body = PipelineReqBody { baton, requests };
        let body_bytes = serde_json::to_vec(&body)
            .map_err(|e| anyhow::anyhow!("Failed to serialize body: {}", e))?;

        let auth = if self.auth_token.is_empty() {
            None
        } else {
            Some(self.auth_token.as_str())
        };
        let response_bytes = runtime::http_post_json(&self.http_url, body_bytes, auth).await?;

        let resp: PipelineRespBody = serde_json::from_slice(&response_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to parse Turso response: {}", e))?;
        Ok(resp)
    }

    async fn create_table(&self) -> Result<()> {
        let response = self
            .execute_pipeline(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some(CREATE_DOCS_TABLE_SQL.to_string()),
                        sql_id: None,
                        args: vec![],
                        named_args: vec![],
                        want_rows: Some(false),
                        replication_index: None,
                    },
                }),
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some(ADD_VERSION_COLUMN_SQL.to_string()),
                        sql_id: None,
                        args: vec![],
                        named_args: vec![],
                        want_rows: Some(false),
                        replication_index: None,
                    },
                }),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        for (idx, result) in response.results.into_iter().enumerate() {
            if let StreamResult::Error { error } = result {
                if idx == 1 && Self::is_duplicate_column_error(&error.message) {
                    continue;
                }
                bail!("Create table error: {}", error.message);
            }
        }

        Ok(())
    }

    fn is_table_not_found_error(error_message: &str) -> bool {
        error_message.contains("no such table")
    }

    fn is_duplicate_column_error(error_message: &str) -> bool {
        error_message.contains("duplicate column name")
    }

    fn is_missing_version_column_error(error_message: &str) -> bool {
        error_message.contains("no such column: version")
            || error_message.contains("has no column named version")
    }

    fn is_schema_error(error_message: &str) -> bool {
        Self::is_table_not_found_error(error_message)
            || Self::is_missing_version_column_error(error_message)
    }

    fn is_busy_error(error_message: &str) -> bool {
        let msg = error_message.to_ascii_lowercase();
        msg.contains("database is locked") || msg.contains("sqlite_busy") || msg.contains("busy")
    }

    async fn busy_backoff(attempt: u32) {
        const BASE_MS: u64 = 5;
        const CAP_MS: u64 = 200;
        let ceiling = BASE_MS.checked_shl(attempt).unwrap_or(CAP_MS).min(CAP_MS);
        let mut buf = [0u8; 8];
        runtime::random_bytes(&mut buf);
        let raw = u64::from_le_bytes(buf);
        let delay_ms = raw % (ceiling + 1);
        runtime::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }

    pub(crate) async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        for retry in 0..2 {
            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some("SELECT data FROM docs WHERE pk = ? AND sk = ?".to_string()),
                            sql_id: None,
                            args: vec![
                                Value::Text {
                                    value: pk.to_string().into(),
                                },
                                Value::Text {
                                    value: sk.to_string().into(),
                                },
                            ],
                            named_args: vec![],
                            want_rows: Some(true),
                            replication_index: None,
                        },
                    }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                match result {
                    StreamResult::Ok { response } => match response {
                        StreamResponse::Execute(exec_resp) => {
                            if let Some(Value::Blob { value }) = exec_resp
                                .result
                                .rows
                                .first()
                                .and_then(|row| row.values.first())
                            {
                                return Ok(Some(value.clone()));
                            }
                            return Ok(None);
                        }
                        StreamResponse::Close(_) => continue,
                        _ => {}
                    },
                    StreamResult::Error { error } => {
                        if retry == 0 && Self::is_table_not_found_error(&error.message) {
                            self.create_table().await?;
                            should_retry = true;
                            break;
                        }
                        bail!("Turso error: {}", error.message);
                    }
                    StreamResult::None => {}
                }
            }

            if !should_retry {
                return Ok(None);
            }
        }

        Ok(None)
    }

    pub(crate) async fn get_with_version(&self, pk: &str, sk: &str) -> Result<Option<StoredDoc>> {
        for retry in 0..2 {
            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some(
                                "SELECT data, version FROM docs WHERE pk = ? AND sk = ?"
                                    .to_string(),
                            ),
                            sql_id: None,
                            args: vec![
                                Value::Text {
                                    value: pk.to_string().into(),
                                },
                                Value::Text {
                                    value: sk.to_string().into(),
                                },
                            ],
                            named_args: vec![],
                            want_rows: Some(true),
                            replication_index: None,
                        },
                    }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                match result {
                    StreamResult::Ok { response } => match response {
                        StreamResponse::Execute(exec_resp) => {
                            if let Some(row) = exec_resp.result.rows.first()
                                && let (
                                    Some(Value::Blob { value: data }),
                                    Some(Value::Integer { value: version }),
                                ) = (row.values.first(), row.values.get(1))
                            {
                                return Ok(Some(StoredDoc {
                                    data: data.clone(),
                                    version: *version,
                                }));
                            }
                            return Ok(None);
                        }
                        StreamResponse::Close(_) => continue,
                        _ => {}
                    },
                    StreamResult::Error { error } => {
                        if retry == 0 && Self::is_schema_error(&error.message) {
                            self.create_table().await?;
                            should_retry = true;
                            break;
                        }
                        bail!("Turso error: {}", error.message);
                    }
                    StreamResult::None => {}
                }
            }

            if !should_retry {
                return Ok(None);
            }
        }

        Ok(None)
    }

    pub(crate) async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        for retry in 0..2 {
            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some(UPSERT_DOC_SQL.to_string()),
                            sql_id: None,
                            args: vec![
                                Value::Text {
                                    value: pk.to_string().into(),
                                },
                                Value::Text {
                                    value: sk.to_string().into(),
                                },
                                Value::Blob {
                                    value: data.to_vec().into(),
                                },
                            ],
                            named_args: vec![],
                            want_rows: Some(false),
                            replication_index: None,
                        },
                    }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                if let StreamResult::Error { error } = result {
                    if retry == 0 && Self::is_schema_error(&error.message) {
                        self.create_table().await?;
                        should_retry = true;
                        break;
                    }
                    bail!("Put error: {}", error.message);
                }
            }

            if !should_retry {
                return Ok(());
            }
        }

        Ok(())
    }

    pub(crate) async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        for retry in 0..2 {
            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some("DELETE FROM docs WHERE pk = ? AND sk = ?".to_string()),
                            sql_id: None,
                            args: vec![
                                Value::Text {
                                    value: pk.to_string().into(),
                                },
                                Value::Text {
                                    value: sk.to_string().into(),
                                },
                            ],
                            named_args: vec![],
                            want_rows: Some(false),
                            replication_index: None,
                        },
                    }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                if let StreamResult::Error { error } = result {
                    if retry == 0 && Self::is_table_not_found_error(&error.message) {
                        self.create_table().await?;
                        should_retry = true;
                        break;
                    }
                    bail!("Delete error: {}", error.message);
                }
            }

            if !should_retry {
                return Ok(());
            }
        }

        Ok(())
    }

    pub(crate) async fn query<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        pk: S1,
        after_sk: Option<S2>,
        limit: usize,
    ) -> Result<Vec<(String, Bytes)>> {
        let pk: Arc<str> = pk.as_ref().to_string().into();
        let after_sk: Option<Arc<str>> = after_sk.map(|s| s.as_ref().to_string().into());

        for retry in 0..2 {
            let (sql, args) = match after_sk.clone() {
                Some(sk) => (
                    "SELECT sk, data FROM docs WHERE pk = ? AND sk > ? ORDER BY sk LIMIT ?"
                        .to_string(),
                    vec![
                        Value::Text { value: pk.clone() },
                        Value::Text { value: sk },
                        Value::Integer {
                            value: limit as i64,
                        },
                    ],
                ),
                None => (
                    "SELECT sk, data FROM docs WHERE pk = ? ORDER BY sk LIMIT ?".to_string(),
                    vec![
                        Value::Text { value: pk.clone() },
                        Value::Integer {
                            value: limit as i64,
                        },
                    ],
                ),
            };

            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some(sql),
                            sql_id: None,
                            args,
                            named_args: vec![],
                            want_rows: Some(true),
                            replication_index: None,
                        },
                    }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                match result {
                    StreamResult::Ok { response } => match response {
                        StreamResponse::Execute(exec_resp) => {
                            let mut items = Vec::new();
                            for row in exec_resp.result.rows {
                                if let (
                                    Some(Value::Text { value: sk }),
                                    Some(Value::Blob { value: data }),
                                ) = (row.values.first(), row.values.get(1))
                                {
                                    items.push((sk.to_string(), data.clone()));
                                }
                            }
                            return Ok(items);
                        }
                        StreamResponse::Close(_) => continue,
                        _ => {}
                    },
                    StreamResult::Error { error } => {
                        if retry == 0 && Self::is_table_not_found_error(&error.message) {
                            self.create_table().await?;
                            should_retry = true;
                            break;
                        }
                        bail!("Query error: {}", error.message);
                    }
                    StreamResult::None => {}
                }
            }

            if !should_retry {
                return Ok(vec![]);
            }
        }

        Ok(vec![])
    }

    pub(crate) async fn scan(
        &self,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<Vec<(String, String, Bytes)>> {
        for retry in 0..2 {
            let (sql, args) = match after {
                None => (
                    "SELECT pk, sk, data FROM docs ORDER BY pk, sk LIMIT ?".to_string(),
                    vec![Value::Integer {
                        value: limit as i64,
                    }],
                ),
                Some((pk, sk)) => (
                    "SELECT pk, sk, data FROM docs WHERE (pk, sk) > (?, ?) ORDER BY pk, sk LIMIT ?"
                        .to_string(),
                    vec![
                        Value::Text {
                            value: pk.to_string().into(),
                        },
                        Value::Text {
                            value: sk.to_string().into(),
                        },
                        Value::Integer {
                            value: limit as i64,
                        },
                    ],
                ),
            };

            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some(sql),
                            sql_id: None,
                            args,
                            named_args: vec![],
                            want_rows: Some(true),
                            replication_index: None,
                        },
                    }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                match result {
                    StreamResult::Ok { response } => match response {
                        StreamResponse::Execute(exec_resp) => {
                            let mut items = Vec::new();
                            for row in exec_resp.result.rows {
                                if let (
                                    Some(Value::Text { value: pk }),
                                    Some(Value::Text { value: sk }),
                                    Some(Value::Blob { value: data }),
                                ) = (row.values.first(), row.values.get(1), row.values.get(2))
                                {
                                    items.push((pk.to_string(), sk.to_string(), data.clone()));
                                }
                            }
                            return Ok(items);
                        }
                        StreamResponse::Close(_) => continue,
                        _ => {}
                    },
                    StreamResult::Error { error } => {
                        if retry == 0 && Self::is_table_not_found_error(&error.message) {
                            self.create_table().await?;
                            should_retry = true;
                            break;
                        }
                        bail!("Scan error: {}", error.message);
                    }
                    StreamResult::None => {}
                }
            }

            if !should_retry {
                return Ok(vec![]);
            }
        }

        Ok(vec![])
    }

    pub(crate) async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        if ops.is_empty() {
            return Ok(());
        }

        // Convert BatchOps to Stmts
        let stmts: Vec<Stmt> = ops
            .iter()
            .map(|op| match op {
                BatchOp::Put { pk, sk, data } => Stmt {
                    sql: Some(UPSERT_DOC_SQL.to_string()),
                    sql_id: None,
                    args: vec![
                        Value::Text {
                            value: pk.to_string().into(),
                        },
                        Value::Text {
                            value: sk.to_string().into(),
                        },
                        Value::Blob {
                            value: data.to_vec().into(),
                        },
                    ],
                    named_args: vec![],
                    want_rows: Some(false),
                    replication_index: None,
                },
                BatchOp::Delete { pk, sk } => Stmt {
                    sql: Some("DELETE FROM docs WHERE pk = ? AND sk = ?".to_string()),
                    sql_id: None,
                    args: vec![
                        Value::Text {
                            value: pk.to_string().into(),
                        },
                        Value::Text {
                            value: sk.to_string().into(),
                        },
                    ],
                    named_args: vec![],
                    want_rows: Some(false),
                    replication_index: None,
                },
            })
            .collect();

        for retry in 0..2 {
            let batch = Batch::transactional(stmts.clone());

            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Batch(BatchStreamReq { batch }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                match result {
                    StreamResult::Ok { response } => match response {
                        StreamResponse::Batch(batch_resp) => {
                            // Check step_errors for any errors
                            if let Some(error) =
                                batch_resp.result.step_errors.into_iter().flatten().next()
                            {
                                bail!("Batch step error: {}", error.message);
                            }
                            return Ok(());
                        }
                        StreamResponse::Close(_) => continue,
                        _ => {}
                    },
                    StreamResult::Error { error } => {
                        if retry == 0 && Self::is_schema_error(&error.message) {
                            self.create_table().await?;
                            should_retry = true;
                            break;
                        }
                        bail!("Batch error: {}", error.message);
                    }
                    StreamResult::None => {}
                }
            }

            if !should_retry {
                return Ok(());
            }
        }

        Ok(())
    }

    pub(crate) async fn execute_ops(&self, ops: Vec<crate::DbOp>) -> Result<Vec<crate::DbResult>> {
        use crate::{DbOp, DbResult};

        if ops.is_empty() {
            return Ok(vec![]);
        }

        for retry in 0..2 {
            let mut requests: Vec<StreamRequest> = ops
                .iter()
                .map(|op| {
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: match op {
                            DbOp::Get { pk, sk } => Stmt {
                                sql: Some(
                                    "SELECT data FROM docs WHERE pk = ? AND sk = ?".to_string(),
                                ),
                                sql_id: None,
                                args: vec![
                                    Value::Text {
                                        value: pk.clone().into(),
                                    },
                                    Value::Text {
                                        value: sk.clone().into(),
                                    },
                                ],
                                named_args: vec![],
                                want_rows: Some(true),
                                replication_index: None,
                            },
                            DbOp::Query {
                                pk,
                                after_sk,
                                limit,
                            } => {
                                let mut sql_parts =
                                    vec!["SELECT sk, data FROM docs WHERE pk = ?".to_string()];
                                let mut args = vec![Value::Text {
                                    value: pk.clone().into(),
                                }];
                                if let Some(sk) = after_sk {
                                    sql_parts.push("AND sk > ?".to_string());
                                    args.push(Value::Text {
                                        value: sk.clone().into(),
                                    });
                                }
                                sql_parts.push("ORDER BY sk".to_string());
                                if let Some(lim) = limit {
                                    sql_parts.push("LIMIT ?".to_string());
                                    args.push(Value::Integer { value: *lim as i64 });
                                }
                                Stmt {
                                    sql: Some(sql_parts.join(" ")),
                                    sql_id: None,
                                    args,
                                    named_args: vec![],
                                    want_rows: Some(true),
                                    replication_index: None,
                                }
                            }
                            DbOp::Put { pk, sk, data } => Stmt {
                                sql: Some(UPSERT_DOC_SQL.to_string()),
                                sql_id: None,
                                args: vec![
                                    Value::Text {
                                        value: pk.clone().into(),
                                    },
                                    Value::Text {
                                        value: sk.clone().into(),
                                    },
                                    Value::Blob {
                                        value: data.clone().into(),
                                    },
                                ],
                                named_args: vec![],
                                want_rows: Some(false),
                                replication_index: None,
                            },
                            DbOp::Delete { pk, sk } => Stmt {
                                sql: Some("DELETE FROM docs WHERE pk = ? AND sk = ?".to_string()),
                                sql_id: None,
                                args: vec![
                                    Value::Text {
                                        value: pk.clone().into(),
                                    },
                                    Value::Text {
                                        value: sk.clone().into(),
                                    },
                                ],
                                named_args: vec![],
                                want_rows: Some(false),
                                replication_index: None,
                            },
                        },
                    })
                })
                .collect();

            requests.push(StreamRequest::Close(CloseStreamReq {}));

            let response = self.execute_pipeline(requests).await?;

            let mut db_results = Vec::new();
            let mut should_retry = false;
            let mut op_idx = 0;

            for result in response.results {
                match result {
                    StreamResult::Ok { response } => match response {
                        StreamResponse::Execute(exec_resp) => {
                            if op_idx < ops.len() {
                                match &ops[op_idx] {
                                    DbOp::Get { .. } => {
                                        let value = exec_resp
                                            .result
                                            .rows
                                            .first()
                                            .and_then(|row| row.values.first())
                                            .and_then(|v| match v {
                                                Value::Blob { value } => Some(value.clone()),
                                                _ => None,
                                            });
                                        db_results.push(DbResult::Single(value));
                                    }
                                    DbOp::Query { .. } => {
                                        let items = exec_resp
                                            .result
                                            .rows
                                            .iter()
                                            .filter_map(|row| {
                                                if let (
                                                    Some(Value::Text { value: sk }),
                                                    Some(Value::Blob { value: data }),
                                                ) = (row.values.first(), row.values.get(1))
                                                {
                                                    Some((sk.to_string(), data.clone()))
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                        db_results.push(DbResult::Multiple(items));
                                    }
                                    DbOp::Put { .. } | DbOp::Delete { .. } => {
                                        db_results.push(DbResult::Done);
                                    }
                                }
                                op_idx += 1;
                            }
                        }
                        StreamResponse::Close(_) => {}
                        _ => {}
                    },
                    StreamResult::Error { error } => {
                        if retry == 0 && Self::is_schema_error(&error.message) {
                            self.create_table().await?;
                            should_retry = true;
                            break;
                        }
                        bail!("Pipeline error: {}", error.message);
                    }
                    StreamResult::None => {}
                }
            }

            if !should_retry {
                return Ok(db_results);
            }
        }

        Ok(vec![])
    }

    pub(crate) async fn execute_raw(
        &self,
        sql: &str,
        args: Vec<Value>,
        want_rows: bool,
    ) -> Result<Vec<Vec<Value>>> {
        for retry in 0..2 {
            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some(sql.to_string()),
                            sql_id: None,
                            args: args.clone(),
                            named_args: vec![],
                            want_rows: Some(want_rows),
                            replication_index: None,
                        },
                    }),
                    StreamRequest::Close(CloseStreamReq {}),
                ])
                .await?;

            let mut should_retry = false;
            for result in response.results {
                match result {
                    StreamResult::Ok { response } => match response {
                        StreamResponse::Execute(exec_resp) => {
                            if want_rows {
                                return Ok(exec_resp
                                    .result
                                    .rows
                                    .into_iter()
                                    .map(|row| row.values)
                                    .collect());
                            }
                            return Ok(vec![]);
                        }
                        StreamResponse::Close(_) => continue,
                        _ => {}
                    },
                    StreamResult::Error { error } => {
                        if retry == 0 && Self::is_table_not_found_error(&error.message) {
                            self.create_table().await?;
                            should_retry = true;
                            break;
                        }
                        bail!("execute_raw error: {}", error.message);
                    }
                    StreamResult::None => {}
                }
            }

            if !should_retry {
                return Ok(vec![]);
            }
        }

        Ok(vec![])
    }

    pub(crate) async fn transaction(&self) -> Result<TursoTransaction> {
        // Ensure table exists before starting transaction
        self.ensure_table().await?;

        // Execute BEGIN and get baton
        let response = self
            .execute_pipeline(vec![StreamRequest::Execute(ExecuteStreamReq {
                stmt: Stmt {
                    sql: Some("BEGIN".to_string()),
                    sql_id: None,
                    args: vec![],
                    named_args: vec![],
                    want_rows: Some(false),
                    replication_index: None,
                },
            })])
            .await?;

        // Check for errors
        for result in &response.results {
            if let StreamResult::Error { error } = result {
                bail!("Transaction begin error: {}", error.message);
            }
        }

        let baton = response
            .baton
            .ok_or_else(|| anyhow::anyhow!("No baton returned from BEGIN"))?;

        Ok(TursoTransaction {
            db: self.clone(),
            baton: Some(baton),
        })
    }

    #[tracing::instrument(skip_all, fields(reads = keys.len()))]
    pub(crate) async fn begin_immediate_with_reads(
        &self,
        keys: &[(String, String)],
    ) -> Result<(TursoTransaction, Vec<Option<StoredDoc>>)> {
        const MAX_BUSY_ATTEMPTS: u32 = 8;
        let mut busy_attempt: u32 = 0;
        let mut schema_retried = false;
        loop {
            let mut requests: Vec<StreamRequest> = Vec::with_capacity(keys.len() + 1);
            requests.push(StreamRequest::Execute(ExecuteStreamReq {
                stmt: Stmt {
                    sql: Some("BEGIN IMMEDIATE".to_string()),
                    sql_id: None,
                    args: vec![],
                    named_args: vec![],
                    want_rows: Some(false),
                    replication_index: None,
                },
            }));
            for (pk, sk) in keys {
                requests.push(StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some(
                            "SELECT data, version FROM docs WHERE pk = ? AND sk = ?".to_string(),
                        ),
                        sql_id: None,
                        args: vec![
                            Value::Text {
                                value: pk.clone().into(),
                            },
                            Value::Text {
                                value: sk.clone().into(),
                            },
                        ],
                        named_args: vec![],
                        want_rows: Some(true),
                        replication_index: None,
                    },
                }));
            }

            let response = self.execute_pipeline(requests).await?;

            let mut docs: Vec<Option<StoredDoc>> = Vec::with_capacity(keys.len());
            let mut idx = 0usize;
            let mut retry_kind: Option<RetryKind> = None;
            let mut fatal_error: Option<String> = None;

            for stream_result in response.results {
                match stream_result {
                    StreamResult::Ok {
                        response: StreamResponse::Execute(exec_resp),
                    } => {
                        if idx == 0 {
                            // BEGIN IMMEDIATE result, ignore
                        } else {
                            let stored = if let Some(row) = exec_resp.result.rows.first()
                                && let (
                                    Some(Value::Blob { value: data }),
                                    Some(Value::Integer { value: version }),
                                ) = (row.values.first(), row.values.get(1))
                            {
                                Some(StoredDoc {
                                    data: data.clone(),
                                    version: *version,
                                })
                            } else {
                                None
                            };
                            docs.push(stored);
                        }
                        idx += 1;
                    }
                    StreamResult::Ok { response: _ } => {}
                    StreamResult::Error { error } => {
                        if !schema_retried && Self::is_schema_error(&error.message) {
                            retry_kind = Some(RetryKind::Schema);
                        } else if Self::is_busy_error(&error.message) {
                            retry_kind = Some(RetryKind::Busy);
                        } else {
                            fatal_error = Some(error.message);
                        }
                        break;
                    }
                    StreamResult::None => {}
                }
            }

            if let Some(msg) = fatal_error {
                bail!("begin_immediate_with_reads error: {}", msg);
            }

            match retry_kind {
                Some(RetryKind::Schema) => {
                    self.create_table().await?;
                    schema_retried = true;
                    continue;
                }
                Some(RetryKind::Busy) => {
                    if busy_attempt + 1 >= MAX_BUSY_ATTEMPTS {
                        bail!(
                            "begin_immediate_with_reads: BUSY after {} attempts",
                            MAX_BUSY_ATTEMPTS
                        );
                    }
                    Self::busy_backoff(busy_attempt).await;
                    busy_attempt += 1;
                    continue;
                }
                None => {}
            }

            let baton = response
                .baton
                .ok_or_else(|| anyhow::anyhow!("No baton returned from BEGIN IMMEDIATE"))?;
            let tx = TursoTransaction {
                db: self.clone(),
                baton: Some(baton),
            };
            return Ok((tx, docs));
        }
    }

    async fn ensure_table(&self) -> Result<()> {
        self.create_table().await
    }
}
