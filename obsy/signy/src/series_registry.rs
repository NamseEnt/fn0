//! The metric part registry (M14, issue #8): the open set of metric parts,
//! modeled on `trace_registry.rs` — the same per-tenant stored-bytes census
//! maintained on registry mutation, because the storage quota reads it on the
//! ingest path and summing segments per write is work proportional to the
//! part count.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::object_storage::MetricManifest;
use crate::series_part::{SeriesPart, SeriesPartReader, discover_series_parts};
use crate::tenant::TenantId;

pub struct SeriesRegistry {
    inner: RwLock<HashMap<String, Arc<SeriesPartReader>>>,
    stored_bytes: RwLock<HashMap<TenantId, u64>>,
    operation_lock: Arc<tokio::sync::RwLock<()>>,
}

fn reader_tenant_bytes(reader: &SeriesPartReader) -> Vec<(TenantId, u64)> {
    reader
        .part()
        .meta
        .tenants
        .iter()
        .map(|segment| (segment.tenant.clone(), segment.bytes.len()))
        .collect()
}

fn census_of(readers: &HashMap<String, Arc<SeriesPartReader>>) -> HashMap<TenantId, u64> {
    let mut totals: HashMap<TenantId, u64> = HashMap::new();
    for reader in readers.values() {
        for (tenant, bytes) in reader_tenant_bytes(reader) {
            *totals.entry(tenant).or_insert(0) += bytes;
        }
    }
    totals
}

impl SeriesRegistry {
    pub fn standalone() -> Self {
        Self::new(Arc::new(tokio::sync::RwLock::new(())))
    }

    pub fn new(operation_lock: Arc<tokio::sync::RwLock<()>>) -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            stored_bytes: RwLock::new(HashMap::new()),
            operation_lock,
        }
    }

    pub fn operation_lock(&self) -> Arc<tokio::sync::RwLock<()>> {
        self.operation_lock.clone()
    }

    pub fn visit_tenants(&self, mut visit: impl FnMut(&TenantId)) {
        for reader in self.inner.read().values() {
            for segment in &reader.part().meta.tenants {
                visit(&segment.tenant);
            }
        }
    }

    pub fn load_from_disk(
        metrics_root: &Path,
        operation_lock: Arc<tokio::sync::RwLock<()>>,
    ) -> Result<Self, String> {
        let registry = Self::new(operation_lock);
        registry.reload_from_disk(metrics_root)?;
        Ok(registry)
    }

    pub fn reload_from_disk(&self, metrics_root: &Path) -> Result<(), String> {
        let parts = discover_series_parts(metrics_root)?;
        let mut readers = HashMap::new();
        for part in parts {
            let id = part.meta.id.clone();
            let reader = self
                .open_part(part, true)
                .map_err(|error| format!("failed to open metric part {id}: {error}"))?;
            readers.insert(id, Arc::new(reader));
        }
        let mut inner = self.inner.write();
        *self.stored_bytes.write() = census_of(&readers);
        *inner = readers;
        Ok(())
    }

    pub fn load_from_manifest(
        metrics_root: &Path,
        manifest: &MetricManifest,
        operation_lock: Arc<tokio::sync::RwLock<()>>,
    ) -> Result<Self, String> {
        let registry = Self::new(operation_lock);
        let mut readers = HashMap::new();
        for descriptor in &manifest.parts {
            let dir = metrics_root
                .join(&descriptor.partition)
                .join(&descriptor.id);
            let part = crate::series_part::load_series_part(&dir).map_err(|error| {
                format!(
                    "failed to load metric manifest part {}: {error}",
                    descriptor.id
                )
            })?;
            if part.meta.id != descriptor.id || part.meta.partition != descriptor.partition {
                return Err(format!(
                    "cached metric part metadata does not match manifest descriptor {}/{}",
                    descriptor.partition, descriptor.id
                ));
            }
            let reader = SeriesPartReader::open_cached(part).map_err(|error| {
                format!(
                    "failed to open metric manifest part {}: {error}",
                    descriptor.id
                )
            })?;
            readers.insert(descriptor.id.clone(), Arc::new(reader));
        }
        *registry.stored_bytes.write() = census_of(&readers);
        *registry.inner.write() = readers;
        Ok(registry)
    }

    fn open_part(&self, part: SeriesPart, require_data: bool) -> Result<SeriesPartReader, String> {
        let _ = self;
        if require_data {
            SeriesPartReader::open(part)
        } else {
            SeriesPartReader::open_cached(part)
        }
    }

    pub fn missing_data_ids(
        &self,
        ids: &std::collections::HashSet<String>,
    ) -> std::collections::HashSet<String> {
        self.inner
            .read()
            .iter()
            .filter(|(id, reader)| ids.contains(*id) && !reader.part().data_path().exists())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Open readers for freshly written parts, without touching the registry —
    /// the same open-outside-the-lock split as the other registries.
    pub fn open_parts(
        parts: Vec<SeriesPart>,
    ) -> Result<Vec<(String, Arc<SeriesPartReader>)>, String> {
        let mut readers = Vec::with_capacity(parts.len());
        for part in parts {
            let id = part.meta.id.clone();
            let reader = SeriesPartReader::open(part)
                .map_err(|error| format!("failed to open metric part {id}: {error}"))?;
            readers.push((id, Arc::new(reader)));
        }
        Ok(readers)
    }

    pub fn register(&self, parts: Vec<SeriesPart>) -> Result<Vec<String>, String> {
        Ok(self.register_opened(Self::open_parts(parts)?))
    }

    pub fn register_opened(&self, readers: Vec<(String, Arc<SeriesPartReader>)>) -> Vec<String> {
        let ids = readers.iter().map(|(id, _)| id.clone()).collect();
        let mut inner = self.inner.write();
        let mut census = self.stored_bytes.write();
        for (id, reader) in readers {
            if let Some(previous) = inner.insert(id, reader.clone()) {
                subtract(&mut census, &previous);
            }
            for (tenant, bytes) in reader_tenant_bytes(&reader) {
                *census.entry(tenant).or_insert(0) += bytes;
            }
        }
        ids
    }

    pub fn unregister(&self, ids: &[String]) {
        let mut inner = self.inner.write();
        let mut census = self.stored_bytes.write();
        for id in ids {
            if let Some(removed) = inner.remove(id) {
                subtract(&mut census, &removed);
            }
        }
    }

    /// Bytes the tenant's chunks occupy across every registered metric part.
    pub fn tenant_stored_bytes(&self, tenant: &TenantId) -> u64 {
        self.stored_bytes.read().get(tenant).copied().unwrap_or(0)
    }

    pub fn snapshot(&self) -> Vec<Arc<SeriesPartReader>> {
        self.inner.read().values().cloned().collect()
    }

    pub fn part_dirs(&self) -> Vec<std::path::PathBuf> {
        self.inner
            .read()
            .values()
            .map(|reader| reader.part().dir.clone())
            .collect()
    }

    pub fn part_ids(&self) -> std::collections::HashSet<String> {
        self.inner.read().keys().cloned().collect()
    }

    pub fn part_count(&self) -> usize {
        self.inner.read().len()
    }
}

fn subtract(census: &mut HashMap<TenantId, u64>, reader: &SeriesPartReader) {
    for (tenant, bytes) in reader_tenant_bytes(reader) {
        if let Some(total) = census.get_mut(&tenant) {
            *total = total.saturating_sub(bytes);
            if *total == 0 {
                census.remove(&tenant);
            }
        }
    }
}
