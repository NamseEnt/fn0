use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::time::Instant;

use dodb_core::{
    DocumentKey, Lsn, Revision, RevisionState, TransactionMutation, TransactionRequest,
};
use dodb_storage::{BTreeStore, DatabaseConfig, WalIdentity, WalLog};
use dodb_testkit::{CrashInjector, CrashableFile, FaultAction, FaultPlan, FileOperation};

fn config() -> DatabaseConfig {
    DatabaseConfig::default().with_cache_capacity(0)
}

fn key(name: &[u8]) -> DocumentKey {
    DocumentKey::new(b"dst", name)
}

fn group_requests() -> [TransactionRequest; 2] {
    [
        TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: key(b"A"),
                value: b"a".to_vec(),
            }],
        ),
        TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: key(b"B"),
                value: b"b".to_vec(),
            }],
        ),
    ]
}

fn identity(config: &DatabaseConfig) -> WalIdentity {
    WalIdentity::new(
        config.database_uuid,
        config.tenant_id,
        config.shard_id,
        config.shard_epoch,
    )
}

fn open_from_images(data: &[u8], wal: &[u8]) -> BTreeStore<CrashableFile, CrashableFile> {
    BTreeStore::open_with_wal(
        CrashableFile::from_durable(data.to_vec()),
        CrashableFile::from_durable(wal.to_vec()),
        config(),
    )
    .expect("database should reopen from candidate images")
}

fn actual_stage(
    store: &mut BTreeStore<CrashableFile, CrashableFile>,
    context: &str,
) -> Vec<(u8, Revision)> {
    let a = store.get(&key(b"A")).unwrap();
    let b = store.get(&key(b"B")).unwrap();
    let mut actual = Vec::new();
    if let RevisionState::Present { value, revision } = a {
        assert_eq!(value, b"a", "{context}");
        actual.push((b'A', revision));
    } else {
        assert_eq!(a, RevisionState::missing(Revision::ZERO), "{context}");
    }
    if let RevisionState::Present { value, revision } = b {
        assert_eq!(value, b"b", "{context}");
        actual.push((b'B', revision));
    } else {
        assert_eq!(b, RevisionState::missing(Revision::ZERO), "{context}");
    }
    assert!(
        actual.len() != 1 || actual[0].0 == b'A',
        "{context}: B recovered without A"
    );
    actual
}

fn assert_stage_matches_wal(
    store: &mut BTreeStore<CrashableFile, CrashableFile>,
    batches: &[(u8, Lsn)],
    context: &str,
) {
    let actual = actual_stage(store, context);
    let expected: Vec<_> = batches
        .iter()
        .map(|(label, lsn)| (*label, Revision::from(*lsn)))
        .collect();
    assert_eq!(actual, expected, "{context}");
    store.check_invariants().unwrap();
}

