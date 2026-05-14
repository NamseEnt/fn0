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

    #[tracing::instrument(skip_all, fields(reads = keys.len()))]
    pub(crate) async fn batch_get_with_version(
        &mut self,
        keys: &[(String, String)],
    ) -> Result<Vec<Option<StoredDoc>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        let requests: Vec<StreamRequest> = keys
            .iter()
            .map(|(pk, sk)| {
                StreamRequest::Execute(ExecuteStreamReq {
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
                })
            })
            .collect();

        let response = self.execute_in_tx(requests).await?;

        let mut docs: Vec<Option<StoredDoc>> = Vec::with_capacity(keys.len());
        for stream_result in response.results {
            match stream_result {
                StreamResult::Ok {
                    response: StreamResponse::Execute(exec_resp),
                } => {
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
                StreamResult::Ok { response: _ } => {}
                StreamResult::Error { error } => {
                    bail!("batch_get_with_version error: {}", error.message);
                }
                StreamResult::None => {}
            }
        }
        Ok(docs)
    }

    #[tracing::instrument(skip_all, fields(writes = writes.len()))]
    pub(crate) async fn apply_writes_and_commit(
        &mut self,
        writes: &[crate::WriteOp],
    ) -> Result<crate::CommitOutcome> {
        use crate::WriteOp;

        let mut steps: Vec<BatchStep> = Vec::with_capacity(writes.len() + 2);

        for op in writes {
            let stmt = match op {
                WriteOp::Insert { pk, sk, data } => Stmt {
                    sql: Some(
                        "INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0)".to_string(),
                    ),
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
                WriteOp::Update {
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
                WriteOp::Delete {
                    pk,
                    sk,
                    expected_version,
                } => Stmt {
                    sql: Some(
                        "DELETE FROM docs WHERE pk = ? AND sk = ? AND version = ?".to_string(),
                    ),
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
            };
            let condition = if steps.is_empty() {
                None
            } else {
                Some(BatchCond::Ok {
                    step: (steps.len() - 1) as u32,
                })
            };
            steps.push(BatchStep { condition, stmt });
        }

        let last_write_step = if writes.is_empty() {
            None
        } else {
            Some((steps.len() - 1) as u32)
        };
        let commit_cond = last_write_step.map(|s| BatchCond::Ok { step: s });
        steps.push(BatchStep {
            condition: commit_cond,
            stmt: Stmt {
                sql: Some("COMMIT".to_string()),
                sql_id: None,
                args: vec![],
                named_args: vec![],
                want_rows: Some(false),
                replication_index: None,
            },
        });
        let commit_step_idx = (steps.len() - 1) as u32;
        steps.push(BatchStep {
            condition: Some(BatchCond::Not {
                cond: Box::new(BatchCond::Ok {
                    step: commit_step_idx,
                }),
            }),
            stmt: Stmt {
                sql: Some("ROLLBACK".to_string()),
                sql_id: None,
                args: vec![],
                named_args: vec![],
                want_rows: Some(false),
                replication_index: None,
            },
        });

        let batch = Batch {
            steps,
            replication_index: None,
        };

        let response = self
            .execute_in_tx(vec![
                StreamRequest::Batch(BatchStreamReq { batch }),
                StreamRequest::Close(CloseStreamReq {}),
            ])
            .await?;

        self.baton = None;

        let mut batch_result: Option<BatchResult> = None;
        for stream_result in response.results {
            match stream_result {
                StreamResult::Ok {
                    response: StreamResponse::Batch(batch_resp),
                } => {
                    batch_result = Some(batch_resp.result);
                    break;
                }
                StreamResult::Ok { response: _ } => {}
                StreamResult::Error { error } => {
                    bail!("apply_writes_and_commit stream error: {}", error.message);
                }
                StreamResult::None => {}
            }
        }

        let batch_result = batch_result
            .ok_or_else(|| anyhow::anyhow!("apply_writes_and_commit: batch response missing"))?;

        let mut conflict: Option<crate::ConflictInfo> = None;
        for (i, error_opt) in batch_result.step_errors.iter().enumerate() {
            if let Some(error) = error_opt
                && i < writes.len()
            {
                conflict = Some(crate::ConflictInfo {
                    step_index: i,
                    message: error.message.clone(),
                });
                break;
            }
        }

        let mut affected_counts: Vec<u64> = Vec::with_capacity(writes.len());
        for (i, stmt_result_opt) in batch_result.step_results.iter().enumerate() {
            if i >= writes.len() {
                break;
            }
            affected_counts.push(
                stmt_result_opt
                    .as_ref()
                    .map(|r| r.affected_row_count)
                    .unwrap_or(0),
            );
        }
        while affected_counts.len() < writes.len() {
            affected_counts.push(0);
        }

        Ok(crate::CommitOutcome {
            affected_counts,
            conflict,
        })
    }
}
