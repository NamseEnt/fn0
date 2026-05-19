mod cloudflare;

use cloudflare::CloudflareClient;
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
        code_version: u64,
    },
    QuotaExceeded {
        reason: String,
    },
    NotLoggedIn,
    NotFound,
    InternalError,
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
            tracing::error!("deploy ProjectDocGet: {e}");
            return Output::InternalError;
        }
    };

    if project.owner_github_id != user.github_id {
        return Output::NotFound;
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

    if let Err(e) = ensure_all_resources(&req.body.project_id).await {
        tracing::error!("deploy ensure_all_resources: {e}");
        return Output::InternalError;
    }

    let bundle_env = match BundleEnv::from_env() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("deploy BundleEnv::from_env: {e}");
            return Output::InternalError;
        }
    };
    let code_version: u64 = u64::try_from(forte_sdk::now().timestamp())
        .expect("system clock returns positive timestamp");
    let object_key = format!("original/{}/{}.tar", project.project_id, code_version);
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
                tracing::error!("deploy StaticEnv::from_env: {e}");
                return Output::InternalError;
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
        tracing::error!("deploy upsert_cron_config: {e}");
        return Output::InternalError;
    }

    Output::Ok {
        presigned_put_url,
        object_key,
        static_uploads,
        code_version,
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

async fn ensure_all_resources(project_id: &str) -> anyhow::Result<()> {
    let cf = CloudflareClient::from_env()?;
    let public_base_domain = std::env::var("FN0_STATIC_ASSET_STORAGE_PUBLIC_BASE_DOMAIN")
        .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_PUBLIC_BASE_DOMAIN not set"))?;
    let zone_id = std::env::var("FN0_CLOUDFLARE_ZONE_ID")
        .map_err(|_| anyhow::anyhow!("FN0_CLOUDFLARE_ZONE_ID not set"))?;

    let bucket = format!("fn0-static-asset-{project_id}");
    let custom_domain = format!("{project_id}.{public_base_domain}");

    cf.create_r2_bucket(&bucket, "apac").await?;
    cf.put_r2_bucket_cors(&bucket, "*").await?;
    cf.register_r2_custom_domain(&bucket, &custom_domain, &zone_id)
        .await?;
    ensure_turso_database(project_id).await?;
    Ok(())
}

async fn ensure_turso_database(project_id: &str) -> anyhow::Result<()> {
    let api_token = std::env::var("FN0_TURSO_API_TOKEN")
        .map_err(|_| anyhow::anyhow!("FN0_TURSO_API_TOKEN not set"))?;
    let org_slug = std::env::var("FN0_TURSO_ORG_SLUG")
        .map_err(|_| anyhow::anyhow!("FN0_TURSO_ORG_SLUG not set"))?;
    let group_name = std::env::var("FN0_TURSO_GROUP_NAME")
        .map_err(|_| anyhow::anyhow!("FN0_TURSO_GROUP_NAME not set"))?;

    let url = format!("https://api.turso.tech/v1/organizations/{org_slug}/databases");
    let body = serde_json::to_vec(&serde_json::json!({
        "name": project_id,
        "group": group_name,
    }))?;
    let req = http::Request::builder()
        .uri(url)
        .method("POST")
        .header("Authorization", format!("Bearer {api_token}"))
        .header("Content-Type", "application/json")
        .body(body)?;
    let resp = http::Client::new().send(req).await?;
    let status = resp.status().as_u16();
    if (200..300).contains(&status) || status == 409 {
        return Ok(());
    }
    let body_bytes = resp.into_body().bytes().await.to_vec();
    anyhow::bail!(
        "turso ensure_database {project_id} failed (status={status}): {}",
        String::from_utf8_lossy(&body_bytes)
    );
}

