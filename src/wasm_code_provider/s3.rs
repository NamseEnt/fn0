use super::{Error, Result, WasmCodeProvider};
use anyhow::anyhow;
use async_singleflight::Group;
use aws_sdk_s3::{Client, operation::get_object::GetObjectError};
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct S3CodeProvider {
    client: Client,
    bucket: String,
    prefix: Option<String>,
    // front is new, back is old
    cache: Arc<Mutex<VecDeque<CacheEntry>>>,
    cache_size: usize,
    singleflight: Arc<Group<String, Bytes, Error>>,
}

impl S3CodeProvider {
    pub fn new(client: Client, bucket: String, prefix: Option<String>, cache_size: usize) -> Self {
        Self {
            client,
            bucket,
            prefix,
            cache: Default::default(),
            cache_size,
            singleflight: Default::default(),
        }
    }

    fn build_key(&self, id: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}/{}", prefix, id),
            None => id.to_string(),
        }
    }

    async fn try_hit_cache(&self, key: &str) -> Option<CacheEntry> {
        let mut cache = self.cache.lock().await;
        let index = cache.iter().position(|entry| entry.key == key)?;
        let entry = cache.remove(index).expect("unreachable");
        cache.push_front(entry.clone());
        Some(entry)
    }

    async fn fetch_from_s3(
        &self,
        key: &str,
        if_none_match: Option<String>,
    ) -> anyhow::Result<(Bytes, String)> {
        let mut req = self.client.get_object().bucket(&self.bucket).key(key);

        if let Some(etag) = if_none_match {
            req = req.if_none_match(etag);
        }

        let output = req.send().await?;

        let data = output.body.collect().await?.into_bytes();

        let etag = output.e_tag.expect("S3 should return e_tag");

        Ok((data, etag))
    }

    async fn on_local_cache_hit(&self, cached: CacheEntry) -> Result<Bytes> {
        match self.fetch_from_s3(&cached.key, Some(cached.etag)).await {
            Ok((data, new_etag)) => {
                self.put_to_cache(cached.key, data.clone(), new_etag).await;
                Ok(data)
            }
            Err(sdk_err) => match &sdk_err.downcast_ref::<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>>() {
                Some(aws_sdk_s3::error::SdkError::ServiceError(service_err)) => {
                    if service_err.raw().status().as_u16() == 304 {
                        return Ok(cached.data);
                    }
                    match service_err.err() {
                        GetObjectError::NoSuchKey(_) => Err(Error::NotFound),
                        _ => Err(Error::ProviderError(sdk_err)),
                    }
                }
                _ => Err(Error::ProviderError(sdk_err)),
            },
        }
    }

    async fn on_local_cache_miss(&self, key: &str) -> Result<Bytes> {
        match self.fetch_from_s3(&key, None).await {
            Ok((data, new_etag)) => {
                self.put_to_cache(key.to_string(), data.clone(), new_etag)
                    .await;
                Ok(data)
            }
            Err(sdk_err) => match &sdk_err.downcast_ref::<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>>() {
                Some(aws_sdk_s3::error::SdkError::ServiceError(service_err)) => match service_err.err() {
                    GetObjectError::NoSuchKey(_) => Err(Error::NotFound),
                    _ => Err(Error::ProviderError(sdk_err)),
                },
                _ => Err(Error::ProviderError(sdk_err)),
            },
        }
    }

    async fn put_to_cache(&self, key: String, data: Bytes, etag: String) {
        let mut cache = self.cache.lock().await;

        if let Some(index) = cache.iter().position(|entry| entry.key == key) {
            cache.remove(index).expect("unreachable");
        };

        cache.push_front(CacheEntry { key, data, etag });

        let mut cached_bytes = 0;
        for (index, entry) in cache.iter().enumerate() {
            cached_bytes += entry.data.len();
            if cached_bytes > self.cache_size {
                cache.drain(index..);
                break;
            }
        }
    }
}