#[test]
fn every_unsynced_wal_prefix_recovers_only_a_logical_commit_prefix() {
    let started = Instant::now();
    let initial =
        BTreeStore::open_with_wal(CrashableFile::new(), CrashableFile::new(), config()).unwrap();
    let (initial_data, initial_wal) = initial.into_files().unwrap();
    let baseline_data = initial_data.durable_bytes().to_vec();
    let baseline_wal = initial_wal.durable_bytes().to_vec();
    let baseline_len = baseline_wal.len();
    assert_eq!(baseline_len, initial_wal.volatile_bytes().len());

    let mut store = BTreeStore::open_with_wal(
        CrashableFile::from_durable(baseline_data.clone()),
        CrashableFile::from_durable(baseline_wal.clone()),
        config(),
    )
    .unwrap();
    store.set_fault_injector(CrashInjector::at("before_wal_sync", 1));
    assert!(store.apply_transaction_group(&group_requests()).is_err());
    let (data_after_group, wal_after_group) = store.into_files().unwrap();
    let full_volatile_wal = wal_after_group.volatile_bytes().to_vec();
    let full_len = full_volatile_wal.len();
    assert!(full_len > baseline_len);
    assert_eq!(wal_after_group.durable_bytes(), baseline_wal);
    assert_eq!(data_after_group.durable_bytes(), baseline_data);
    let full_store = WalLog::open(
        CrashableFile::from_durable(full_volatile_wal.clone()),
        identity(&config()),
    )
    .unwrap();
    assert_eq!(full_store.committed_batches().len(), 2);
    let full_commit_lsns: Vec<_> = full_store
        .committed_batches()
        .iter()
        .map(|batch| batch.commit_lsn)
        .collect();
    drop(full_store);

    let mut previous_count = 0;
    let mut previous_next_lsn = Lsn::ZERO;
    let mut complete_uncommitted_frame_advanced_lsn = false;
    let mut stage_counts = [0usize; 3];
    let mut first_a_offset = None;
    let mut first_ab_offset = None;
    let mut a_only_prefix = None;
    let mut semantic_boundary_prefixes = BTreeSet::new();

    for prefix_len in baseline_len..=full_len {
        let prefix = &full_volatile_wal[..prefix_len];
        let wal = WalLog::open(
            CrashableFile::from_durable(prefix.to_vec()),
            identity(&config()),
        )
        .unwrap_or_else(|error| panic!("WalLog open failed at prefix {prefix_len}: {error}"));
        let commits = wal.committed_batches().len();
        let next_lsn = wal.next_lsn();
        assert!(commits <= 2, "prefix_len={prefix_len} count={commits}");
        assert!(
            commits >= previous_count,
            "committed count decreased at {prefix_len}"
        );
        assert!(
            next_lsn >= previous_next_lsn,
            "next_lsn decreased at {prefix_len}"
        );
        assert_eq!(wal.history_start_lsn(), Lsn::ZERO);
        if prefix_len > baseline_len && commits == previous_count && next_lsn > previous_next_lsn {
            complete_uncommitted_frame_advanced_lsn = true;
        }
        let expected_batches: Vec<_> = wal
            .committed_batches()
            .iter()
            .enumerate()
            .map(|(commit_index, batch)| {
                (
                    if commit_index == 0 { b'A' } else { b'B' },
                    batch.commit_lsn,
                )
            })
            .collect();
        assert!(commits != 1 || expected_batches[0].0 == b'A');
        stage_counts[commits] += 1;
        if commits == 1 {
            first_a_offset.get_or_insert(prefix_len);
            a_only_prefix.get_or_insert(prefix_len);
        }
        if commits == 2 {
            first_ab_offset.get_or_insert(prefix_len);
        }
        if commits != previous_count {
            for nearby in [
                prefix_len.saturating_sub(1),
                prefix_len,
                prefix_len.saturating_add(1),
            ] {
                if (baseline_len..=full_len).contains(&nearby) {
                    semantic_boundary_prefixes.insert(nearby);
                }
            }
        }
        previous_count = commits;
        previous_next_lsn = next_lsn;
        drop(wal);

        let mut candidate_data = CrashableFile::from_durable(baseline_data.clone());
        candidate_data.crash();
        let candidate_wal = CrashableFile::from_durable(prefix.to_vec());
        let mut recovered = BTreeStore::open_with_wal(candidate_data, candidate_wal, config())
            .unwrap_or_else(|error| {
                panic!("prefix_len={prefix_len} baseline_len={baseline_len} full_len={full_len} WalLog committed count={commits} next_lsn={next_lsn} expected stage={commits} actual A/B state unavailable: {error}")
            });
        assert_stage_matches_wal(
            &mut recovered,
            &expected_batches,
            &format!(
                "prefix_len={prefix_len} baseline_len={baseline_len} full_len={full_len} WalLog committed count={commits} next_lsn={next_lsn} expected stage={commits}"
            ),
        );
        if semantic_boundary_prefixes.contains(&prefix_len) || prefix_len == full_len {
            let next_revision = recovered.put(key(b"continuation"), b"writable").unwrap();
            assert!(next_revision.get() > 0);
            recovered.check_invariants().unwrap();
        }
    }

    assert_eq!(stage_counts.iter().filter(|count| **count > 0).count(), 3);
    assert_eq!(previous_count, 2);
    assert!(first_a_offset.is_some());
    assert!(first_ab_offset.is_some());
    assert_eq!(first_a_offset, a_only_prefix);
    assert!(complete_uncommitted_frame_advanced_lsn);
    assert_eq!(full_commit_lsns.len(), 2);
    println!(
        "WAL prefix DST baseline_len={baseline_len} full_len={full_len} prefixes={} stages={stage_counts:?} first_A={} first_AB={} runtime={:?}",
        full_len - baseline_len + 1,
        first_a_offset.unwrap(),
        first_ab_offset.unwrap(),
        started.elapsed(),
    );
}

