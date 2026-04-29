use crate::doc_db::DocDb;
use crate::docs::{
    DbRequest, HqJobPendingDoc, HqJobPendingDocGet, HqJobPendingDocQuery, HqJobProcessingDoc,
    HqJobProcessingDocGet, HqJobProcessingDocQuery,
};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

#[derive(Clone)]
pub struct HqJob {
    pub id: String,
    pub task_name: String,
    pub payload: String,
    pub retry_count: i64,
    pub max_retries: i64,
    pub processing_updated_at: String,
}

const STALE_PROCESSING_SECS: i64 = 60;

impl DocDb {
    pub async fn ensure_job_tables(&self) -> Result<()> {
        Ok(())
    }

    async fn list_pending(&self, limit: usize) -> Result<Vec<HqJobPendingDoc>> {
        let prepared = HqJobPendingDocQuery {
            created_at: None,
            id: None,
            limit: Some(limit),
        }
        .prepare();
        let mut iter = self
            .forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?
            .into_iter();
        (prepared.parse)(&mut iter).map_err(|e| eyre!("{}", e))
    }

    async fn list_processing(&self, limit: usize) -> Result<Vec<HqJobProcessingDoc>> {
        let prepared = HqJobProcessingDocQuery {
            updated_at: None,
            id: None,
            limit: Some(limit),
        }
        .prepare();
        let mut iter = self
            .forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?
            .into_iter();
        (prepared.parse)(&mut iter).map_err(|e| eyre!("{}", e))
    }

