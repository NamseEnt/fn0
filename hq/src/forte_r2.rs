use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::presigning::PresigningConfig;
use color_eyre::eyre::{Result, eyre};

use crate::args::ForteR2Args;

#[derive(Clone)]
pub struct ForteR2 {
    client: S3Client,
    bucket: String,
    public_base_url: String,
}

impl ForteR2 {
    pub async fn new(args: &ForteR2Args) -> Self {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new("auto"))
            .endpoint_url(&args.endpoint)
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                &args.access_key_id,
                &args.secret_access_key,
                None,
                None,
                "hq-r2",
            ))
            .load()
            .await;

        let client = S3Client::from_conf(
            S3ConfigBuilder::from(&config)
                .force_path_style(false)
                .build(),
        );

        Self {
            client,
            bucket: args.bucket.clone(),
            public_base_url: args.public_base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn public_base_url(&self) -> &str {
        &self.public_base_url
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
        let presigning = PresigningConfig::expires_in(expires_in)
            .map_err(|e| eyre!("presigning config failed: {e}"))?;
        let mut req = self.client.put_object().bucket(&self.bucket).key(&key);
        if let Some(ct) = content_type {
            req = req.content_type(ct);
        }
        let presigned = req
            .presigned(presigning)
            .await
            .map_err(|e| eyre!("r2 presign failed for {key}: {e}"))?;
        Ok(presigned.uri().to_string())
    }

    pub async fn delete_prefix(&self, prefix: &str) -> Result<()> {
        let prefix = prefix.trim_start_matches('/').to_string();
        let mut continuation: Option<String> = None;
        loop {
            let mut req = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);
            if let Some(token) = &continuation {
                req = req.continuation_token(token);
            }
            let resp = req
                .send()
                .await
                .map_err(|e| eyre!("r2 list_objects failed for prefix {prefix}: {e}"))?;
            let contents = resp.contents.unwrap_or_default();
            for object in contents {
                if let Some(key) = object.key {
                    self.client
                        .delete_object()
                        .bucket(&self.bucket)
                        .key(&key)
                        .send()
                        .await
                        .map_err(|e| eyre!("r2 delete_object failed for {key}: {e}"))?;
                }
            }
            if resp.is_truncated.unwrap_or(false) {
                continuation = resp.next_continuation_token;
                if continuation.is_none() {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}