#[test]
fn wal_sync_errors_allow_none_partial_or_full_persistence() {
    let initialized =
        BTreeStore::open_with_wal(CrashableFile::new(), CrashableFile::new(), config()).unwrap();
    let (data, wal) = initialized.into_files().unwrap();
    let data_image = data.durable_bytes().to_vec();
    let baseline_wal = wal.durable_bytes().to_vec();
    let baseline_len = baseline_wal.len();

    let mut unsynced = BTreeStore::open_with_wal(
        CrashableFile::from_durable(data_image.clone()),
        CrashableFile::from_durable(baseline_wal.clone()),
        config(),
    )
    .unwrap();
    unsynced.set_fault_injector(CrashInjector::at("before_wal_sync", 1));
    assert!(unsynced.apply_transaction_group(&group_requests()).is_err());
    let (_, unsynced_wal) = unsynced.into_files().unwrap();
    let full_len = unsynced_wal.volatile_bytes().len();
    let mut only_a = None;
    for prefix_len in baseline_len..=full_len {
        let wal = WalLog::open(
            CrashableFile::from_durable(unsynced_wal.volatile_bytes()[..prefix_len].to_vec()),
            identity(&config()),
        )
        .unwrap();
        if wal.committed_batches().len() == 1 {
            only_a = Some(prefix_len);
            break;
        }
    }
    let cases = [
        (baseline_len, 0usize, false),
        (
            only_a.expect("prefix sweep must find A-only boundary"),
            1usize,
            false,
        ),
        (full_len, 2usize, false),
        (full_len, 2usize, true),
    ];
    for (persisted_len, expected_count, persist_all) in cases {
        let data = CrashableFile::from_durable(data_image.clone());
        let wal = CrashableFile::from_durable(baseline_wal.clone());
        let initialized = BTreeStore::open_with_wal(data, wal, config()).unwrap();
        let (data, mut wal) = initialized.into_files().unwrap();
        let fault = if persist_all {
            FaultAction::SyncPersistAllThenIo {
                kind: ErrorKind::Other,
                message: "sync returned an error after all bytes persisted".into(),
            }
        } else {
            FaultAction::SyncPersistPrefixThenIo {
                length: persisted_len,
                kind: ErrorKind::Other,
                message: "sync returned an error after a prefix persisted".into(),
            }
        };
        wal.set_fault_plan(FaultPlan::default().on_next(FileOperation::SyncData, fault));
        let mut trial = BTreeStore::open_with_wal(data, wal, config()).unwrap();
        assert!(trial.apply_transaction_group(&group_requests()).is_err());
        let (mut data, mut wal) = trial.into_files().unwrap();
        data.crash();
        wal.crash();
        let mut reopened = BTreeStore::open_with_wal(data, wal, config()).unwrap_or_else(|error| {
            panic!("persisted prefix length={persisted_len} expected recovered logical commit count={expected_count} actual state=open error {error}")
        });
        let stage = actual_stage(
            &mut reopened,
            &format!(
                "persisted prefix length={persisted_len} expected recovered logical commit count={expected_count}"
            ),
        );
        assert_eq!(
            stage.len(),
            expected_count,
            "persisted prefix length={persisted_len} expected recovered logical commit count={expected_count} actual state={stage:?}"
        );
        assert!(expected_count != 1 || stage[0].0 == b'A');
        reopened.check_invariants().unwrap();
        println!(
            "sync-error matrix persisted_prefix={persisted_len} expected_count={expected_count} persist_all={persist_all} actual={stage:?}"
        );
    }
}

fn checkpoint_points() -> [&'static str; 25] {
    [
        "before_checkpoint_gate",
        "before_checkpoint_data_flush",
        "before_data_page_write",
        "during_data_page_write",
        "after_data_page_write",
        "before_data_file_sync",
        "during_data_file_sync",
        "after_data_file_sync",
        "before_checkpoint_superblock_write",
        "after_checkpoint_superblock_write",
        "before_checkpoint_metadata_sync",
        "during_checkpoint_metadata_sync",
        "after_checkpoint_metadata_sync",
        "before_wal_reset",
        "during_wal_truncate",
        "after_wal_truncate",
        "before_wal_reset_truncate_sync",
        "during_wal_reset_truncate_sync",
        "after_wal_reset_truncate_sync",
        "before_wal_reinitialization",
        "during_wal_reinitialization",
        "after_wal_reset_write",
        "before_wal_reset_sync",
        "during_wal_reset_sync",
        "after_wal_reset_sync",
    ]
}

