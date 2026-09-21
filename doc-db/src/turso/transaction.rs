use super::*;
use anyhow::{Result, bail};
use bytes::Bytes;
use libsql_hrana::proto::*;

impl TursoTransaction {
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

    pub(crate) async fn execute_stmt(
        &mut self,
        sql: &str,
        args: Vec<Value>,
        want_rows: bool,
    ) -> Result<StmtResult> {
        let response = self
            .execute_in_tx(vec![StreamRequest::Execute(ExecuteStreamReq {
                stmt: Stmt {
                    sql: Some(sql.to_string()),
                    sql_id: None,
                    args,
                    named_args: vec![],
                    want_rows: Some(want_rows),
                    replication_index: None,
                },
            })])
            .await?;

        for result in response.results {
            match result {
                StreamResult::Ok { response } => {
                    if let StreamResponse::Execute(exec_resp) = response {
                        return Ok(exec_resp.result);
                    }
                }
                StreamResult::Error { error } => {
                    bail!("Transaction execute error: {}", error.message);
                }
                StreamResult::None => {}
            }
        }

        bail!("Missing transaction execute result")
    }

    pub(crate) async fn get(&mut self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        let result = self
            .execute_stmt(
                "SELECT data FROM docs WHERE pk = ? AND sk = ?",
                vec![
                    Value::Text {
                        value: pk.to_string().into(),
                    },
                    Value::Text {
                        value: sk.to_string().into(),
                    },
                ],
                true,
            )
            .await?;

        if let Some(Value::Blob { value }) = result.rows.first().and_then(|row| row.values.first())
        {
            return Ok(Some(value.clone()));
        }

        Ok(None)
    }

    pub(crate) async fn put(&mut self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        self.execute_stmt(
            UPSERT_DOC_SQL,
            vec![
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
            false,
        )
        .await?;

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

    #[tracing::instrument(skip_all, fields(items = items.len()))]
    pub(crate) async fn transact(
        mut self,
        items: &[crate::TransactItem],
    ) -> Result<crate::TransactOutcome> {
        let validation = self.validate_items(items).await;
        let conflict = match validation {
            Ok(conflict) => conflict,
            Err(error) => {
                return Err(rollback_after_error(self, error).await);
            }
        };
        if let Some(conflict) = conflict {
            self.rollback().await?;
            return Ok(crate::TransactOutcome {
                conflict: Some(conflict),
            });
        }

        let write_conflict = match self.apply_writes(items).await {
            Ok(conflict) => conflict,
            Err(error) => {
                return Err(rollback_after_error(self, error).await);
            }
        };
        if let Some(conflict) = write_conflict {
            self.rollback().await?;
            return Ok(crate::TransactOutcome {
                conflict: Some(conflict),
            });
        }

        self.commit().await?;
        Ok(crate::TransactOutcome { conflict: None })
    }

    async fn validate_items(
        &mut self,
        items: &[crate::TransactItem],
    ) -> Result<Option<crate::TransactConflict>> {
        if items.is_empty() {
            return Ok(None);
        }

        let requests = items
            .iter()
            .map(|item| {
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: validation_stmt(item),
                })
            })
            .collect();
        let results = self.execute_statements(requests).await?;
        if results.len() != items.len() {
            bail!(
                "transaction validation result count mismatch: expected {}, got {}",
                items.len(),
                results.len()
            );
        }

        for (step_index, (item, result)) in items.iter().zip(results.iter()).enumerate() {
            if !condition_holds(item, result) {
                return Ok(Some(crate::TransactConflict { step_index }));
            }
        }
        Ok(None)
    }

    async fn apply_writes(
        &mut self,
        items: &[crate::TransactItem],
    ) -> Result<Option<crate::TransactConflict>> {
        let writes: Vec<(usize, Stmt)> = items
            .iter()
            .enumerate()
            .filter_map(|(step_index, item)| write_stmt(item).map(|stmt| (step_index, stmt)))
            .collect();
        if writes.is_empty() {
            return Ok(None);
        }

        let requests = writes
            .iter()
            .map(|(_, stmt)| StreamRequest::Execute(ExecuteStreamReq { stmt: stmt.clone() }))
            .collect();
        let results = self.execute_statements(requests).await?;
        if results.len() != writes.len() {
            bail!(
                "transaction write result count mismatch: expected {}, got {}",
                writes.len(),
                results.len()
            );
        }

        for ((step_index, _), result) in writes.iter().zip(results.iter()) {
            if result.affected_row_count != 1 {
                return Ok(Some(crate::TransactConflict {
                    step_index: *step_index,
                }));
            }
        }
        Ok(None)
    }

