//! S3-API access to the three R2 stores control manages: the bundle store
//! (`original/`/`compiled/` deploy artifacts), the shared static asset bucket
//! (`{project_id}/{code_version}/` prefixes), and the per-project
//! `fn0-object-storage-{project_id}` buckets. Used by `bundle_gc` (superseded
//! artifact GC) and `project_teardown` (full project deletion).

use crate::common::aws_sign;
use forte_sdk::*;

const R2_REGION: &str = "auto";

pub const OBJECT_STORAGE_BUCKET_PREFIX: &str = "fn0-object-storage-";

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

/// One project's view of one of its buckets, in whichever Cloudflare account
/// the project belongs to. Replaces the three env-configured stores for
/// everything that is per-project; the bundle store stays on the platform
/// account and keeps its own type.
pub struct ProjectR2Store {
    account_id: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
}

impl ProjectR2Store {
    pub fn objects(storage: &crate::common::byoc::ProjectStorage) -> Self {
        Self {
            account_id: storage.account_id.clone(),
            bucket: storage.object_bucket.clone(),
            access_key_id: storage.object_keys.access_key_id.clone(),
            secret_access_key: storage.object_keys.secret_access_key.clone(),
        }
    }

    pub fn assets(storage: &crate::common::byoc::ProjectStorage) -> Self {
        Self {
            account_id: storage.account_id.clone(),
            bucket: storage.asset_bucket.clone(),
            access_key_id: storage.asset_keys.access_key_id.clone(),
            secret_access_key: storage.asset_keys.secret_access_key.clone(),
        }
    }

    pub fn pages(storage: &crate::common::byoc::ProjectStorage) -> Self {
        Self {
            account_id: storage.account_id.clone(),
            bucket: storage.page_bucket.clone(),
            access_key_id: storage.page_keys.access_key_id.clone(),
            secret_access_key: storage.page_keys.secret_access_key.clone(),
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

pub struct StaticAssetStore {
    account_id: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
}

impl StaticAssetStore {
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            account_id: std::env::var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID not set"))?,
            bucket: std::env::var("FN0_STATIC_ASSET_STORAGE_BUCKET")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_BUCKET not set"))?,
            access_key_id: std::env::var("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID not set"))?,
            secret_access_key: std::env::var("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY")
                .map_err(|_| {
                    anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY not set")
                })?,
        })
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

// Lazily generated page HTML, written by the worker. A different bucket from
// `StaticAssetStore`: that one is public through static.fn0.dev, this one is
// private and reachable only through fn0.
pub struct StaticPageStore {
    account_id: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
}

impl StaticPageStore {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            account_id: std::env::var("FN0_STATIC_PAGE_STORAGE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_PAGE_STORAGE_ACCOUNT_ID not set"))?,
            bucket: std::env::var("FN0_STATIC_PAGE_STORAGE_BUCKET")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_PAGE_STORAGE_BUCKET not set"))?,
            access_key_id: std::env::var("FN0_STATIC_PAGE_STORAGE_ACCESS_KEY_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_PAGE_STORAGE_ACCESS_KEY_ID not set"))?,
            secret_access_key: std::env::var("FN0_STATIC_PAGE_STORAGE_SECRET_ACCESS_KEY").map_err(
                |_| anyhow::anyhow!("FN0_STATIC_PAGE_STORAGE_SECRET_ACCESS_KEY not set"),
            )?,
        })
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

pub struct ObjectStorageStore {
    account_id: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
}

impl ObjectStorageStore {
    pub fn from_env(project_id: &str) -> anyhow::Result<Self> {
        Ok(Self {
            account_id: std::env::var("FN0_OBJECT_STORAGE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_OBJECT_STORAGE_ACCOUNT_ID not set"))?,
            bucket: format!("{OBJECT_STORAGE_BUCKET_PREFIX}{project_id}"),
            access_key_id: std::env::var("FN0_OBJECT_STORAGE_ACCESS_KEY_ID")
                .map_err(|_| anyhow::anyhow!("FN0_OBJECT_STORAGE_ACCESS_KEY_ID not set"))?,
            secret_access_key: std::env::var("FN0_OBJECT_STORAGE_SECRET_ACCESS_KEY")
                .map_err(|_| anyhow::anyhow!("FN0_OBJECT_STORAGE_SECRET_ACCESS_KEY not set"))?,
        })
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

pub fn parse_compiled_key(key: &str) -> Option<(String, u64)> {
    let rest = key.strip_prefix("compiled/")?.strip_suffix(".tar.zst")?;
    let mut segments = rest.rsplitn(3, '/');
    let code_version: u64 = segments.next()?.parse().ok()?;
    let project_id = segments.next()?.to_string();
    segments.next()?;
    Some((project_id, code_version))
}
