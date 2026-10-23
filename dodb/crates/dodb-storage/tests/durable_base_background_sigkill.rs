use std::fs;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dodb_core::{DocumentKey, Revision, RevisionState};
use dodb_storage::{BlinkStore, DatabaseConfig, FaultInjector, ProductionFile};

const CHILD_DIRECTORY: &str = "DODB_BASE_BACKGROUND_CHILD_DIRECTORY";
const CHILD_BOUNDARY: &str = "DODB_BASE_BACKGROUND_CHILD_BOUNDARY";

fn key(sequence: u64) -> DocumentKey {
    DocumentKey::new(
        b"checkpoint-sigkill".to_vec(),
        sequence.to_be_bytes().to_vec(),
    )
}

fn expected_value(sequence: u64) -> Vec<u8> {
    vec![
        sequence as u8;
        if sequence == 20 {
            9 * 1024 * 1024
        } else {
            8192
        }
    ]
}

fn open(path: &Path) -> BlinkStore<ProductionFile, ProductionFile> {
    BlinkStore::<ProductionFile, ProductionFile>::open_segmented_base_path(
        path,
        DatabaseConfig::default(),
        1,
        true,
    )
    .unwrap()
}

struct PauseAt {
    boundary: String,
    ready_path: PathBuf,
}

impl FaultInjector for PauseAt {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == self.boundary {
            let temporary = self.ready_path.with_extension("pending");
            fs::write(&temporary, point)?;
            fs::rename(temporary, &self.ready_path)?;
            loop {
                thread::park();
            }
        }
        Ok(())
    }
}

struct ChildGuard(Child);

struct FailAt(String);

impl FaultInjector for FailAt {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == self.0 {
            return Err(dodb_core::Error::durability(format!("injected {point}")));
        }
        Ok(())
    }
}

#[test]
fn failed_background_phases_preserve_post_target_commits_and_allow_recovery() {
    let boundaries = [
        "before_checkpoint_data_flush",
        "during_checkpoint_page_write",
        "after_checkpoint_page_write",
        "before_checkpoint_data_sync",
        "after_checkpoint_data_sync",
        "before_base_pack_sync",
        "after_base_pack_directory_sync",
        "before_checkpoint_manifest_publish",
        "before_wal_manifest_file_sync",
        "after_wal_manifest_file_sync",
        "before_wal_manifest_directory_sync",
        "after_wal_manifest_directory_sync",
        "after_checkpoint_manifest_publish",
        "before_checkpoint_wal_unlink",
        "after_checkpoint_wal_unlink",
        "before_checkpoint_wal_directory_sync",
        "after_checkpoint_wal_directory_sync",
    ];
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "dodb-base-background-faults-{}-{unique}",
        std::process::id()
    ));
    for boundary in boundaries {
        let directory = root.join(boundary);
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("tenant.db");
        let mut store = open(&path);
        store.put(key(1), vec![1; 8192]).unwrap();
        let mut snapshot = store.prepare_segmented_checkpoint().unwrap().unwrap();
        let latest = store.put(key(1), vec![2; 8192]).unwrap();
        let second = store.put(key(2), vec![3; 8192]).unwrap();
        snapshot.set_boxed_fault_injector(Box::new(FailAt(boundary.to_owned())));
        let output = snapshot.run();
        store.set_boxed_fault_injector(Box::new(FailAt(boundary.to_owned())));
        assert!(
            store.complete_segmented_checkpoint(output).is_err(),
            "{boundary}"
        );
        assert!(store.put(key(3), vec![4; 8192]).is_err(), "{boundary}");
        drop(store);
        let mut recovered = open(&path);
        assert_eq!(
            recovered.get(&key(1)).unwrap(),
            RevisionState::Present {
                value: vec![2; 8192],
                revision: latest
            },
            "{boundary}"
        );
        assert_eq!(
            recovered.get(&key(2)).unwrap(),
            RevisionState::Present {
                value: vec![3; 8192],
                revision: second
            },
            "{boundary}"
        );
        recovered.put(key(3), vec![4; 8192]).unwrap();
        let snapshot = recovered.prepare_segmented_checkpoint().unwrap().unwrap();
        recovered
            .complete_segmented_checkpoint(snapshot.run())
            .unwrap();
        recovered.check_invariants().unwrap();
        drop(recovered);
        let mut verified = open(&path);
        assert_eq!(
            verified.get(&key(1)).unwrap().value(),
            Some(vec![2; 8192].as_slice())
        );
        assert_eq!(
            verified.get(&key(3)).unwrap().value(),
            Some(vec![4; 8192].as_slice())
        );
        drop(verified);
        fs::remove_dir_all(directory).unwrap();
    }
    fs::remove_dir(root).unwrap();
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn background_checkpoint_child() {
    let Some(directory) = std::env::var_os(CHILD_DIRECTORY).map(PathBuf::from) else {
        return;
    };
    let boundary = std::env::var(CHILD_BOUNDARY).unwrap();
    let mut store = open(&directory.join("tenant.db"));
    for sequence in 1..=5 {
        store.put(key(sequence), vec![0; 4096]).unwrap();
    }
    store.checkpoint().unwrap();
    let mut acknowledgements = fs::File::create(directory.join("acknowledged.csv")).unwrap();
    for sequence in 1..=5 {
        let revision = store.put(key(sequence), expected_value(sequence)).unwrap();
        writeln!(acknowledgements, "{sequence},{revision}").unwrap();
        acknowledgements.sync_all().unwrap();
    }
    if boundary.contains("chunk_sync") {
        let revision = store.put(key(20), expected_value(20)).unwrap();
        writeln!(acknowledgements, "20,{revision}").unwrap();
        acknowledgements.sync_all().unwrap();
    }
    let mut snapshot = store.prepare_segmented_checkpoint().unwrap().unwrap();
    for sequence in 6..=10 {
        let revision = store.put(key(sequence), expected_value(sequence)).unwrap();
        writeln!(acknowledgements, "{sequence},{revision}").unwrap();
        acknowledgements.sync_all().unwrap();
    }
    snapshot.set_boxed_fault_injector(Box::new(PauseAt {
        boundary: boundary.clone(),
        ready_path: directory.join("ready"),
    }));
    let worker = thread::spawn(move || snapshot.run());
    store.set_boxed_fault_injector(Box::new(PauseAt {
        boundary,
        ready_path: directory.join("ready"),
    }));
    store
        .complete_segmented_checkpoint(worker.join().unwrap())
        .unwrap();
    panic!("checkpoint did not reach the requested boundary");
}

