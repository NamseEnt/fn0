use dodb_core::{
    DocumentKey, Error, PrimaryKey, Revision, RevisionState, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use std::time::Duration;

use super::*;

#[derive(Clone, Debug, Default)]
struct MemoryFile {
    bytes: Vec<u8>,
}

impl DurableFile for MemoryFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let start = usize::try_from(offset).map_err(|_| Error::invalid_input("offset overflow"))?;
        if start >= self.bytes.len() {
            return Ok(0);
        }
        let count = buffer.len().min(self.bytes.len() - start);
        buffer[..count].copy_from_slice(&self.bytes[start..start + count]);
        Ok(count)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
        let start = usize::try_from(offset).map_err(|_| Error::invalid_input("offset overflow"))?;
        let end = start
            .checked_add(bytes.len())
            .ok_or_else(|| Error::invalid_input("write range overflow"))?;
        if self.bytes.len() < end {
            self.bytes.resize(end, 0);
        }
        self.bytes[start..end].copy_from_slice(bytes);
        Ok(bytes.len())
    }

    fn len(&self) -> Result<u64> {
        Ok(self.bytes.len() as u64)
    }

    fn set_len(&mut self, length: u64) -> Result<()> {
        let length =
            usize::try_from(length).map_err(|_| Error::invalid_input("length overflow"))?;
        self.bytes.resize(length, 0);
        Ok(())
    }

    fn sync_data(&mut self) -> Result<()> {
        Ok(())
    }

    fn sync_all(&mut self) -> Result<()> {
        Ok(())
    }
}

fn key(pk: impl Into<Vec<u8>>, sk: impl Into<Vec<u8>>) -> DocumentKey {
    DocumentKey::new(pk, sk)
}

fn config(cache_capacity: usize) -> DatabaseConfig {
    DatabaseConfig::default().with_cache_capacity(cache_capacity)
}

#[test]
fn basic_operations_preserve_missing_revisions_and_order() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(16)).unwrap();
    let first = key(vec![0, 1], vec![0]);
    let second = key(vec![0, 1], vec![1]);
    let other = key(vec![0, 2], vec![0]);

    assert_eq!(
        store.get(&first).unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    assert_eq!(store.delete(first.clone()).unwrap(), Revision::new(1));
    assert_eq!(
        store.get(&first).unwrap(),
        RevisionState::missing(Revision::new(1))
    );
    assert_eq!(
        store.put(first.clone(), b"first").unwrap(),
        Revision::new(2)
    );
    store.put(second.clone(), b"second").unwrap();
    store.put(other.clone(), b"other").unwrap();
    store.delete(second.clone()).unwrap();

    let rows = store.query(&PrimaryKey::new(vec![0, 1]), None, 10).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, first);
    assert_eq!(rows[0].value, b"first");
    let scan = store.scan(None, 10).unwrap();
    assert_eq!(
        scan.iter().map(|row| row.key.clone()).collect::<Vec<_>>(),
        vec![key(vec![0, 1], vec![0]), other]
    );
    store.check_invariants().unwrap();
}

#[test]
fn canonical_key_boundary_includes_zero_byte_expansion() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let boundary = key(vec![0; 1_994], Vec::new());
    assert_eq!(boundary.encoded_len(), MAX_ENCODED_KEY_SIZE);
    assert_eq!(
        store.get(&boundary).unwrap(),
        RevisionState::missing(Revision::ZERO)
    );

    let oversized = key(vec![0; 1_995], Vec::new());
    assert!(matches!(
        store.put(oversized, b"rejected"),
        Err(Error::InvalidInput(_))
    ));
}

#[test]
fn splits_root_and_internal_pages_and_reopens() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(0)).unwrap();
    for value in 0u16..3000 {
        let key = key(
            (value % 17).to_le_bytes().to_vec(),
            value.to_le_bytes().to_vec(),
        );
        store.put(key, value.to_le_bytes()).unwrap();
    }
    let report = store.check_invariants().unwrap();
    assert!(report.reachable_pages > 50);
    let file = store.into_file();
    let mut reopened = BTreeStore::open(file, config(1)).unwrap();
    assert_eq!(reopened.scan(None, usize::MAX).unwrap().len(), 3000);
    assert_eq!(
        reopened.check_invariants().unwrap().leaked_pages,
        Vec::new()
    );
}

