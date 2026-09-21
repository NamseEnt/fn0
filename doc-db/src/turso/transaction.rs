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
}
