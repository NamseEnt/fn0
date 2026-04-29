use crate::doc_db::DocDb;
use crate::docs::{DbRequest, HostStatusDocGet, HostStatusDocQuery};
use color_eyre::eyre::{Result, eyre};
use forte_db::TrxResult;

pub use crate::docs::{GracefulPurpose, HostStatusDoc, HostStatusDoc as HostStatus};

impl DocDb {
    pub async fn upsert_host_status_if_absent(
        &self,
        host_id: &str,
        site_name: &str,
        addr: &str,
        dns_addr: Option<&str>,
    ) -> Result<()> {
        let host_id = host_id.to_string();
        let site_name = site_name.to_string();
        let addr = addr.to_string();
        let dns_addr = dns_addr.map(|s| s.to_string());
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let host_id = host_id.clone();
                let site_name = site_name.clone();
                let addr = addr.clone();
                let dns_addr = dns_addr.clone();
                async move {
                    let existing = trx
                        .get(HostStatusDocGet {
                            site_name: site_name.as_str(),
                            host_id: host_id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    if existing.is_none() {
                        trx.create(HostStatusDoc {
                            site_name,
                            host_id,
                            addr,
                            dns_addr,
                            generation: None,
                            instances: None,
                            healthy: false,
                            consecutive_failures: 0,
                            last_verified_at: None,
                            graceful: false,
                            graceful_purpose: None,
                            graceful_since_at: None,
                        })
                        .map_err(anyhow::Error::from)?;
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("upsert_host_status_if_absent conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn get_host_status(
        &self,
        site_name: &str,
        host_id: &str,
    ) -> Result<Option<HostStatusDoc>> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let doc = trx
                    .get(HostStatusDocGet { site_name, host_id })
                    .await
                    .map_err(anyhow::Error::from)?
                    .map(|h| (*h).clone());
                trx.commit(doc).map_err(anyhow::Error::from)
            })
            .await
        {
            TrxResult::Committed(d) => Ok(d),
            TrxResult::Conflict(d) => Err(eyre!("get_host_status conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn list_host_statuses(&self, site_name: &str) -> Result<Vec<HostStatusDoc>> {
        let prepared = HostStatusDocQuery {
            site_name,
            host_id: None,
            limit: Some(1000),
        }
        .prepare();
        let mut results = self
            .forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?
            .into_iter();
        let docs: Vec<HostStatusDoc> =
            (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;
        Ok(docs)
    }

    pub async fn delete_host_status(&self, site_name: &str, host_id: &str) -> Result<()> {
        let site_name = site_name.to_string();
        let host_id = host_id.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let site_name = site_name.clone();
                let host_id = host_id.clone();
                async move {
                    let h = trx
                        .get(HostStatusDocGet {
                            site_name: site_name.as_str(),
                            host_id: host_id.as_str(),
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
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("delete_host_status conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn mark_host_verified(
        &self,
        site_name: &str,
        host_id: &str,
        generation: u64,
        instances: u64,
        now_rfc3339: &str,
    ) -> Result<()> {
        let site_name = site_name.to_string();
        let host_id = host_id.to_string();
        let now = now_rfc3339.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let site_name = site_name.clone();
                let host_id = host_id.clone();
                let now = now.clone();
                async move {
                    let h = trx
                        .get(HostStatusDocGet {
                            site_name: site_name.as_str(),
                            host_id: host_id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    if let Some(mut s) = h {
                        s.generation = Some(generation);
                        s.instances = Some(instances);
                        s.healthy = true;
                        s.consecutive_failures = 0;
                        s.last_verified_at = Some(now);
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("mark_host_verified conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn mark_host_failed(&self, site_name: &str, host_id: &str) -> Result<u32> {
        let site_name = site_name.to_string();
        let host_id = host_id.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let site_name = site_name.clone();
                let host_id = host_id.clone();
                async move {
                    let h = trx
                        .get(HostStatusDocGet {
                            site_name: site_name.as_str(),
                            host_id: host_id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    let failures = if let Some(mut s) = h {
                        s.healthy = false;
                        s.consecutive_failures = s.consecutive_failures.saturating_add(1);
                        s.consecutive_failures
                    } else {
                        0
                    };
                    trx.commit(failures).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(f) => Ok(f),
            TrxResult::Conflict(d) => Err(eyre!("mark_host_failed conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn clear_host_graceful(&self, site_name: &str, host_id: &str) -> Result<()> {
        let site_name = site_name.to_string();
        let host_id = host_id.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let site_name = site_name.clone();
                let host_id = host_id.clone();
                async move {
                    let h = trx
                        .get(HostStatusDocGet {
                            site_name: site_name.as_str(),
                            host_id: host_id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    if let Some(mut s) = h {
                        s.graceful = false;
                        s.graceful_purpose = None;
                        s.graceful_since_at = None;
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("clear_host_graceful conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn set_host_graceful(
        &self,
        site_name: &str,
        host_id: &str,
        purpose: GracefulPurpose,
        now_rfc3339: &str,
    ) -> Result<bool> {
        let site_name = site_name.to_string();
        let host_id = host_id.to_string();
        let now = now_rfc3339.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let site_name = site_name.clone();
                let host_id = host_id.clone();
                let now = now.clone();
                async move {
                    let h = trx
                        .get(HostStatusDocGet {
                            site_name: site_name.as_str(),
                            host_id: host_id.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    let did = if let Some(mut s) = h {
                        if s.graceful {
                            false
                        } else {
                            s.graceful = true;
                            s.graceful_purpose = Some(purpose);
                            s.graceful_since_at = Some(now);
                            true
                        }
                    } else {
                        false
                    };
                    trx.commit(did).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(d) => Ok(d),
            TrxResult::Conflict(d) => Err(eyre!("set_host_graceful conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }
}
