//! S3-API access to the R2 stores control manages: the bundle store
//! (`original/`/`compiled/` deploy artifacts, on fn0's own account) and the
//! project's own buckets in its owner's account. Used by `bundle_gc`
//! (superseded artifact GC) and `project_teardown` (full project deletion).

use crate::common::aws_sign;
use forte_sdk::*;

const R2_REGION: &str = "auto";

async fn r2_list_all(
    account_id: &str,
    bucket: &str,
    access_key_id: &str,
    secret_access_key: &str,
    prefix: &str,
    now: DateTime,
) -> anyhow::Result<Vec<aws_sign::R2ListedObject>> {
    let mut objects = Vec::new();
    let mut continuation_token: Option<String> = None;
    loop {
        let page = aws_sign::r2_list_objects(aws_sign::R2ListArgs {
            account_id,
            bucket,
            region: R2_REGION,
            prefix,
            continuation_token: continuation_token.as_deref(),
            access_key_id,
            secret_access_key,
            now,
        })
        .await?;
        objects.extend(page.objects);
        match page.next_continuation_token {
            Some(token) => continuation_token = Some(token),
            None => break,
        }
    }
    Ok(objects)
}

async fn r2_delete(
    account_id: &str,
    bucket: &str,
    access_key_id: &str,
    secret_access_key: &str,
    key: &str,
    now: DateTime,
) -> anyhow::Result<()> {
    aws_sign::r2_delete_object(aws_sign::R2ObjectRef {
        account_id,
        bucket,
        region: R2_REGION,
        key,
        access_key_id,
        secret_access_key,
        now,
    })
    .await
}

/// One project's view of one of its buckets, in its owner's Cloudflare account.
/// The bundle store is fn0's own and keeps its own type.
///
/// Which credential a constructor picks is the access boundary, not a detail:
/// [`Self::frontend_assets`] cannot open a bucket holding user data, and the
/// other three cannot open the frontend-asset bucket.
pub struct ProjectR2Store {
    account_id: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
}

impl ProjectR2Store {
    fn with_worker_keys(storage: &crate::common::byoc::ProjectStorage, bucket: String) -> Self {
        Self {
            account_id: storage.account_id.clone(),
            bucket,
            access_key_id: storage.worker_keys.access_key_id.clone(),
            secret_access_key: storage.worker_keys.secret_access_key.clone(),
        }
    }

    pub fn private_objects(storage: &crate::common::byoc::ProjectStorage) -> Self {
        Self::with_worker_keys(storage, storage.private_object_storage_bucket.clone())
    }

    pub fn public_objects(storage: &crate::common::byoc::ProjectStorage) -> Self {
        Self::with_worker_keys(storage, storage.public_object_storage_bucket.clone())
    }

    pub fn rendered_html_cache(storage: &crate::common::byoc::ProjectStorage) -> Self {
        Self::with_worker_keys(storage, storage.rendered_html_cache_bucket.clone())
    }

    pub fn frontend_assets(storage: &crate::common::byoc::ProjectStorage) -> Self {
        Self {
            account_id: storage.account_id.clone(),
            bucket: storage.frontend_asset_bucket.clone(),
            access_key_id: storage.frontend_asset_keys.access_key_id.clone(),
            secret_access_key: storage.frontend_asset_keys.secret_access_key.clone(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub async fn list_all(
        &self,
        prefix: &str,
        now: DateTime,
    ) -> anyhow::Result<Vec<aws_sign::R2ListedObject>> {
        r2_list_all(
            &self.account_id,
            &self.bucket,
            &self.access_key_id,
            &self.secret_access_key,
            prefix,
            now,
        )
        .await
    }

    pub async fn delete(&self, key: &str, now: DateTime) -> anyhow::Result<()> {
        r2_delete(
            &self.account_id,
            &self.bucket,
            &self.access_key_id,
            &self.secret_access_key,
            key,
            now,
        )
        .await
    }
}

pub struct BundleStore {
    account_id: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
}

impl BundleStore {
    pub fn from_env() -> anyhow::Result<Self> {
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

    pub async fn delete(&self, key: &str, now: DateTime) -> anyhow::Result<()> {
        r2_delete(
            &self.account_id,
            &self.bucket,
            &self.access_key_id,
            &self.secret_access_key,
            key,
            now,
        )
        .await
    }

    pub async fn list_all(
        &self,
        prefix: &str,
        now: DateTime,
    ) -> anyhow::Result<Vec<aws_sign::R2ListedObject>> {
        r2_list_all(
            &self.account_id,
            &self.bucket,
            &self.access_key_id,
            &self.secret_access_key,
            prefix,
            now,
        )
        .await
    }
}

pub fn parse_compiled_key(key: &str) -> Option<(String, u64)> {
    let rest = key.strip_prefix("compiled/")?.strip_suffix(".tar.zst")?;
    let mut segments = rest.rsplitn(3, '/');
    let code_version: u64 = segments.next()?.parse().ok()?;
    let project_id = segments.next()?.to_string();
    segments.next()?;
    Some((project_id, code_version))
}
