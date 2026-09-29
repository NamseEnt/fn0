use std::path::PathBuf;

use dodb_core::{DocumentKey, TransactionMutation, TransactionRequest};
use dodb_storage::{BTreeStore, BlinkStore, DatabaseConfig, ProductionFile};

fn probe_key(index: u64) -> DocumentKey {
    DocumentKey::new(
        format!("pk-{:03}", index % 37).into_bytes(),
        format!("sk-{index:08}").into_bytes(),
    )
}

fn put_request(keys: &[u64], round: u64) -> TransactionRequest {
    TransactionRequest::new(
        Vec::new(),
        keys.iter()
            .map(|key_index| TransactionMutation::Put {
                key: probe_key(*key_index),
                value: vec![(key_index ^ round) as u8; 64],
            })
            .collect(),
    )
}

#[test]
fn wal_byte_probe() {
    let directory = PathBuf::from(std::env::var("WAL_BYTE_PROBE_DIR").expect("WAL_BYTE_PROBE_DIR"));
    std::fs::create_dir_all(&directory).unwrap();
    for name in ["blink.db", "blink.wal", "btree.db", "btree.wal"] {
        let _ = std::fs::remove_file(directory.join(name));
    }

    let mut blink = BlinkStore::open_with_wal(
        ProductionFile::open(directory.join("blink.db")).unwrap(),
        ProductionFile::open(directory.join("blink.wal")).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    blink.enable_planned_execution();
    for index in 0..2_000u64 {
        blink
            .put(probe_key(index % 700), vec![index as u8; 64])
            .unwrap();
    }
    blink.put(probe_key(5), vec![7u8; 2_000]).unwrap();
    blink.delete(probe_key(6)).unwrap();
    for round in 0..40u64 {
        let requests = (0..16u64)
            .map(|slot| {
                if slot % 4 == 0 {
                    put_request(&(0..16).map(|lane| (round * 131 + slot * 17 + lane * 41) % 900).collect::<Vec<_>>(), round)
                } else {
                    put_request(&[(round * 16 + slot) % 900], round)
                }
            })
            .collect::<Vec<_>>();
        for result in blink.apply_transaction_group(&requests).unwrap() {
            result.unwrap();
        }
    }
    let checkpoint = blink.checkpoint().unwrap();
    println!("blink_checkpoint_lsn={}", checkpoint.checkpoint_lsn.get());
    for index in 0..500u64 {
        blink
            .put(probe_key(index * 3 % 900), vec![(index + 1) as u8; 64])
            .unwrap();
    }
    let mut width_one_deltas = Vec::new();
    for index in 0..100u64 {
        let before = blink.wal_metrics().unwrap().unwrap().wal_bytes;
        blink
            .put(probe_key(index * 7 % 900), vec![0x5a; 64])
            .unwrap();
        width_one_deltas.push(blink.wal_metrics().unwrap().unwrap().wal_bytes - before);
    }
    width_one_deltas.sort_unstable();
    println!(
        "blink_width1_existing_update_wal_bytes min={} median={} max={}",
        width_one_deltas[0],
        width_one_deltas[width_one_deltas.len() / 2],
        width_one_deltas[width_one_deltas.len() - 1]
    );
    drop(blink.into_files());

    let mut btree = BTreeStore::open_with_wal(
        ProductionFile::open(directory.join("btree.db")).unwrap(),
        ProductionFile::open(directory.join("btree.wal")).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    for index in 0..1_500u64 {
        btree
            .put(probe_key(index % 600), vec![index as u8; 64])
            .unwrap();
    }
    btree.checkpoint().unwrap();
    for index in 0..300u64 {
        btree
            .put(probe_key(index * 5 % 600), vec![(index + 3) as u8; 64])
            .unwrap();
    }
    drop(btree.into_files());
}
