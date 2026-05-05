use crate::doc_db::DocDb;
use crate::docs::{DbRequest, RenameJobDoc, RenameJobDocGet, RenameJobDocQuery};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

pub use crate::docs::{RenameJobDoc as RenameJob, RenameJobPhase};

impl DocDb {
    pub async fn insert_rename_job(&self, job: RenameJobDoc) -> Result<()> {
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
            TrxResult::Conflict(d) => Err(eyre!("insert_rename_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn get_rename_job(&self, job_id: &str) -> Result<Option<RenameJobDoc>> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let doc = trx
                    .get(RenameJobDocGet { job_id })
                    .await
                    .map_err(anyhow::Error::from)?
                    .map(|h| (*h).clone());
                trx.commit(doc).map_err(anyhow::Error::from)
            })
            .await
        {
            TrxResult::Committed(d) => Ok(d),
            TrxResult::Conflict(d) => Err(eyre!("get_rename_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn update_rename_job(&self, job: &RenameJobDoc) -> Result<()> {
        let job_id = job.job_id.clone();
        let updated = job.clone();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let job_id = job_id.clone();
                let updated = updated.clone();
                async move {
                    let handle = trx
                        .get(RenameJobDocGet {
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
            TrxResult::Conflict(d) => Err(eyre!("update_rename_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn list_active_rename_jobs(&self) -> Result<Vec<RenameJobDoc>> {
        let prepared = RenameJobDocQuery {
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
        let docs: Vec<RenameJobDoc> = (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;
        let jobs: Vec<RenameJobDoc> = docs.into_iter().filter(|d| !d.is_terminal()).collect();
        Ok(jobs)
    }

    pub async fn find_active_rename_job_for(
        &self,
        old_subdomain: &str,
    ) -> Result<Option<RenameJobDoc>> {
        let jobs = self.list_active_rename_jobs().await?;
        Ok(jobs.into_iter().find(|j| j.old_subdomain == old_subdomain))
    }
}