#[test]
fn large_value_replacement_reclaims_overflow_pages() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(4)).unwrap();
    let document_key = key(vec![7], vec![9]);
    let large = vec![0xa5; 10_000];
    store.put(document_key.clone(), large.clone()).unwrap();
    assert_eq!(
        store.get(&document_key).unwrap().value(),
        Some(large.as_slice())
    );
    store.put(document_key.clone(), b"small").unwrap();
    let report = store.check_invariants().unwrap();
    assert!(report.free_pages >= 3);
    assert_eq!(
        store.get(&document_key).unwrap().value(),
        Some(&b"small"[..])
    );
    store.delete(document_key.clone()).unwrap();
    assert!(store.check_invariants().unwrap().free_pages >= 3);
    store.put(document_key, vec![0x11; 9000]).unwrap();
    assert!(store.check_invariants().unwrap().free_pages < 3);
}

fn split_update_key(index: u16) -> DocumentKey {
    key(b"split-update", index.to_be_bytes().to_vec())
}

#[test]
fn growing_existing_value_splits_leaf_and_preserves_order() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(0)).unwrap();
    let original_value = vec![0x31; 300];
    for index in 0..11 {
        store
            .put(split_update_key(index), original_value.clone())
            .unwrap();
    }
    assert_eq!(store.check_invariants().unwrap().reachable_pages, 1);

    let replacement = vec![0x42; 500];
    store.put(split_update_key(5), replacement.clone()).unwrap();

    assert!(store.check_invariants().unwrap().reachable_pages >= 3);
    let rows = store.scan(None, usize::MAX).unwrap();
    assert_eq!(rows.len(), 11);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row.key, split_update_key(index as u16));
        let expected = if index == 5 {
            replacement.as_slice()
        } else {
            original_value.as_slice()
        };
        assert_eq!(row.value, expected);
    }
    let queried = store
        .query(&PrimaryKey::new(b"split-update".to_vec()), None, usize::MAX)
        .unwrap();
    assert_eq!(queried, rows);
    store.check_invariants().unwrap();
}

#[test]
fn growing_overflow_value_splits_leaf_and_reclaims_old_chain() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(0)).unwrap();
    let large = vec![0x71; 10_000];
    let neighbor = vec![0x72; 300];
    for index in 0..11 {
        let value = if index == 5 {
            large.clone()
        } else {
            neighbor.clone()
        };
        store.put(split_update_key(index), value).unwrap();
    }
    let before = store.check_invariants().unwrap();
    assert_eq!(before.reachable_pages, 4);

    store.put(split_update_key(5), vec![0x73; 500]).unwrap();

    let after = store.check_invariants().unwrap();
    assert!(after.reachable_pages >= 3);
    assert!(after.free_pages >= 3);
    assert!(after.leaked_pages.is_empty());
    assert_eq!(
        store.get(&split_update_key(5)).unwrap().value(),
        Some(&vec![0x73; 500][..])
    );
}

#[test]
fn wal_recovery_preserves_existing_value_leaf_split() {
    let mut store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let original_value = vec![0x51; 300];
    for index in 0..11 {
        store
            .put(split_update_key(index), original_value.clone())
            .unwrap();
    }
    let replacement = vec![0x62; 500];
    let revision = store.put(split_update_key(5), replacement.clone()).unwrap();
    let (data, wal) = store.into_files().unwrap();

    let mut reopened = BTreeStore::open_with_wal(data, wal, config(0)).unwrap();
    assert_eq!(
        reopened.get(&split_update_key(5)).unwrap(),
        RevisionState::present(replacement, revision)
    );
    let rows = reopened.scan(None, usize::MAX).unwrap();
    assert_eq!(rows.len(), 11);
    assert_eq!(
        rows.iter().map(|row| row.key.clone()).collect::<Vec<_>>(),
        (0..11).map(split_update_key).collect::<Vec<_>>()
    );
    assert_eq!(
        reopened.check_invariants().unwrap().leaked_pages,
        Vec::new()
    );
}

