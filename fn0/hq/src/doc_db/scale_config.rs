use crate::doc_db::DocDb;
use crate::docs::{ScaleConfigDoc, ScaleConfigDocGet};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

impl DocDb {
    pub async fn get_scale_config(&self, site_name: &str) -> Result<Option<ScaleConfigDoc>> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let doc = trx
                    .get(ScaleConfigDocGet { site_name })
                    .await?
                    .map(|h| (*h).clone());
                trx.commit(doc)
            })
            .await
        {
            TrxResult::Committed(doc) => Ok(doc),
            TrxResult::Conflict(d) => return Err(eyre!("get_scale_config conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }
}
