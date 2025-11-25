use super::*;
use async_singleflight::Group;
use bytes::Bytes;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

pub struct FsAdaptCache<T, E> {
    base_path: PathBuf,
    cache: Arc<Mutex<VecDeque<CacheEntry<T>>>>,
    cache_size: usize,
    singleflight: Arc<Group<String, T, Error<E>>>,
}

impl<T, E> Clone for FsAdaptCache<T, E> {
    fn clone(&self) -> Self {
        Self {
            base_path: self.base_path.clone(),
            cache: self.cache.clone(),
            cache_size: self.cache_size,
            singleflight: self.singleflight.clone(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static, E> FsAdaptCache<T, E> {
    pub fn new(base_path: PathBuf, cache_size: usize) -> Self {
        Self {
            base_path,
            cache: Default::default(),
            cache_size,
            singleflight: Default::default(),
        }
    }

    async fn try_hit_cache(&self, path: &str) -> Option<CacheEntry<T>> {
        let mut cache = self.cache.lock().await;
        let index = cache.iter().position(|entry| entry.key == path)?;
        let entry = cache.remove(index).expect("unreachable");
        cache.push_front(entry.clone());
        Some(entry)
    }

    async fn read_from_fs(&self, path: &str) -> anyhow::Result<(Bytes, SystemTime, u64)> {
        let full_path = self.base_path.join(path);
        let metadata = tokio::fs::metadata(&full_path).await?;
        let mtime = metadata.modified()?;
        let file_size = metadata.len();
        let data = tokio::fs::read(&full_path).await?;
        Ok((Bytes::from(data), mtime, file_size))
    }

    async fn fetch_and_cache(
        &self,
        path: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> Result<T, Error<E>> {
        let (data, mtime, file_size) =
            self.read_from_fs(path).await.map_err(Error::StorageError)?;
        let (value, byte_len) = convert(data).map_err(Error::ConvertError)?;

        self.put_to_cache(CacheEntry {
            key: path.to_string(),
            value: value.clone(),
            byte_len,
            mtime,
            file_size,
        })
        .await;

        Ok(value)
    }

    async fn get_impl(
        &self,
        path: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> Result<T, Error<E>> {
        let cached = self.try_hit_cache(path).await;

        let full_path = self.base_path.join(path);
        let metadata_result = tokio::fs::metadata(&full_path).await;

        match metadata_result {
            Ok(metadata) => {
                let mtime = metadata
                    .modified()
                    .map_err(|e| Error::StorageError(anyhow::anyhow!(e)))?;
                let file_size = metadata.len();

                if let Some(cache_entry) = cached {
                    if cache_entry.mtime == mtime && cache_entry.file_size == file_size {
                        return Ok(cache_entry.value);
                    }
                }

                self.fetch_and_cache(path, convert).await
            }
            Err(error) => {
                if error.kind() == std::io::ErrorKind::NotFound {
                    return Err(Error::NotFound);
                }
                Err(Error::StorageError(anyhow::anyhow!(error)))
            }
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

impl<T, E> AdaptCache<T, E> for FsAdaptCache<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Send + 'static,
{
    async fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> Result<T, Error<E>> {
        let path = id.to_string();

        let provider = self.clone();
        self.singleflight
            .work(&path.clone(), async move {
                provider.get_impl(&path, convert).await
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
    mtime: SystemTime,
    file_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use tokio::time::sleep;

    #[derive(Debug, Clone)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    fn string_converter(bytes: Bytes) -> Result<(String, usize), TestError> {
        let len = bytes.len();
        String::from_utf8(bytes.to_vec())
            .map(|s| (s, len))
            .map_err(|e| TestError(e.to_string()))
    }

    async fn create_test_file(dir: &TempDir, name: &str, content: &str) -> String {
        let path = dir.path().join(name);
        tokio::fs::write(&path, content).await.unwrap();
        name.to_string()
    }

    async fn update_file_content(dir: &TempDir, name: &str, content: &str) {
        sleep(Duration::from_millis(10)).await;
        let path = dir.path().join(name);
        tokio::fs::write(&path, content).await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_miss_read_from_fs() {
        let temp_dir = TempDir::new().unwrap();
        let file_name = create_test_file(&temp_dir, "test.txt", "test-content").await;

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

        let result = cache.get(&file_name, string_converter).await.unwrap();
        assert_eq!(result, "test-content");
    }

    #[tokio::test]
    async fn test_cache_hit_with_same_mtime() {
        let temp_dir = TempDir::new().unwrap();
        let file_name = create_test_file(&temp_dir, "test.txt", "cached-content").await;

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

        let val1 = cache.get(&file_name, string_converter).await.unwrap();
        let val2 = cache.get(&file_name, string_converter).await.unwrap();

        assert_eq!(val1, "cached-content");
        assert_eq!(val2, "cached-content");
    }

    #[tokio::test]
    async fn test_not_found_error() {
        let temp_dir = TempDir::new().unwrap();
        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

        let result = cache.get("missing.txt", string_converter).await;
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let temp_dir = TempDir::new().unwrap();

        for i in 0..5 {
            create_test_file(
                &temp_dir,
                &format!("file{}.txt", i),
                &format!("content-{}", i),
            )
            .await;
        }

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 30);

        for i in 0..5 {
            cache
                .get(&format!("file{}.txt", i), string_converter)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_cache_update_on_mtime_change() {
        let temp_dir = TempDir::new().unwrap();
        let file_name = create_test_file(&temp_dir, "test.txt", "version-1").await;

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

        let val1 = cache.get(&file_name, string_converter).await.unwrap();
        assert_eq!(val1, "version-1");

        update_file_content(&temp_dir, &file_name, "version-2").await;

        let val2 = cache.get(&file_name, string_converter).await.unwrap();
        assert_eq!(val2, "version-2");
    }

    #[tokio::test]
    async fn test_concurrent_same_key_requests() {
        use tokio::sync::Barrier;

        let temp_dir = TempDir::new().unwrap();
        let file_name = create_test_file(&temp_dir, "test.txt", "shared-content").await;

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];
        for _ in 0..10 {
            let cache_clone = cache.clone();
            let barrier_clone = barrier.clone();
            let file_name_clone = file_name.clone();
            let handle = tokio::spawn(async move {
                barrier_clone.wait().await;
                cache_clone.get(&file_name_clone, string_converter).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            assert_eq!(result, "shared-content");
        }
    }

    #[tokio::test]
    async fn test_cache_size_zero() {
        let temp_dir = TempDir::new().unwrap();
        let file_name = create_test_file(&temp_dir, "test.txt", "test-content").await;

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 0);

        cache.get(&file_name, string_converter).await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_different_keys() {
        let temp_dir = TempDir::new().unwrap();

        for i in 0..10 {
            create_test_file(
                &temp_dir,
                &format!("file{}.txt", i),
                &format!("content-{}", i),
            )
            .await;
        }

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

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
    async fn test_convert_error() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.txt");
        tokio::fs::write(&path, vec![0xFF, 0xFE, 0xFD])
            .await
            .unwrap();

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

        let result = cache.get("invalid.txt", string_converter).await;
        assert!(matches!(result, Err(Error::ConvertError(_))));
    }

    #[tokio::test]
    async fn test_cache_mru_behavior() {
        let temp_dir = TempDir::new().unwrap();

        for i in 0..3 {
            create_test_file(
                &temp_dir,
                &format!("file{}.txt", i),
                &format!("content-{}", i),
            )
            .await;
        }

        let cache: FsAdaptCache<String, TestError> =
            FsAdaptCache::new(temp_dir.path().to_path_buf(), 1024);

        cache.get("file0.txt", string_converter).await.unwrap();
        cache.get("file1.txt", string_converter).await.unwrap();
        cache.get("file2.txt", string_converter).await.unwrap();

        let val = cache.get("file0.txt", string_converter).await.unwrap();
        assert_eq!(val, "content-0");
    }
}
