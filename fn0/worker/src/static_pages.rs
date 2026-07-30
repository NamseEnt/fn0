use crate::storage_resolver::ManifestStorageResolver;
use bytes::Bytes;
use fn0::StaticPageStorage;
use opendal::ErrorKind;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub struct StaticPageStore {
    resolver: Arc<ManifestStorageResolver>,
}

impl StaticPageStore {
    pub fn new(resolver: Arc<ManifestStorageResolver>) -> Self {
        Self { resolver }
    }
}

impl StaticPageStorage for StaticPageStore {
    fn read<'storage>(
        &'storage self,
        project_id: &'storage str,
        key: &'storage str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Bytes>>> + Send + 'storage>> {
        Box::pin(async move {
            match self.resolver.page_operator(project_id).read(key).await {
                Ok(buffer) => Ok(Some(buffer.to_bytes())),
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error.into()),
            }
        })
    }

    fn write<'storage>(
        &'storage self,
        project_id: &'storage str,
        key: &'storage str,
        body: Bytes,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'storage>> {
        Box::pin(async move {
            self.resolver
                .page_operator(project_id)
                .write(key, body)
                .await?;
            Ok(())
        })
    }
}
