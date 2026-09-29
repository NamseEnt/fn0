use std::io::ErrorKind;

use dodb_core::{DocumentKey, Error, Result};
use dodb_storage::blink::BlinkStore;
use dodb_storage::btree::DatabaseConfig;
use dodb_storage::fault::FaultInjector;
use dodb_testkit::{CrashableFile, FaultAction, FaultPlan, FileOperation};

struct FailAt {
    point: &'static str,
}

impl FaultInjector for FailAt {
    fn hit(&mut self, point: &str) -> Result<()> {
        if point == self.point {
            return Err(Error::Io(std::io::Error::other(format!(
                "injected checkpoint failure at {point}"
            ))));
        }
        Ok(())
    }
}

fn value_after_crash(data: CrashableFile, wal: CrashableFile, key: &DocumentKey) {
    let mut data = data;
    let mut wal = wal;
    data.crash();
    wal.crash();
    let mut reopened =
        BlinkStore::open_with_logical_wal(data, wal, DatabaseConfig::default()).unwrap();
    assert_eq!(
        reopened.get(key).unwrap().value(),
        Some(&b"durable logical value"[..])
    );
    assert_eq!(reopened.get(key).unwrap().revision().get(), 1);
    assert!(reopened.check_invariants().unwrap().leaked_pages.is_empty());
}

#[test]
fn logical_checkpoint_crashes_recover_before_and_after_watermark_and_wal_reclaim() {
    for point in [
        "during_checkpoint_page_write",
        "before_checkpoint_data_sync",
        "after_checkpoint_data_sync",
        "after_checkpoint_metadata_sync",
        "before_wal_reset",
        "after_wal_truncate",
        "after_wal_reset_truncate_sync",
        "after_wal_reset_sync",
    ] {
        let key = DocumentKey::new(b"phase-j-crash".to_vec(), point.as_bytes().to_vec());
        let mut store = BlinkStore::open_with_logical_wal(
            CrashableFile::new(),
            CrashableFile::new(),
            DatabaseConfig::default(),
        )
        .unwrap();
        store
            .put(key.clone(), b"durable logical value".to_vec())
            .unwrap();
        store.set_fault_injector(FailAt { point });
        assert!(store.checkpoint().is_err(), "fault did not fire at {point}");
        let (data, wal) = store.into_files();
        value_after_crash(data, wal.unwrap(), &key);
    }
}

#[test]
fn logical_checkpoint_write_and_data_sync_io_failures_keep_wal_recoverable() {
    for (operation, action) in [
        (
            FileOperation::WriteAt,
            FaultAction::io(ErrorKind::StorageFull, "injected materializer ENOSPC"),
        ),
        (
            FileOperation::SyncData,
            FaultAction::io(ErrorKind::Other, "injected materializer data sync failure"),
        ),
    ] {
        let key = DocumentKey::new(
            b"phase-j-io-failure".to_vec(),
            format!("{operation:?}").into_bytes(),
        );
        let data_fault = match operation {
            FileOperation::WriteAt => FaultPlan::default().on_nth(operation, 4, action),
            FileOperation::SyncData => FaultPlan::default().on_next(operation, action),
            _ => unreachable!(),
        };
        let mut store = BlinkStore::open_with_logical_wal(
            CrashableFile::new().with_fault_plan(data_fault),
            CrashableFile::new(),
            DatabaseConfig::default(),
        )
        .unwrap();
        store
            .put(key.clone(), b"durable logical value".to_vec())
            .unwrap();
        assert!(store.checkpoint().is_err());
        let (data, wal) = store.into_files();
        value_after_crash(data, wal.unwrap(), &key);
    }
}
