//! Copies a project's existing objects from the platform's Cloudflare account
//! into the account the owner just connected.
//!
//! Without this, connecting an account would silently strand every object the
//! project has already written: the worker starts signing against the new
//! account immediately, so a `get` of an existing key would 404 against an
//! empty bucket.
//!
//! The queue is at-least-once, so every step is a copy that overwrites, never a
//! move. Source objects are deliberately left in place: reclaiming them is a
//! separate decision from making the new account work, and a redelivery that
//! found them already deleted could not finish the copy.

use crate::common::aws_sign;
use crate::common::byoc::{self, ProjectStorage};
use crate::common::r2_store::{ObjectStorageStore, StaticAssetStore, StaticPageStore};
use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const R2_REGION: &str = "auto";

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub project_id: String,
}

pub async fn handle(input: Input) -> anyhow::Result<()> {
    let db = doc_db::turso();
    let now = forte_sdk::now();
    let Some(storage) = ProjectStorage::resolve_connected(&db, &input.project_id).await? else {
        return Ok(());
    };

    let object_copied = copy_object_storage(&input.project_id, &storage, now).await?;
    let asset_copied = copy_static_assets(&input.project_id, &storage, now).await?;
    let page_copied = copy_static_pages(&input.project_id, &storage, now).await?;

    // Only now is the new account a complete copy, so only now may workers be
    // pointed at it. Marking the config ready first would serve 404s for every
    // object written before the account was connected.
    let Some(config) = (ProjectCloudflareConfigDocGet {
        project_id: input.project_id.as_str(),
    })
    .send_with(&db)
    .await?
    else {
        return Ok(());
    };
    let ready = ProjectCloudflareConfigDoc {
        state: CloudflareConnectionState::Ok,
        checked_at: now,
        ..config
    };
    (ProjectCloudflareConfigDocPut(ready.clone()))
        .send_with(&db)
        .await?;
    byoc::publish_storage_to_manifest(&db, &input.project_id, Some(&ready)).await?;

    tracing::info!(
        project_id = %input.project_id,
        object_copied,
        asset_copied,
        page_copied,
        "byoc migration complete; project switched to its own account"
    );
    Ok(())
}

async fn copy_object_storage(
    project_id: &str,
    storage: &ProjectStorage,
    now: DateTime,
) -> anyhow::Result<u64> {
    let source = ObjectStorageStore::from_env(project_id)?;
    // The platform bucket for a project that never used object storage does
    // not exist, and ListObjectsV2 errors on a missing bucket rather than
    // returning an empty page.
    if !crate::common::cloudflare::CloudflareClient::from_env()?
        .r2_bucket_exists(source.bucket())
        .await?
    {
        return Ok(0);
    }
    let source_account_id = std::env::var("FN0_OBJECT_STORAGE_ACCOUNT_ID")
        .map_err(|_| anyhow::anyhow!("FN0_OBJECT_STORAGE_ACCOUNT_ID not set"))?;
    let source_keys = platform_keys(
        "FN0_OBJECT_STORAGE_ACCESS_KEY_ID",
        "FN0_OBJECT_STORAGE_SECRET_ACCESS_KEY",
    )?;

    let mut copied = 0;
    for object in source.list_all("", now).await? {
        copy_object(
            CopyArgs {
                source_account_id: &source_account_id,
                source_bucket: source.bucket(),
                source_access_key_id: &source_keys.0,
                source_secret_access_key: &source_keys.1,
                target_account_id: &storage.account_id,
                target_bucket: &storage.object_bucket,
                target_access_key_id: &storage.object_keys.access_key_id,
                target_secret_access_key: &storage.object_keys.secret_access_key,
                key: &object.key,
            },
            now,
        )
        .await?;
        copied += 1;
    }
    Ok(copied)
}

async fn copy_static_assets(
    project_id: &str,
    storage: &ProjectStorage,
    now: DateTime,
) -> anyhow::Result<u64> {
    let source = StaticAssetStore::from_env()?;
    let source_account_id = std::env::var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")
        .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID not set"))?;
    let source_keys = platform_keys(
        "FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID",
        "FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY",
    )?;

    let mut copied = 0;
    for object in source.list_all(&format!("{project_id}/"), now).await? {
        copy_object(
            CopyArgs {
                source_account_id: &source_account_id,
                source_bucket: source.bucket(),
                source_access_key_id: &source_keys.0,
                source_secret_access_key: &source_keys.1,
                target_account_id: &storage.account_id,
                target_bucket: &storage.asset_bucket,
                target_access_key_id: &storage.asset_keys.access_key_id,
                target_secret_access_key: &storage.asset_keys.secret_access_key,
                key: &object.key,
            },
            now,
        )
        .await?;
        copied += 1;
    }
    Ok(copied)
}

/// Cached pages are regenerated on demand, so they are not copied: a miss in
/// the new account costs one SSR render, and copying them would carry a
/// `code_version` that the next deploy invalidates anyway.
async fn copy_static_pages(
    project_id: &str,
    _storage: &ProjectStorage,
    now: DateTime,
) -> anyhow::Result<u64> {
    let source = StaticPageStore::from_env()?;
    let stale = source.list_all(&format!("{project_id}/"), now).await?.len();
    tracing::info!(%project_id, stale, "byoc migration: cached pages left to regenerate");
    Ok(0)
}

fn platform_keys(key_id_var: &str, secret_var: &str) -> anyhow::Result<(String, String)> {
    Ok((
        std::env::var(key_id_var).map_err(|_| anyhow::anyhow!("{key_id_var} not set"))?,
        std::env::var(secret_var).map_err(|_| anyhow::anyhow!("{secret_var} not set"))?,
    ))
}

struct CopyArgs<'a> {
    source_account_id: &'a str,
    source_bucket: &'a str,
    source_access_key_id: &'a str,
    source_secret_access_key: &'a str,
    target_account_id: &'a str,
    target_bucket: &'a str,
    target_access_key_id: &'a str,
    target_secret_access_key: &'a str,
    key: &'a str,
}

async fn copy_object(args: CopyArgs<'_>, now: DateTime) -> anyhow::Result<()> {
    let object = aws_sign::r2_get_object(aws_sign::R2ObjectRef {
        account_id: args.source_account_id,
        bucket: args.source_bucket,
        region: R2_REGION,
        key: args.key,
        access_key_id: args.source_access_key_id,
        secret_access_key: args.source_secret_access_key,
        now,
    })
    .await?;

    aws_sign::r2_put_object(
        aws_sign::R2ObjectRef {
            account_id: args.target_account_id,
            bucket: args.target_bucket,
            region: R2_REGION,
            key: args.key,
            access_key_id: args.target_access_key_id,
            secret_access_key: args.target_secret_access_key,
            now,
        },
        object.body,
        object.content_type.as_deref(),
        object.cache_control.as_deref(),
    )
    .await
}
