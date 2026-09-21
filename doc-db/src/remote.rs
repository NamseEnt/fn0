use crate::{BatchOp, DbOp, DbResult, TransactCondition, TransactMutation, TransactRequest};
use anyhow::{Result, anyhow, bail};
use bytes::Bytes;
use doc_db_protocol::{
    DocDbBasicOperation, DocDbBasicResult, DocDbBatchOperation, DocDbCondition, DocDbKey,
    DocDbMutation, DocDbObservedDocument, DocDbOperation, DocDbRequest, DocDbResponse, DocDbResult,
    DocDbTransactOutcome, decode_response, encode_request,
};

#[derive(Clone)]
pub(crate) struct RemoteDatabase {
    url: String,
}

impl RemoteDatabase {
    pub(crate) fn new(url: String) -> Self {
        Self { url }
    }

    async fn execute(&self, operation: DocDbOperation) -> Result<DocDbResponse> {
        let request = DocDbRequest::new(operation);
        let body = encode_request(&request).map_err(|error| anyhow!(error.to_string()))?;
        let (status, response_body) = crate::runtime::http_post_doc_db(&self.url, body).await?;
        let response =
            decode_response(&response_body).map_err(|error| anyhow!(error.to_string()))?;
        if !(200..300).contains(&status)
            && !matches!(
                response.result,
                DocDbResult::Transact {
                    outcome: DocDbTransactOutcome::Conflict { .. }
                }
            )
        {
            if let DocDbResult::Error { error } = &response.result {
                bail!("{error}");
            }
            bail!("semantic doc-db request failed with HTTP status {status}");
        }
        Ok(response)
    }