    pub async fn claim_job(&self) -> Result<Option<HqJob>> {
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(pending) = self.list_pending(1).await?.into_iter().next() {
            let now_owned = now.clone();
            match self
                .forte
                .trx::<_, _, _, (), anyhow::Error>(|trx| {
                    let pending = pending.clone();
                    let now = now_owned.clone();
                    async move {
                        let h = trx
                            .get(HqJobPendingDocGet {
                                created_at: pending.created_at.as_str(),
                                id: pending.id.as_str(),
                            })
                            .await
                            .map_err(anyhow::Error::from)?;
                        if let Some(handle) = h {
                            handle.delete();
                            trx.create(HqJobProcessingDoc {
                                updated_at: now.clone(),
                                id: pending.id.clone(),
                                task_name: pending.task_name.clone(),
                                payload: pending.payload.clone(),
                                retry_count: pending.retry_count,
                                max_retries: pending.max_retries,
                                original_created_at: pending.created_at.clone(),
                            })
                            .map_err(anyhow::Error::from)?;
                            trx.commit(Some(now)).map_err(anyhow::Error::from)
                        } else {
                            trx.commit(None).map_err(anyhow::Error::from)
                        }
                    }
                })
                .await
            {
                TrxResult::Committed(Some(updated_at)) => {
                    return Ok(Some(HqJob {
                        id: pending.id,
                        task_name: pending.task_name,
                        payload: pending.payload,
                        retry_count: pending.retry_count as i64,
                        max_retries: pending.max_retries as i64,
                        processing_updated_at: updated_at,
                    }));
                }
                TrxResult::Committed(None) => {} // raced; fall through
                TrxResult::Conflict(_) => {} // OCC retried internally; treat as race
                TrxResult::Err(e) => return Err(eyre!("{}", e)),
                TrxResult::Cancelled(_) => unreachable!(),
            }
        }

        // Look for stale processing jobs to re-claim
        if let Some(stale) = self.list_processing(1).await?.into_iter().next() {
            let updated = chrono::DateTime::parse_from_rfc3339(&stale.updated_at).ok();
            let now_dt = chrono::Utc::now();
            let is_stale = updated
                .map(|u| {
                    now_dt
                        .signed_duration_since(u.with_timezone(&chrono::Utc))
                        .num_seconds()
                        >= STALE_PROCESSING_SECS
                })
                .unwrap_or(true);
            if !is_stale {
                return Ok(None);
            }
            let now_owned = now;
            match self
                .forte
                .trx::<_, _, _, (), anyhow::Error>(|trx| {
                    let stale = stale.clone();
                    let now = now_owned.clone();
                    async move {
                        let h = trx
                            .get(HqJobProcessingDocGet {
                                updated_at: stale.updated_at.as_str(),
                                id: stale.id.as_str(),
                            })
                            .await
                            .map_err(anyhow::Error::from)?;
                        if let Some(handle) = h {
                            handle.delete();
                            trx.create(HqJobProcessingDoc {
                                updated_at: now.clone(),
                                id: stale.id.clone(),
                                task_name: stale.task_name.clone(),
                                payload: stale.payload.clone(),
                                retry_count: stale.retry_count,
                                max_retries: stale.max_retries,
                                original_created_at: stale.original_created_at.clone(),
                            })
                            .map_err(anyhow::Error::from)?;
                            trx.commit(Some(now)).map_err(anyhow::Error::from)
                        } else {
                            trx.commit(None).map_err(anyhow::Error::from)
                        }
                    }
                })
                .await
            {
                TrxResult::Committed(Some(updated_at)) => Ok(Some(HqJob {
                    id: stale.id,
                    task_name: stale.task_name,
                    payload: stale.payload,
                    retry_count: stale.retry_count as i64,
                    max_retries: stale.max_retries as i64,
                    processing_updated_at: updated_at,
                })),
                TrxResult::Committed(None) => Ok(None),
                TrxResult::Conflict(_) => Ok(None),
                TrxResult::Err(e) => Err(eyre!("{}", e)),
                TrxResult::Cancelled(_) => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn complete_job(&self, id: &str) -> Result<()> {
        self.delete_processing_by_id(id).await
    }

    pub async fn fail_job(&self, id: &str) -> Result<()> {
        self.delete_processing_by_id(id).await
    }

    pub async fn retry_job(&self, id: &str) -> Result<()> {
        let target = self
            .list_processing(1000)
            .await?
            .into_iter()
            .find(|p| p.id == id);
        let Some(p) = target else {
            return Ok(());
        };
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let p = p.clone();
                async move {
                    let h = trx
                        .get(HqJobProcessingDocGet {
                            updated_at: p.updated_at.as_str(),
                            id: p.id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    if let Some(handle) = h {
                        handle.delete();
                        trx.create(HqJobPendingDoc {
                            created_at: chrono::Utc::now().to_rfc3339(),
                            id: p.id.clone(),
                            task_name: p.task_name.clone(),
                            payload: p.payload.clone(),
                            retry_count: p.retry_count + 1,
                            max_retries: p.max_retries,
                        })
                        .map_err(anyhow::Error::from)?;
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) | TrxResult::Conflict(_) => Ok(()),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    async fn delete_processing_by_id(&self, id: &str) -> Result<()> {
        let target = self
            .list_processing(1000)
            .await?
            .into_iter()
            .find(|p| p.id == id);
        let Some(p) = target else {
            return Ok(());
        };
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let p = p.clone();
                async move {
                    let h = trx
                        .get(HqJobProcessingDocGet {
                            updated_at: p.updated_at.as_str(),
                            id: p.id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    if let Some(handle) = h {
                        handle.delete();
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) | TrxResult::Conflict(_) => Ok(()),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn enqueue_pending(
        &self,
        id: &str,
        task_name: &str,
        payload: &str,
        max_retries: u32,
    ) -> Result<()> {
        let doc = HqJobPendingDoc {
            created_at: chrono::Utc::now().to_rfc3339(),
            id: id.to_string(),
            task_name: task_name.to_string(),
            payload: payload.to_string(),
            retry_count: 0,
            max_retries,
        };
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let doc = doc.clone();
                async move {
                    trx.create(doc).map_err(anyhow::Error::from)?;
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("enqueue_pending conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }
}
