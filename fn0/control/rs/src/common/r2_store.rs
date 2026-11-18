//! S3-API access to the R2 stores control manages: the bundle store
//! (`original/`/`compiled/` deploy artifacts, on fn0's own account) and the
//! project's own buckets in its owner's account. Used by `bundle_gc`
//! (superseded artifact GC) and `project_teardown` (full project deletion).

use crate::common::{
    aws_sign,
    byoc::{ProjectStorage, R2KeySource, R2Keys},
};
use forte_sdk::*;

const R2_REGION: &str = "auto";

async fn r2_list_all(
    account_id: &str,
    bucket: &str,
    source: &R2KeySource,
    prefix: &str,
    now: DateTime,
) -> anyhow::Result<Vec<aws_sign::R2ListedObject>> {
    let mut objects = Vec::new();
    let mut continuation_token: Option<String> = None;
    loop {
        let keys = source.for_bucket(bucket, 0).await?;
        let now = if keys.session_token.is_some() {
            forte_sdk::now()
        } else {
            now
        };
        let page = aws_sign::r2_list_objects(aws_sign::R2ListArgs {
            account_id,
            bucket,
            region: R2_REGION,
            prefix,
            continuation_token: continuation_token.as_deref(),
            access_key_id: &keys.access_key_id,
            secret_access_key: &keys.secret_access_key,
            session_token: keys.session_token.as_deref(),
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
    source: &R2KeySource,
    key: &str,
    now: DateTime,
) -> anyhow::Result<()> {
    let keys = source.for_bucket(bucket, 0).await?;
    let now = if keys.session_token.is_some() {
        forte_sdk::now()
    } else {
        now
    };
    aws_sign::r2_delete_object(aws_sign::R2ObjectRef {
        account_id,
        bucket,
        region: R2_REGION,
        key,
        access_key_id: &keys.access_key_id,
        secret_access_key: &keys.secret_access_key,
        session_token: keys.session_token.as_deref(),
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
    keys: R2KeySource,
}

impl ProjectR2Store {
    fn with_worker_keys(storage: &ProjectStorage, bucket: String) -> Self {
        Self {
            account_id: storage.account_id.clone(),
            bucket,
            keys: storage.worker_keys.clone(),
        }
    }

    pub fn private_objects(storage: &ProjectStorage) -> Self {
        Self::with_worker_keys(storage, storage.private_object_storage_bucket.clone())
    }

    pub fn public_objects(storage: &ProjectStorage) -> Self {
        Self::with_worker_keys(storage, storage.public_object_storage_bucket.clone())
    }

    pub fn frontend_assets(storage: &ProjectStorage) -> Self {
        Self {
            account_id: storage.account_id.clone(),
            bucket: storage.frontend_asset_bucket.clone(),
            keys: storage.frontend_asset_keys.clone(),
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
        r2_list_all(&self.account_id, &self.bucket, &self.keys, prefix, now).await
    }

    pub async fn delete(&self, key: &str, now: DateTime) -> anyhow::Result<()> {
        r2_delete(&self.account_id, &self.bucket, &self.keys, key, now).await
    }
}

pub struct BundleStore {
    account_id: String,
    bucket: String,
    keys: R2KeySource,
}

impl BundleStore {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            account_id: std::env::var("FN0_BUNDLE_STORE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_ACCOUNT_ID not set"))?,
            bucket: std::env::var("FN0_BUNDLE_STORE_BUCKET")
                .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_BUCKET not set"))?,
            keys: R2KeySource::Fixed(R2Keys {
                access_key_id: std::env::var("FN0_BUNDLE_STORE_ACCESS_KEY_ID")
                    .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_ACCESS_KEY_ID not set"))?,
                secret_access_key: std::env::var("FN0_BUNDLE_STORE_SECRET_ACCESS_KEY")
                    .map_err(|_| anyhow::anyhow!("FN0_BUNDLE_STORE_SECRET_ACCESS_KEY not set"))?,
                session_token: None,
            }),
        })
    }

    pub async fn delete(&self, key: &str, now: DateTime) -> anyhow::Result<()> {
        r2_delete(&self.account_id, &self.bucket, &self.keys, key, now).await
    }

    pub async fn list_all(
        &self,
        prefix: &str,
        now: DateTime,
    ) -> anyhow::Result<Vec<aws_sign::R2ListedObject>> {
        r2_list_all(&self.account_id, &self.bucket, &self.keys, prefix, now).await
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
