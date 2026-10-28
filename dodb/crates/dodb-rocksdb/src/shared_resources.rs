use std::collections::BTreeMap;
use std::ffi::{c_char, c_void};
use std::ptr::NonNull;

use dodb_core::Error;
use serde::Serialize;

pub const SHARED_RESOURCE_FLAGS: [&str; 6] = [
    "--rocksdb-shared-cache-bytes",
    "--rocksdb-shared-wbm-bytes",
    "--rocksdb-write-buffer-bytes",
    "--rocksdb-flush-threads",
    "--rocksdb-compaction-threads",
    "--rocksdb-max-open-files",
];

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SharedResourceConfig {
    pub cache_bytes: usize,
    pub wbm_bytes: usize,
    pub write_buffer_bytes: usize,
    pub flush_threads: i32,
    pub compaction_threads: i32,
    pub max_open_files: i32,
}

impl SharedResourceConfig {
    pub fn validate(self) -> Result<(), String> {
        if self.write_buffer_bytes < 65_536
            || self.write_buffer_bytes > self.wbm_bytes
            || self.wbm_bytes > self.cache_bytes
        {
            return Err(
                "require 65536 <= write-buffer bytes <= WBM bytes <= cache bytes".to_owned(),
            );
        }
        if !(1..=64).contains(&self.flush_threads)
            || !(1..=64).contains(&self.compaction_threads)
            || self.max_open_files < 20
        {
            return Err(
                "flush/compaction threads must be 1..64 and max-open-files at least 20".to_owned(),
            );
        }
        Ok(())
    }

    pub fn from_arguments(backend: &str, arguments: &[String]) -> Result<Option<Self>, String> {
        let mut values = BTreeMap::new();
        for (argument_index, name) in arguments.iter().enumerate() {
            if !name.starts_with("--rocksdb-") {
                continue;
            }
            if !SHARED_RESOURCE_FLAGS.contains(&name.as_str()) {
                return Err(format!("unknown resource option {name}"));
            }
            let value = arguments
                .get(argument_index + 1)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| format!("missing value for {name}"))?;
            if values.insert(name.as_str(), value.as_str()).is_some() {
                return Err(format!("duplicate resource option {name}"));
            }
        }
        if backend != "minimal-native-shared" {
            return if values.is_empty() {
                Ok(None)
            } else {
                Err("RocksDB resource options require minimal-native-shared".to_owned())
            };
        }
        let parse = |name: &str| -> Result<usize, String> {
            values
                .get(name)
                .ok_or_else(|| format!("required resource option {name}"))?
                .parse()
                .map_err(|_| format!("invalid unsigned resource value for {name}"))
        };
        let integer = |name: &str| -> Result<i32, String> {
            i32::try_from(parse(name)?)
                .map_err(|_| format!("resource value exceeds i32 for {name}"))
        };
        let config = Self {
            cache_bytes: parse(SHARED_RESOURCE_FLAGS[0])?,
            wbm_bytes: parse(SHARED_RESOURCE_FLAGS[1])?,
            write_buffer_bytes: parse(SHARED_RESOURCE_FLAGS[2])?,
            flush_threads: integer(SHARED_RESOURCE_FLAGS[3])?,
            compaction_threads: integer(SHARED_RESOURCE_FLAGS[4])?,
            max_open_files: integer(SHARED_RESOURCE_FLAGS[5])?,
        };
        config.validate()?;
        Ok(Some(config))
    }

    pub fn arguments(self) -> Vec<String> {
        SHARED_RESOURCE_FLAGS
            .into_iter()
            .zip([
                self.cache_bytes.to_string(),
                self.wbm_bytes.to_string(),
                self.write_buffer_bytes.to_string(),
                self.flush_threads.to_string(),
                self.compaction_threads.to_string(),
                self.max_open_files.to_string(),
            ])
            .flat_map(|(name, value)| [name.to_owned(), value])
            .collect()
    }

    pub fn report(self) -> serde_json::Value {
        serde_json::json!({
            "configured": self,
            "scope": "one cache and WBM per service; shared across all opened tenant databases",
            "cache_type": "LRU", "cache_shard_bits": 0,
            "cache_strict_capacity_limit": false, "wbm_allow_stall": false,
            "memtables_charged_to_cache": true, "cache_index_and_filter_blocks": true,
            "pin_l0_filter_and_index_blocks_in_cache": false,
            "pin_top_level_index_and_filter": false,
            "max_write_buffer_number_per_tenant": 2,
            "max_write_buffer_size_to_maintain_per_tenant": 1,
            "transaction_snapshot_mode": "no Transaction::SetSnapshot; exclusive GetForUpdate conditions; Query/Scan use DB snapshots",
            "max_subcompactions_per_tenant": 1,
            "max_background_jobs_per_tenant": self.flush_threads + self.compaction_threads,
            "background_pool_scope": "process-global Env::Default HIGH flush and LOW compaction pools; not an application or total-thread limit",
            "process_memory_limit_enforced": false,
            "budget_limitations": "WBM is a soft flush threshold with stalls disabled; idle dirty or retained-history memtables may keep usage above it; non-strict cache may exceed capacity with pinned blocks and reservations; excludes other native allocations, request/runtime memory, OS cache and ARC; WBM reservations are already included in cache usage"
        })
    }
}