#[test]
fn deleting_all_entries_keeps_empty_leaves_searchable() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(1)).unwrap();
    let keys: Vec<_> = (0..240)
        .map(|index| key(vec![0], (index as u16).to_le_bytes().to_vec()))
        .collect();
    for document_key in &keys {
        store.put(document_key.clone(), b"value").unwrap();
    }
    for document_key in keys.iter().rev() {
        store.delete(document_key.clone()).unwrap();
    }
    assert!(store.scan(None, 10).unwrap().is_empty());
    assert!(
        store
            .query(&PrimaryKey::new(vec![0]), None, 10)
            .unwrap()
            .is_empty()
    );
    let report = store.check_invariants().unwrap();
    assert!(report.reachable_pages > 1);
}

#[test]
fn prepared_batch_is_private_until_published() {
    let file = MemoryFile::default();
    let mut store = BTreeStore::open(file, config(8)).unwrap();
    let document_key = key(vec![1], vec![2]);
    let requests = [
        BatchRequest::Put {
            key: document_key.clone(),
            value: b"value".to_vec(),
        },
        BatchRequest::Get {
            key: document_key.clone(),
        },
    ];
    let prepared = store.prepare_batch(&requests).unwrap();
    assert!(!prepared.changed_pages().is_empty());
    assert_eq!(
        store.get(&document_key).unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
    store.publish_prepared(prepared).unwrap();
    assert_eq!(
        store.get(&document_key).unwrap().value(),
        Some(&b"value"[..])
    );
}

#[test]
fn failed_batch_does_not_publish_earlier_requests() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(0)).unwrap();
    let first_key = key(vec![1], vec![1]);
    let second_key = key(vec![1], vec![2]);
    let result = store.apply_batch(&[
        BatchRequest::Put {
            key: first_key.clone(),
            value: b"first".to_vec(),
        },
        BatchRequest::Put {
            key: second_key,
            value: vec![0; MAX_VALUE_SIZE + 1],
        },
    ]);
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    assert_eq!(
        store.get(&first_key).unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
}

#[test]
fn malformed_page_is_corruption_not_a_panic() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(0)).unwrap();
    store.put(key(vec![1], vec![1]), b"value").unwrap();
    let mut file = store.into_file();
    file.bytes[2 * PAGE_SIZE + 80] ^= 1;
    let result = BTreeStore::open(file, config(0));
    assert!(matches!(result, Err(Error::Corruption(_))));
}

