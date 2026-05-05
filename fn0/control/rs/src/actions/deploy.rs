use crate::common::auth;
use crate::common::aws_sign;
use crate::docs::*;
use crate::quota;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub build_id: String,
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub jobs: Vec<CronJob>,
    pub cron_updated_at: DateTime,
}

#[derive(Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
}

#[derive(Serialize)]
pub enum Output {
    Ok {
        presigned_put_url: String,
        object_key: String,
        static_uploads: Vec<StaticUpload>,
    },
    QuotaExceeded {
        reason: String,
    },
    NotLoggedIn,
    NotFound,
    Forbidden,
    Error {
        message: String,
    },
}

#[derive(Serialize)]
pub struct StaticUpload {
    pub path: String,
    pub presigned_url: String,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    let Some(user) = auth::bearer_user(req.headers).await else {
        return Output::NotLoggedIn;
    };

    let db = doc_db::turso();
    let project = match (ProjectDocGet {
        project_id: &req.body.project_id,
    })
    .send_with(&db)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return Output::NotFound,
        Err(e) => {
            return Output::Error {
                message: e.to_string(),
            };
        }
    };

    if project.owner_github_id != user.github_id {
        return Output::Forbidden;
    }

    if req.body.files.len() > quota::MAX_FILES_PER_BUILD {
        return Output::QuotaExceeded {
            reason: format!(
                "file count {} exceeds limit {}",
                req.body.files.len(),
                quota::MAX_FILES_PER_BUILD
            ),
        };
    }
    let total_size: u64 = req.body.files.iter().map(|f| f.size).sum();
    if total_size > quota::MAX_TOTAL_SIZE_PER_BUILD {
        return Output::QuotaExceeded {
            reason: format!(
                "total size {} bytes exceeds limit {}",
                total_size,
                quota::MAX_TOTAL_SIZE_PER_BUILD
            ),
        };
    }

    let bundle_env = match BundleEnv::from_env() {
        Ok(v) => v,
        Err(e) => {
            return Output::Error {
                message: e.to_string(),
            };
        }
    };
    let object_key = format!("original/{}.tar", project.project_id);
    let presigned_put_url = aws_sign::r2_presign_put(aws_sign::R2PresignArgs {
        account_id: &bundle_env.account_id,
        bucket: &bundle_env.bucket,
        region: "auto",
        key: &object_key,
        access_key_id: &bundle_env.access_key_id,
        secret_access_key: &bundle_env.secret_access_key,
        expires_seconds: 600,
        now: forte_sdk::now(),
    });

    let static_uploads = if req.body.files.is_empty() {
        Vec::new()
    } else {
        let static_env = match StaticEnv::from_env() {
            Ok(v) => v,
            Err(e) => {
                return Output::Error {
                    message: e.to_string(),
                };
            }
        };
        let static_bucket = format!("fn0-static-asset-{}", project.project_id);
        let now_dt = forte_sdk::now();
        req.body
            .files
            .iter()
            .map(|f| {
                let key = format!("{}/{}", req.body.build_id, f.path);
                let url = aws_sign::r2_presign_put(aws_sign::R2PresignArgs {
                    account_id: &static_env.account_id,
                    bucket: &static_bucket,
                    region: "auto",
                    key: &key,
                    access_key_id: &static_env.access_key_id,
                    secret_access_key: &static_env.secret_access_key,
                    expires_seconds: 600,
                    now: now_dt,
                });
                StaticUpload {
                    path: f.path.clone(),
                    presigned_url: url,
                }
            })
            .collect()
    };

    if let Err(e) = upsert_cron_config(
        &db,
        req.body.project_id.clone(),
        req.body.jobs.clone(),
        req.body.cron_updated_at,
    )
    .await
    {
        return Output::Error { message: e };
    }

    Output::Ok {
        presigned_put_url,
        object_key,
        static_uploads,
    }
}

async fn upsert_cron_config(
    db: &doc_db::Database,
    project_id: String,
    jobs: Vec<CronJob>,
    updated_at: DateTime,
) -> Result<(), String> {
    let result = db
        .trx(|trx| {
            let project_id = project_id.clone();
            let jobs = jobs.clone();
            async move {
                let existing = trx
                    .get(CronConfigDocGet {
                        project_id: project_id.as_str(),
                    })
                    .await?;
                match existing {
                    Some(mut handle) => {
                        if handle.updated_at < updated_at {
                            handle.jobs = jobs;
                            handle.updated_at = updated_at;
                        }
                    }
                    None => {
                        trx.create(CronConfigDoc {
                            project_id,
                            jobs,
                            updated_at,
                        })?;
                    }
                }
                trx.commit::<_, ()>(())
            }
        })
        .await;
    match result {
        doc_db::TrxResult::Committed(()) => Ok(()),
        doc_db::TrxResult::Cancelled(()) => unreachable!(),
        doc_db::TrxResult::Conflict(_) => Err("cron config conflict".to_string()),
        doc_db::TrxResult::Err(e) => Err(e.to_string()),
    }
}

struct BundleEnv {
    account_id: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
}

impl BundleEnv {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            account_id: std::env::var("FN0_BUNDLE_STORE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_ACCOUNT_ID not set"))?,
            bucket: std::env::var("FN0_BUNDLE_STORE_BUCKET")
                .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_BUCKET not set"))?,
            access_key_id: std::env::var("FN0_BUNDLE_STORE_ACCESS_KEY_ID")
                .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_ACCESS_KEY_ID not set"))?,
            secret_access_key: std::env::var("FN0_BUNDLE_STORE_SECRET_ACCESS_KEY")
                .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_SECRET_ACCESS_KEY not set"))?,
        })
    }
}

struct StaticEnv {
    account_id: String,
    access_key_id: String,
    secret_access_key: String,
}

impl StaticEnv {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            account_id: std::env::var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID not set"))?,
            access_key_id: std::env::var("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID").map_err(
                |_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID not set"),
            )?,
            secret_access_key: std::env::var("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY")
                .map_err(|_| {
                    anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY not set")
                })?,
        })
    }
}
