use std::env;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use dodb_core::{DocumentKey, TransactionMutation, TransactionRequest};
use dodb_storage::{BTreeStore, BlinkStore, DatabaseConfig, FaultInjector, ProductionFile};

struct BlockingFault {
    point: String,
}

impl FaultInjector for BlockingFault {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == self.point {
            println!("HOOK {point}");
            io::stdout().flush().map_err(dodb_core::Error::from)?;
            let mut release = [0_u8; 1];
            io::stdin()
                .read_exact(&mut release)
                .map_err(dodb_core::Error::from)?;
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
        Ok(())
    }
}

enum Store {
    Main(BTreeStore<ProductionFile, ProductionFile>),
    Planned(BlinkStore<ProductionFile, ProductionFile>),
}

impl Store {
    fn transact(
        &mut self,
        request: TransactionRequest,
    ) -> dodb_core::Result<dodb_core::TransactionResult> {
        match self {
            Self::Main(store) => store.transact(request),
            Self::Planned(store) => store.transact(request),
        }
    }
}

fn main() {
    let mut arguments = env::args_os().skip(1);
    let engine = arguments.next().expect("engine is required");
    let data_path = PathBuf::from(arguments.next().expect("data path is required"));
    let wal_path = PathBuf::from(arguments.next().expect("WAL path is required"));
    let boundary = arguments.next().expect("boundary is required");
    let width: usize = arguments
        .next()
        .expect("width is required")
        .to_string_lossy()
        .parse()
        .expect("width must be an integer");
    let seed: u64 = arguments
        .next()
        .expect("seed is required")
        .to_string_lossy()
        .parse()
        .expect("seed must be an integer");
    let mutations = target_mutations(seed, width);
    let injector = BlockingFault {
        point: match boundary.to_string_lossy().as_ref() {
            "before_wal_append" => "before_wal_append".to_owned(),
            "before_wal_sync" => "before_wal_sync".to_owned(),
            "after_wal_sync" => "after_wal_sync".to_owned(),
            _ => String::new(),
        },
    };
    let mut store = match engine.to_string_lossy().as_ref() {
        "main-btree" => {
            let mut store = BTreeStore::open_with_wal(
                ProductionFile::open(data_path).expect("database file should open"),
                ProductionFile::open(wal_path).expect("WAL file should open"),
                DatabaseConfig::default(),
            )
            .expect("database should open");
            store.set_fault_injector(injector);
            Store::Main(store)
        }
        "planned-blink" => {
            let mut store = BlinkStore::open_with_wal(
                ProductionFile::open(data_path).expect("database file should open"),
                ProductionFile::open(wal_path).expect("WAL file should open"),
                DatabaseConfig::default(),
            )
            .expect("database should open");
            store.enable_planned_execution();
            store.set_fault_injector(injector);
            Store::Planned(store)
        }
        _ => panic!("unknown engine"),
    };
    let result = store
        .transact(TransactionRequest::new(Vec::new(), mutations))
        .expect("target transaction should succeed");
    let revision = result.revision.expect("target revision should be returned");
    println!("SUCCESS {}", revision.get());
    io::stdout().flush().expect("success response should flush");
    if boundary.to_string_lossy() == "returned" {
        let mut release = [0_u8; 1];
        io::stdin()
            .read_exact(&mut release)
            .expect("parent should hold child after success");
    }
}

fn target_mutations(seed: u64, width: usize) -> Vec<TransactionMutation> {
    (0..width)
        .map(|mutation_index| {
            let row_index = (mutation_index * 503 + 113) % 4096;
            TransactionMutation::Put {
                key: row_key(row_index),
                value: value_bytes(seed, 128, mutation_index),
            }
        })
        .collect()
}

fn row_key(row_index: usize) -> DocumentKey {
    let partition = (row_index as u64).to_be_bytes();
    let sort = (row_index as u64 ^ 0xd0db_2026).to_be_bytes();
    DocumentKey::new(partition.to_vec(), sort.to_vec())
}

fn value_bytes(seed: u64, transaction_index: usize, mutation_index: usize) -> Vec<u8> {
    let mut value = vec![0_u8; 64];
    value[..8].copy_from_slice(&seed.to_be_bytes());
    value[8..16].copy_from_slice(&(transaction_index as u64).to_be_bytes());
    value[16..24].copy_from_slice(&(mutation_index as u64).to_be_bytes());
    for (byte_index, byte) in value[24..].iter_mut().enumerate() {
        *byte = seed
            .wrapping_add((transaction_index as u64).wrapping_mul(31))
            .wrapping_add((mutation_index as u64).wrapping_mul(17))
            .wrapping_add(byte_index as u64) as u8;
    }
    value
}