#[test]
fn all_cache_sizes_use_the_same_results() {
    for capacity in [0, 1, 8, 64] {
        let mut store = BTreeStore::open(MemoryFile::default(), config(capacity)).unwrap();
        for index in 0..120 {
            store
                .put(key(vec![index % 4], vec![0, index]), vec![index; 10])
                .unwrap();
        }
        for index in 0..120 {
            let state = store.get(&key(vec![index % 4], vec![0, index])).unwrap();
            assert_eq!(state.value(), Some(vec![index; 10].as_slice()));
        }
        assert!(store.cache_len() <= capacity);
        store.check_invariants().unwrap();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn async_shard_makes_committed_writes_visible_after_success() {
    let store = BTreeStore::open(MemoryFile::default(), config(2)).unwrap();
    let shard = AsyncShard::start(store, 8);
    let first = shard
        .execute(BatchRequest::Put {
            key: key(vec![1], vec![1]),
            value: b"first".to_vec(),
        })
        .await
        .unwrap();
    assert_eq!(first, BatchResponse::Put(Revision::new(1)));
    let second = shard
        .execute(BatchRequest::Get {
            key: key(vec![1], vec![1]),
        })
        .await
        .unwrap();
    assert_eq!(
        second,
        BatchResponse::Get(RevisionState::present(b"first", Revision::new(1)))
    );
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn async_shard_reads_use_one_committed_view_and_bypass_write_queue() {
    let store = BTreeStore::open(MemoryFile::default(), config(0)).unwrap();
    let shard = AsyncShard::start(store, 8);
    for index in 0..240usize {
        shard
            .execute(BatchRequest::Put {
                key: key(b"read-view", index.to_le_bytes().to_vec()),
                value: vec![index as u8],
            })
            .await
            .unwrap();
    }

    let get = shard
        .execute(BatchRequest::Get {
            key: key(b"read-view", 239usize.to_le_bytes().to_vec()),
        })
        .await
        .unwrap();
    assert_eq!(
        get,
        BatchResponse::Get(RevisionState::present([239], Revision::new(240)))
    );
    let query = shard
        .execute(BatchRequest::Query {
            pk: PrimaryKey::new(b"read-view".to_vec()),
            exclusive_after_sk: None,
            limit: 240,
        })
        .await
        .unwrap();
    let BatchResponse::Query(query) = query else {
        panic!("expected query response");
    };
    assert_eq!(query.len(), 240);
    let scan = shard
        .execute(BatchRequest::Scan {
            exclusive_after_key: None,
            limit: 240,
        })
        .await
        .unwrap();
    let BatchResponse::Scan(scan) = scan else {
        panic!("expected scan response");
    };
    assert_eq!(scan.len(), 240);
    let metrics = shard.coordinator_metrics();
    assert_eq!(metrics.queued_requests, 240);
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn response_budgets_bound_query_scan_and_transact_get_without_truncation() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let first = key(b"budget", b"first");
    let second = key(b"budget", b"second");
    let value = vec![7; 2 * 1024 * 1024];
    store.put(first.clone(), value.clone()).unwrap();
    store.put(second.clone(), value).unwrap();
    let shard = AsyncShard::start(store, 8);

    for request in [
        BatchRequest::Query {
            pk: PrimaryKey::new(b"budget".to_vec()),
            exclusive_after_sk: None,
            limit: 2,
        },
        BatchRequest::Scan {
            exclusive_after_key: None,
            limit: 2,
        },
    ] {
        assert!(matches!(
            shard
                .execute_with_response_budget(request, 3 * 1024 * 1024)
                .await,
            Err(Error::ResponseTooLarge(_))
        ));
    }
    assert!(matches!(
        shard
            .transact_get_with_response_budget(vec![first.clone(), second.clone()], 3 * 1024 * 1024)
            .await,
        Err(Error::ResponseTooLarge(_))
    ));

    let query = shard
        .execute_with_response_budget(
            BatchRequest::Query {
                pk: PrimaryKey::new(b"budget".to_vec()),
                exclusive_after_sk: None,
                limit: 2,
            },
            5 * 1024 * 1024,
        )
        .await
        .unwrap();
    let BatchResponse::Query(rows) = query else {
        panic!("expected query response");
    };
    assert_eq!(rows.len(), 2);
    let states = shard
        .transact_get_with_response_budget(vec![second, first], 5 * 1024 * 1024)
        .await
        .unwrap();
    assert_eq!(states.len(), 2);
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn one_near_maximum_value_remains_readable_with_a_response_budget() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let key = key(b"large", b"value");
    let value = vec![3; MAX_VALUE_SIZE];
    store.put(key.clone(), value.clone()).unwrap();
    let shard = AsyncShard::start(store, 8);
    let response = shard
        .execute_with_response_budget(BatchRequest::Get { key }, MAX_VALUE_SIZE + 1024 * 1024)
        .await
        .unwrap();
    let BatchResponse::Get(state) = response else {
        panic!("expected get response");
    };
    assert_eq!(state.value().map(|bytes| bytes.len()), Some(value.len()));
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn async_shard_keeps_logical_transaction_lsns_distinct() {
    let store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let shard = AsyncShard::start(store, 8);
    let a = key(b"async", b"a");
    let b = key(b"async", b"b");
    let first = shard.execute_transaction(put_transaction(a.clone(), b"a"));
    let second = shard.execute_transaction(put_transaction(b.clone(), b"b"));
    let (first, second) = tokio::join!(first, second);
    let first = first.unwrap().commit_lsn.unwrap();
    let second = second.unwrap().commit_lsn.unwrap();
    assert_ne!(first, second);
    assert_eq!(shard.transact_get(vec![a, b]).await.unwrap().len(), 2);
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn async_checkpoint_is_fifo_and_reads_remain_available() {
    let store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let shard = AsyncShard::start(store, 8);
    let document_key = key(b"async-checkpoint", b"key");
    let before = shard
        .execute(BatchRequest::Put {
            key: document_key.clone(),
            value: b"before".to_vec(),
        })
        .await
        .unwrap();
    let BatchResponse::Put(before_revision) = before else {
        panic!("expected PUT response");
    };
    let checkpoint = shard.checkpoint().await.unwrap();
    assert_eq!(
        checkpoint.checkpoint_lsn,
        dodb_core::Lsn::new(before_revision.get())
    );
    let after = shard
        .execute(BatchRequest::Put {
            key: document_key.clone(),
            value: b"after".to_vec(),
        })
        .await
        .unwrap();
    let BatchResponse::Put(after_revision) = after else {
        panic!("expected PUT response");
    };
    assert!(after_revision > before_revision);
    assert_eq!(
        shard
            .execute(BatchRequest::Get { key: document_key })
            .await
            .unwrap(),
        BatchResponse::Get(RevisionState::present(b"after", after_revision))
    );
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn async_shard_groups_mutations_around_transact_get_barriers() {
    let store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let shard = AsyncShard::start_with_config(
        store,
        CoordinatorConfig {
            max_collection_delay: Duration::from_millis(10),
            ..CoordinatorConfig::default()
        },
    );
    let first_key = key(b"barrier", b"first");
    let second_key = key(b"barrier", b"second");
    let third_key = key(b"barrier", b"third");
    let first = shard.execute(BatchRequest::Put {
        key: first_key.clone(),
        value: b"first".to_vec(),
    });
    let second = shard.execute(BatchRequest::Put {
        key: second_key.clone(),
        value: b"second".to_vec(),
    });
    let barrier = shard.transact_get(vec![first_key.clone(), second_key.clone()]);
    let third = shard.execute(BatchRequest::Put {
        key: third_key.clone(),
        value: b"third".to_vec(),
    });
    let (first, second, barrier, third) = tokio::join!(first, second, barrier, third);
    let BatchResponse::Put(first_revision) = first.unwrap() else {
        panic!("expected first PUT response");
    };
    let BatchResponse::Put(second_revision) = second.unwrap() else {
        panic!("expected second PUT response");
    };
    assert!(first_revision < second_revision);
    assert_eq!(
        barrier.unwrap(),
        vec![
            RevisionState::present(b"first", first_revision),
            RevisionState::present(b"second", second_revision),
        ]
    );
    let BatchResponse::Put(third_revision) = third.unwrap() else {
        panic!("expected third PUT response");
    };
    assert!(second_revision < third_revision);
    assert_eq!(
        shard
            .execute(BatchRequest::Get { key: third_key })
            .await
            .unwrap(),
        BatchResponse::Get(RevisionState::present(b"third", third_revision))
    );
    assert_eq!(shard.wal_metrics().unwrap().wal_syncs, 3);
    shard.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn async_shard_groups_mutations_around_checkpoint_barriers() {
    let store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let shard = AsyncShard::start_with_config(
        store,
        CoordinatorConfig {
            max_collection_delay: Duration::from_millis(10),
            ..CoordinatorConfig::default()
        },
    );
    let first_key = key(b"checkpoint-barrier", b"first");
    let second_key = key(b"checkpoint-barrier", b"second");
    let third_key = key(b"checkpoint-barrier", b"third");
    let first = shard.execute(BatchRequest::Put {
        key: first_key.clone(),
        value: b"first".to_vec(),
    });
    let second = shard.execute(BatchRequest::Put {
        key: second_key.clone(),
        value: b"second".to_vec(),
    });
    let checkpoint = shard.checkpoint();
    let third = shard.execute(BatchRequest::Put {
        key: third_key.clone(),
        value: b"third".to_vec(),
    });
    let (first, second, checkpoint, third) = tokio::join!(first, second, checkpoint, third);
    let BatchResponse::Put(first_revision) = first.unwrap() else {
        panic!("expected first PUT response");
    };
    let BatchResponse::Put(second_revision) = second.unwrap() else {
        panic!("expected second PUT response");
    };
    assert!(first_revision < second_revision);
    assert_eq!(
        checkpoint.unwrap().checkpoint_lsn,
        dodb_core::Lsn::new(second_revision.get())
    );
    let BatchResponse::Put(third_revision) = third.unwrap() else {
        panic!("expected third PUT response");
    };
    assert!(second_revision < third_revision);
    assert_eq!(
        shard
            .execute(BatchRequest::Get { key: third_key })
            .await
            .unwrap(),
        BatchResponse::Get(RevisionState::present(b"third", third_revision))
    );
    assert_eq!(shard.wal_metrics().unwrap().wal_syncs, 4);
    shard.close().await.unwrap();
}

#[test]
fn wal_backed_store_reopens_from_committed_page_images() {
    let mut store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let document_key = key(vec![9], vec![9]);
    let revision = store.put(document_key.clone(), b"wal".to_vec()).unwrap();
    assert!(revision.get() > 1);
    let (data, wal) = store.into_files().unwrap();
    let mut reopened = BTreeStore::open_with_wal(data, wal, config(0)).unwrap();
    assert_eq!(
        reopened.get(&document_key).unwrap(),
        RevisionState::present(b"wal", revision)
    );
}

#[test]
fn checkpoint_flushes_state_resets_wal_and_preserves_lsn_monotonicity() {
    let mut store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let document_key = key(b"checkpoint", b"key");
    let first_revision = store.put(document_key.clone(), b"before").unwrap();
    let before = store.wal_metrics().unwrap().unwrap().wal_bytes;
    let report = store.checkpoint().unwrap();

    assert_eq!(
        report.checkpoint_lsn,
        dodb_core::Lsn::new(first_revision.get())
    );
    assert!(report.pages_flushed > 0);
    assert_eq!(
        report.bytes_written,
        report.pages_flushed as u64 * PAGE_SIZE as u64 + 2 * PAGE_SIZE as u64
    );
    assert!(report.wal_bytes_reclaimed > 0);
    assert!(store.wal_metrics().unwrap().unwrap().wal_bytes < before);
    assert_eq!(
        store.current_superblock().checkpoint_lsn,
        report.checkpoint_lsn
    );

    let second_revision = store.put(document_key.clone(), b"after").unwrap();
    assert!(second_revision > first_revision);
    let (data, wal) = store.into_files().unwrap();
    let mut reopened = BTreeStore::open_with_wal(data, wal, config(0)).unwrap();
    assert_eq!(
        reopened.current_superblock().checkpoint_lsn,
        report.checkpoint_lsn
    );
    assert_eq!(
        reopened.get(&document_key).unwrap(),
        RevisionState::present(b"after", second_revision)
    );
    let repeated = reopened.checkpoint().unwrap();
    assert_eq!(
        repeated.checkpoint_lsn,
        dodb_core::Lsn::new(second_revision.get())
    );
}

fn put_transaction(key: DocumentKey, value: &[u8]) -> TransactionRequest {
    TransactionRequest::new(
        Vec::new(),
        vec![TransactionMutation::Put {
            key,
            value: value.to_vec(),
        }],
    )
}

#[test]
fn transaction_workflow_validates_all_point_dependencies_and_commits_atomically() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let a = key(b"p", b"a");
    let b = key(b"p", b"b");
    let c = key(b"p", b"c");
    let a_revision = store.put(a.clone(), b"old-a").unwrap();
    let b_revision = store.put(b.clone(), b"old-b").unwrap();

    let request = TransactionRequest::new(
        vec![
            TransactionCondition::RevisionEquals {
                key: a.clone(),
                expected_revision: a_revision,
            },
            TransactionCondition::RevisionEquals {
                key: b.clone(),
                expected_revision: b_revision,
            },
            TransactionCondition::NotExists { key: c.clone() },
        ],
        vec![
            TransactionMutation::Put {
                key: a.clone(),
                value: b"new-a".to_vec(),
            },
            TransactionMutation::Put {
                key: c.clone(),
                value: b"new-c".to_vec(),
            },
        ],
    );
    let result = store.transact(request).unwrap();
    let commit_lsn = result.commit_lsn.unwrap();
    assert_eq!(store.get(&a).unwrap().revision(), commit_lsn.into());
    assert_eq!(store.get(&c).unwrap().revision(), commit_lsn.into());
    assert_eq!(store.get(&b).unwrap().revision(), b_revision);
}

#[test]
fn transaction_conflict_on_a_read_only_dependency_rolls_back_all_mutations() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let a = key(b"p", b"a");
    let b = key(b"p", b"b");
    let a_revision = store.put(a.clone(), b"a").unwrap();
    let request = TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: a.clone(),
            expected_revision: a_revision,
        }],
        vec![TransactionMutation::Put {
            key: b.clone(),
            value: b"b".to_vec(),
        }],
    );
    store.put(a.clone(), b"changed").unwrap();
    let error = store.transact(request).unwrap_err();
    assert!(matches!(error, Error::Conflict(_)));
    assert_eq!(
        store.get(&b).unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
}

#[test]
fn revision_equals_distinguishes_never_existing_from_deleted_and_not_exists_does_not() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let x = key(b"p", b"x");
    let never_existing = store.get(&x).unwrap().revision();
    store.put(x.clone(), b"x").unwrap();
    store.delete(x.clone()).unwrap();

    let stale = TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: x.clone(),
            expected_revision: never_existing,
        }],
        vec![TransactionMutation::Put {
            key: key(b"p", b"y"),
            value: b"y".to_vec(),
        }],
    );
    assert!(matches!(store.transact(stale), Err(Error::Conflict(_))));

    let insert = TransactionRequest::new(
        vec![TransactionCondition::NotExists { key: x.clone() }],
        vec![TransactionMutation::Put {
            key: x.clone(),
            value: b"reinserted".to_vec(),
        }],
    );
    store.transact(insert).unwrap();
}

