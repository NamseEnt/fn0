use crate::doc_db::DocDb;
use crate::docs::{ForteBuildDoc, ForteBuildDocGet, ForteBuildDocPut};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

impl DocDb {
    pub async fn register_build(
        &self,
        subdomain: &str,
        build_id: &str,
    ) -> Result<Vec<String>> {
        let subdomain_owned = subdomain.to_string();
        let build_id_owned = build_id.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let subdomain = subdomain_owned.clone();
                let build_id = build_id_owned.clone();
                async move {
                    let prev = trx
                        .get(ForteBuildDocGet {
                            subdomain: subdomain.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    let mut old_ids = Vec::new();
                    if let Some(mut handle) = prev {
                        old_ids.push(handle.build_id.clone());
                        handle.build_id = build_id;
                    } else {
                        trx.create(ForteBuildDoc {
                            subdomain: subdomain.clone(),
                            build_id,
                        })
                        .map_err(anyhow::Error::from)?;
                    }
                    trx.commit(old_ids).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(ids) => Ok(ids),
            TrxResult::Conflict(d) => Err(eyre!("register_build conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn enqueue_r2_prefix_delete(&self, subdomain: &str, build_id: &str) -> Result<()> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let prefix = format!("{subdomain}/{build_id}/");
        let payload = serde_json::json!({ "prefix": prefix }).to_string();
        self.enqueue_pending(&job_id, "delete_r2_prefix", &payload, 10)
            .await
    }

    pub async fn clear_build(&self, subdomain: &str) -> Result<()> {
        let subdomain_owned = subdomain.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let subdomain = subdomain_owned.clone();
                async move {
                    let prev = trx
                        .get(ForteBuildDocGet {
                            subdomain: subdomain.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    if let Some(handle) = prev {
                        handle.delete();
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("clear_build conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn enqueue_r2_subdomain_delete(&self, subdomain: &str) -> Result<()> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let prefix = format!("{subdomain}/");
        let payload = serde_json::json!({ "prefix": prefix }).to_string();
        self.enqueue_pending(&job_id, "delete_r2_prefix", &payload, 10)
            .await
    }
}
