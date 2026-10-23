use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dodb_core::{
    DocumentKey, Error, RevisionState, TransactionCondition, TransactionMutation,
    TransactionRequest,
};
use dodb_storage::blink::segmented_wal::SegmentedWalManifest;
use dodb_storage::{BlinkStore, DatabaseConfig, FaultInjector, ProductionFile};

type Store = BlinkStore<ProductionFile, ProductionFile>;

fn directory(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("dodb-bases-{name}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn open(path: &Path, coalesce: bool) -> Store {
    Store::open_segmented_base_path(path, DatabaseConfig::default(), 1, coalesce).unwrap()
}

fn key(number: u64) -> DocumentKey {
    DocumentKey::new("bases", number.to_be_bytes().to_vec())
}

fn manifest(path: &Path) -> SegmentedWalManifest {
    SegmentedWalManifest::read(&path.with_file_name("tenant.db.segments.manifest")).unwrap()
}

fn seed(store: &mut Store) -> BTreeMap<u64, RevisionState> {
    let mut expected = BTreeMap::new();
    for number in 0..192 {
        let value = vec![number as u8; 512];
        let revision = store.put(key(number), value.clone()).unwrap();
        expected.insert(number, RevisionState::Present { value, revision });
    }
    expected
}

fn verify(store: &mut Store, expected: &BTreeMap<u64, RevisionState>) {
    for (number, state) in expected {
        assert_eq!(store.get(&key(*number)).unwrap(), *state, "key {number}");
    }
    store.check_invariants().unwrap();
}

#[test]
fn checkpoints_rotations_and_reopen_preserve_page_delta_bases() {
    for coalesce in [false, true] {
        let directory = directory("delta-reuse");
        let path = directory.join("tenant.db");
        let mut store = open(&path, coalesce);
        let mut expected = seed(&mut store);
        let first = store.checkpoint().unwrap();
        assert!(first.base_bytes_written > 0);
        let cache_before = fs::read(&path).unwrap();
        for generation in 1..=12 {
            let before = store.wal_metrics().unwrap().unwrap().redo;
            let value = vec![generation as u8; 512];
            let revision = store.put(key(90), value.clone()).unwrap();
            expected.insert(90, RevisionState::Present { value, revision });
            let after = store.wal_metrics().unwrap().unwrap().redo;
            assert!(after.page_delta_records > before.page_delta_records);
            assert_eq!(after.image_first_touch, before.image_first_touch);
            store.checkpoint().unwrap();
            assert_eq!(fs::read(&path).unwrap(), cache_before);
        }
        drop(store);
        let mut reopened = open(&path, coalesce);
        verify(&mut reopened, &expected);
        assert_eq!(fs::read(&path).unwrap(), cache_before);
        let before = reopened.wal_metrics().unwrap().unwrap().redo;
        let value = vec![44; 512];
        let revision = reopened.put(key(90), value.clone()).unwrap();
        expected.insert(90, RevisionState::Present { value, revision });
        let after = reopened.wal_metrics().unwrap().unwrap().redo;
        assert!(after.page_delta_records > before.page_delta_records);
        assert_eq!(after.image_first_touch, before.image_first_touch);
        drop(reopened);
        let mut verified = open(&path, coalesce);
        verify(&mut verified, &expected);
        drop(verified);
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn immutable_packs_reuse_cold_pages_and_consolidate_with_bounded_generation_count() {
    let directory = directory("consolidation");
    let path = directory.join("tenant.db");
    let mut store = open(&path, true);
    let mut expected = seed(&mut store);
    store.checkpoint().unwrap();
    let initial_base = manifest(&path).bases.unwrap()[0].number;
    let mut consolidations = 0;
    let mut clean_page_overlap_checked = false;
    for generation in 0..24 {
        let number = (generation * 7) % 192;
        let value = vec![(generation + 1) as u8; 512];
        let revision = store.put(key(number), value.clone()).unwrap();
        expected.insert(number, RevisionState::Present { value, revision });
        let report = if !clean_page_overlap_checked && manifest(&path).bases.unwrap().len() == 8 {
            let snapshot = store.prepare_segmented_checkpoint().unwrap().unwrap();
            let value = vec![99; 512];
            let revision = store.put(key(191), value.clone()).unwrap();
            expected.insert(191, RevisionState::Present { value, revision });
            let report = store.complete_segmented_checkpoint(snapshot.run()).unwrap();
            assert_eq!(report.full_base_snapshots, 1);
            clean_page_overlap_checked = true;
            report
        } else {
            store.checkpoint().unwrap()
        };
        consolidations += report.full_base_snapshots;
        let current = manifest(&path);
        let bases = current.bases.unwrap();
        assert!(bases.len() <= 8);
        if generation == 0 {
            assert!(bases.iter().any(|base| base.number == initial_base));
            assert!(report.base_bytes_written < report.base_bytes_retained);
        }
        let actual_bytes = fs::read_dir(path.with_file_name("tenant.db.segments"))
            .unwrap()
            .map(|entry| entry.unwrap().metadata().unwrap().len())
            .sum::<u64>();
        assert!(actual_bytes <= 512 * 1024 * 1024);
    }
    assert!(consolidations > 0);
    assert!(clean_page_overlap_checked);
    drop(store);
    let mut reopened = open(&path, true);
    verify(&mut reopened, &expected);
    drop(reopened);
    fs::remove_dir_all(directory).unwrap();
}

struct FailAt(&'static str);

impl FaultInjector for FailAt {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == self.0 {
            return Err(Error::durability(format!("injected at {point}")));
        }
        Ok(())
    }
}

#[test]
fn checkpoint_failures_preserve_acknowledged_writes_and_allow_write_then_second_recovery() {
    for boundary in [
        "before_base_pack_create",
        "during_base_pack_page_write",
        "before_base_pack_sync",
        "after_base_pack_sync",
        "before_base_pack_directory_sync",
        "after_base_pack_directory_sync",
        "before_checkpoint_manifest_publish",
        "after_checkpoint_manifest_publish",
        "before_checkpoint_wal_unlink",
        "after_checkpoint_wal_unlink",
        "before_base_pack_unlink",
        "after_base_pack_unlink",
        "before_checkpoint_wal_directory_sync",
        "after_checkpoint_wal_directory_sync",
    ] {
        let directory = directory(boundary);
        let path = directory.join("tenant.db");
        let mut store = open(&path, true);
        let revision = store.put(key(0), vec![0; 512]).unwrap();
        store.checkpoint().unwrap();
        let next = store.put(key(0), vec![1; 512]).unwrap();
        store.set_boxed_fault_injector(Box::new(FailAt(boundary)));
        assert!(store.checkpoint().is_err(), "{boundary}");
        assert!(store.put(key(1), vec![2; 512]).is_err());
        drop(store);
        let mut recovered = open(&path, true);
        assert_eq!(
            recovered.get(&key(0)).unwrap(),
            RevisionState::Present {
                value: vec![1; 512],
                revision: next
            }
        );
        assert!(next > revision);
        let last = recovered.put(key(2), vec![3; 512]).unwrap();
        recovered.checkpoint().unwrap();
        drop(recovered);
        let mut verified = open(&path, true);
        assert_eq!(
            verified.get(&key(2)).unwrap(),
            RevisionState::Present {
                value: vec![3; 512],
                revision: last
            }
        );
        verified.check_invariants().unwrap();
        drop(verified);
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn listed_base_loss_corruption_identity_and_truncation_are_rejected() {
    for variant in ["missing", "truncated", "corrupt", "identity", "cache"] {
        let directory = directory(variant);
        let path = directory.join("tenant.db");
        let mut store = open(&path, true);
        let expected = seed(&mut store);
        store.checkpoint().unwrap();
        drop(store);
        let current = manifest(&path);
        let entry = &current.bases.as_ref().unwrap()[0];
        let pack = path
            .with_file_name("tenant.db.segments")
            .join(format!("base-{:020}.pages", entry.number));
        match variant {
            "missing" => fs::remove_file(&pack).unwrap(),
            "truncated" => fs::OpenOptions::new()
                .write(true)
                .open(&pack)
                .unwrap()
                .set_len(100)
                .unwrap(),
            "cache" => fs::write(&path, vec![0xaa; 4096]).unwrap(),
            _ => {
                let mut bytes = fs::read(&pack).unwrap();
                let offset = if variant == "identity" { 16 } else { 500 };
                bytes[offset] ^= 0xff;
                fs::write(&pack, bytes).unwrap();
            }
        }
        let result = Store::open_segmented_base_path(&path, DatabaseConfig::default(), 1, true);
        if variant == "cache" {
            let mut recovered = result.unwrap();
            verify(&mut recovered, &expected);
            assert_eq!(fs::read(&path).unwrap(), vec![0xaa; 4096]);
        } else {
            assert!(result.is_err(), "{variant}");
        }
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn empty_immutable_database_reopens_without_trusting_or_rewriting_the_cache() {
    let directory = directory("empty-cache");
    let path = directory.join("tenant.db");
    drop(open(&path, true));
    fs::write(&path, [0xaa]).unwrap();
    let mut store = open(&path, true);
    assert!(store.get(&key(0)).unwrap().is_missing());
    assert_eq!(fs::read(&path).unwrap(), [0xaa]);
    let revision = store.put(key(0), vec![1; 512]).unwrap();
    drop(store);
    let mut recovered = open(&path, true);
    assert_eq!(
        recovered.get(&key(0)).unwrap(),
        RevisionState::Present {
            value: vec![1; 512],
            revision,
        }
    );
    recovered.checkpoint().unwrap();
    drop(recovered);
    drop(open(&path, true));
    assert_eq!(fs::read(&path).unwrap(), [0xaa]);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn intermediate_pack_sync_failures_preserve_post_target_writes_and_allow_second_recovery() {
    for boundary in ["before_base_pack_chunk_sync", "after_base_pack_chunk_sync"] {
        let directory = directory(boundary);
        let path = directory.join("tenant.db");
        let mut store = open(&path, true);
        let value = vec![7; 9 * 1024 * 1024];
        let original = store.put(key(0), value.clone()).unwrap();
        let mut snapshot = store.prepare_segmented_checkpoint().unwrap().unwrap();
        let latest = store.put(key(1), vec![8; 512]).unwrap();
        snapshot.set_boxed_fault_injector(Box::new(FailAt(boundary)));
        assert!(store.complete_segmented_checkpoint(snapshot.run()).is_err());
        assert!(store.put(key(2), vec![9; 512]).is_err());
        drop(store);
        let mut recovered = open(&path, true);
        assert_eq!(
            recovered.get(&key(0)).unwrap(),
            RevisionState::Present {
                value,
                revision: original,
            }
        );
        assert_eq!(
            recovered.get(&key(1)).unwrap(),
            RevisionState::Present {
                value: vec![8; 512],
                revision: latest,
            }
        );
        let last = recovered.put(key(2), vec![9; 512]).unwrap();
        recovered.checkpoint().unwrap();
        drop(recovered);
        let mut verified = open(&path, true);
        assert_eq!(
            verified.get(&key(2)).unwrap(),
            RevisionState::Present {
                value: vec![9; 512],
                revision: last,
            }
        );
        verified.check_invariants().unwrap();
        drop(verified);
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn incomplete_transaction_tail_can_be_recovered_written_and_recovered_again() {
    let directory = directory("incomplete-tail");
    let path = directory.join("tenant.db");
    let mut store = open(&path, true);
    let mut expected = seed(&mut store);
    store.checkpoint().unwrap();
    let revision = store.put(key(90), vec![4; 512]).unwrap();
    expected.insert(
        90,
        RevisionState::Present {
            value: vec![4; 512],
            revision,
        },
    );
    store.put(key(92), vec![6; 512]).unwrap();
    let current = manifest(&path);
    drop(store);
    let active_path = path
        .with_file_name("tenant.db.segments")
        .join(format!("segment-{:020}.wal", current.active_segment));
    let bytes = fs::read(&active_path).unwrap();
    let mut frame_offset = 0;
    let mut final_frame_offset = 0;
    while frame_offset < bytes.len() {
        final_frame_offset = frame_offset;
        let frame_length = u32::from_le_bytes(
            bytes[frame_offset + 8..frame_offset + 12]
                .try_into()
                .unwrap(),
        ) as usize;
        frame_offset += frame_length;
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .open(&active_path)
        .unwrap();
    file.set_len(final_frame_offset as u64).unwrap();
    drop(file);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&active_path)
        .unwrap();
    file.write_all(&bytes[final_frame_offset..final_frame_offset + 10])
        .unwrap();
    file.sync_all().unwrap();
    let mut recovered = open(&path, true);
    verify(&mut recovered, &expected);
    let revision = recovered.put(key(91), vec![5; 512]).unwrap();
    expected.insert(
        91,
        RevisionState::Present {
            value: vec![5; 512],
            revision,
        },
    );
    drop(recovered);
    let mut verified = open(&path, true);
    verify(&mut verified, &expected);
    drop(verified);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn conditional_multikey_deletes_and_reuse_survive_base_replacement() {
    let directory = directory("conditions");
    let path = directory.join("tenant.db");
    let mut store = open(&path, true);
    seed(&mut store);
    store.checkpoint().unwrap();
    let prior = store.get(&key(1)).unwrap().revision();
    let transaction = store
        .transact(TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: key(1),
                expected_revision: prior,
            }],
            vec![
                TransactionMutation::Put {
                    key: key(1),
                    value: vec![9; 512],
                },
                TransactionMutation::Delete { key: key(2) },
            ],
        ))
        .unwrap();
    let expected_first = store.get(&key(1)).unwrap();
    let expected_second = store.get(&key(2)).unwrap();
    assert_eq!(expected_first.revision(), transaction.revision.unwrap());
    assert_eq!(expected_second.revision(), transaction.revision.unwrap());
    assert!(
        store
            .transact(TransactionRequest::new(
                vec![TransactionCondition::RevisionEquals {
                    key: key(1),
                    expected_revision: prior
                }],
                vec![TransactionMutation::Put {
                    key: key(3),
                    value: vec![99; 512]
                }],
            ))
            .is_err()
    );
    for generation in 0..4 {
        store
            .put(key(4), vec![generation as u8; 32 * 1024])
            .unwrap();
        store.checkpoint().unwrap();
        store.put(key(4), vec![generation as u8; 512]).unwrap();
        store.checkpoint().unwrap();
    }
    drop(store);
    let mut recovered = open(&path, true);
    assert_eq!(recovered.get(&key(1)).unwrap(), expected_first);
    assert_eq!(recovered.get(&key(2)).unwrap(), expected_second);
    recovered.check_invariants().unwrap();
    drop(recovered);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn experimental_and_existing_layouts_never_convert_implicitly() {
    let directory = directory("layout-rejection");
    let path = directory.join("tenant.db");
    drop(open(&path, true));
    assert!(Store::open_segmented_path(&path, DatabaseConfig::default(), 1).is_err());
    assert!(Store::open_path(&path, DatabaseConfig::default()).is_err());
    let legacy = directory.join("legacy.db");
    drop(Store::open_segmented_path(&legacy, DatabaseConfig::default(), 1).unwrap());
    assert!(Store::open_segmented_base_path(&legacy, DatabaseConfig::default(), 1, true).is_err());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn replayed_overflow_growth_is_included_in_the_next_checkpoint_before_wal_reclamation() {
    let directory = directory("replayed-growth");
    let path = directory.join("tenant.db");
    let mut store = open(&path, true);
    store.put(key(0), vec![0; 512]).unwrap();
    store.checkpoint().unwrap();
    let revision = store.put(key(1), vec![1; 32 * 1024]).unwrap();
    drop(store);
    let mut recovered = open(&path, true);
    recovered.put(key(2), vec![2; 32]).unwrap();
    recovered.checkpoint().unwrap();
    drop(recovered);
    let mut verified = open(&path, true);
    assert_eq!(
        verified.get(&key(1)).unwrap(),
        RevisionState::Present {
            value: vec![1; 32 * 1024],
            revision
        }
    );
    verified.check_invariants().unwrap();
    drop(verified);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn base_storage_and_pending_snapshot_are_reserved_before_accepting_more_writes() {
    let directory = directory("capacity");
    let path = directory.join("tenant.db");
    let mut store = open(&path, true);
    let limit = 128 * 1024;
    store.set_checkpoint_retention_limit(limit).unwrap();
    store.put(key(0), vec![0; 512]).unwrap();
    store.checkpoint().unwrap();
    assert!(matches!(
        store.put(key(1), vec![1; 128 * 1024]),
        Err(Error::Overloaded(_))
    ));
    assert!(store.get(&key(1)).unwrap().is_missing());
    store.put(key(2), vec![2; 512]).unwrap();
    let snapshot = store.prepare_segmented_checkpoint().unwrap().unwrap();
    assert!(matches!(
        store.put(key(3), vec![3; 128 * 1024]),
        Err(Error::Overloaded(_))
    ));
    store.complete_segmented_checkpoint(snapshot.run()).unwrap();
    store.put(key(4), vec![4; 512]).unwrap();
    store.checkpoint().unwrap();
    let actual_bytes = fs::read_dir(path.with_file_name("tenant.db.segments"))
        .unwrap()
        .map(|entry| entry.unwrap().metadata().unwrap().len())
        .sum::<u64>();
    assert!(actual_bytes <= limit);
    drop(store);
    let mut reopened = open(&path, true);
    assert!(reopened.get(&key(1)).unwrap().is_missing());
    assert!(reopened.get(&key(3)).unwrap().is_missing());
    assert_eq!(
        reopened.get(&key(4)).unwrap().value(),
        Some(vec![4; 512].as_slice())
    );
    drop(reopened);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn interrupted_reclamation_removes_valid_obsolete_files_and_preserves_unknown_artifacts() {
    let directory = directory("obsolete-cleanup");
    let path = directory.join("tenant.db");
    let segment_directory = path.with_file_name("tenant.db.segments");
    let mut store = open(&path, true);
    store.put(key(0), vec![0; 512]).unwrap();
    store.checkpoint().unwrap();
    let prior = manifest(&path);
    let prior_base = prior.bases.as_ref().unwrap()[0].number;
    let prior_active = prior.active_segment;
    store.put(key(0), vec![1; 512]).unwrap();
    store.set_boxed_fault_injector(Box::new(FailAt("after_checkpoint_manifest_publish")));
    assert!(store.checkpoint().is_err());
    drop(store);
    let old_base = segment_directory.join(format!("base-{prior_base:020}.pages"));
    let old_wal = segment_directory.join(format!("segment-{prior_active:020}.wal"));
    assert!(old_base.exists() && old_wal.exists());
    let unknown = segment_directory.join("base-00000000000000090000.pages");
    fs::write(&unknown, b"preserved invalid orphan").unwrap();
    let mut recovered = open(&path, true);
    assert_eq!(
        recovered.get(&key(0)).unwrap().value(),
        Some(vec![1; 512].as_slice())
    );
    assert!(!old_base.exists() && !old_wal.exists());
    assert_eq!(fs::read(&unknown).unwrap(), b"preserved invalid orphan");
    drop(recovered);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn live_primary_pages_and_extra_recovery_budget_are_accounted_separately() {
    let directory = directory("primary-budget");
    let path = directory.join("tenant.db");
    let mut store = open(&path, true);
    for number in 0..96 {
        store.put(key(number), vec![number as u8; 512]).unwrap();
    }
    let first = store.checkpoint().unwrap();
    let live_bytes = first.base_bytes_written;
    let extra_limit = live_bytes * 3 / 2;
    store.set_checkpoint_retention_limit(extra_limit).unwrap();
    for generation in 0..6 {
        store.put(key(50), vec![generation as u8; 512]).unwrap();
        let snapshot = store.prepare_segmented_checkpoint().unwrap().unwrap();
        store.put(key(51), vec![generation as u8; 512]).unwrap();
        store.complete_segmented_checkpoint(snapshot.run()).unwrap();
        let actual_bytes = fs::read_dir(path.with_file_name("tenant.db.segments"))
            .unwrap()
            .map(|entry| entry.unwrap().metadata().unwrap().len())
            .sum::<u64>();
        assert!(actual_bytes <= live_bytes + extra_limit);
    }
    store.checkpoint().unwrap();
    drop(store);
    let mut verified = open(&path, true);
    assert_eq!(
        verified.get(&key(50)).unwrap().value(),
        Some(vec![5; 512].as_slice())
    );
    assert_eq!(
        verified.get(&key(51)).unwrap().value(),
        Some(vec![5; 512].as_slice())
    );
    verified.check_invariants().unwrap();
    drop(verified);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn failed_public_write_leaves_an_incomplete_transaction_that_can_be_recovered_twice() {
    for boundary in ["during_page_delta_payload_write", "before_commit_record"] {
        let directory = directory(boundary);
        let path = directory.join("tenant.db");
        let mut store = open(&path, true);
        let prior = store.put(key(0), vec![0; 512]).unwrap();
        store.checkpoint().unwrap();
        store.set_boxed_fault_injector(Box::new(FailAt(boundary)));
        assert!(store.put(key(0), vec![1; 512]).is_err());
        drop(store);
        let mut recovered = open(&path, true);
        assert_eq!(
            recovered.get(&key(0)).unwrap(),
            RevisionState::Present {
                value: vec![0; 512],
                revision: prior
            }
        );
        let next = recovered.put(key(1), vec![2; 512]).unwrap();
        assert!(next > prior);
        recovered.checkpoint().unwrap();
        drop(recovered);
        let mut verified = open(&path, true);
        assert_eq!(
            verified.get(&key(0)).unwrap(),
            RevisionState::Present {
                value: vec![0; 512],
                revision: prior
            }
        );
        assert_eq!(
            verified.get(&key(1)).unwrap(),
            RevisionState::Present {
                value: vec![2; 512],
                revision: next
            }
        );
        verified.check_invariants().unwrap();
        drop(verified);
        fs::remove_dir_all(directory).unwrap();
    }
}
