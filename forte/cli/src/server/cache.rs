use adapt_cache::AdaptCache;
use anyhow::Result;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SimpleCache {
    memory: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    backend_path: String,
    frontend_path: String,
}

impl SimpleCache {
    pub fn new(backend_path: String, frontend_path: String) -> Self {
        Self {
            memory: Arc::new(Mutex::new(HashMap::new())),
            backend_path,
            frontend_path,
        }
    }

    pub async fn invalidate(&self, id: &str) {
        let mut cache = self.memory.lock().await;
        cache.remove(id);
    }

    #[allow(dead_code)]
    pub async fn invalidate_all(&self) {
        let mut cache = self.memory.lock().await;
        cache.clear();
    }

    async fn load_file(&self, path: &str) -> Result<Vec<u8>> {
        tokio::fs::read(path).await.map_err(|e| anyhow::anyhow!(e))
    }
}

impl<T: Clone + Send + Sync + 'static, E: Send + 'static> AdaptCache<T, E> for SimpleCache {
    async fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> std::result::Result<T, adapt_cache::Error<E>> {
        let mut cache = self.memory.lock().await;

        let bytes = if let Some(data) = cache.get(id) {
            Bytes::copy_from_slice(data)
        } else {
            let (path, is_backend) = match id {
                "app::backend" => (&self.backend_path, true),
                "app::frontend" => (&self.frontend_path, false),
                _ => return Err(adapt_cache::Error::NotFound),
            };

            let mut data = match self.load_file(path).await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to read '{path}': {e}");
                    return Err(adapt_cache::Error::StorageError(anyhow::anyhow!(e)));
                }
            };

            if is_backend {
                match fn0::compile(&data) {
                    Ok(cwasm) => {
                        data = cwasm;
                    }
                    Err(e) => {
                        eprintln!("[wasm] Compilation failed: {:?}", e);
                        return Err(adapt_cache::Error::StorageError(e));
                    }
                }
            }

            cache.insert(id.to_string(), data.clone());
            Bytes::from(data)
        };

        let (converted, _) = convert(bytes).map_err(adapt_cache::Error::ConvertError)?;
        Ok(converted)
    }
}
