use bytes::Bytes;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct BundleStore {
    inner: RwLock<HashMap<String, Bytes>>,
}

impl BundleStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, artifact_id: &str, bytes: Bytes) {
        self.inner
            .write()
            .unwrap()
            .insert(artifact_id.to_string(), bytes);
    }

    pub fn remove(&self, artifact_id: &str) {
        self.inner.write().unwrap().remove(artifact_id);
    }

    pub fn get(&self, artifact_id: &str) -> Option<Bytes> {
        self.inner.read().unwrap().get(artifact_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_after_insert() {
        let store = BundleStore::new();
        store.insert("a::backend", Bytes::from_static(b"hello"));
        assert_eq!(store.get("a::backend"), Some(Bytes::from_static(b"hello")));
    }

    #[test]
    fn remove_clears_entry() {
        let store = BundleStore::new();
        store.insert("a::backend", Bytes::from_static(b"hello"));
        store.remove("a::backend");
        assert_eq!(store.get("a::backend"), None);
    }
}
