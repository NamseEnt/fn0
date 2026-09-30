#[path = "../../common/mod.rs"]
mod common;

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::PathBuf;

use common::driver::{self, Args, Attempt, Engine, EngineFactory};
use common::workload::Mutation;
use serde_json::{Value, json};

const BLOCK_CACHE_BYTES: u64 = 64 * 1024 * 1024;
const ROCKSDB_TAG: &str = env!("CROSSDB_ROCKSDB_TAG");
const ROCKSDB_COMMIT: &str = env!("CROSSDB_ROCKSDB_COMMIT");

unsafe extern "C" {
    fn crossdb_rocksdb_open(
        path: *const c_char,
        pipelined: i32,
        block_cache_bytes: u64,
        error: *mut *mut c_char,
    ) -> *mut c_void;
    fn crossdb_rocksdb_write(
        handle: *mut c_void,
        keys: *const u8,
        values: *const u8,
        count: usize,
        key_length: usize,
        value_length: usize,
        error: *mut *mut c_char,
    ) -> i32;
    fn crossdb_rocksdb_get(
        handle: *mut c_void,
        key: *const u8,
        key_length: usize,
        value: *mut *mut c_char,
        value_length: *mut usize,
    ) -> i32;
    fn crossdb_rocksdb_query(
        handle: *mut c_void,
        lower: *const u8,
        upper: *const u8,
        key_length: usize,
        value_length: usize,
        limit: usize,
        keys_out: *mut u8,
        values_out: *mut u8,
    ) -> i32;
    fn crossdb_rocksdb_count(handle: *mut c_void) -> u64;
    fn crossdb_rocksdb_property(handle: *mut c_void, name: *const c_char) -> *mut c_char;
    fn crossdb_rocksdb_map_property_json(handle: *mut c_void, name: *const c_char) -> *mut c_char;
    fn crossdb_rocksdb_events(handle: *mut c_void) -> *mut c_char;
    fn crossdb_rocksdb_options(handle: *mut c_void) -> *mut c_char;
    fn crossdb_monotonic_seconds() -> f64;
    fn crossdb_rocksdb_close(handle: *mut c_void) -> *mut c_char;
    fn crossdb_free(value: *mut c_char);
}

fn take_string(pointer: *mut c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    let value = unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned();
    unsafe { crossdb_free(pointer) };
    Some(value)
}

fn pipelined_from_args(args: &Args) -> bool {
    match args.option("pipelined").unwrap_or("off") {
        "on" => true,
        "off" => false,
        other => panic!("unknown pipelined value {other}"),
    }
}

struct RocksEngine {
    handle: *mut c_void,
    data_dir: PathBuf,
    value_size: usize,
}

unsafe impl Send for RocksEngine {}
unsafe impl Sync for RocksEngine {}

impl RocksEngine {
    fn property(&self, name: &str) -> Option<String> {
        let name = CString::new(name).unwrap();
        take_string(unsafe { crossdb_rocksdb_property(self.handle, name.as_ptr()) })
    }

    fn map_property(&self, name: &str) -> Value {
        let name = CString::new(name).unwrap();
        take_string(unsafe { crossdb_rocksdb_map_property_json(self.handle, name.as_ptr()) })
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or(Value::Null)
    }

    fn integer_properties(&self) -> Value {
        let mut properties = serde_json::Map::new();
        for name in [
            "rocksdb.num-files-at-level0",
            "rocksdb.num-files-at-level1",
            "rocksdb.num-files-at-level2",
            "rocksdb.estimate-pending-compaction-bytes",
            "rocksdb.is-write-stopped",
            "rocksdb.actual-delayed-write-rate",
            "rocksdb.cur-size-all-mem-tables",
            "rocksdb.num-immutable-mem-table",
            "rocksdb.num-running-flushes",
            "rocksdb.num-running-compactions",
            "rocksdb.compaction-pending",
            "rocksdb.mem-table-flush-pending",
            "rocksdb.total-sst-files-size",
            "rocksdb.live-sst-files-size",
            "rocksdb.estimate-num-keys",
            "rocksdb.block-cache-usage",
        ] {
            let value = self
                .property(name)
                .and_then(|text| text.trim().parse::<u64>().ok());
            properties.insert(name.to_owned(), json!(value));
        }
        Value::Object(properties)
    }
}

impl Engine for RocksEngine {
    type Writer = (Vec<u8>, Vec<u8>);

    fn open_writer(&self, _writer_id: usize) -> Self::Writer {
        (
            Vec::with_capacity(16 * 16),
            Vec::with_capacity(16 * self.value_size),
        )
    }

    fn attempt(&self, writer: &mut Self::Writer, mutations: &[Mutation]) -> Attempt {
        let (keys, values) = writer;
        keys.clear();
        values.clear();
        let key_length = mutations[0].key.len();
        let value_length = mutations[0].value.len();
        for mutation in mutations {
            assert_eq!(mutation.key.len(), key_length);
            assert_eq!(mutation.value.len(), value_length);
            keys.extend_from_slice(&mutation.key);
            values.extend_from_slice(&mutation.value);
        }
        let mut error: *mut c_char = std::ptr::null_mut();
        let status = unsafe {
            crossdb_rocksdb_write(
                self.handle,
                keys.as_ptr(),
                values.as_ptr(),
                mutations.len(),
                key_length,
                value_length,
                &mut error,
            )
        };
        if status == 0 {
            Attempt::Committed
        } else {
            Attempt::Failed(take_string(error).unwrap_or_default())
        }
    }