#[test]
fn transaction_group_assigns_separate_lsns_and_stages_shared_pages_in_order() {
    let mut store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(0)).unwrap();
    let a = key(b"same", b"a");
    let b = key(b"same", b"b");
    let results = store
        .apply_transaction_group(&[
            put_transaction(a.clone(), b"a"),
            put_transaction(b.clone(), b"b"),
            put_transaction(a.clone(), b"a-again"),
        ])
        .unwrap();
    let first = results[0].as_ref().unwrap().commit_lsn.unwrap();
    let second = results[1].as_ref().unwrap().commit_lsn.unwrap();
    let third = results[2].as_ref().unwrap().commit_lsn.unwrap();
    assert!(first < second && second < third);
    let metrics = store.wal_metrics().unwrap().unwrap();
    assert_eq!(metrics.committed_batches, 3);
    assert_eq!(metrics.wal_syncs, 2);
    assert_eq!(store.get(&a).unwrap().revision(), third.into());
    assert_eq!(store.get(&b).unwrap().revision(), second.into());
    store.check_invariants().unwrap();

    let (data, wal) = store.into_files().unwrap();
    let mut reopened = BTreeStore::open_with_wal(data, wal, config(0)).unwrap();
    assert_eq!(reopened.get(&a).unwrap().value(), Some(&b"a-again"[..]));
    assert_eq!(reopened.get(&b).unwrap().value(), Some(&b"b"[..]));
    reopened.check_invariants().unwrap();
}

