use anyhow::{Context, Result};
use bytes::Bytes;
use fn0::StaticPageStorage;
use opendal::{ErrorKind, Operator};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct StaticPageStore {
    operator: Operator,
}

impl StaticPageStore {
    pub fn from_env() -> Result<Self> {
        let account_id = std::env::var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")
            .context("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID is required")?;
        let bucket = std::env::var("FN0_STATIC_ASSET_STORAGE_BUCKET")
            .context("FN0_STATIC_ASSET_STORAGE_BUCKET is required")?;
        let access_key_id = std::env::var("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID")
            .context("FN0_STATIC_ASSET_STORAGE_ACCESS_KEY_ID is required")?;
        let secret_access_key = std::env::var("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY")
            .context("FN0_STATIC_ASSET_STORAGE_SECRET_ACCESS_KEY is required")?;
        let endpoint = std::env::var("FN0_STATIC_ASSET_STORAGE_ENDPOINT")
            .unwrap_or_else(|_| format!("https://{account_id}.r2.cloudflarestorage.com"));

        let operator = Operator::new(
            opendal::services::S3::default()
                .bucket(&bucket)
                .region("auto")
                .endpoint(&endpoint)
                .access_key_id(&access_key_id)
                .secret_access_key(&secret_access_key)
                .disable_config_load()
                .disable_ec2_metadata(),
        )?
        .finish();

        Ok(Self { operator })
    }
}

impl StaticPageStorage for StaticPageStore {
    fn read<'storage>(
        &'storage self,
        key: &'storage str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Bytes>>> + Send + 'storage>> {
        Box::pin(async move {
            match self.operator.read(key).await {
                Ok(buffer) => Ok(Some(buffer.to_bytes())),
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error.into()),
            }
        })
    }

    fn write<'storage>(
        &'storage self,
        key: &'storage str,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'storage>> {
        Box::pin(async move {
            self.operator.write(key, body).await?;
            Ok(())
        })
    }
}
