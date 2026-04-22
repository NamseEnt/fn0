use super::*;
use async_singleflight::Group;
use aws_sdk_s3::{Client, operation::get_object::GetObjectError};
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct S3AdaptCache<T, E> {
    client: Client,
    bucket: String,
    prefix: Option<String>,
    // front is new, back is old
    cache: Arc<Mutex<VecDeque<CacheEntry<T>>>>,
    cache_size: usize,
    singleflight: Arc<Group<String, T, Error<E>>>,
}

impl<T: Clone + Send + Sync + 'static, E> S3AdaptCache<T, E> {
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

    async fn try_hit_cache(&self, key: &str) -> Option<CacheEntry<T>> {
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

    async fn fetch_and_cache(
        &self,
        key: &str,
        if_none_match: Option<String>,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> Result<T, Error<E>> {
        let (data, etag) = self
            .fetch_from_s3(key, if_none_match)
            .await
            .map_err(Error::StorageError)?;
        let (value, byte_len) = convert(data).map_err(Error::ConvertError)?;

        self.put_to_cache(CacheEntry {
            key: key.to_string(),
            value: value.clone(),
            byte_len,
            etag,
        })
        .await;

        Ok(value)
    }

    async fn get_impl(
        &self,
        key: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> Result<T, Error<E>> {
        let cached = self.try_hit_cache(key).await;

        let if_none_match_param = cached.as_ref().map(|entry| entry.etag.clone());

        let result = self
            .fetch_and_cache(key, if_none_match_param, convert)
            .await;
        if let Ok(value) = result {
            return Ok(value);
        }

        let error = result.err().unwrap();
        match error {
            Error::StorageError(error) => {
                match &error.downcast_ref::<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>>() {
                Some(aws_sdk_s3::error::SdkError::ServiceError(service_err)) => {
                    if service_err.raw().status().as_u16() == 304 {
                        return Ok(cached.unwrap().value);
                    }
                    match service_err.err() {
                        GetObjectError::NoSuchKey(_) => Err(Error::NotFound),
                        _ => Err(Error::StorageError(error)),
                    }
                }
                _ => Err(Error::StorageError(error)),
            }
            }
            _ => Err(error),
        }
    }

    async fn put_to_cache(&self, new_entry: CacheEntry<T>) {
        let mut cache = self.cache.lock().await;

        if let Some(index) = cache.iter().position(|entry| entry.key == new_entry.key) {
            cache.remove(index).expect("unreachable");
        };

        cache.push_front(new_entry);

        let mut cached_bytes = 0;
        for (index, entry) in cache.iter().enumerate() {
            cached_bytes += entry.byte_len;
            if cached_bytes > self.cache_size {
                cache.drain(index..);
                break;
            }
        }
    }
}

impl<T, E> AdaptCache<T, E> for S3AdaptCache<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static + Clone,
{
    async fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> Result<T, Error<E>> {
        let key = self.build_key(id);

        let provider = self.clone();
        self.singleflight
            .work(&key.clone(), async move {
                provider.get_impl(&key, convert).await
            })
            .await
            .map_err(|opt_err| opt_err.unwrap_or(Error::SingleflightLeaderFailed))
    }
}

#[derive(Clone)]
struct CacheEntry<T> {
    key: String,
    value: T,
    byte_len: usize,
    etag: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::operation::get_object::{GetObjectError, GetObjectOutput};
    use aws_sdk_s3::primitives::ByteStream;
    use aws_smithy_mocks::{mock, mock_client};

    #[derive(Debug, Clone)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    fn create_test_string_data(value: &str) -> Vec<u8> {
        value.as_bytes().to_vec()
    }

    fn string_converter(bytes: Bytes) -> Result<(String, usize), TestError> {
        let len = bytes.len();
        String::from_utf8(bytes.to_vec())
            .map(|s| (s, len))
            .map_err(|e| TestError(e.to_string()))
    }

    #[tokio::test]
    async fn test_cache_miss_fetch_from_s3() {
        let data = create_test_string_data("test-content");
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let result = cache.get("test.txt", string_converter).await.unwrap();
        assert_eq!(result, "test-content");
    }

    #[tokio::test]
    async fn test_cache_hit_with_304_response() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let data = create_test_string_data("cached-content");
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
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let val1 = cache.get("test.txt", string_converter).await.unwrap();
        let val2 = cache.get("test.txt", string_converter).await.unwrap();

        assert_eq!(val1, "cached-content");
        assert_eq!(val2, "cached-content");
    }

    #[tokio::test]
    async fn test_not_found_error() {
        let rule = mock!(aws_sdk_s3::Client::get_object).then_error(|| {
            GetObjectError::NoSuchKey(aws_sdk_s3::types::error::NoSuchKey::builder().build())
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let result = cache.get("missing.txt", string_converter).await;
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[tokio::test]
    async fn test_prefix_handling() {
        let data = create_test_string_data("test-content");
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> = S3AdaptCache::new(
            client,
            "test-bucket".to_string(),
            Some("data".to_string()),
            1024,
        );

        let result = cache.get("test.txt", string_converter).await.unwrap();
        assert_eq!(result, "test-content");
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let mut rules = Vec::new();
        for i in 0..5 {
            let data = create_test_string_data(&format!("content-{}", i));
            rules.push(mock!(aws_sdk_s3::Client::get_object).then_output(move || {
                GetObjectOutput::builder()
                    .body(ByteStream::from(data.clone()))
                    .e_tag(format!("etag-{}", i))
                    .build()
            }));
        }

        let client = mock_client!(aws_sdk_s3, &rules);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 30);

        for i in 0..5 {
            cache
                .get(&format!("file{}.txt", i), string_converter)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_cache_update_on_etag_change() {
        let data1 = create_test_string_data("version-1");
        let data2 = create_test_string_data("version-2");
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
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let val1 = cache.get("test.txt", string_converter).await.unwrap();
        assert_eq!(val1, "version-1");

        let val2 = cache.get("test.txt", string_converter).await.unwrap();
        assert_eq!(val2, "version-2");
    }

    #[tokio::test]
    async fn test_concurrent_same_key_requests() {
        use tokio::sync::Barrier;

        let data = create_test_string_data("shared-content");

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
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];
        for _ in 0..10 {
            let cache_clone = cache.clone();
            let barrier_clone = barrier.clone();
            let handle = tokio::spawn(async move {
                barrier_clone.wait().await;
                cache_clone.get("test.txt", string_converter).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            assert_eq!(result, "shared-content");
        }
    }

    #[tokio::test]
    async fn test_cache_hit_then_not_found() {
        let data = create_test_string_data("original-content");
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
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        cache.get("test.txt", string_converter).await.unwrap();

        let result2 = cache.get("test.txt", string_converter).await;
        assert!(matches!(result2, Err(Error::NotFound)));
    }

    #[tokio::test]
    async fn test_s3_error_invalid_object_state() {
        let rule = mock!(aws_sdk_s3::Client::get_object).then_error(|| {
            GetObjectError::InvalidObjectState(
                aws_sdk_s3::types::error::InvalidObjectState::builder().build(),
            )
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let result = cache.get("test.txt", string_converter).await;
        assert!(matches!(result, Err(Error::StorageError(_))));
    }

    #[tokio::test]
    async fn test_network_error_500() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let rule = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(500).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let result = cache.get("test.txt", string_converter).await;
        assert!(matches!(result, Err(Error::StorageError(_))));
    }

    #[tokio::test]
    async fn test_cache_size_zero() {
        let data = create_test_string_data("test-content");
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 0);

        cache.get("test.txt", string_converter).await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_size_smaller_than_file() {
        let data = create_test_string_data("large-content-that-exceeds-cache");
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 5);

        cache.get("test.txt", string_converter).await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_different_keys() {
        let mut rules = Vec::new();
        for i in 0..10 {
            let data = create_test_string_data(&format!("content-{}", i));
            rules.push(mock!(aws_sdk_s3::Client::get_object).then_output(move || {
                GetObjectOutput::builder()
                    .body(ByteStream::from(data.clone()))
                    .e_tag(format!("etag-{}", i))
                    .build()
            }));
        }

        let client = mock_client!(aws_sdk_s3, &rules);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let mut handles = vec![];
        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                cache_clone
                    .get(&format!("file{}.txt", i), string_converter)
                    .await
            });
            handles.push(handle);
        }

        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap().unwrap();
            assert_eq!(result, format!("content-{}", i));
        }
    }

    #[tokio::test]
    async fn test_cache_key_isolation() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let content1 = create_test_string_data("content-1");
        let content2 = create_test_string_data("content-2");

        let content1_clone = content1.clone();
        let content2_clone = content2.clone();

        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(content1_clone.clone()))
                .e_tag("etag-1")
                .build()
        });

        let rule2 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(content2_clone.clone()))
                .e_tag("etag-2")
                .build()
        });

        let rule3 = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(304).unwrap(), SdkBody::empty())
        });

        let rule4 = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(304).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule1, &rule2, &rule3, &rule4]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let val1_first = cache.get("key1.txt", string_converter).await.unwrap();
        let val2_first = cache.get("key2.txt", string_converter).await.unwrap();

        let val1_second = cache.get("key1.txt", string_converter).await.unwrap();
        let val2_second = cache.get("key2.txt", string_converter).await.unwrap();

        assert_eq!(val1_first, "content-1");
        assert_eq!(val1_second, "content-1");
        assert_eq!(val2_first, "content-2");
        assert_eq!(val2_second, "content-2");
        assert_ne!(val1_second, val2_second);
    }

    #[tokio::test]
    async fn test_convert_error() {
        let data = vec![0xFF, 0xFE, 0xFD];
        let data_for_rule = data.clone();
        let rule = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let result = cache.get("invalid.txt", string_converter).await;
        assert!(matches!(result, Err(Error::ConvertError(_))));
    }

    #[tokio::test]
    async fn test_cache_mru_behavior() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let mut rules = Vec::new();
        for i in 0..3 {
            let data = create_test_string_data(&format!("content-{}", i));
            rules.push(mock!(aws_sdk_s3::Client::get_object).then_output(move || {
                GetObjectOutput::builder()
                    .body(ByteStream::from(data.clone()))
                    .e_tag(format!("etag-{}", i))
                    .build()
            }));
        }

        rules.push(mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(304).unwrap(), SdkBody::empty())
        }));

        let client = mock_client!(aws_sdk_s3, &rules);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        cache.get("file0.txt", string_converter).await.unwrap();
        cache.get("file1.txt", string_converter).await.unwrap();
        cache.get("file2.txt", string_converter).await.unwrap();

        let val = cache.get("file0.txt", string_converter).await.unwrap();
        assert_eq!(val, "content-0");
    }

    #[tokio::test]
    async fn test_cache_duplicate_key_update() {
        let data1 = create_test_string_data("version-1");
        let data2 = create_test_string_data("version-2");
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
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        cache.get("test.txt", string_converter).await.unwrap();
        cache.get("test.txt", string_converter).await.unwrap();
    }

    #[tokio::test]
    async fn test_empty_prefix_vs_none() {
        let data = create_test_string_data("test-content");

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
        let cache_none: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client1, "test-bucket".to_string(), None, 1024);

        let client2 = mock_client!(aws_sdk_s3, [&rule2]);
        let cache_empty: S3AdaptCache<String, TestError> = S3AdaptCache::new(
            client2,
            "test-bucket".to_string(),
            Some("".to_string()),
            1024,
        );

        let val1 = cache_none.get("test.txt", string_converter).await.unwrap();
        let val2 = cache_empty.get("test.txt", string_converter).await.unwrap();

        assert_eq!(val1, "test-content");
        assert_eq!(val2, "test-content");
    }

    #[tokio::test]
    async fn test_forbidden_error() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let rule = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(403).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let result = cache.get("test.txt", string_converter).await;
        assert!(matches!(result, Err(Error::StorageError(_))));
    }

    #[tokio::test]
    async fn test_timeout_error_on_cache_miss() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let rule = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(408).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        let result = cache.get("test.txt", string_converter).await;
        assert!(matches!(result, Err(Error::StorageError(_))));
    }

    #[tokio::test]
    async fn test_timeout_error_on_cache_hit() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let data = create_test_string_data("cached-content");
        let data_for_rule = data.clone();
        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let rule2 = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(408).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule1, &rule2]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        cache.get("test.txt", string_converter).await.unwrap();

        let result = cache.get("test.txt", string_converter).await;
        assert!(matches!(result, Err(Error::StorageError(_))));
    }

    #[tokio::test]
    async fn test_service_unavailable_on_cache_hit() {
        use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
        use aws_smithy_runtime_api::http::StatusCode;
        use aws_smithy_types::body::SdkBody;

        let data = create_test_string_data("cached-content");
        let data_for_rule = data.clone();
        let rule1 = mock!(aws_sdk_s3::Client::get_object).then_output(move || {
            GetObjectOutput::builder()
                .body(ByteStream::from(data_for_rule.clone()))
                .e_tag("etag-123")
                .build()
        });

        let rule2 = mock!(Client::get_object).then_http_response(|| {
            HttpResponse::new(StatusCode::try_from(503).unwrap(), SdkBody::empty())
        });

        let client = mock_client!(aws_sdk_s3, [&rule1, &rule2]);
        let cache: S3AdaptCache<String, TestError> =
            S3AdaptCache::new(client, "test-bucket".to_string(), None, 1024);

        cache.get("test.txt", string_converter).await.unwrap();

        let result = cache.get("test.txt", string_converter).await;
        assert!(matches!(result, Err(Error::StorageError(_))));
    }
}