#[repr(C)]
#[derive(Default, Debug, Serialize)]
pub struct SharedResourceSnapshot {
    pub cache_capacity_bytes: u64,
    pub cache_usage_bytes: u64,
    pub cache_pinned_usage_bytes: u64,
    pub wbm_limit_bytes: u64,
    pub wbm_usage_bytes: u64,
    pub wbm_mutable_bytes: u64,
    pub wbm_cache_reservation_bytes: u64,
    pub flush_threads: u64,
    pub compaction_threads: u64,
}

pub(crate) struct SharedResources(NonNull<c_void>);

unsafe impl Send for SharedResources {}
unsafe impl Sync for SharedResources {}

impl SharedResources {
    pub(crate) fn new(config: SharedResourceConfig) -> Result<Self, Error> {
        config.validate().map_err(Error::invalid_input)?;
        let mut error = std::ptr::null_mut();
        let pointer = unsafe { dodb_rocksdb_shared_new(&config, &mut error) };
        NonNull::new(pointer)
            .map(Self)
            .ok_or_else(|| crate::ffi_error(error, "shared resource creation failed"))
    }

    pub(crate) fn pointer(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    pub(crate) fn snapshot(&self) -> SharedResourceSnapshot {
        let mut snapshot = SharedResourceSnapshot::default();
        unsafe { dodb_rocksdb_shared_snapshot(self.pointer(), &mut snapshot) };
        snapshot
    }
}

impl Drop for SharedResources {
    fn drop(&mut self) {
        unsafe { dodb_rocksdb_shared_delete(self.pointer()) };
    }
}

unsafe extern "C" {
    fn dodb_rocksdb_shared_new(
        config: *const SharedResourceConfig,
        error: *mut *mut c_char,
    ) -> *mut c_void;
    fn dodb_rocksdb_shared_delete(resources: *mut c_void);
    fn dodb_rocksdb_shared_snapshot(resources: *mut c_void, snapshot: *mut SharedResourceSnapshot);
    pub(crate) fn dodb_rocksdb_open_shared(
        path: *const c_char,
        resources: *mut c_void,
        error: *mut *mut c_char,
    ) -> *mut c_void;
    pub(crate) fn dodb_rocksdb_uses_shared(database: *mut c_void, resources: *mut c_void) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_resource_options_round_trip_and_reject_incompatible_inputs() {
        let config = SharedResourceConfig {
            cache_bytes: 16 * 1024 * 1024,
            wbm_bytes: 4 * 1024 * 1024,
            write_buffer_bytes: 256 * 1024,
            flush_threads: 1,
            compaction_threads: 1,
            max_open_files: 64,
        };
        let arguments = config.arguments();
        assert_eq!(
            SharedResourceConfig::from_arguments("minimal-native-shared", &arguments).unwrap(),
            Some(config)
        );
        for backend in ["blink", "minimal-native", "core", "raw", "rocksdb-service"] {
            assert!(SharedResourceConfig::from_arguments(backend, &arguments).is_err());
            assert_eq!(
                SharedResourceConfig::from_arguments(backend, &[]).unwrap(),
                None
            );
        }
        for count in 0..arguments.len() {
            assert!(
                SharedResourceConfig::from_arguments("minimal-native-shared", &arguments[..count])
                    .is_err()
            );
        }
        let mut duplicate = arguments.clone();
        duplicate.extend_from_slice(&arguments[..2]);
        assert!(SharedResourceConfig::from_arguments("minimal-native-shared", &duplicate).is_err());
        for position in (1..arguments.len()).step_by(2) {
            for value in ["0", "-1", "18446744073709551616"] {
                let mut invalid = arguments.clone();
                invalid[position] = value.to_owned();
                assert!(
                    SharedResourceConfig::from_arguments("minimal-native-shared", &invalid)
                        .is_err()
                );
            }
        }
        for invalid in [
            SharedResourceConfig {
                wbm_bytes: config.cache_bytes + 1,
                ..config
            },
            SharedResourceConfig {
                write_buffer_bytes: config.wbm_bytes + 1,
                ..config
            },
            SharedResourceConfig {
                write_buffer_bytes: 65_535,
                ..config
            },
            SharedResourceConfig {
                flush_threads: 65,
                ..config
            },
            SharedResourceConfig {
                max_open_files: 19,
                ..config
            },
        ] {
            assert!(invalid.validate().is_err());
        }
    }
}
