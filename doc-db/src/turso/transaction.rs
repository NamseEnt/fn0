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

    #[tracing::instrument(
        skip_all,
        fields(conditions = request.conditions.len(), mutations = request.mutations.len())
    )]
    pub(crate) async fn transact(
        mut self,
        request: &crate::TransactRequest,
    ) -> Result<crate::TransactOutcome> {
        let validation = self.validate_conditions(&request.conditions).await;
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

        if let Err(error) = self.apply_mutations(&request.mutations).await {
            return Err(rollback_after_error(self, error).await);
        }

        self.commit().await?;
        Ok(crate::TransactOutcome { conflict: None })
    }

    async fn validate_conditions(
        &mut self,
        conditions: &[crate::TransactCondition],
    ) -> Result<Option<crate::TransactConflict>> {
        if conditions.is_empty() {
            return Ok(None);
        }

        let requests = conditions
            .iter()
            .map(|condition| {
                Ok(StreamRequest::Execute(ExecuteStreamReq {
                    stmt: validation_stmt(condition)?,
                }))
            })
            .collect::<Result<Vec<_>>>()?;
        let results = self.execute_statements(requests).await?;
        if results.len() != conditions.len() {
            bail!(
                "transaction validation result count mismatch: expected {}, got {}",
                conditions.len(),
                results.len()
            );
        }

        for (condition_index, (condition, result)) in
            conditions.iter().zip(results.iter()).enumerate()
        {
            if !condition_holds(condition, result)? {
                return Ok(Some(crate::TransactConflict { condition_index }));
            }
        }
        Ok(None)
    }

    async fn apply_mutations(&mut self, mutations: &[crate::TransactMutation]) -> Result<()> {
        if mutations.is_empty() {
            return Ok(());
        }

        let requests = mutations
            .iter()
            .map(|mutation| {
                StreamRequest::Execute(ExecuteStreamReq {
                    stmt: write_stmt(mutation),
                })
            })
            .collect();
        let results = self.execute_statements(requests).await?;
        if results.len() != mutations.len() {
            bail!(
                "transaction mutation result count mismatch: expected {}, got {}",
                mutations.len(),
                results.len()
            );
        }
        Ok(())
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

fn validation_stmt(condition: &crate::TransactCondition) -> Result<Stmt> {
    let (sql, want_rows) = match condition {
        crate::TransactCondition::RevisionEquals { .. }
        | crate::TransactCondition::Exists { .. } => {
            ("SELECT version FROM docs WHERE pk = ? AND sk = ?", true)
        }
        crate::TransactCondition::NotExists { .. } => {
            ("SELECT 1 FROM docs WHERE pk = ? AND sk = ?", true)
        }
    };
    let (pk, sk) = condition_key(condition);
    let args = vec![
        Value::Text {
            value: pk.to_string().into(),
        },
        Value::Text {
            value: sk.to_string().into(),
        },
    ];
    Ok(Stmt {
        sql: Some(sql.to_string()),
        sql_id: None,
        args,
        named_args: vec![],
        want_rows: Some(want_rows),
        replication_index: None,
    })
}

fn condition_holds(condition: &crate::TransactCondition, result: &StmtResult) -> Result<bool> {
    Ok(match condition {
        crate::TransactCondition::NotExists { .. } => result.rows.is_empty(),
        crate::TransactCondition::Exists { .. } => !result.rows.is_empty(),
        crate::TransactCondition::RevisionEquals {
            expected_revision, ..
        } => {
            let expected_revision = crate::revision_to_backend(*expected_revision)?;
            matches!(
                result.rows.first().and_then(|row| row.values.first()),
                Some(Value::Integer { value }) if *value == expected_revision
            )
        }
    })
}

fn write_stmt(mutation: &crate::TransactMutation) -> Stmt {
    match mutation {
        crate::TransactMutation::Put { pk, sk, data } => Stmt {
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
        crate::TransactMutation::Delete { pk, sk } => Stmt {
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
    }
}

fn condition_key(condition: &crate::TransactCondition) -> (&str, &str) {
    match condition {
        crate::TransactCondition::RevisionEquals { pk, sk, .. }
        | crate::TransactCondition::Exists { pk, sk }
        | crate::TransactCondition::NotExists { pk, sk } => (pk, sk),
    }
}
