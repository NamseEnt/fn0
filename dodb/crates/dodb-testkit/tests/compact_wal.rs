use std::io::ErrorKind;

use dodb_core::{DocumentKey, TransactionMutation, TransactionRequest};
use dodb_storage::{BTreeStore, DatabaseConfig};
use dodb_testkit::{CrashInjector, CrashableFile, FaultAction, FaultPlan, FileOperation};

fn key(sort_key: &[u8]) -> DocumentKey {
    DocumentKey::new(b"compact-faults", sort_key)
}

fn open(data: CrashableFile, wal: CrashableFile) -> BTreeStore<CrashableFile, CrashableFile> {
    BTreeStore::open_with_compact_wal(data, wal, DatabaseConfig::default()).unwrap()
}

fn seeded_files() -> (Vec<u8>, Vec<u8>) {
    let mut store = open(CrashableFile::new(), CrashableFile::new());
    store.put(key(b"first"), vec![0x11; 512]).unwrap();
    store.put(key(b"second"), vec![0x22; 512]).unwrap();
    let (data, wal) = store.into_files().unwrap();
    (data.durable_bytes().to_vec(), wal.durable_bytes().to_vec())
}

fn requests() -> [TransactionRequest; 2] {
    [
        TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: key(b"first"),
                value: vec![0x33; 512],
            }],
        ),
        TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: key(b"second"),
                value: vec![0x44; 512],
            }],
        ),
    ]
}

fn crash_and_reopen(
    store: BTreeStore<CrashableFile, CrashableFile>,
) -> BTreeStore<CrashableFile, CrashableFile> {
    let (mut data, mut wal) = store.into_files().unwrap();
    data.crash();
    wal.crash();
    open(data, wal)
}

#[test]
fn compact_wal_crash_boundaries_preserve_group_durability() {
    let (data_image, wal_image) = seeded_files();
    for point in [
        "before_wal_append",
        "during_wal_header_write",
        "during_wal_payload_write",
        "after_page_delta_record",
        "after_page_images_written",
        "before_commit_record",
        "after_commit_record_write",
        "before_wal_sync",
        "during_wal_sync",
        "after_wal_sync",
        "before_publish",
        "after_publish",
    ] {
        let mut store = open(
            CrashableFile::from_durable(data_image.clone()),
            CrashableFile::from_durable(wal_image.clone()),
        );
        store.set_fault_injector(CrashInjector::at(point, 1));
        assert!(
            store.apply_transaction_group(&requests()).is_err(),
            "{point}"
        );
        let mut reopened = crash_and_reopen(store);
        let durable = matches!(point, "after_wal_sync" | "before_publish" | "after_publish");
        assert_eq!(
            reopened.get(&key(b"first")).unwrap().value(),
            Some(vec![if durable { 0x33 } else { 0x11 }; 512].as_slice()),
            "{point}"
        );
        assert_eq!(
            reopened.get(&key(b"second")).unwrap().value(),
            Some(vec![if durable { 0x44 } else { 0x22 }; 512].as_slice()),
            "{point}"
        );
        reopened.check_invariants().unwrap();
    }
}

#[test]
fn compact_wal_uncertain_sync_recovers_only_a_complete_commit_prefix() {
    let (data_image, wal_image) = seeded_files();
    let mut successful = open(
        CrashableFile::from_durable(data_image.clone()),
        CrashableFile::from_durable(wal_image.clone()),
    );
    successful
        .apply_transaction_group(&requests())
        .unwrap()
        .into_iter()
        .collect::<dodb_core::Result<Vec<_>>>()
        .unwrap();
    let final_length = successful.wal_metrics().unwrap().unwrap().wal_bytes as usize;
    for persisted_length in wal_image.len()..=final_length {
        let fault = FaultPlan::default().on_next(
            FileOperation::SyncData,
            FaultAction::SyncPersistPrefixThenIo {
                length: persisted_length,
                kind: ErrorKind::Other,
                message: "uncertain compact WAL sync".to_owned(),
            },
        );
        let mut store = open(
            CrashableFile::from_durable(data_image.clone()),
            CrashableFile::from_durable(wal_image.clone()).with_fault_plan(fault),
        );
        assert!(store.apply_transaction_group(&requests()).is_err());
        let mut reopened = crash_and_reopen(store);
        let first = reopened.get(&key(b"first")).unwrap();
        let second = reopened.get(&key(b"second")).unwrap();
        let first_new = first.value() == Some(vec![0x33; 512].as_slice());
        let second_new = second.value() == Some(vec![0x44; 512].as_slice());
        assert!(first_new || first.value() == Some(vec![0x11; 512].as_slice()));
        assert!(second_new || second.value() == Some(vec![0x22; 512].as_slice()));
        assert!(
            !second_new || first_new,
            "recovered a non-prefix at {persisted_length}"
        );
        if persisted_length == final_length {
            assert!(first_new && second_new);
        }
        reopened.check_invariants().unwrap();
    }
}

#[test]
fn compact_wal_checkpoint_faults_preserve_values_and_next_commit() {
    let (data_image, wal_image) = seeded_files();
    for point in [
        "before_checkpoint_gate",
        "before_data_page_write",
        "during_data_page_write",
        "during_data_file_sync",
        "before_checkpoint_superblock_write",
        "before_checkpoint_metadata_sync",
        "after_checkpoint_metadata_sync",
        "during_wal_truncate",
        "after_wal_reset_truncate_sync",
        "after_wal_reset_write",
        "after_wal_reset_sync",
        "before_checkpoint_complete",
    ] {
        let mut store = open(
            CrashableFile::from_durable(data_image.clone()),
            CrashableFile::from_durable(wal_image.clone()),
        );
        store
            .apply_transaction_group(&requests())
            .unwrap()
            .into_iter()
            .collect::<dodb_core::Result<Vec<_>>>()
            .unwrap();
        store.set_fault_injector(CrashInjector::at(point, 1));
        assert!(store.checkpoint().is_err(), "{point}");
        let mut reopened = crash_and_reopen(store);
        assert_eq!(
            reopened.get(&key(b"first")).unwrap().value(),
            Some(vec![0x33; 512].as_slice()),
            "{point}"
        );
        assert_eq!(
            reopened.get(&key(b"second")).unwrap().value(),
            Some(vec![0x44; 512].as_slice()),
            "{point}"
        );
        reopened.put(key(b"first"), vec![0x55; 512]).unwrap();
        let mut reopened = crash_and_reopen(reopened);
        assert_eq!(
            reopened.get(&key(b"first")).unwrap().value(),
            Some(vec![0x55; 512].as_slice()),
            "{point}"
        );
        reopened.check_invariants().unwrap();
    }
}
