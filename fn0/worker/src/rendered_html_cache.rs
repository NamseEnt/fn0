//! The worker's R2-backed rendered-HTML cache, one bucket per project.

use crate::storage_resolver::ManifestStorageResolver;
use bytes::Bytes;
use fn0::RenderedHtmlCache;
use opendal::ErrorKind;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub struct R2RenderedHtmlCache {
    resolver: Arc<ManifestStorageResolver>,
}

impl R2RenderedHtmlCache {
    pub fn new(resolver: Arc<ManifestStorageResolver>) -> Self {
        Self { resolver }
    }
}

impl RenderedHtmlCache for R2RenderedHtmlCache {
    fn read<'storage>(
        &'storage self,
        project_id: &'storage str,
        key: &'storage str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Bytes>>> + Send + 'storage>> {
        Box::pin(async move {
            let Some(operator) = self.resolver.rendered_html_cache_operator(project_id) else {
                return Ok(None);
            };
            match operator.read(key).await {
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
            let operator = self
                .resolver
                .rendered_html_cache_operator(project_id)
                .ok_or_else(|| {
                    anyhow::anyhow!("no rendered-HTML cache for project {project_id}")
                })?;
            operator.write(key, body).await?;
            Ok(())
        })
    }
}
