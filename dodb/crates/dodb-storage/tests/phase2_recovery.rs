use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use dodb_core::{DocumentKey, Error, RevisionState};
use dodb_storage::{BTreeStore, DatabaseConfig, ProductionFile};

struct FailOnce {
    point: &'static str,
    fired: bool,
}

impl dodb_storage::FaultInjector for FailOnce {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if !self.fired && point == self.point {
            self.fired = true;
            return Err(Error::recovery(format!("injected failure at {point}")));
        }
        Ok(())
    }
}

fn paths(iteration: usize) -> (PathBuf, PathBuf, PathBuf) {
    let prefix = std::env::temp_dir().join(format!(
        "dodb-phase2-subprocess-{}-{iteration}",
        std::process::id()
    ));
    (
        prefix.with_extension("db"),
        prefix.with_extension("wal"),
        prefix.with_extension("ready"),
    )
}

#[test]
fn successful_write_survives_sigkill_and_restart() {
    let executable = std::env::var_os("CARGO_BIN_EXE_phase2-crash-child")
        .expect("Cargo should expose the crash child binary to integration tests");
    for iteration in 0..1_024 {
        let (data_path, wal_path, marker_path) = paths(iteration);
        let _ = std::fs::remove_file(&data_path);
        let _ = std::fs::remove_file(&wal_path);
        let _ = std::fs::remove_file(&marker_path);
        let mut child = Command::new(&executable)
            .arg(&data_path)
            .arg(&wal_path)
            .arg(&marker_path)
            .spawn()
            .expect("crash child should spawn");
        for _ in 0..200 {
            if marker_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(marker_path.exists(), "child did not report success");
        child.kill().expect("SIGKILL should terminate child");
        let status = child.wait().expect("child status should be available");
        assert!(!status.success(), "child must be terminated abruptly");

        let mut store = BTreeStore::open_with_wal(
            ProductionFile::open(&data_path).unwrap(),
            ProductionFile::open(&wal_path).unwrap(),
            DatabaseConfig::default(),
        )
        .unwrap();
        assert_eq!(
            store
                .get(&DocumentKey::new(b"subprocess".to_vec(), b"key".to_vec()))
                .unwrap()
                .value(),
            Some(&b"value"[..])
        );
        store.check_invariants().unwrap();
        let _ = std::fs::remove_file(data_path);
        let _ = std::fs::remove_file(wal_path);
        let _ = std::fs::remove_file(marker_path);
    }
}

#[test]
fn checkpoint_boundaries_survive_sigkill_and_restart() {
    let executable = std::env::var_os("CARGO_BIN_EXE_phase2-crash-child")
        .expect("Cargo should expose the crash child binary to integration tests");
    let checkpoint_points = [
        "before_checkpoint_data_flush",
        "before_data_page_write",
        "during_data_page_write",
        "before_data_file_sync",
        "before_checkpoint_superblock_write",
        "before_checkpoint_metadata_sync",
        "before_wal_reset",
        "during_wal_truncate",
        "before_wal_reset_truncate_sync",
        "before_wal_reinitialization",
        "during_wal_reinitialization",
        "before_wal_reset_sync",
        "during_wal_reset_sync",
    ];
    for (iteration, checkpoint_point) in checkpoint_points.into_iter().enumerate() {
        let (data_path, wal_path, marker_path) = paths(20_000 + iteration);
        let _ = std::fs::remove_file(&data_path);
        let _ = std::fs::remove_file(&wal_path);
        let _ = std::fs::remove_file(&marker_path);
        let mut child = Command::new(&executable)
            .arg(&data_path)
            .arg(&wal_path)
            .arg(&marker_path)
            .arg(checkpoint_point)
            .spawn()
            .expect("checkpoint crash child should spawn");
        for _ in 0..200 {
            if marker_path.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            marker_path.exists(),
            "child did not reach checkpoint boundary {checkpoint_point}"
        );
        child.kill().expect("SIGKILL should terminate child");
        let status = child.wait().expect("child status should be available");
        assert!(!status.success(), "child must be terminated abruptly");

        let mut store = BTreeStore::open_with_wal(
            ProductionFile::open(&data_path).unwrap(),
            ProductionFile::open(&wal_path).unwrap(),
            DatabaseConfig::default(),
        )
        .unwrap();
        assert_eq!(
            store
                .get(&DocumentKey::new(b"subprocess".to_vec(), b"key".to_vec()))
                .unwrap()
                .value(),
            Some(&b"value"[..])
        );
        store.check_invariants().unwrap();
        let _ = std::fs::remove_file(data_path);
        let _ = std::fs::remove_file(wal_path);
        let _ = std::fs::remove_file(marker_path);
    }
}

#[test]
fn wrong_wal_identity_is_rejected() {
    let data_path =
        std::env::temp_dir().join(format!("dodb-phase2-identity-{}.db", std::process::id()));
    let wal_path = data_path.with_extension("wal");
    let config = DatabaseConfig {
        database_uuid: [1; 16],
        ..DatabaseConfig::default()
    };
    let _store = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        config,
    )
    .unwrap();
    drop(_store);
    let wrong = DatabaseConfig {
        database_uuid: [2; 16],
        ..DatabaseConfig::default()
    };
    let result = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        wrong,
    );
    assert!(matches!(result, Err(Error::Corruption(_))));
    let _ = std::fs::remove_file(data_path);
    let _ = std::fs::remove_file(wal_path);
}