    pub(crate) async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        let response = self
            .execute(DocDbOperation::Get {
                key: DocDbKey::new(pk, sk),
            })
            .await?;
        match response.result {
            DocDbResult::Get { data } => Ok(data.map(|document| document.data.into())),
            other => bail!("unexpected semantic doc-db get response: {other:?}"),
        }
    }

    pub(crate) async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        let response = self
            .execute(DocDbOperation::Put {
                key: DocDbKey::new(pk, sk),
                data: data.to_vec(),
            })
            .await?;
        match response.result {
            DocDbResult::Put => Ok(()),
            other => bail!("unexpected semantic doc-db put response: {other:?}"),
        }
    }

    pub(crate) async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        let response = self
            .execute(DocDbOperation::Delete {
                key: DocDbKey::new(pk, sk),
            })
            .await?;
        match response.result {
            DocDbResult::Delete => Ok(()),
            other => bail!("unexpected semantic doc-db delete response: {other:?}"),
        }
    }

    pub(crate) async fn query(
        &self,
        pk: &str,
        after_sk: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, Bytes)>> {
        let response = self
            .execute(DocDbOperation::Query {
                pk: pk.to_string(),
                after_sk: after_sk.map(str::to_string),
                limit: limit as u64,
            })
            .await?;
        match response.result {
            DocDbResult::Query { documents } => documents
                .into_iter()
                .map(|document| {
                    if document.key.pk != pk {
                        bail!("semantic doc-db query returned an unexpected partition key");
                    }
                    Ok((document.key.sk, document.data.into()))
                })
                .collect(),
            other => bail!("unexpected semantic doc-db query response: {other:?}"),
        }
    }

    pub(crate) async fn scan(
        &self,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<Vec<(String, String, Bytes)>> {
        let response = self
            .execute(DocDbOperation::Scan {
                after: after.map(|(pk, sk)| DocDbKey::new(pk, sk)),
                limit: limit as u64,
            })
            .await?;
        match response.result {
            DocDbResult::Scan { documents } => Ok(documents
                .into_iter()
                .map(|document| (document.key.pk, document.key.sk, document.data.into()))
                .collect()),
            other => bail!("unexpected semantic doc-db scan response: {other:?}"),
        }
    }

    pub(crate) async fn batch(&self, operations: &[BatchOp<'_>]) -> Result<()> {
        let operations = operations
            .iter()
            .map(|operation| match operation {
                BatchOp::Put { pk, sk, data } => DocDbBatchOperation::Put {
                    key: DocDbKey::new(*pk, *sk),
                    data: data.to_vec(),
                },
                BatchOp::Delete { pk, sk } => DocDbBatchOperation::Delete {
                    key: DocDbKey::new(*pk, *sk),
                },
            })
            .collect();
        let response = self.execute(DocDbOperation::Batch { operations }).await?;
        match response.result {
            DocDbResult::Batch => Ok(()),
            other => bail!("unexpected semantic doc-db batch response: {other:?}"),
        }
    }

    pub(crate) async fn execute_ops(&self, operations: Vec<DbOp>) -> Result<Vec<DbResult>> {
        let protocol_operations = operations
            .iter()
            .map(db_op_to_protocol_operation)
            .collect::<Result<Vec<_>>>()?;
        let response = self
            .execute(DocDbOperation::ExecuteOps {
                operations: protocol_operations,
            })
            .await?;
        let DocDbResult::ExecuteOps { results } = response.result else {
            bail!("unexpected semantic doc-db execute_ops response")
        };
        if results.len() != operations.len() {
            bail!(
                "semantic execute_ops result count mismatch: expected {}, got {}",
                operations.len(),
                results.len()
            );
        }
        operations
            .into_iter()
            .zip(results)
            .map(|(operation, result)| db_result_from_protocol_result(operation, result))
            .collect()
    }

    pub(crate) async fn batch_get_observed(
        &self,
        keys: &[(String, String)],
    ) -> Result<Vec<crate::ObservedDocument>> {
        let response = self
            .execute(DocDbOperation::BatchGetObserved {
                keys: keys.iter().map(|(pk, sk)| DocDbKey::new(pk, sk)).collect(),
            })
            .await?;
        match response.result {
            DocDbResult::BatchGetObserved { documents } => documents
                .into_iter()
                .map(|document| match document {
                    DocDbObservedDocument::Present { data, revision } => {
                        Ok(crate::ObservedDocument::Present {
                            data: data.into(),
                            revision,
                        })
                    }
                    DocDbObservedDocument::Missing { revision } => {
                        Ok(crate::ObservedDocument::Missing { revision })
                    }
                })
                .collect(),
            other => bail!("unexpected semantic doc-db observed response: {other:?}"),
        }
    }

    pub(crate) async fn transact(
        &self,
        request: &TransactRequest,
    ) -> Result<crate::TransactOutcome> {
        let conditions = request
            .conditions
            .iter()
            .map(|condition| match condition {
                TransactCondition::RevisionEquals {
                    pk,
                    sk,
                    expected_revision,
                } => DocDbCondition::RevisionEquals {
                    key: DocDbKey::new(pk, sk),
                    expected_revision: *expected_revision,
                },
                TransactCondition::Exists { pk, sk } => DocDbCondition::Exists {
                    key: DocDbKey::new(pk, sk),
                },
                TransactCondition::NotExists { pk, sk } => DocDbCondition::NotExists {
                    key: DocDbKey::new(pk, sk),
                },
            })
            .collect();
        let mutations = request
            .mutations
            .iter()
            .map(|mutation| match mutation {
                TransactMutation::Put { pk, sk, data } => DocDbMutation::Put {
                    key: DocDbKey::new(pk, sk),
                    data: data.clone(),
                },
                TransactMutation::Delete { pk, sk } => DocDbMutation::Delete {
                    key: DocDbKey::new(pk, sk),
                },
            })
            .collect();
        let response = self
            .execute(DocDbOperation::Transact {
                conditions,
                mutations,
            })
            .await?;
        match response.result {
            DocDbResult::Transact { outcome } => Ok(match outcome {
                DocDbTransactOutcome::Committed => crate::TransactOutcome { conflict: None },
                DocDbTransactOutcome::Conflict { condition_index } => crate::TransactOutcome {
                    conflict: Some(crate::TransactConflict { condition_index }),
                },
            }),
            other => bail!("unexpected semantic doc-db transaction response: {other:?}"),
        }
    }

    pub(crate) async fn get_observed(&self, pk: &str, sk: &str) -> Result<crate::ObservedDocument> {
        let mut documents = self
            .batch_get_observed(&[(pk.to_string(), sk.to_string())])
            .await?;
        documents
            .pop()
            .ok_or_else(|| anyhow!("semantic doc-db observed response was empty"))
    }
}

fn db_op_to_protocol_operation(operation: &DbOp) -> Result<DocDbBasicOperation> {
    Ok(match operation {
        DbOp::Get { pk, sk } => DocDbBasicOperation::Get {
            key: DocDbKey::new(pk, sk),
        },
        DbOp::Query {
            pk,
            after_sk,
            limit,
        } => DocDbBasicOperation::Query {
            pk: pk.clone(),
            after_sk: after_sk.clone(),
            limit: limit.map(|value| value as u64),
        },
        DbOp::Put { pk, sk, data } => DocDbBasicOperation::Put {
            key: DocDbKey::new(pk, sk),
            data: data.clone(),
        },
        DbOp::Delete { pk, sk } => DocDbBasicOperation::Delete {
            key: DocDbKey::new(pk, sk),
        },
    })
}

fn db_result_from_protocol_result(operation: DbOp, result: DocDbBasicResult) -> Result<DbResult> {
    match (operation, result) {
        (DbOp::Get { .. }, DocDbBasicResult::Get { data }) => {
            Ok(DbResult::Single(data.map(|document| document.data.into())))
        }
        (DbOp::Query { .. }, DocDbBasicResult::Query { documents }) => Ok(DbResult::Multiple(
            documents
                .into_iter()
                .map(|document| (document.key.sk, document.data.into()))
                .collect(),
        )),
        (DbOp::Put { .. }, DocDbBasicResult::Put)
        | (DbOp::Delete { .. }, DocDbBasicResult::Delete) => Ok(DbResult::Done),
        (operation, result) => bail!(
            "semantic execute_ops result did not match operation: {} / {}",
            db_op_name(&operation),
            basic_result_name(&result)
        ),
    }
}

fn db_op_name(operation: &DbOp) -> &'static str {
    match operation {
        DbOp::Get { .. } => "get",
        DbOp::Query { .. } => "query",
        DbOp::Put { .. } => "put",
        DbOp::Delete { .. } => "delete",
    }
}

fn basic_result_name(result: &DocDbBasicResult) -> &'static str {
    match result {
        DocDbBasicResult::Get { .. } => "get",
        DocDbBasicResult::Query { .. } => "query",
        DocDbBasicResult::Put => "put",
        DocDbBasicResult::Delete => "delete",
    }
}
