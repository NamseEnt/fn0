use crate::bundle_store::BundleStore;
use adapt_cache::{AdaptCache, Error};
use bytes::Bytes;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BundleCache<T, E> {
    store: Arc<BundleStore>,
    converted: Arc<Mutex<HashMap<String, T>>>,
    _phantom: PhantomData<fn() -> E>,
}

impl<T, E> Clone for BundleCache<T, E> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            converted: self.converted.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T, E> BundleCache<T, E> {
    pub fn new(store: Arc<BundleStore>) -> Self {
        Self {
            store,
            converted: Arc::new(Mutex::new(HashMap::new())),
            _phantom: PhantomData,
        }
    }

    pub async fn invalidate(&self, artifact_id: &str) {
        self.converted.lock().await.remove(artifact_id);
    }
}

impl<T, E> AdaptCache<T, E> for BundleCache<T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Send + 'static,
{
    async fn get(
        &self,
        id: &str,
        convert: impl FnOnce(Bytes) -> std::result::Result<(T, usize), E> + Send,
    ) -> std::result::Result<T, Error<E>> {
        let mut guard = self.converted.lock().await;
        if let Some(v) = guard.get(id) {
            return Ok(v.clone());
        }
        let bytes = self.store.get(id).ok_or(Error::NotFound)?;
        let (value, _size) = convert(bytes).map_err(Error::ConvertError)?;
        guard.insert(id.to_string(), value.clone());
        Ok(value)
    }
}
