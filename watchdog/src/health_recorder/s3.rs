use aws_config::BehaviorVersion;
use aws_sdk_s3::primitives::ByteStream;
use std::collections::BTreeMap;

use super::*;

pub struct S3HealthRecorder {
    client: aws_sdk_s3::Client,
    bucket_name: String,
    object_key: String,
}

impl S3HealthRecorder {
    pub async fn new() -> Self {
        let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        Self {
            client: aws_sdk_s3::Client::new(&sdk_config),
            bucket_name: std::env::var("HEALTH_RECORD_BUCKET_NAME")
                .expect("env var HEALTH_RECORD_BUCKET_NAME is not set"),
            object_key: "health_records.json".to_string(),
        }
    }
}

impl HealthRecorder for S3HealthRecorder {
    fn read_all<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<HealthRecords>> + 'a + Send>> {
        Box::pin(async move {
            let result = self
                .client
                .get_object()
                .bucket(&self.bucket_name)
                .key(&self.object_key)
                .send()
                .await;

            match result {
                Ok(response) => {
                    let bytes = response.body.collect().await?.into_bytes();
                    let records: HealthRecords = serde_json::from_slice(&bytes)?;
                    Ok(records)
                }
                Err(err) => {
                    // If the object doesn't exist, return an empty map
                    if err
                        .as_service_error()
                        .map(|e| e.is_no_such_key())
                        .unwrap_or(false)
                    {
                        return Ok(BTreeMap::new());
                    }
                    Err(err.into())
                }
            }
        })
    }

    fn write_all<'a>(
        &'a self,
        records: HealthRecords,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a + Send>> {
        Box::pin(async move {
            let json_data = serde_json::to_string(&records)?;

            self.client
                .put_object()
                .bucket(&self.bucket_name)
                .key(&self.object_key)
                .body(ByteStream::from(json_data.into_bytes()))
                .content_type("application/json")
                .send()
                .await?;

            Ok(())
        })
    }
}
