use crate::doc_db::DocDb;
use crate::docs::{
    CustomDomainAddJobDoc, CustomDomainAddJobDocGet, CustomDomainAddJobDocQuery,
    CustomDomainRemoveJobDoc, CustomDomainRemoveJobDocGet, CustomDomainRemoveJobDocQuery,
    DbRequest,
};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

pub use crate::docs::{
    CustomDomainAddJobDoc as CustomDomainAddJob, CustomDomainAddPhase,
    CustomDomainRemoveJobDoc as CustomDomainRemoveJob, CustomDomainRemovePhase,
};

impl DocDb {
    pub async fn insert_custom_domain_add_job(&self, job: CustomDomainAddJobDoc) -> Result<()> {
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
            TrxResult::Conflict(d) => Err(eyre!("insert_custom_domain_add_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn update_custom_domain_add_job(&self, job: &CustomDomainAddJobDoc) -> Result<()> {
        let job_id = job.job_id.clone();
        let updated = job.clone();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let job_id = job_id.clone();
                let updated = updated.clone();
                async move {
                    let handle = trx
                        .get(CustomDomainAddJobDocGet {
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
            TrxResult::Conflict(d) => Err(eyre!("update_custom_domain_add_job conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn list_active_custom_domain_add_jobs(&self) -> Result<Vec<CustomDomainAddJobDoc>> {
        let prepared = CustomDomainAddJobDocQuery {
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
        let docs: Vec<CustomDomainAddJobDoc> =
            (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;
        Ok(docs.into_iter().filter(|d| !d.is_terminal()).collect())
    }

    pub async fn find_active_custom_domain_add_job_for(
        &self,
        subdomain: &str,
    ) -> Result<Option<CustomDomainAddJobDoc>> {
        let jobs = self.list_active_custom_domain_add_jobs().await?;
        Ok(jobs.into_iter().find(|j| j.subdomain == subdomain))
    }

    pub async fn insert_custom_domain_remove_job(
        &self,
        job: CustomDomainRemoveJobDoc,
    ) -> Result<()> {
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
            TrxResult::Conflict(d) => Err(eyre!(
                "insert_custom_domain_remove_job conflict: {:?}",
                d
            )),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn update_custom_domain_remove_job(
        &self,
        job: &CustomDomainRemoveJobDoc,
    ) -> Result<()> {
        let job_id = job.job_id.clone();
        let updated = job.clone();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let job_id = job_id.clone();
                let updated = updated.clone();
                async move {
                    let handle = trx
                        .get(CustomDomainRemoveJobDocGet {
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
            TrxResult::Conflict(d) => Err(eyre!(
                "update_custom_domain_remove_job conflict: {:?}",
                d
            )),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn list_active_custom_domain_remove_jobs(
        &self,
    ) -> Result<Vec<CustomDomainRemoveJobDoc>> {
        let prepared = CustomDomainRemoveJobDocQuery {
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
        let docs: Vec<CustomDomainRemoveJobDoc> =
            (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;
        Ok(docs.into_iter().filter(|d| !d.is_terminal()).collect())
    }

    pub async fn find_active_custom_domain_remove_job_for(
        &self,
        subdomain: &str,
    ) -> Result<Option<CustomDomainRemoveJobDoc>> {
        let jobs = self.list_active_custom_domain_remove_jobs().await?;
        Ok(jobs.into_iter().find(|j| j.subdomain == subdomain))
    }
}