    async fn execute_statements(
        &mut self,
        requests: Vec<StreamRequest>,
    ) -> Result<Vec<StmtResult>> {
        let response = self.execute_in_tx(requests).await?;
        let mut results = Vec::new();
        for stream_result in response.results {
            match stream_result {
                StreamResult::Ok {
                    response: StreamResponse::Execute(exec_resp),
                } => results.push(exec_resp.result),
                StreamResult::Ok { response: _ } => {}
                StreamResult::Error { error } => {
                    bail!("transaction statement error: {}", error.message);
                }
                StreamResult::None => {}
            }
        }
        Ok(results)
    }
}

async fn rollback_after_error(tx: TursoTransaction, error: anyhow::Error) -> anyhow::Error {
    match tx.rollback().await {
        Ok(()) => error,
        Err(rollback_error) => {
            anyhow::anyhow!("{error}; transaction rollback failed: {rollback_error}")
        }
    }
}

fn validation_stmt(item: &crate::TransactItem) -> Stmt {
    let (sql, want_rows) = match item {
        crate::TransactItem::CheckVersion { .. }
        | crate::TransactItem::Update { .. }
        | crate::TransactItem::Delete { .. } => {
            ("SELECT version FROM docs WHERE pk = ? AND sk = ?", true)
        }
        crate::TransactItem::CheckMissing { .. } | crate::TransactItem::Insert { .. } => {
            ("SELECT 1 FROM docs WHERE pk = ? AND sk = ?", true)
        }
    };
    let (pk, sk) = item_key(item);
    Stmt {
        sql: Some(sql.to_string()),
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
        want_rows: Some(want_rows),
        replication_index: None,
    }
}

fn condition_holds(item: &crate::TransactItem, result: &StmtResult) -> bool {
    match item {
        crate::TransactItem::CheckMissing { .. } | crate::TransactItem::Insert { .. } => {
            result.rows.is_empty()
        }
        crate::TransactItem::CheckVersion {
            expected_version, ..
        }
        | crate::TransactItem::Update {
            expected_version, ..
        }
        | crate::TransactItem::Delete {
            expected_version, ..
        } => matches!(
            result.rows.first().and_then(|row| row.values.first()),
            Some(Value::Integer { value }) if value == expected_version
        ),
    }
}

fn write_stmt(item: &crate::TransactItem) -> Option<Stmt> {
    let stmt = match item {
        crate::TransactItem::Insert { pk, sk, data } => Stmt {
            sql: Some("INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0)".to_string()),
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
        crate::TransactItem::Update {
            pk,
            sk,
            expected_version,
            data,
        } => Stmt {
            sql: Some(
                "UPDATE docs SET data = ?, version = version + 1 \
                 WHERE pk = ? AND sk = ? AND version = ?"
                    .to_string(),
            ),
            sql_id: None,
            args: vec![
                Value::Blob {
                    value: data.clone().into(),
                },
                Value::Text {
                    value: pk.clone().into(),
                },
                Value::Text {
                    value: sk.clone().into(),
                },
                Value::Integer {
                    value: *expected_version,
                },
            ],
            named_args: vec![],
            want_rows: Some(false),
            replication_index: None,
        },
        crate::TransactItem::Delete {
            pk,
            sk,
            expected_version,
        } => Stmt {
            sql: Some("DELETE FROM docs WHERE pk = ? AND sk = ? AND version = ?".to_string()),
            sql_id: None,
            args: vec![
                Value::Text {
                    value: pk.clone().into(),
                },
                Value::Text {
                    value: sk.clone().into(),
                },
                Value::Integer {
                    value: *expected_version,
                },
            ],
            named_args: vec![],
            want_rows: Some(false),
            replication_index: None,
        },
        crate::TransactItem::CheckVersion { .. } | crate::TransactItem::CheckMissing { .. } => {
            return None;
        }
    };
    Some(stmt)
}

fn item_key(item: &crate::TransactItem) -> (&str, &str) {
    match item {
        crate::TransactItem::CheckVersion { pk, sk, .. }
        | crate::TransactItem::CheckMissing { pk, sk }
        | crate::TransactItem::Insert { pk, sk, .. }
        | crate::TransactItem::Update { pk, sk, .. }
        | crate::TransactItem::Delete { pk, sk, .. } => (pk, sk),
    }
}
