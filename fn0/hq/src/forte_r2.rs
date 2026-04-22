use color_eyre::eyre::{Result, eyre};
use futures::TryStreamExt;
use opendal::Operator;
use opendal::services::S3;

use crate::args::ForteR2Args;

#[derive(Clone)]
pub struct ForteR2 {
    operator: Operator,
    public_base_url: String,
}

impl ForteR2 {
    pub async fn new(args: &ForteR2Args) -> Result<Self> {
        let operator = Operator::new(
            S3::default()
                .bucket(&args.bucket)
                .region("auto")
                .endpoint(&args.endpoint)
                .access_key_id(&args.access_key_id)
                .secret_access_key(&args.secret_access_key)
                .enable_virtual_host_style()
                .disable_config_load()
                .disable_ec2_metadata(),
        )?
        .finish();

        Ok(Self {
            operator,
            public_base_url: args.public_base_url.trim_end_matches('/').to_string(),
        })
    }

    pub fn static_base_url(&self, subdomain: &str, build_id: &str) -> String {
        format!("{}/{subdomain}/{build_id}/", self.public_base_url)
    }

    pub async fn presign_put(
        &self,
        subdomain: &str,
        build_id: &str,
        path: &str,
        content_type: Option<&str>,
        expires_in: std::time::Duration,
    ) -> Result<String> {
        let key = format!(
            "{}/{}/{}",
            subdomain,
            build_id,
            path.trim_start_matches('/')
        );
        let builder = self.operator.presign_write_with(&key, expires_in);
        let presigned = match content_type {
            Some(ct) => builder.content_type(ct).await,
            None => builder.await,
        }
        .map_err(|e| eyre!("r2 presign failed for {key}: {e}"))?;
        Ok(presigned.uri().to_string())
    }

    pub async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let prefix = prefix.trim_start_matches('/').to_string();
        let mut lister = self
            .operator
            .lister_with(&prefix)
            .recursive(true)
            .await
            .map_err(|e| eyre!("r2 list_objects failed for prefix {prefix}: {e}"))?;
        while let Some(entry) = lister
            .try_next()
            .await
            .map_err(|e| eyre!("r2 list_objects iter failed for prefix {prefix}: {e}"))?
        {
            if entry.metadata().is_file() {
                self.operator
                    .delete(entry.path())
                    .await
                    .map_err(|e| eyre!("r2 delete_object failed for {}: {e}", entry.path()))?;
            }
        }
        Ok(())
    }
}