#[test]
fn wrong_identity_does_not_initialize_an_empty_wal_for_an_existing_database() {
    let data_path = std::env::temp_dir().join(format!(
        "dodb-phase2-empty-wal-identity-{}.db",
        std::process::id()
    ));
    let wal_path = data_path.with_extension("wal");
    let valid = DatabaseConfig {
        database_uuid: [7; 16],
        ..DatabaseConfig::default()
    };
    let store = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        valid.clone(),
    )
    .unwrap();
    drop(store);
    std::fs::remove_file(&wal_path).unwrap();
    let database_before = std::fs::read(&data_path).unwrap();

    let wrong = DatabaseConfig {
        database_uuid: [8; 16],
        ..valid.clone()
    };
    let result = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        wrong,
    );
    assert!(matches!(result, Err(Error::Corruption(_))));
    assert_eq!(std::fs::read(&data_path).unwrap(), database_before);
    assert_eq!(std::fs::metadata(&wal_path).unwrap().len(), 0);

    let mut reopened = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        valid,
    )
    .unwrap();
    reopened.check_invariants().unwrap();
    let _ = std::fs::remove_file(data_path);
    let _ = std::fs::remove_file(wal_path);
}

#[test]
fn repaired_database_page_uses_the_committed_wal_image() {
    let data_path =
        std::env::temp_dir().join(format!("dodb-phase2-page-repair-{}.db", std::process::id()));
    let wal_path = data_path.with_extension("wal");
    let key = DocumentKey::new(b"repair".to_vec(), b"page".to_vec());
    let mut store = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    store.put(key.clone(), b"repair me").unwrap();
    store.flush().unwrap();
    drop(store);

    let mut bytes = std::fs::read(&data_path).unwrap();
    bytes[2 * dodb_storage::PAGE_SIZE + 80] ^= 1;
    std::fs::write(&data_path, bytes).unwrap();
    let mut reopened = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    assert_eq!(reopened.get(&key).unwrap().value(), Some(&b"repair me"[..]));
    assert!(!matches!(
        reopened.get(&key).unwrap(),
        RevisionState::Missing { .. }
    ));
    let _ = std::fs::remove_file(data_path);
    let _ = std::fs::remove_file(wal_path);
}

#[test]
fn recovery_page_write_failure_is_recoverable_on_retry() {
    let data_path = std::env::temp_dir().join(format!(
        "dodb-phase2-recovery-fault-{}.db",
        std::process::id()
    ));
    let wal_path = data_path.with_extension("wal");
    let key = DocumentKey::new(b"recovery".to_vec(), b"retry".to_vec());
    let mut store = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    store.put(key.clone(), b"retry me").unwrap();
    store.flush().unwrap();
    drop(store);

    let mut bytes = std::fs::read(&data_path).unwrap();
    bytes[2 * dodb_storage::PAGE_SIZE + 80] ^= 1;
    std::fs::write(&data_path, bytes).unwrap();
    let result = BTreeStore::open_with_wal_and_fault_injector(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
        FailOnce {
            point: "during_recovery_page_write",
            fired: false,
        },
    );
    assert!(matches!(result, Err(Error::RecoveryFailure(_))));

    let mut reopened = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    assert_eq!(reopened.get(&key).unwrap().value(), Some(&b"retry me"[..]));
    reopened.check_invariants().unwrap();
    let _ = std::fs::remove_file(data_path);
    let _ = std::fs::remove_file(wal_path);
}

#[test]
fn corrupted_wal_middle_is_not_silently_skipped() {
    let data_path = std::env::temp_dir().join(format!(
        "dodb-phase2-wal-corruption-{}.db",
        std::process::id()
    ));
    let wal_path = data_path.with_extension("wal");
    let mut store = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    store
        .put(
            DocumentKey::new(b"corrupt".to_vec(), b"middle".to_vec()),
            b"value".to_vec(),
        )
        .unwrap();
    drop(store);

    let mut wal_bytes = std::fs::read(&wal_path).unwrap();
    let first_commit_payload_byte = dodb_storage::WAL_HEADER_SIZE * 3 + 8;
    wal_bytes[first_commit_payload_byte] ^= 1;
    std::fs::write(&wal_path, wal_bytes).unwrap();
    let result = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    );
    assert!(matches!(result, Err(Error::Corruption(_))));
    let _ = std::fs::remove_file(data_path);
    let _ = std::fs::remove_file(wal_path);
}

#[test]
fn corrupt_initial_data_without_a_committed_wal_image_fails_open() {
    let data_path = std::env::temp_dir().join(format!(
        "dodb-phase2-unprotected-page-{}.db",
        std::process::id()
    ));
    let wal_path = data_path.with_extension("wal");
    let mut store = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    )
    .unwrap();
    store.flush().unwrap();
    drop(store);
    let mut bytes = std::fs::read(&data_path).unwrap();
    bytes[2 * dodb_storage::PAGE_SIZE + 80] ^= 1;
    std::fs::write(&data_path, bytes).unwrap();
    let result = BTreeStore::open_with_wal(
        ProductionFile::open(&data_path).unwrap(),
        ProductionFile::open(&wal_path).unwrap(),
        DatabaseConfig::default(),
    );
    assert!(matches!(result, Err(Error::Corruption(_))));
    let _ = std::fs::remove_file(data_path);
    let _ = std::fs::remove_file(wal_path);
}
