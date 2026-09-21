use crate::{BatchOp, DbOp, DbResult};
use anyhow::{Result, anyhow, bail};
use bytes::Bytes;
use doc_db_protocol::{
    DocDbBatchOperation, DocDbKey, DocDbObservedDocument, DocDbOperation, DocDbRequest,
    DocDbResponse, DocDbResult, DocDbTransactItem, DocDbTransactOutcome, decode_response,
    encode_request,
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
        let mut results = Vec::with_capacity(operations.len());
        for operation in operations {
            let result = match operation {
                DbOp::Get { pk, sk } => DbResult::Single(self.get(&pk, &sk).await?),
                DbOp::Query {
                    pk,
                    after_sk,
                    limit,
                } => DbResult::Multiple(
                    self.query(&pk, after_sk.as_deref(), limit.unwrap_or(usize::MAX))
                        .await?,
                ),
                DbOp::Put { pk, sk, data } => {
                    self.put(&pk, &sk, &data).await?;
                    DbResult::Done
                }
                DbOp::Delete { pk, sk } => {
                    self.delete(&pk, &sk).await?;
                    DbResult::Done
                }
            };
            results.push(result);
        }
        Ok(results)
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
                    DocDbObservedDocument::Present { data, version } => {
                        Ok(crate::ObservedDocument::Present {
                            data: data.into(),
                            version,
                        })
                    }
                    DocDbObservedDocument::Missing => Ok(crate::ObservedDocument::Missing),
                })
                .collect(),
            other => bail!("unexpected semantic doc-db observed response: {other:?}"),
        }
    }

    pub(crate) async fn transact(
        &self,
        items: &[crate::TransactItem],
    ) -> Result<crate::TransactOutcome> {
        let items = items
            .iter()
            .map(|item| match item {
                crate::TransactItem::CheckVersion {
                    pk,
                    sk,
                    expected_version,
                } => DocDbTransactItem::CheckVersion {
                    key: DocDbKey::new(pk, sk),
                    expected_version: *expected_version,
                },
                crate::TransactItem::CheckMissing { pk, sk } => DocDbTransactItem::CheckMissing {
                    key: DocDbKey::new(pk, sk),
                },
                crate::TransactItem::Insert { pk, sk, data } => DocDbTransactItem::Insert {
                    key: DocDbKey::new(pk, sk),
                    data: data.clone(),
                },
                crate::TransactItem::Update {
                    pk,
                    sk,
                    expected_version,
                    data,
                } => DocDbTransactItem::Update {
                    key: DocDbKey::new(pk, sk),
                    expected_version: *expected_version,
                    data: data.clone(),
                },
                crate::TransactItem::Delete {
                    pk,
                    sk,
                    expected_version,
                } => DocDbTransactItem::Delete {
                    key: DocDbKey::new(pk, sk),
                    expected_version: *expected_version,
                },
            })
            .collect();
        let response = self.execute(DocDbOperation::Transact { items }).await?;
        match response.result {
            DocDbResult::Transact { outcome } => Ok(match outcome {
                DocDbTransactOutcome::Committed => crate::TransactOutcome { conflict: None },
                DocDbTransactOutcome::Conflict { step_index } => crate::TransactOutcome {
                    conflict: Some(crate::TransactConflict { step_index }),
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
