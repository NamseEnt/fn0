use crate::BatchOp;
use anyhow::{Result, bail};
use libsql_hrana::proto::*;
use std::{str::FromStr, sync::Arc};
use wstd::http::{Client, HeaderValue, Request, Uri, body::Bytes};

pub(crate) struct TursoDatabase {
    http_url: Uri,
    client: Client,
    auth_token: String,
}

impl TursoDatabase {
    pub(crate) fn new(url: String, auth_token: String) -> Self {
        let url = Uri::from_str(&url).expect("Failed to parse URI");
        let mut parts = url.into_parts();
        // Convert libsql: scheme to https, preserve http for local development
        match parts.scheme.as_ref().map(|s| s.as_str()) {
            Some("libsql") | None => {
                parts.scheme = Some("https".parse().unwrap());
            }
            _ => {} // Keep http or other schemes as-is
        }
        parts.path_and_query = Some("/v2/pipeline".parse().unwrap());
        let http_url = Uri::from_parts(parts).expect("Failed to assemble URI");

        Self {
            http_url,
            client: Client::new(),
            auth_token,
        }
    }
    async fn execute_pipeline(&self, requests: Vec<StreamRequest>) -> Result<PipelineRespBody> {
        self.execute_pipeline_with_baton(None, requests).await
    }

    async fn execute_pipeline_with_baton(
        &self,
        baton: Option<String>,
        requests: Vec<StreamRequest>,
    ) -> Result<PipelineRespBody> {
        let body = PipelineReqBody { baton, requests };

        let mut request_builder = Request::post(&self.http_url);

        if !self.auth_token.is_empty() {
            request_builder = request_builder.header(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", self.auth_token))?,
            );
        }

        let request = request_builder
            .header("Content-Type", HeaderValue::from_str("application/json")?)
            .body(
                serde_json::to_vec(&body)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize body: {}", e))?,
            )?;

        let response = self.client.send(request).await?;
        if !response.status().is_success() {
            bail!("Turso request failed with status: {}", response.status());
        }

        response.into_body().json().await
    }

    async fn create_table(&self) -> Result<()> {
        let response = self
            .execute_pipeline(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some(
                            "CREATE TABLE IF NOT EXISTS docs (pk TEXT, sk TEXT, data BLOB, PRIMARY KEY (pk, sk))"
                                .to_string(),
                        ),
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

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("Create table error: {}", error.message);
            }
        }

        Ok(())
    }

    fn is_table_not_found_error(error_message: &str) -> bool {
        error_message.contains("no such table")
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

    pub(crate) async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        for retry in 0..2 {
            let response = self
                .execute_pipeline(vec![
                    StreamRequest::Execute(ExecuteStreamReq {
                        stmt: Stmt {
                            sql: Some(
                                "INSERT OR REPLACE INTO docs (pk, sk, data) VALUES (?, ?, ?)"
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
                    if retry == 0 && Self::is_table_not_found_error(&error.message) {
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
                    sql: Some(
                        "INSERT OR REPLACE INTO docs (pk, sk, data) VALUES (?, ?, ?)".to_string(),
                    ),
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
                        if retry == 0 && Self::is_table_not_found_error(&error.message) {
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

    pub(crate) async fn transaction(&self) -> Result<TursoTransaction<'_>> {
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
            db: self,
            baton: Some(baton),
        })
    }

    async fn ensure_table(&self) -> Result<()> {
        // Try a simple query to check if table exists
        let response = self
            .execute_pipeline(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some("SELECT 1 FROM docs LIMIT 1".to_string()),
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

        for result in response.results {
            if let StreamResult::Error { error } = result {
                if Self::is_table_not_found_error(&error.message) {
                    self.create_table().await?;
                    return Ok(());
                }
                bail!("Table check error: {}", error.message);
            }
        }

        Ok(())
    }
}

pub(crate) struct TursoTransaction<'a> {
    db: &'a TursoDatabase,
    baton: Option<String>,
}

impl<'a> TursoTransaction<'a> {
    async fn execute_in_tx(&mut self, requests: Vec<StreamRequest>) -> Result<PipelineRespBody> {
        let baton = self
            .baton
            .take()
            .ok_or_else(|| anyhow::anyhow!("Transaction already finished"))?;

        let response = self
            .db
            .execute_pipeline_with_baton(Some(baton), requests)
            .await?;

        // Update baton for next request
        self.baton = response.baton.clone();

        Ok(response)
    }

    pub(crate) async fn get(&mut self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        let response = self
            .execute_in_tx(vec![StreamRequest::Execute(ExecuteStreamReq {
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
            })])
            .await?;

        for result in response.results {
            match result {
                StreamResult::Ok { response } => {
                    if let StreamResponse::Execute(exec_resp) = response {
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
                }
                StreamResult::Error { error } => {
                    bail!("Transaction get error: {}", error.message);
                }
                StreamResult::None => {}
            }
        }

        Ok(None)
    }

    pub(crate) async fn put(&mut self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        let response = self
            .execute_in_tx(vec![StreamRequest::Execute(ExecuteStreamReq {
                stmt: Stmt {
                    sql: Some(
                        "INSERT OR REPLACE INTO docs (pk, sk, data) VALUES (?, ?, ?)".to_string(),
                    ),
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
            })])
            .await?;

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("Transaction put error: {}", error.message);
            }
        }

        Ok(())
    }

    pub(crate) async fn delete(&mut self, pk: &str, sk: &str) -> Result<()> {
        let response = self
            .execute_in_tx(vec![StreamRequest::Execute(ExecuteStreamReq {
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
            })])
            .await?;

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("Transaction delete error: {}", error.message);
            }
        }

        Ok(())
    }

    pub(crate) async fn commit(mut self) -> Result<()> {
        let response = self
            .execute_in_tx(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some("COMMIT".to_string()),
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

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("Transaction commit error: {}", error.message);
            }
        }

        self.baton = None; // Mark as finished
        Ok(())
    }

    pub(crate) async fn rollback(mut self) -> Result<()> {
        let response = self
            .execute_in_tx(vec![
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: Stmt {
                        sql: Some("ROLLBACK".to_string()),
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

        for result in response.results {
            if let StreamResult::Error { error } = result {
                bail!("Transaction rollback error: {}", error.message);
            }
        }

        self.baton = None; // Mark as finished
        Ok(())
    }
}
