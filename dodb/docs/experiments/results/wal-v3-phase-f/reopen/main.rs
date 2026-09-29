use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use dodb_core::{DocumentKey, TransactionMutation, TransactionRequest};
use dodb_storage::{BlinkStore, DatabaseConfig, ProductionFile};

struct CountingAllocator;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn key(index: u64) -> DocumentKey {
    let mut primary = vec![0x51u8; 8];
    primary.copy_from_slice(&(index % 128).to_be_bytes());
    DocumentKey::new(primary, index.to_be_bytes().to_vec())
}

fn main() {
    let directory = std::env::temp_dir().join(format!("dodb-reopen-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("store.db");
    {
        let mut store = BlinkStore::<ProductionFile, ProductionFile>::open_path(&path, DatabaseConfig::default()).unwrap();
        store.enable_planned_execution();
        let requests = (0..100_000u64)
            .collect::<Vec<_>>()
            .chunks(16)
            .map(|chunk| {
                TransactionRequest::new(
                    Vec::new(),
                    chunk
                        .iter()
                        .map(|index| TransactionMutation::Put { key: key(*index), value: vec![*index as u8; 64] })
                        .collect(),
                )
            })
            .collect::<Vec<_>>();
        for group in requests.chunks(64) {
            store.apply_transaction_group(group).unwrap();
        }
        store.checkpoint().unwrap();
    }
    let data_bytes = std::fs::metadata(&path).unwrap().len();
    for run in 0..5 {
        let allocations_before = ALLOCATIONS.load(Ordering::Relaxed);
        let bytes_before = ALLOCATED_BYTES.load(Ordering::Relaxed);
        let started = Instant::now();
        let store = BlinkStore::<ProductionFile, ProductionFile>::open_path(&path, DatabaseConfig::default()).unwrap();
        let elapsed = started.elapsed();
        let allocations = ALLOCATIONS.load(Ordering::Relaxed) - allocations_before;
        let bytes = ALLOCATED_BYTES.load(Ordering::Relaxed) - bytes_before;
        println!(
            "run={run} data_pages={} open_ms={:.2} allocations={allocations} allocated_bytes={bytes}",
            data_bytes / 4096,
            elapsed.as_secs_f64() * 1000.0
        );
        drop(store);
    }
    let _ = std::fs::remove_dir_all(directory);
}
