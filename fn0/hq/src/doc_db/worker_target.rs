use crate::doc_db::DocDb;
use crate::docs::{
    CwasmReadyDoc, CwasmReadyDocGet, CwasmReadyDocPut, DbRequest, WorkerLastStableDoc,
    WorkerLastStableDocGet, WorkerLastStableDocPut, WorkerTargetDoc, WorkerTargetDocGet,
};
use color_eyre::eyre::{Result, eyre};
use forte_db::TrxResult;

pub use crate::docs::{
    CwasmReadyDoc as CwasmReady, WorkerLastStableDoc as WorkerLastStable,
    WorkerTargetDoc as WorkerTarget,
};

impl WorkerTargetDoc {
    pub fn image_ref(&self) -> String {
        format!(
            "{}/{}:{}",
            self.image_registry, self.image_repository, self.image_tag
        )
    }
}

impl WorkerLastStableDoc {
    pub fn image_ref(&self) -> String {
        format!(
            "{}/{}:{}",
            self.image_registry, self.image_repository, self.image_tag
        )
    }
}

impl DocDb {
    pub async fn get_worker_target(&self, site_name: &str) -> Result<Option<WorkerTargetDoc>> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let doc = trx
                    .get(WorkerTargetDocGet { site_name })
                    .await
                    .map_err(anyhow::Error::from)?
                    .map(|h| (*h).clone());
                trx.commit(doc).map_err(anyhow::Error::from)
            })
            .await
        {
            TrxResult::Committed(doc) => Ok(doc),
            TrxResult::Conflict(d) => return Err(eyre!("get_worker_target conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn get_cwasm_ready(&self, site_name: &str) -> Result<Option<CwasmReadyDoc>> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let doc = trx
                    .get(CwasmReadyDocGet { site_name })
                    .await
                    .map_err(anyhow::Error::from)?
                    .map(|h| (*h).clone());
                trx.commit(doc).map_err(anyhow::Error::from)
            })
            .await
        {
            TrxResult::Committed(doc) => Ok(doc),
            TrxResult::Conflict(d) => return Err(eyre!("get_cwasm_ready conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn set_cwasm_ready(&self, ready: CwasmReadyDoc) -> Result<()> {
        let prepared = CwasmReadyDocPut(ready).prepare();
        self.forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?;
        Ok(())
    }

    pub async fn get_worker_last_stable(
        &self,
        site_name: &str,
    ) -> Result<Option<WorkerLastStableDoc>> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let doc = trx
                    .get(WorkerLastStableDocGet { site_name })
                    .await
                    .map_err(anyhow::Error::from)?
                    .map(|h| (*h).clone());
                trx.commit(doc).map_err(anyhow::Error::from)
            })
            .await
        {
            TrxResult::Committed(doc) => Ok(doc),
            TrxResult::Conflict(d) => return Err(eyre!("get_worker_last_stable conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn set_worker_last_stable(&self, last_stable: WorkerLastStableDoc) -> Result<()> {
        let prepared = WorkerLastStableDocPut(last_stable).prepare();
        self.forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?;
        Ok(())
    }
}
