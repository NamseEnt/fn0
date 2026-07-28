//! In-process backend for tests. Same API as the HTTP backend, backed by a
//! `BTreeMap` so listings are naturally key-ordered.

use crate::body::Body;
use crate::{ListEntry, ObjectList, ObjectMetadata, Result};
use bytes::Bytes;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone)]
struct StoredObject {
    data: Bytes,
    content_type: Option<String>,
}

#[derive(Clone)]
pub(crate) struct MemoryBucket {
    store: Arc<Mutex<BTreeMap<String, StoredObject>>>,
}

impl MemoryBucket {
    pub(crate) fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub(crate) async fn put(&self, key: &str, body: Body, content_type: Option<&str>) -> Result<()> {
        let data = body.collect().await;
        self.store.lock().unwrap().insert(
            key.to_string(),
            StoredObject {
                data,
                content_type: content_type.map(str::to_string),
            },
        );
        Ok(())
    }

    pub(crate) async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .get(key)
            .map(|object| object.data.clone()))
    }

    pub(crate) async fn head(&self, key: &str) -> Result<Option<ObjectMetadata>> {
        Ok(self
            .store
            .lock()
            .unwrap()
            .get(key)
            .map(|object| ObjectMetadata {
                size: object.data.len() as u64,
                content_type: object.content_type.clone(),
                etag: None,
            }))
    }

    pub(crate) async fn delete(&self, key: &str) -> Result<()> {
        self.store.lock().unwrap().remove(key);
        Ok(())
    }

    pub(crate) async fn list(
        &self,
        prefix: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<ObjectList> {
        let store = self.store.lock().unwrap();
        let mut entries = Vec::new();
        for (key, object) in store.iter() {
            if !key.starts_with(prefix) {
                continue;
            }
            if let Some(after) = after
                && key.as_str() <= after
            {
                continue;
            }
            if entries.len() == limit {
                return Ok(ObjectList {
                    next_cursor: entries.last().map(|e: &ListEntry| e.key.clone()),
                    entries,
                });
            }
            entries.push(ListEntry {
                key: key.clone(),
                size: object.data.len() as u64,
            });
        }
        Ok(ObjectList {
            entries,
            next_cursor: None,
        })
    }

    pub(crate) async fn presigned_url(
        &self,
        key: &str,
        method: &str,
        _expires: Duration,
        _content_length: Option<u64>,
    ) -> Result<String> {
        Ok(format!("memory://{}/{key}", method.to_ascii_lowercase()))
    }
}
