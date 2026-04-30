use crate::doc_db::DocDb;
use crate::docs::{
    CodeVersionDoc, CodeVersionDocGet, CodeVersionDocPut, NextCodeIdDoc, NextCodeIdDocGet,
    NextCodeIdDocPut, ProjectDoc, ProjectDocGet, ProjectDocPut,
};
use color_eyre::eyre::{Result, eyre};
use doc_db::TrxResult;

pub struct ProjectInfo {
    pub code_id: u64,
    pub subdomain: String,
}

impl DocDb {
    pub async fn get_project(
        &self,
        github_username: &str,
        project_name: &str,
    ) -> Result<Option<ProjectInfo>> {
        let subdomain = format!("{github_username}-{project_name}");
        let github_username = github_username.to_string();
        let project_name = project_name.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let github_username = github_username.clone();
                let project_name = project_name.clone();
                async move {
                    let doc = trx
                        .get(ProjectDocGet {
                            github_username: github_username.as_str(),
                            project_name: project_name.as_str(),
                        })
                        .await
                        .map_err(anyhow::Error::from)?
                        .map(|h| h.code_id);
                    trx.commit(doc).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(Some(code_id)) => Ok(Some(ProjectInfo { code_id, subdomain })),
            TrxResult::Committed(None) => Ok(None),
            TrxResult::Conflict(d) => Err(eyre!("get_project conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn get_or_create_project(
        &self,
        github_username: &str,
        project_name: &str,
    ) -> Result<ProjectInfo> {
        let subdomain = format!("{github_username}-{project_name}");
        let github_username_owned = github_username.to_string();
        let project_name_owned = project_name.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let github_username = github_username_owned.clone();
                let project_name = project_name_owned.clone();
                async move {
                    let (project_handle, counter_handle) = trx
                        .get((
                            ProjectDocGet {
                                github_username: github_username.as_str(),
                                project_name: project_name.as_str(),
                            },
                            NextCodeIdDocGet {},
                        ))
                        .await
                        .map_err(anyhow::Error::from)?;

                    let code_id = if let Some(p) = project_handle {
                        p.code_id
                    } else {
                        let next = counter_handle.as_ref().map(|c| c.count + 1).unwrap_or(1);
                        if let Some(mut c) = counter_handle {
                            c.count = next;
                        } else {
                            trx.create(NextCodeIdDoc { count: next })
                                .map_err(anyhow::Error::from)?;
                        }
                        trx.create(ProjectDoc {
                            github_username: github_username.clone(),
                            project_name: project_name.clone(),
                            code_id: next,
                        })
                        .map_err(anyhow::Error::from)?;
                        next
                    };

                    trx.commit(code_id).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(code_id) => Ok(ProjectInfo { code_id, subdomain }),
            TrxResult::Conflict(d) => Err(eyre!("get_or_create_project conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn rename_project(
        &self,
        github_username: &str,
        old_project_name: &str,
        new_project_name: &str,
    ) -> Result<u64> {
        let username = github_username.to_string();
        let old_name = old_project_name.to_string();
        let new_name = new_project_name.to_string();
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| {
                let username = username.clone();
                let old_name = old_name.clone();
                let new_name = new_name.clone();
                async move {
                    let (old_handle, new_handle) = trx
                        .get((
                            ProjectDocGet {
                                github_username: username.as_str(),
                                project_name: old_name.as_str(),
                            },
                            ProjectDocGet {
                                github_username: username.as_str(),
                                project_name: new_name.as_str(),
                            },
                        ))
                        .await
                        .map_err(anyhow::Error::from)?;

                    let code_id = match (old_handle, new_handle) {
                        (Some(old), None) => {
                            let id = old.code_id;
                            old.delete();
                            trx.create(ProjectDoc {
                                github_username: username.clone(),
                                project_name: new_name.clone(),
                                code_id: id,
                            })
                            .map_err(anyhow::Error::from)?;
                            id
                        }
                        (None, Some(new)) => new.code_id,
                        (Some(old), Some(new)) => {
                            if old.code_id != new.code_id {
                                return Err(anyhow::anyhow!(
                                    "rename_project conflict: old code_id={} new code_id={}",
                                    old.code_id,
                                    new.code_id
                                ));
                            }
                            let id = old.code_id;
                            old.delete();
                            id
                        }
                        (None, None) => {
                            return Err(anyhow::anyhow!(
                                "rename_project: neither old ({}, {}) nor new ({}, {}) project exists",
                                username,
                                old_name,
                                username,
                                new_name
                            ));
                        }
                    };

                    trx.commit(code_id).map_err(anyhow::Error::from)
                }
            })
            .await
        {
            TrxResult::Committed(id) => Ok(id),
            TrxResult::Conflict(d) => Err(eyre!("rename_project conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }

    pub async fn next_code_version(&self, code_id: u64) -> Result<u64> {
        match self
            .forte
            .trx::<_, _, _, (), anyhow::Error>(|trx| async move {
                let handle = trx
                    .get(CodeVersionDocGet { code_id })
                    .await
                    .map_err(anyhow::Error::from)?;
                let next = handle.as_ref().map(|h| h.count + 1).unwrap_or(1);
                if let Some(mut h) = handle {
                    h.count = next;
                } else {
                    trx.create(CodeVersionDoc {
                        code_id,
                        count: next,
                    })
                    .map_err(anyhow::Error::from)?;
                }
                trx.commit(next).map_err(anyhow::Error::from)
            })
            .await
        {
            TrxResult::Committed(v) => Ok(v),
            TrxResult::Conflict(d) => Err(eyre!("next_code_version conflict: {:?}", d)),
            TrxResult::Err(e) => Err(eyre!("{}", e)),
            TrxResult::Cancelled(_) => unreachable!(),
        }
    }
}