#[test]
fn checkpoint_faults_recover_across_durable_and_volatile_file_images() {
    let points = [
        checkpoint_points().as_slice(),
        &["before_checkpoint_complete"],
    ]
    .concat();
    let mut combinations = 0usize;
    let mut metadata_survival = false;
    let mut truncate_survival = false;
    let mut reset_init_survival = false;
    let mut complete_error_survival = false;
    for point in points.iter().copied() {
        let mut base =
            BTreeStore::open_with_wal(CrashableFile::new(), CrashableFile::new(), config())
                .unwrap();
        let a_revision = base.put(key(b"A"), b"a").unwrap();
        let b_revision = base.put(key(b"B"), b"b").unwrap();
        base.set_fault_injector(CrashInjector::at(point, 1));
        assert!(base.checkpoint().is_err(), "fault did not fire at {point}");
        let (data, wal) = base.into_files().unwrap();
        let data_images = unique_images(data.durable_bytes(), data.volatile_bytes());
        let wal_images = unique_images(wal.durable_bytes(), wal.volatile_bytes());
        if point == "after_checkpoint_superblock_write" {
            metadata_survival = data_images
                .iter()
                .any(|(_, bytes)| bytes == data.volatile_bytes());
        }
        if point == "after_wal_truncate" {
            truncate_survival = wal_images.iter().any(|(_, bytes)| bytes.is_empty())
                && wal_images
                    .iter()
                    .any(|(_, bytes)| bytes == wal.durable_bytes());
        }
        if point == "after_wal_reset_write" {
            reset_init_survival = wal_images
                .iter()
                .any(|(_, bytes)| bytes == wal.volatile_bytes());
        }
        if point == "before_checkpoint_complete" {
            complete_error_survival = true;
        }
        for (data_kind, data_image) in &data_images {
            for (wal_kind, wal_image) in &wal_images {
                combinations += 1;
                let context = format!(
                    "fault point={point} data candidate={data_kind} wal candidate={wal_kind} previous revisions=({}, {})",
                    a_revision.get(),
                    b_revision.get()
                );
                let mut reopened = open_from_images(data_image, wal_image);
                assert_eq!(
                    reopened.get(&key(b"A")).unwrap(),
                    RevisionState::present(b"a", a_revision),
                    "{context}"
                );
                assert_eq!(
                    reopened.get(&key(b"B")).unwrap(),
                    RevisionState::present(b"b", b_revision),
                    "{context}"
                );
                reopened
                    .check_invariants()
                    .unwrap_or_else(|error| panic!("{context}: {error}"));
                let next_revision = reopened
                    .put(key(b"C"), b"after recovery")
                    .unwrap_or_else(|error| panic!("{context}: {error}"));
                assert!(
                    next_revision > a_revision.max(b_revision),
                    "{context}: next revision={}",
                    next_revision.get()
                );
                reopened
                    .check_invariants()
                    .unwrap_or_else(|error| panic!("{context}: {error}"));
            }
        }
    }
    assert!(
        metadata_survival,
        "unsynced checkpoint metadata candidate was absent"
    );
    assert!(
        truncate_survival,
        "unsynced WAL truncate candidates were absent"
    );
    assert!(
        reset_init_survival,
        "unsynced reset INIT candidate was absent"
    );
    assert!(
        complete_error_survival,
        "completion error point was not exercised"
    );
    println!(
        "checkpoint DST fault_points={} candidate_combinations={combinations} metadata_survival={metadata_survival} truncate_survival={truncate_survival} reset_init_survival={reset_init_survival} completion_error={complete_error_survival}",
        points.len()
    );
}

fn unique_images(first: &[u8], second: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    if first == second {
        vec![("durable", first.to_vec())]
    } else {
        vec![("durable", first.to_vec()), ("volatile", second.to_vec())]
    }
}

#[test]
fn every_wal_reset_init_prefix_recovers_checkpointed_state_and_lsn_floor() {
    let mut store =
        BTreeStore::open_with_wal(CrashableFile::new(), CrashableFile::new(), config()).unwrap();
    let a_revision = store.put(key(b"A"), b"a").unwrap();
    let b_revision = store.put(key(b"B"), b"b").unwrap();
    let last_revision = a_revision.max(b_revision);
    store.set_fault_injector(CrashInjector::at("after_wal_reset_write", 1));
    assert!(store.checkpoint().is_err());
    let (data, wal) = store.into_files().unwrap();
    let durable_data = data.durable_bytes().to_vec();
    let init = wal.volatile_bytes().to_vec();
    assert!(wal.durable_bytes().is_empty());
    let init_len = init.len();
    for prefix_len in 0..=init_len {
        let mut reopened = BTreeStore::open_with_wal(
            CrashableFile::from_durable(durable_data.clone()),
            CrashableFile::from_durable(init[..prefix_len].to_vec()),
            config(),
        )
        .unwrap_or_else(|error| {
            panic!("reset INIT prefix_len={prefix_len} init_len={init_len}: {error}")
        });
        assert_eq!(
            reopened.get(&key(b"A")).unwrap(),
            RevisionState::present(b"a", a_revision),
            "reset INIT prefix_len={prefix_len} init_len={init_len}"
        );
        assert_eq!(
            reopened.get(&key(b"B")).unwrap(),
            RevisionState::present(b"b", b_revision),
            "reset INIT prefix_len={prefix_len} init_len={init_len}"
        );
        reopened.check_invariants().unwrap();
        let next_revision = reopened.put(key(b"C"), b"after reset").unwrap();
        assert!(
            next_revision > last_revision,
            "reset INIT prefix_len={prefix_len} next revision={} floor={}",
            next_revision.get(),
            last_revision.get()
        );
    }
    println!(
        "reset INIT exhaustive prefix count={} init_len={} passed",
        init_len + 1,
        init_len
    );
}
