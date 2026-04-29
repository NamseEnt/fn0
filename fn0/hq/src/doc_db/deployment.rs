use crate::doc_db::DocDb;
use crate::docs::{
    DbRequest, DeploymentDoc, DeploymentDocQuery, DeploymentKind, NextDeploymentSeqDoc,
    NextDeploymentSeqDocGet,
};
use color_eyre::eyre::{Result, eyre};
use forte_db::TrxResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Deployment {
    Deploy {
        subdomain: String,
        code_id: u64,
        code_version: u64,
    },
    Undeploy {
        subdomain: String,
    },
}

fn doc_to_deployment(doc: DeploymentDoc) -> Option<Deployment> {
    match doc.kind {
        DeploymentKind::Deploy => Some(Deployment::Deploy {
            subdomain: doc.subdomain,
            code_id: doc.code_id?,
            code_version: doc.code_version?,
        }),
        DeploymentKind::Undeploy => Some(Deployment::Undeploy {
            subdomain: doc.subdomain,
        }),
    }
}

impl DocDb {
    pub async fn all_deployments(&self) -> Result<Vec<Deployment>> {
        let prepared = DeploymentDocQuery {
            seq: None,
            limit: Some(100_000),
        }
        .prepare();
        let mut results = self
            .forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?
            .into_iter();
        let docs: Vec<DeploymentDoc> =
            (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;
        Ok(docs.into_iter().filter_map(doc_to_deployment).collect())
    }

    pub async fn deployment_exists(
        &self,
        subdomain: &str,
        code_id: u64,
        code_version: u64,
    ) -> Result<bool> {
        let all = self.all_deployments().await?;
        Ok(all.iter().any(|d| match d {
            Deployment::Deploy {
                subdomain: s,
                code_id: c,
                code_version: v,
            } => s == subdomain && *c == code_id && *v == code_version,
            _ => false,
        }))
    }

    pub async fn insert_deployment(
        &self,
        subdomain: &str,
        code_id: u64,
        code_version: u64,
    ) -> Result<()> {
        let subdomain = subdomain.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let subdomain = subdomain.clone();
                async move {
                    let counter = trx
                        .get(NextDeploymentSeqDocGet {})
                        .await
                        .map_err(anyhow::Error::from)?;
                    let next = counter.as_ref().map(|c| c.count + 1).unwrap_or(1);
                    if let Some(mut c) = counter {
                        c.count = next;
                    } else {
                        trx.create(NextDeploymentSeqDoc { count: next })
                            .map_err(anyhow::Error::from)?;
                    }
                    trx.create(DeploymentDoc {
                        seq: next,
                        subdomain,
                        kind: DeploymentKind::Deploy,
                        code_id: Some(code_id),
                        code_version: Some(code_version),
                    })
                    .map_err(anyhow::Error::from)?;
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("insert_deployment conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn insert_undeployment_with_job(
        &self,
        subdomain: &str,
        job_id: &str,
        job_payload: &str,
    ) -> Result<()> {
        let subdomain = subdomain.to_string();
        let job_id = job_id.to_string();
        let job_payload = job_payload.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let subdomain = subdomain.clone();
                let job_id = job_id.clone();
                let job_payload = job_payload.clone();
                async move {
                    let counter = trx
                        .get(NextDeploymentSeqDocGet {})
                        .await
                        .map_err(anyhow::Error::from)?;
                    let next = counter.as_ref().map(|c| c.count + 1).unwrap_or(1);
                    if let Some(mut c) = counter {
                        c.count = next;
                    } else {
                        trx.create(NextDeploymentSeqDoc { count: next })
                            .map_err(anyhow::Error::from)?;
                    }
                    trx.create(DeploymentDoc {
                        seq: next,
                        subdomain,
                        kind: DeploymentKind::Undeploy,
                        code_id: None,
                        code_version: None,
                    })
                    .map_err(anyhow::Error::from)?;
                    trx.create(crate::docs::HqJobPendingDoc {
                        created_at: chrono::Utc::now().to_rfc3339(),
                        id: job_id,
                        task_name: "delete_wasm".to_string(),
                        payload: job_payload,
                        retry_count: 0,
                        max_retries: 3,
                    })
                    .map_err(anyhow::Error::from)?;
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("insert_undeployment_with_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn deployments_after(&self, sk: u64) -> Result<Vec<Deployment>> {
        let prepared = DeploymentDocQuery {
            seq: Some(sk),
            limit: Some(100_000),
        }
        .prepare();
        let mut results = self
            .forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?
            .into_iter();
        let docs: Vec<DeploymentDoc> =
            (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;
        Ok(docs.into_iter().filter_map(doc_to_deployment).collect())
    }
}