#[test]
fn insert_if_absent_race_has_one_winner_in_one_coordinator_group() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let key = key(b"race", b"x");
    let request = |value: &[u8]| {
        TransactionRequest::new(
            vec![TransactionCondition::NotExists { key: key.clone() }],
            vec![TransactionMutation::Put {
                key: key.clone(),
                value: value.to_vec(),
            }],
        )
    };
    let results = store
        .apply_transaction_group(&[request(b"first"), request(b"second")])
        .unwrap();
    assert!(results[0].is_ok());
    assert!(matches!(results[1], Err(Error::Conflict(_))));
    assert_eq!(store.get(&key).unwrap().value(), Some(&b"first"[..]));
}

#[test]
fn same_group_revision_dependency_conflicts_against_the_staged_commit() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let a = key(b"group", b"a");
    let b = key(b"group", b"b");
    let old_revision = store.put(a.clone(), b"old").unwrap();
    let first = TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: a.clone(),
            expected_revision: old_revision,
        }],
        vec![TransactionMutation::Put {
            key: a.clone(),
            value: b"new".to_vec(),
        }],
    );
    let second = TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: a.clone(),
            expected_revision: old_revision,
        }],
        vec![TransactionMutation::Put {
            key: b.clone(),
            value: b"should-not-appear".to_vec(),
        }],
    );
    let results = store.apply_transaction_group(&[first, second]).unwrap();
    assert!(results[0].is_ok());
    assert!(matches!(results[1], Err(Error::Conflict(_))));
    assert_eq!(
        store.get(&b).unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
}

