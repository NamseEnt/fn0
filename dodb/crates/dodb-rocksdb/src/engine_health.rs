use std::ffi::{c_char, c_uchar, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

const MAX_ENGINE_HEALTH_SAMPLES: usize = 1_024;

const ENGINE_PROPERTIES: [(&str, &str); 9] = [
    (
        "estimate-pending-compaction-bytes",
        "rocksdb.estimate-pending-compaction-bytes",
    ),
    ("num-running-compactions", "rocksdb.num-running-compactions"),
    ("num-running-flushes", "rocksdb.num-running-flushes"),
    ("mem-table-flush-pending", "rocksdb.mem-table-flush-pending"),
    ("compaction-pending", "rocksdb.compaction-pending"),
    ("is-write-stopped", "rocksdb.is-write-stopped"),
    (
        "actual-delayed-write-rate",
        "rocksdb.actual-delayed-write-rate",
    ),
    ("block-cache-usage", "rocksdb.block-cache-usage"),
    ("cur-size-all-mem-tables", "rocksdb.cur-size-all-mem-tables"),
];

unsafe extern "C" {
    fn dodb_rocksdb_get_int_property(
        database: *mut c_void,
        property: *const c_uchar,
        property_length: usize,
        value: *mut u64,
        property_text: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_get_int_property_for_db(
        database: *mut c_void,
        property: *const c_uchar,
        property_length: usize,
        value: *mut u64,
        property_text: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_error_free(error: *mut c_char);
}

pub(crate) trait EngineHealthDatabase: Send + Sync {
    fn database_pointer(&self) -> *mut c_void;
    fn database_kind(&self) -> EngineHealthDatabaseKind;
}

#[derive(Clone, Copy)]
pub(crate) enum EngineHealthDatabaseKind {
    TransactionDb,
    Db,
}

pub struct EngineHealthObserver {
    databases: Vec<(String, Weak<dyn EngineHealthDatabase>)>,
}

impl EngineHealthObserver {
    pub(crate) fn new() -> Self {
        Self {
            databases: Vec::new(),
        }
    }

    pub(crate) fn add_database<T>(&mut self, database_name: String, database: &Arc<T>)
    where
        T: EngineHealthDatabase + 'static,
    {
        let erased_database: Arc<dyn EngineHealthDatabase> = database.clone();
        self.databases
            .push((database_name, Arc::downgrade(&erased_database)));
    }

    pub(crate) fn append(&mut self, mut other: Self) {
        self.databases.append(&mut other.databases);
    }

    pub fn combine(observers: impl IntoIterator<Item = Self>) -> Self {
        let mut combined = Self::new();
        for observer in observers {
            combined.append(observer);
        }
        combined
    }

    pub fn sample(&self) -> Value {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
            .unwrap_or(0);
        let databases = self
            .databases
            .iter()
            .map(|(database_name, weak_database)| {
                let Some(open_check) = weak_database.upgrade() else {
                    return json!({
                        "database": database_name,
                        "status": "closed",
                        "properties": {},
                    });
                };
                drop(open_check);
                let properties = ENGINE_PROPERTIES
                    .iter()
                    .map(|(field_name, property_name)| {
                        let Some(database) = weak_database.upgrade() else {
                            return (
                                (*field_name).to_owned(),
                                json!({
                                    "status": "closed",
                                    "value": null,
                                    "unit": property_unit(field_name),
                                }),
                            );
                        };
                        let property_bytes = property_name.as_bytes();
                        let mut value = 0_u64;
                        let mut property_text = std::ptr::null_mut();
                        let property_function = match database.database_kind() {
                            EngineHealthDatabaseKind::TransactionDb => {
                                dodb_rocksdb_get_int_property
                            }
                            EngineHealthDatabaseKind::Db => dodb_rocksdb_get_int_property_for_db,
                        };
                        let status = unsafe {
                            property_function(
                                database.database_pointer(),
                                property_bytes.as_ptr(),
                                property_bytes.len(),
                                &mut value,
                                &mut property_text,
                            )
                        };
                        drop(database);
                        let property_text_value = if property_text.is_null() {
                            None
                        } else {
                            let text = unsafe { std::ffi::CStr::from_ptr(property_text) }
                                .to_string_lossy()
                                .into_owned();
                            unsafe { dodb_rocksdb_error_free(property_text) };
                            Some(text)
                        };
                        let (status_name, property_value) = match status {
                            0 => ("value", json!(value)),
                            1 => ("unsupported", Value::Null),
                            2 => ("get_int_property_unavailable", Value::Null),
                            _ => ("native_property_error", Value::Null),
                        };
                        let mut property = json!({
                            "status": status_name,
                            "value": property_value,
                            "unit": property_unit(field_name),
                            "get_int_property_status": match status {
                                0 => "value",
                                1 | 2 => "returned_false",
                                _ => "native_error",
                            },
                            "get_property_status": match status {
                                0 => "not_needed",
                                1 => "returned_false",
                                2 => "value",
                                _ => "not_completed",
                            },
                        });
                        if let Some(property_text_value) = property_text_value {
                            property["get_property_text"] = json!(property_text_value);
                        }
                        ((*field_name).to_owned(), property)
                    })
                    .collect::<serde_json::Map<_, _>>();
                json!({
                    "database": database_name,
                    "status": "open",
                    "properties": properties,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "timestamp_unix_ms": timestamp_ms,
            "databases": databases,
        })
    }

    pub fn start_sampling(
        &self,
        interval: Duration,
        measurement_duration: Duration,
    ) -> Result<EngineHealthSampler, String> {
        if interval < Duration::from_secs(1) {
            return Err("engine health sampling interval must be at least 1000 ms".to_owned());
        }
        let maximum_sample_count = measurement_duration
            .as_nanos()
            .div_ceil(interval.as_nanos())
            .saturating_add(2)
            .min(MAX_ENGINE_HEALTH_SAMPLES as u128) as usize;
        let stopped = Arc::new(AtomicBool::new(false));
        let samples = Arc::new(Mutex::new(Vec::with_capacity(maximum_sample_count)));
        let worker_stopped = Arc::clone(&stopped);
        let worker_samples = Arc::clone(&samples);
        let worker_truncated = Arc::new(AtomicBool::new(false));
        let sampler_truncated = Arc::clone(&worker_truncated);
        let observer = self.clone_weak();
        let started = Instant::now();
        let worker = thread::Builder::new()
            .name("rocksdb-engine-health".to_owned())
            .spawn(move || {
                while !worker_stopped.load(Ordering::Relaxed) {
                    let sleep_started = Instant::now();
                    while !worker_stopped.load(Ordering::Relaxed)
                        && sleep_started.elapsed() < interval
                    {
                        thread::sleep(
                            interval
                                .saturating_sub(sleep_started.elapsed())
                                .min(Duration::from_millis(50)),
                        );
                    }
                    if worker_stopped.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Ok(mut sample_values) = worker_samples.lock() {
                        if sample_values.len() < maximum_sample_count {
                            sample_values.push(observer.sample());
                        } else {
                            sampler_truncated.store(true, Ordering::Relaxed);
                        }
                    }
                }
            })
            .map_err(|error| format!("failed to start engine health sampler: {error}"))?;
        Ok(EngineHealthSampler {
            stopped,
            samples,
            truncated: worker_truncated,
            started,
            interval,
            maximum_sample_count,
            worker: Some(worker),
        })
    }

    fn clone_weak(&self) -> Self {
        Self {
            databases: self
                .databases
                .iter()
                .map(|(database_name, database)| (database_name.clone(), database.clone()))
                .collect(),
        }
    }
}

pub struct EngineHealthSampler {
    stopped: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<Value>>>,
    truncated: Arc<AtomicBool>,
    started: Instant,
    interval: Duration,
    maximum_sample_count: usize,
    worker: Option<thread::JoinHandle<()>>,
}

impl EngineHealthSampler {
    pub fn finish(mut self) -> Result<Value, String> {
        self.stop_and_join()?;
        let mut sample_values = self
            .samples
            .lock()
            .map_err(|_| "engine health sample lock poisoned".to_owned())?;
        let samples = std::mem::take(&mut *sample_values);
        Ok(json!({
            "enabled": true,
            "status": "sampled",
            "sampling_interval_ms": self.interval.as_millis(),
            "sampling_scope": "measurement admission and drain only; warmup and validation excluded",
            "timestamp_unit": "milliseconds since Unix epoch",
            "properties": property_units(),
            "sample_count": samples.len(),
            "sample_capacity_limit": self.maximum_sample_count,
            "truncated": self.truncated.load(Ordering::Relaxed),
            "coverage_complete": !self.truncated.load(Ordering::Relaxed),
            "elapsed_ms": self.started.elapsed().as_millis(),
            "samples": samples,
        }))
    }

    fn stop_and_join(&mut self) -> Result<(), String> {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| "engine health sampler panicked".to_owned())?;
        }
        Ok(())
    }
}

impl Drop for EngineHealthSampler {
    fn drop(&mut self) {
        let _ = self.stop_and_join();
    }
}

impl Clone for EngineHealthObserver {
    fn clone(&self) -> Self {
        self.clone_weak()
    }
}

pub fn disabled_report() -> Value {
    json!({
        "enabled": false,
        "status": "disabled",
        "sampling_interval_ms": 0,
        "sampling_scope": "no engine query or sampler thread",
        "sample_count": 0,
        "samples": [],
    })
}

pub fn unsupported_report(interval: Duration) -> Value {
    json!({
        "enabled": interval > Duration::ZERO,
        "status": "unsupported",
        "sampling_interval_ms": interval.as_millis(),
        "sampling_scope": "engine properties are unavailable for this backend",
        "sample_count": 0,
        "samples": [],
    })
}

fn property_unit(field_name: &str) -> &'static str {
    match field_name {
        "estimate-pending-compaction-bytes" | "block-cache-usage" | "cur-size-all-mem-tables" => {
            "bytes"
        }
        "actual-delayed-write-rate" => "bytes_per_second",
        "num-running-compactions" | "num-running-flushes" => "count",
        "mem-table-flush-pending" | "compaction-pending" | "is-write-stopped" => {
            "boolean_as_integer"
        }
        _ => "unknown",
    }
}

fn property_units() -> Value {
    ENGINE_PROPERTIES
        .iter()
        .map(|(field_name, _)| ((*field_name).to_owned(), json!(property_unit(field_name))))
        .collect::<serde_json::Map<_, _>>()
        .into()
}
