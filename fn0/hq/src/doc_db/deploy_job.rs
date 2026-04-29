use crate::doc_db::DocDb;
use crate::docs::{DbRequest, DeployJobDoc, DeployJobDocGet, DeployJobDocPut, DeployJobDocQuery};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

pub use crate::docs::{DeployJobDoc as DeployJob, DeployJobPhase};

impl DocDb {
    pub async fn insert_deploy_job(&self, job: DeployJobDoc) -> Result<()> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let job = job.clone();
                async move {
                    trx.create(job).map_err(anyhow::Error::from)?;
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("insert_deploy_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn get_deploy_job(&self, job_id: &str) -> Result<Option<DeployJobDoc>> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let doc = trx
                    .get(DeployJobDocGet { job_id })
                    .await
                    .map_err(anyhow::Error::from)?
                    .map(|h| (*h).clone());
                trx.commit(doc).map_err(anyhow::Error::from)
            })
            .await
        {
            TrxResult::Committed(d) => Ok(d),
            TrxResult::Conflict(d) => Err(eyre!("get_deploy_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn update_deploy_job(&self, job: &DeployJobDoc) -> Result<()> {
        let job_id = job.job_id.clone();
        let updated = job.clone();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let job_id = job_id.clone();
                let updated = updated.clone();
                async move {
                    let handle = trx
                        .get(DeployJobDocGet {
                            job_id: job_id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    if let Some(mut h) = handle {
                        *h = updated;
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("update_deploy_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn list_active_deploy_jobs(&self) -> Result<Vec<DeployJobDoc>> {
        let prepared = DeployJobDocQuery {
            job_id: None,
            limit: Some(1000),
        }
        .prepare();
        let mut results = self
            .forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?
            .into_iter();
        let docs: Vec<DeployJobDoc> =
            (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;
        let jobs: Vec<DeployJobDoc> = docs.into_iter().filter(|d| !d.is_terminal()).collect();
        Ok(jobs)
    }
}