#[test]
fn transact_get_preserves_input_order_and_revisions() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let first = key(b"p", b"1");
    let second = key(b"p", b"2");
    let first_revision = store.put(first.clone(), b"one").unwrap();
    let second_revision = store.put(second.clone(), b"two").unwrap();
    assert_eq!(
        store
            .transact_get(&[second.clone(), first.clone()])
            .unwrap(),
        vec![
            RevisionState::present(b"two", second_revision),
            RevisionState::present(b"one", first_revision),
        ]
    );
}

#[test]
fn condition_only_transaction_succeeds_without_a_logical_commit() {
    let mut store =
        BTreeStore::open_with_wal(MemoryFile::default(), MemoryFile::default(), config(8)).unwrap();
    let existing = key(b"condition-only", b"existing");
    let other_existing = key(b"condition-only", b"other-existing");
    let missing = key(b"condition-only", b"missing");
    let revision = store.put(existing.clone(), b"value").unwrap();
    store.put(other_existing.clone(), b"other-value").unwrap();
    let state_before = store.get(&existing).unwrap();
    let generation_before = store.current_superblock().generation;
    let wal_before = store.wal_metrics().unwrap().unwrap();

    let result = store
        .transact(TransactionRequest::new(
            vec![
                TransactionCondition::RevisionEquals {
                    key: existing.clone(),
                    expected_revision: revision,
                },
                TransactionCondition::Exists {
                    key: other_existing,
                },
                TransactionCondition::NotExists { key: missing },
            ],
            Vec::new(),
        ))
        .unwrap();

    assert_eq!(result.commit_lsn, None);
    assert_eq!(store.get(&existing).unwrap(), state_before);
    assert_eq!(store.current_superblock().generation, generation_before);
    assert_eq!(store.wal_metrics().unwrap().unwrap(), wal_before);
}