impl WasmCodeProvider for S3CodeProvider {
    async fn get_wasm_code(&self, id: &str) -> Result<Bytes> {
        let key = self.build_key(id);

        let provider = self.clone();
        self.singleflight
            .work(&key.clone(), async move {
                if let Some(entry) = provider.try_hit_cache(&key).await {
                    provider.on_local_cache_hit(entry).await
                } else {
                    provider.on_local_cache_miss(&key).await
                }
            })
            .await
            .map_err(|opt_err| {
                opt_err
                    .unwrap_or_else(|| Error::ProviderError(anyhow!("Singleflight leader failed")))
            })
    }
}

#[derive(Clone)]
struct CacheEntry {
    key: String,
    data: Bytes,
    etag: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
    use aws_sdk_s3::primitives::ByteStream;
    use aws_smithy_mocks::{mock, mock_client};

    #[tokio::test]
    async fn test_cache_miss_fetch_from_s3() {
        let data = b"wasm code content".to_vec();
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let result = provider.get_wasm_code("test.wasm").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), data.as_slice());

        let cache = provider.cache.lock().await;
        assert_eq!(cache.len(), 1);
        assert_eq!(cache[0].key, "test.wasm");
        assert_eq!(cache[0].data.as_ref(), data.as_slice());
        assert_eq!(cache[0].etag, "etag-123".to_string());
    }

    #[tokio::test]
    async fn test_cache_hit_with_304_response() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let data = b"wasm code content".to_vec();

        let data1 = data.clone();
        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data1.clone()))
                .e_tag("etag-123")
                .build()
        });

        let rule2 = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(304).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule1, &rule2]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        provider.get_wasm_code("test.wasm").await.unwrap();

        let result = provider.get_wasm_code("test.wasm").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), data.as_slice());
    }

    #[tokio::test]
    async fn test_not_found_error() {
        let rule = mock!(aws_sdk_s3::Client::get_object).then_error(|| {
            GetObjectError::NoSuchKey(aws_sdk_s3::types::error::NoSuchKey::builder().build())
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let result = provider.get_wasm_code("missing.wasm").await;
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let mut rules = Vec::new();
        for i in 0..5 {
            let data = format!("data-{}", i).into_bytes();
            rules.push(mock!(aws_sdk_s3::Client::get_object).then_output(move || {
                GetObjectOutput::builder()
                    .body(ByteStream::from(data.clone()))
                    .e_tag(format!("etag-{}", i))
                    .build()
            }));
        }

        let client = mock_client!(aws_sdk_s3, &rules);
        let cache_size = "data-0".len() + "data-1".len() + "data-2".len();
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, cache_size);

        for i in 0..5 {
            provider
                .get_wasm_code(&format!("file{}.wasm", i))
                .await
                .unwrap();
        }

        let cache = provider.cache.lock().await;
        assert_eq!(cache.len(), 3);
        assert_eq!(cache[0].key, "file4.wasm");
        assert_eq!(cache[1].key, "file3.wasm");
        assert_eq!(cache[2].key, "file2.wasm");
    }

    #[tokio::test]
    async fn test_prefix_handling() {
        let data = b"wasm code".to_vec();
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let provider = S3CodeProvider::new(
            client,
            "test-bucket".to_string(),
            Some("wasm-modules".to_string()),
            1024 * 1024,
        );

        let result = provider.get_wasm_code("test.wasm").await;
        assert!(result.is_ok());

        let cache = provider.cache.lock().await;
        assert_eq!(cache[0].key, "wasm-modules/test.wasm");
    }

    #[tokio::test]
    async fn test_cache_update_on_etag_change() {
        let data1 = b"version 1".to_vec();
        let data2 = b"version 2".to_vec();
        let data1_clone = data1.clone();
        let data2_clone = data2.clone();

        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data1_clone.clone()))
                .e_tag("etag-v1")
                .build()
        });

        let rule2 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data2_clone.clone()))
                .e_tag("etag-v2")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule1, &rule2]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let result1 = provider.get_wasm_code("test.wasm").await.unwrap();
        assert_eq!(result1.as_ref(), data1.as_slice());

        let result2 = provider.get_wasm_code("test.wasm").await.unwrap();
        assert_eq!(result2.as_ref(), data2.as_slice());

        let cache = provider.cache.lock().await;
        assert_eq!(cache.len(), 1);
        assert_eq!(cache[0].data.as_ref(), data2.as_slice());
        assert_eq!(cache[0].etag, "etag-v2".to_string());
    }

    #[tokio::test]
    async fn test_concurrent_same_key_requests() {
        use tokio::sync::Barrier;

        let data = b"wasm code".to_vec();

        let mut rules = Vec::new();
        for _ in 0..10 {
            let data_for_rule = data.clone();
            rules.push(mock!(aws_sdk_s3::Client::get_object).then_output(move || {
                GetObjectOutput::builder()
                    .body(ByteStream::from(data_for_rule.clone()))
                    .e_tag("etag-123")
                    .build()
            }));
        }

        let client = mock_client!(aws_sdk_s3, &rules);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];
        for _ in 0..10 {
            let provider_clone = provider.clone();
            let barrier_clone = barrier.clone();
            let handle = tokio::spawn(async move {
                barrier_clone.wait().await;
                provider_clone.get_wasm_code("test.wasm").await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_ref(), data.as_slice());
        }
    }

    #[tokio::test]
    async fn test_cache_mru_behavior() {
        let mut rules = Vec::new();
        for i in 0..3 {
            let data = format!("data-{}", i).into_bytes();
            rules.push(mock!(aws_sdk_s3::Client::get_object).then_output(move || {
                GetObjectOutput::builder()
                    .body(ByteStream::from(data.clone()))
                    .e_tag(format!("etag-{}", i))
                    .build()
            }));
        }

        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;
        rules.push(mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(304).unwrap(), SdkBody::empty())
        }));

        let client = mock_client!(aws_sdk_s3, &rules);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        provider.get_wasm_code("file0.wasm").await.unwrap();
        provider.get_wasm_code("file1.wasm").await.unwrap();
        provider.get_wasm_code("file2.wasm").await.unwrap();

        {
            let cache = provider.cache.lock().await;
            assert_eq!(cache[0].key, "file2.wasm");
            assert_eq!(cache[1].key, "file1.wasm");
            assert_eq!(cache[2].key, "file0.wasm");
        }

        provider.get_wasm_code("file0.wasm").await.unwrap();

        let cache = provider.cache.lock().await;
        assert_eq!(cache[0].key, "file0.wasm");
        assert_eq!(cache[1].key, "file2.wasm");
        assert_eq!(cache[2].key, "file1.wasm");
    }

    #[tokio::test]
    async fn test_cache_hit_then_not_found() {
        let data = b"wasm code".to_vec();
        let data_for_rule = data.clone();
        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let rule2 = mock!(aws_sdk_s3::Client::get_object).then_error(|| {
            GetObjectError::NoSuchKey(aws_sdk_s3::types::error::NoSuchKey::builder().build())
        });

        let client = mock_client!(aws_sdk_s3, [&rule1, &rule2]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let result1 = provider.get_wasm_code("test.wasm").await;
        assert!(result1.is_ok());

        let result2 = provider.get_wasm_code("test.wasm").await;
        assert!(matches!(result2, Err(Error::NotFound)));
    }

    #[tokio::test]
    async fn test_cache_duplicate_key_update() {
        let data1 = b"version 1".to_vec();
        let data2 = b"version 2".to_vec();
        let data1_clone = data1.clone();
        let data2_clone = data2.clone();

        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data1_clone.clone()))
                .e_tag("etag-v1")
                .build()
        });

        let rule2 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data2_clone.clone()))
                .e_tag("etag-v2")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule1, &rule2]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        provider.get_wasm_code("test.wasm").await.unwrap();
        provider.get_wasm_code("test.wasm").await.unwrap();

        let cache = provider.cache.lock().await;
        assert_eq!(cache.len(), 1);
        assert_eq!(cache[0].key, "test.wasm");
        assert_eq!(cache[0].data.as_ref(), data2.as_slice());
    }

    #[tokio::test]
    async fn test_s3_access_denied_error() {
        let rule = mock!(aws_sdk_s3::Client::get_object).then_error(|| {
            GetObjectError::InvalidObjectState(
                aws_sdk_s3::types::error::InvalidObjectState::builder().build(),
            )
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let result = provider.get_wasm_code("test.wasm").await;
        assert!(matches!(result, Err(Error::ProviderError(_))));
    }

    #[tokio::test]
    async fn test_sdk_network_error() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let rule = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(500).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let result = provider.get_wasm_code("test.wasm").await;
        assert!(matches!(result, Err(Error::ProviderError(_))));
    }

    #[tokio::test]
    async fn test_cache_size_zero() {
        let data = b"wasm code".to_vec();
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 0);

        let result = provider.get_wasm_code("test.wasm").await;
        assert!(result.is_ok());

        let cache = provider.cache.lock().await;
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_cache_size_smaller_than_file() {
        let data = b"large wasm code content".to_vec();
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache_size = 5;
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, cache_size);

        let result = provider.get_wasm_code("test.wasm").await;
        assert!(result.is_ok());

        let cache = provider.cache.lock().await;
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_empty_data() {
        let data = b"".to_vec();
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-empty")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let result = provider.get_wasm_code("empty.wasm").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_empty_prefix_vs_none() {
        let data = b"wasm code".to_vec();

        let data1 = data.clone();
        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data1.clone()))
                .e_tag("etag-1")
                .build()
        });

        let data2 = data.clone();
        let rule2 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data2.clone()))
                .e_tag("etag-2")
                .build()
        });

        let client1 = mock_client!(aws_sdk_s3, [&rule1]);
        let provider_none = S3CodeProvider::new(client1, "test-bucket".to_string(), None, 1024 * 1024);

        let client2 = mock_client!(aws_sdk_s3, [&rule2]);
        let provider_empty =
            S3CodeProvider::new(client2, "test-bucket".to_string(), Some("".to_string()), 1024 * 1024);

        provider_none.get_wasm_code("test.wasm").await.unwrap();
        provider_empty.get_wasm_code("test.wasm").await.unwrap();

        let cache_none = provider_none.cache.lock().await;
        assert_eq!(cache_none[0].key, "test.wasm");

        let cache_empty = provider_empty.cache.lock().await;
        assert_eq!(cache_empty[0].key, "/test.wasm");
    }

    #[tokio::test]
    async fn test_concurrent_different_keys() {
        let mut rules = Vec::new();
        for i in 0..20 {
            let data = format!("data-{}", i).into_bytes();
            rules.push(mock!(aws_sdk_s3::Client::get_object).then_output(move || {
                GetObjectOutput::builder()
                    .body(ByteStream::from(data.clone()))
                    .e_tag(format!("etag-{}", i))
                    .build()
            }));
        }

        let client = mock_client!(aws_sdk_s3, &rules);
        let provider = S3CodeProvider::new(client, "test-bucket".to_string(), None, 1024 * 1024);

        let mut handles = vec![];
        for i in 0..20 {
            let provider_clone = provider.clone();
            let handle = tokio::spawn(async move {
                provider_clone
                    .get_wasm_code(&format!("file{}.wasm", i))
                    .await
            });
            handles.push(handle);
        }

        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap().as_ref(),
                format!("data-{}", i).as_bytes()
            );
        }

        let cache = provider.cache.lock().await;
        assert_eq!(cache.len(), 20);
    }
}
