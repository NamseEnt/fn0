use crate::doc_db::DocDb;
use crate::docs::{
    CustomDomainByDomainDoc, CustomDomainByDomainDocGet, CustomDomainBySubdomainDoc,
    CustomDomainBySubdomainDocGet, CustomDomainBySubdomainDocQuery, DbRequest,
};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

pub struct CustomDomainEntry {
    pub subdomain: String,
    pub domain: String,
}

pub enum InsertCustomDomainOutcome {
    Inserted,
    AlreadyExists,
    SubdomainTaken { existing_domain: String },
    DomainTaken { existing_subdomain: String },
}

impl DocDb {
    pub async fn insert_custom_domain(
        &self,
        subdomain: &str,
        domain: &str,
        cloudflare_hostname_id: &str,
    ) -> Result<InsertCustomDomainOutcome> {
        let subdomain_owned = subdomain.to_string();
        let domain_owned = domain.to_string();
        let hostname_id_owned = cloudflare_hostname_id.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let subdomain = subdomain_owned.clone();
                let domain = domain_owned.clone();
                let hostname_id = hostname_id_owned.clone();
                async move {
                    let (by_sub, by_dom) = trx
                        .get((
                            CustomDomainBySubdomainDocGet {
                                subdomain: subdomain.as_str(),
                            },
                            CustomDomainByDomainDocGet {
                                domain: domain.as_str(),
                            },
                        ))
                        .await
                        .map_err(anyhow::Error::from)?;

                    let outcome = match (by_sub, by_dom) {
                        (Some(sub_doc), Some(dom_doc)) => {
                            if sub_doc.domain == domain && dom_doc.subdomain == subdomain {
                                InsertCustomDomainOutcome::AlreadyExists
                            } else if sub_doc.domain != domain {
                                InsertCustomDomainOutcome::SubdomainTaken {
                                    existing_domain: sub_doc.domain.clone(),
                                }
                            } else {
                                InsertCustomDomainOutcome::DomainTaken {
                                    existing_subdomain: dom_doc.subdomain.clone(),
                                }
                            }
                        }
                        (Some(sub_doc), None) => {
                            if sub_doc.domain == domain {
                                InsertCustomDomainOutcome::AlreadyExists
                            } else {
                                InsertCustomDomainOutcome::SubdomainTaken {
                                    existing_domain: sub_doc.domain.clone(),
                                }
                            }
                        }
                        (None, Some(dom_doc)) => InsertCustomDomainOutcome::DomainTaken {
                            existing_subdomain: dom_doc.subdomain.clone(),
                        },
                        (None, None) => {
                            trx.create(CustomDomainBySubdomainDoc {
                                subdomain: subdomain.clone(),
                                domain: domain.clone(),
                            })
                            .map_err(anyhow::Error::from)?;
                            trx.create(CustomDomainByDomainDoc {
                                domain: domain.clone(),
                                subdomain: subdomain.clone(),
                                cloudflare_hostname_id: hostname_id.clone(),
                            })
                            .map_err(anyhow::Error::from)?;
                            InsertCustomDomainOutcome::Inserted
                        }
                    };

                    trx.commit(outcome).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(o) => Ok(o),
            TrxResult::Conflict(d) => Err(eyre!("insert_custom_domain conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn delete_custom_domain(&self, subdomain: &str, domain: &str) -> Result<()> {
        let subdomain_owned = subdomain.to_string();
        let domain_owned = domain.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let subdomain = subdomain_owned.clone();
                let domain = domain_owned.clone();
                async move {
                    let (by_sub, by_dom) = trx
                        .get((
                            CustomDomainBySubdomainDocGet {
                                subdomain: subdomain.as_str(),
                            },
                            CustomDomainByDomainDocGet {
                                domain: domain.as_str(),
                            },
                        ))
                        .await
                        .map_err(anyhow::Error::from)?;
                    if let Some(h) = by_sub {
                        if h.domain == domain {
                            h.delete();
                        }
                    }
                    if let Some(h) = by_dom {
                        if h.subdomain == subdomain {
                            h.delete();
                        }
                    }
                    trx.commit(()).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(_) => Ok(()),
            TrxResult::Conflict(d) => Err(eyre!("delete_custom_domain conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn get_custom_domain_by_subdomain(&self, subdomain: &str) -> Result<Option<String>> {
        let subdomain_owned = subdomain.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let subdomain = subdomain_owned.clone();
                async move {
                    let h = trx
                        .get(CustomDomainBySubdomainDocGet {
                            subdomain: subdomain.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    let domain = h.map(|h| h.domain.clone());
                    trx.commit(domain).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(d) => Ok(d),
            TrxResult::Conflict(d) => {
                Err(eyre!("get_custom_domain_by_subdomain conflict: {:?}", d))
            }
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn get_custom_domain_by_domain(
        &self,
        domain: &str,
    ) -> Result<Option<(String, String)>> {
        let domain_owned = domain.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let domain = domain_owned.clone();
                async move {
                    let h = trx
                        .get(CustomDomainByDomainDocGet {
                            domain: domain.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?;
                    let v = h.map(|h| (h.subdomain.clone(), h.cloudflare_hostname_id.clone()));
                    trx.commit(v).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(v) => Ok(v),
            TrxResult::Conflict(d) => Err(eyre!("get_custom_domain_by_domain conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn list_custom_domains(&self) -> Result<Vec<CustomDomainEntry>> {
        let prepared = CustomDomainBySubdomainDocQuery {
            subdomain: None,
            limit: Some(100_000),
        }
        .prepare();
        let mut results = self
            .forte
            .execute_ops(prepared.ops)
            .await
            .map_err(|e| eyre!("{}", e))?
            .into_iter();
        let by_sub_docs: Vec<CustomDomainBySubdomainDoc> =
            (prepared.parse)(&mut results).map_err(|e| eyre!("{}", e))?;

        Ok(by_sub_docs
            .into_iter()
            .map(|d| CustomDomainEntry {
                subdomain: d.subdomain,
                domain: d.domain,
            })
            .collect())
    }
}