    fn seed(&self, writer: &mut Self::Writer, rows: &[Mutation]) {
        match self.attempt(writer, rows) {
            Attempt::Committed => {}
            other => panic!("seed write failed: {other:?}"),
        }
    }

    fn read(&self, _writer: &mut Self::Writer, key: &[u8]) -> Option<Vec<u8>> {
        let mut value: *mut c_char = std::ptr::null_mut();
        let mut value_length = 0usize;
        let status = unsafe {
            crossdb_rocksdb_get(
                self.handle,
                key.as_ptr(),
                key.len(),
                &mut value,
                &mut value_length,
            )
        };
        match status {
            0 => {
                let bytes = unsafe { std::slice::from_raw_parts(value as *const u8, value_length) }
                    .to_vec();
                unsafe { crossdb_free(value) };
                Some(bytes)
            }
            1 => None,
            _ => panic!("rocksdb get failed"),
        }
    }

    fn query(&self, _writer: &mut Self::Writer, primary_key: &[u8], limit: usize) -> Vec<Mutation> {
        let mut lower = primary_key.to_vec();
        lower.extend_from_slice(&[0; 8]);
        let mut upper = primary_key.to_vec();
        upper.extend_from_slice(&[u8::MAX; 8]);
        let mut keys = vec![0; limit * 16];
        let mut values = vec![0; limit * self.value_size];
        let count = unsafe {
            crossdb_rocksdb_query(
                self.handle,
                lower.as_ptr(),
                upper.as_ptr(),
                16,
                self.value_size,
                limit,
                keys.as_mut_ptr(),
                values.as_mut_ptr(),
            )
        };
        assert!(count >= 0, "rocksdb range query failed");
        (0..count as usize)
            .map(|row_index| Mutation {
                key: keys[row_index * 16..(row_index + 1) * 16].to_vec(),
                value: values[row_index * self.value_size..(row_index + 1) * self.value_size]
                    .to_vec(),
            })
            .collect()
    }

    fn count_rows(&self, _writer: &mut Self::Writer) -> u64 {
        unsafe { crossdb_rocksdb_count(self.handle) }
    }

    fn settings(&self, _writer: &mut Self::Writer) -> Value {
        take_string(unsafe { crossdb_rocksdb_options(self.handle) })
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or(Value::Null)
    }

    fn metrics(&self) -> Value {
        let events = take_string(unsafe { crossdb_rocksdb_events(self.handle) })
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .unwrap_or(Value::Null);
        json!({
            "t_mono": unsafe { crossdb_monotonic_seconds() },
            "dbstats": self.property("rocksdb.dbstats"),
            "cfstats_map": self.map_property("rocksdb.cfstats"),
            "levelstats": self.property("rocksdb.levelstats"),
            "integer_properties": self.integer_properties(),
            "events": events,
            "files": driver::directory_listing(&self.data_dir),
        })
    }

    fn monitor_sample(&self) -> Value {
        json!({
            "t_mono": unsafe { crossdb_monotonic_seconds() },
            "properties": self.integer_properties(),
        })
    }
}

struct RocksFactory;

impl RocksFactory {
    fn open(args: &Args) -> RocksEngine {
        let path = CString::new(args.data_dir.join("rocksdb").to_str().unwrap()).unwrap();
        let mut error: *mut c_char = std::ptr::null_mut();
        let handle = unsafe {
            crossdb_rocksdb_open(
                path.as_ptr(),
                i32::from(pipelined_from_args(args)),
                BLOCK_CACHE_BYTES,
                &mut error,
            )
        };
        assert!(
            !handle.is_null(),
            "rocksdb open failed: {}",
            take_string(error).unwrap_or_default()
        );
        RocksEngine {
            handle,
            data_dir: args.data_dir.join("rocksdb"),
            value_size: args.value_size,
        }
    }
}

impl EngineFactory for RocksFactory {
    type Engine = RocksEngine;

    fn engine_name(args: &Args) -> String {
        if pipelined_from_args(args) {
            "rocksdb-pipelined".to_owned()
        } else {
            "rocksdb".to_owned()
        }
    }

    fn build_info() -> Value {
        json!({
            "database": "rocksdb",
            "tag": ROCKSDB_TAG,
            "commit": ROCKSDB_COMMIT,
            "binding": "C++ API through a static in-process shim (DB::Write with one WriteBatch per logical transaction)",
        })
    }

    fn create(args: &Args) -> RocksEngine {
        Self::open(args)
    }

    fn reopen(args: &Args) -> RocksEngine {
        Self::open(args)
    }

    fn close(engine: RocksEngine) -> Value {
        let data_dir = engine.data_dir.clone();
        let status = take_string(unsafe { crossdb_rocksdb_close(engine.handle) });
        json!({
            "close_status": status,
            "files_after_close": driver::directory_listing(&data_dir),
        })
    }
}

fn main() {
    driver::bench_main::<RocksFactory>();
}