#[test]
fn sigkill_background_boundaries_recover_post_target_writes_and_checkpoint_again() {
    let boundaries = [
        "before_base_pack_chunk_sync",
        "after_base_pack_chunk_sync",
        "after_checkpoint_page_write",
        "before_checkpoint_data_sync",
        "after_checkpoint_data_sync",
        "before_base_pack_sync",
        "after_base_pack_directory_sync",
        "before_checkpoint_manifest_publish",
        "after_checkpoint_manifest_publish",
        "before_checkpoint_wal_unlink",
        "after_checkpoint_wal_unlink",
        "before_checkpoint_wal_directory_sync",
        "after_checkpoint_wal_directory_sync",
    ];
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "dodb-base-background-checkpoint-sigkill-{}-{unique}",
        std::process::id()
    ));
    for boundary in boundaries {
        let directory = root.join(boundary);
        fs::create_dir_all(&directory).unwrap();
        let mut child = ChildGuard(
            Command::new(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("background_checkpoint_child")
                .arg("--nocapture")
                .env(CHILD_DIRECTORY, &directory)
                .env(CHILD_BOUNDARY, boundary)
                .spawn()
                .unwrap(),
        );
        let ready_path = directory.join("ready");
        let deadline = Instant::now() + Duration::from_secs(30);
        while !ready_path.exists() && Instant::now() < deadline {
            if let Some(status) = child.0.try_wait().unwrap() {
                panic!("checkpoint child exited at {boundary}: {status}");
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            ready_path.exists(),
            "checkpoint child timed out at {boundary}"
        );
        assert_eq!(fs::read_to_string(ready_path).unwrap(), boundary);
        child.0.kill().unwrap();
        assert_eq!(child.0.wait().unwrap().signal(), Some(9));
        let expected = fs::read_to_string(directory.join("acknowledged.csv"))
            .unwrap()
            .lines()
            .map(|line| {
                let (sequence, revision) = line.split_once(',').unwrap();
                (
                    sequence.parse::<u64>().unwrap(),
                    Revision::new(revision.parse().unwrap()),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            expected.len(),
            10 + usize::from(boundary.contains("chunk_sync"))
        );
        let path = directory.join("tenant.db");
        let mut recovered = open(&path);
        for (sequence, revision) in &expected {
            assert_eq!(
                recovered.get(&key(*sequence)).unwrap(),
                RevisionState::Present {
                    value: expected_value(*sequence),
                    revision: *revision,
                },
                "{boundary}"
            );
        }
        let next = recovered.put(key(11), expected_value(11)).unwrap();
        recovered.checkpoint().unwrap();
        recovered.check_invariants().unwrap();
        drop(recovered);
        let mut verified = open(&path);
        assert_eq!(
            verified.get(&key(11)).unwrap(),
            RevisionState::Present {
                value: expected_value(11),
                revision: next
            }
        );
        for (sequence, revision) in &expected {
            assert_eq!(
                verified.get(&key(*sequence)).unwrap(),
                RevisionState::Present {
                    value: expected_value(*sequence),
                    revision: *revision
                }
            );
        }
        drop(verified);
        fs::remove_dir_all(&directory).unwrap();
    }
    fs::remove_dir(root).unwrap();
}