#[test]
fn condition_only_transaction_reports_stale_and_presence_conflicts() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let existing = key(b"condition-only", b"existing");
    let missing = key(b"condition-only", b"missing");
    let revision = store.put(existing.clone(), b"value").unwrap();

    let stale = store.transact(TransactionRequest::new(
        vec![TransactionCondition::RevisionEquals {
            key: existing.clone(),
            expected_revision: Revision::new(revision.get().saturating_sub(1)),
        }],
        Vec::new(),
    ));
    assert!(matches!(stale, Err(Error::Conflict(_))));

    let missing_exists = store.transact(TransactionRequest::new(
        vec![TransactionCondition::Exists {
            key: missing.clone(),
        }],
        Vec::new(),
    ));
    assert!(matches!(missing_exists, Err(Error::Conflict(_))));

    let existing_not_exists = store.transact(TransactionRequest::new(
        vec![TransactionCondition::NotExists { key: existing }],
        Vec::new(),
    ));
    assert!(matches!(existing_not_exists, Err(Error::Conflict(_))));
}

#[test]
fn multiple_condition_only_predicates_are_atomic() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let existing = key(b"condition-only", b"existing");
    let missing = key(b"condition-only", b"missing");
    let other_missing = key(b"condition-only", b"other-missing");
    let revision = store.put(existing.clone(), b"value").unwrap();

    let result = store.transact(TransactionRequest::new(
        vec![
            TransactionCondition::RevisionEquals {
                key: existing.clone(),
                expected_revision: revision,
            },
            TransactionCondition::NotExists {
                key: missing.clone(),
            },
            TransactionCondition::Exists { key: other_missing },
        ],
        Vec::new(),
    ));
    match result {
        Err(Error::Conflict(conflict)) => {
            assert_eq!(conflict.key, key(b"condition-only", b"other-missing"));
        }
        other => panic!("expected a condition conflict, got {other:?}"),
    }
    assert_eq!(store.get(&existing).unwrap().revision(), revision);
}

#[test]
fn ambiguous_transaction_mutations_are_rejected_without_changes() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let key = key(b"p", b"duplicate");
    let request = TransactionRequest::new(
        Vec::new(),
        vec![
            TransactionMutation::Put {
                key: key.clone(),
                value: b"one".to_vec(),
            },
            TransactionMutation::Delete { key: key.clone() },
        ],
    );
    assert!(matches!(
        store.transact(request),
        Err(Error::InvalidRequest(_))
    ));
    assert_eq!(
        store.get(&key).unwrap(),
        RevisionState::missing(Revision::ZERO)
    );
}

#[test]
fn duplicate_conditions_and_empty_transactions_are_rejected() {
    let mut store = BTreeStore::open(MemoryFile::default(), config(8)).unwrap();
    let key = key(b"p", b"duplicate-condition");
    let duplicate_conditions = TransactionRequest::new(
        vec![
            TransactionCondition::Exists { key: key.clone() },
            TransactionCondition::NotExists { key },
        ],
        Vec::new(),
    );
    assert!(matches!(
        store.transact(duplicate_conditions),
        Err(Error::InvalidRequest(_))
    ));
    assert!(matches!(
        store.transact(TransactionRequest::new(Vec::new(), Vec::new())),
        Err(Error::InvalidRequest(_))
    ));
}
