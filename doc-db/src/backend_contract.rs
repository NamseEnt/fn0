use crate::{
    AdminScanRequest, AdminWriteOp, AdminWriteOutcome, BatchOp, Database, DbOp, DbRequest,
    DbResult, DocGet, DocKey, Document, Prepared, TrxResult, WriteOp,
};
use bytes::Bytes;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ContractDocument {
    id: String,
    value: String,
}

impl Document for ContractDocument {
    fn key(&self) -> DocKey {
        DocKey::new("contract", self.id.clone())
    }
}

struct ContractDocumentGet {
    id: String,
}

impl DocGet for ContractDocumentGet {
    type Doc = ContractDocument;

    fn key(&self) -> DocKey {
        DocKey::new("contract", self.id.clone())
    }
}

pub async fn run_backend_contract(create_db: impl Fn() -> Database) {
    run_basic_kv_contract(&create_db).await;
    run_ordering_contract(&create_db).await;
    run_batch_contract(&create_db).await;
    run_version_and_admin_contract(&create_db).await;
    run_explicit_transaction_contract(&create_db).await;
    run_optimistic_transaction_contract(&create_db).await;
    run_execute_ops_contract(&create_db).await;
}

async fn run_basic_kv_contract(create_db: &impl Fn() -> Database) {
    let database = create_db();
    assert_eq!(database.get("missing", "missing").await.unwrap(), None);

    let original = [0_u8, 1, 255, 0];
    database.put("basic", "key", &original).await.unwrap();
    assert_eq!(
        database.get("basic", "key").await.unwrap(),
        Some(Bytes::from(original.to_vec()))
    );

    database.put("basic", "key", b"replacement").await.unwrap();
    assert_eq!(
        database.get("basic", "key").await.unwrap(),
        Some(Bytes::from_static(b"replacement"))
    );

    database.delete("basic", "key").await.unwrap();
    assert_eq!(database.get("basic", "key").await.unwrap(), None);
    database.delete("basic", "key").await.unwrap();
}

async fn run_ordering_contract(create_db: &impl Fn() -> Database) {
    let database = create_db();
    let sort_keys = vec!["", "a", "a\0b", "a/b", "a&b", "한글", "日本語", "😀"];
    for (key_index, sort_key) in sort_keys.iter().enumerate() {
        database
            .put("ordering", sort_key, &[key_index as u8])
            .await
            .unwrap();
    }

    let mut expected_keys = sort_keys.clone();
    expected_keys.sort_unstable();
    let all_items = database.query("ordering", None::<&str>, 100).await.unwrap();
    assert_eq!(
        all_items
            .iter()
            .map(|(sort_key, _)| sort_key.as_str())
            .collect::<Vec<_>>(),
        expected_keys
    );

    let first_page = database
        .query("ordering", Some(expected_keys[0]), 2)
        .await
        .unwrap();
    assert_eq!(first_page.len(), 2);
    assert!(
        !first_page
            .iter()
            .any(|(sort_key, _)| sort_key == expected_keys[0])
    );
    assert_eq!(first_page[0].0, expected_keys[1]);
    assert_eq!(first_page[1].0, expected_keys[2]);

    let scan_database = create_db();
    let scan_keys = vec![
        ("", "\0"),
        ("a", ""),
        ("a\0b", "a/b"),
        ("a/b", "a&b"),
        ("한글", "日本語"),
        ("😀", "😀"),
    ];
    for (key_index, (partition_key, sort_key)) in scan_keys.iter().enumerate() {
        scan_database
            .put(partition_key, sort_key, &[key_index as u8])
            .await
            .unwrap();
    }
    let mut expected_scan = scan_keys.clone();
    expected_scan.sort_unstable();
    let scan_page = scan_database.scan(None, 100).await.unwrap();
    let actual_scan = scan_page
        .iter()
        .map(|(partition_key, sort_key, _)| (partition_key.as_str(), sort_key.as_str()))
        .collect::<Vec<_>>();
    let expected_scan_refs = expected_scan
        .iter()
        .map(|(partition_key, sort_key)| (*partition_key, *sort_key))
        .collect::<Vec<_>>();
    assert_eq!(actual_scan, expected_scan_refs);

    let cursor = &expected_scan[1];
    let after_cursor = scan_database
        .scan(Some((cursor.0, cursor.1)), 2)
        .await
        .unwrap();
    assert_eq!(after_cursor.len(), 2);
    assert_eq!(
        (after_cursor[0].0.as_str(), after_cursor[0].1.as_str()),
        expected_scan_refs[2]
    );
}

async fn run_batch_contract(create_db: &impl Fn() -> Database) {
    let database = create_db();
    database.put("batch", "delete", b"old").await.unwrap();
    database
        .batch(&[
            BatchOp::Put {
                pk: "batch",
                sk: "a",
                data: b"A",
            },
            BatchOp::Put {
                pk: "batch",
                sk: "b",
                data: b"B",
            },
            BatchOp::Delete {
                pk: "batch",
                sk: "delete",
            },
        ])
        .await
        .unwrap();
    assert_eq!(
        database.get("batch", "a").await.unwrap(),
        Some(Bytes::from_static(b"A"))
    );
    assert_eq!(
        database.get("batch", "b").await.unwrap(),
        Some(Bytes::from_static(b"B"))
    );
    assert_eq!(database.get("batch", "delete").await.unwrap(), None);
}

async fn run_version_and_admin_contract(create_db: &impl Fn() -> Database) {
    let database = create_db();
    database.put("version", "document", b"v0").await.unwrap();
    let initial_page = database
        .admin_scan(AdminScanRequest {
            after: None,
            limit: 10,
            pk_prefix: Some("version".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(initial_page.documents[0].version, 0);
    assert_eq!(initial_page.documents[0].data, Bytes::from_static(b"v0"));

    database.put("version", "document", b"v1").await.unwrap();
    let updated_page = database
        .admin_scan(AdminScanRequest {
            after: None,
            limit: 10,
            pk_prefix: Some("version".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(updated_page.documents[0].version, 1);

    let applied = database
        .admin_write_batch(&[AdminWriteOp::Put {
            pk: "version".to_string(),
            sk: "document".to_string(),
            expected_version: 1,
            data: b"v2".to_vec(),
        }])
        .await
        .unwrap();
    assert_eq!(applied, AdminWriteOutcome::Applied);
    let version_two = database
        .admin_scan(AdminScanRequest {
            after: None,
            limit: 10,
            pk_prefix: Some("version".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(version_two.documents[0].version, 2);

    let deleted = database
        .admin_write_batch(&[AdminWriteOp::Delete {
            pk: "version".to_string(),
            sk: "document".to_string(),
            expected_version: 2,
        }])
        .await
        .unwrap();
    assert_eq!(deleted, AdminWriteOutcome::Applied);
    let recreated = database
        .admin_write_batch(&[AdminWriteOp::Create {
            pk: "version".to_string(),
            sk: "document".to_string(),
            data: b"new-incarnation".to_vec(),
        }])
        .await
        .unwrap();
    assert_eq!(recreated, AdminWriteOutcome::Applied);
    let incarnation_page = database
        .admin_scan(AdminScanRequest {
            after: None,
            limit: 10,
            pk_prefix: Some("version".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(incarnation_page.documents[0].version, 0);

    for sort_key in ["a", "b", "c"] {
        database
            .put("cursor", sort_key, sort_key.as_bytes())
            .await
            .unwrap();
    }
    let cursor_page = database
        .admin_scan(AdminScanRequest {
            after: None,
            limit: 2,
            pk_prefix: Some("cursor".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(cursor_page.documents.len(), 2);
    assert_eq!(cursor_page.documents[0].sk, "a");
    assert_eq!(cursor_page.documents[1].sk, "b");
    let next_cursor = cursor_page.next.clone().expect("admin scan cursor");
    let next_page = database
        .admin_scan(AdminScanRequest {
            after: Some(next_cursor),
            limit: 2,
            pk_prefix: Some("cursor".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(next_page.documents.len(), 1);
    assert_eq!(next_page.documents[0].sk, "c");

    database.put("migration", "a", b"a0").await.unwrap();
    database.put("migration", "b", b"b0").await.unwrap();
    let conflict = database
        .admin_write_batch(&[
            AdminWriteOp::Put {
                pk: "migration".to_string(),
                sk: "a".to_string(),
                expected_version: 0,
                data: b"a1".to_vec(),
            },
            AdminWriteOp::Put {
                pk: "migration".to_string(),
                sk: "b".to_string(),
                expected_version: 99,
                data: b"b1".to_vec(),
            },
        ])
        .await
        .unwrap();
    assert!(matches!(conflict, AdminWriteOutcome::Conflict(_)));
    assert_eq!(
        database.get("migration", "a").await.unwrap(),
        Some(Bytes::from_static(b"a0"))
    );
    assert_eq!(
        database.get("migration", "b").await.unwrap(),
        Some(Bytes::from_static(b"b0"))
    );
}

async fn run_explicit_transaction_contract(create_db: &impl Fn() -> Database) {
    let database = create_db();
    let mut transaction = database.transaction().await.unwrap();
    transaction.put("transaction", "a", b"A").await.unwrap();
    transaction.put("transaction", "b", b"B").await.unwrap();
    assert_eq!(
        transaction.get("transaction", "a").await.unwrap(),
        Some(Bytes::from_static(b"A"))
    );
    transaction.commit().await.unwrap();
    assert_eq!(
        database.get("transaction", "a").await.unwrap(),
        Some(Bytes::from_static(b"A"))
    );
    assert_eq!(
        database.get("transaction", "b").await.unwrap(),
        Some(Bytes::from_static(b"B"))
    );

    let mut rollback_transaction = database.transaction().await.unwrap();
    rollback_transaction
        .put("transaction", "a", b"modified")
        .await
        .unwrap();
    rollback_transaction
        .put("transaction", "created", b"created")
        .await
        .unwrap();
    rollback_transaction
        .delete("transaction", "a")
        .await
        .unwrap();
    assert_eq!(
        rollback_transaction.get("transaction", "a").await.unwrap(),
        None
    );
    rollback_transaction.rollback().await.unwrap();
    assert_eq!(
        database.get("transaction", "a").await.unwrap(),
        Some(Bytes::from_static(b"A"))
    );
    assert_eq!(database.get("transaction", "created").await.unwrap(), None);
}

async fn run_optimistic_transaction_contract(create_db: &impl Fn() -> Database) {
    run_optimistic_conflict_pair_contract(create_db).await;
    let database = create_db();
    let initial_document = ContractDocument {
        id: "existing".to_string(),
        value: "before".to_string(),
    };
    database
        .put(
            "contract",
            "existing",
            &serde_json::to_vec(&initial_document).unwrap(),
        )
        .await
        .unwrap();
    let update_result = database
        .trx(|transaction| async move {
            let mut handle = transaction
                .get(ContractDocumentGet {
                    id: "existing".to_string(),
                })
                .await?
                .expect("existing document");
            handle.value = "after".to_string();
            drop(handle);
            transaction.commit::<(), ()>(())
        })
        .await;
    assert!(matches!(update_result, TrxResult::Committed(())));
    let update_page = database
        .admin_scan(AdminScanRequest {
            after: None,
            limit: 10,
            pk_prefix: Some("contract".to_string()),
        })
        .await
        .unwrap();
    assert_eq!(update_page.documents[0].version, 1);

    let stale_database = database.clone();
    let stale_result = stale_database
        .trx(|transaction| {
            let database = stale_database.clone();
            async move {
                let mut handle = transaction
                    .get(ContractDocumentGet {
                        id: "existing".to_string(),
                    })
                    .await?
                    .expect("existing document");
                handle.value = "stale".to_string();
                drop(handle);
                database
                    .put(
                        "contract",
                        "existing",
                        &serde_json::to_vec(&ContractDocument {
                            id: "existing".to_string(),
                            value: "concurrent".to_string(),
                        })
                        .unwrap(),
                    )
                    .await?;
                transaction.commit::<(), ()>(())
            }
        })
        .await;
    assert!(
        matches!(stale_result, TrxResult::Conflict(details) if details.keys.iter().any(|conflict| conflict.actual_version == Some(6)))
    );

    let create_database = create_db();
    let create_result = create_database
        .trx(|transaction| {
            let create_database = create_database.clone();
            async move {
                create_database.delete("contract", "new").await?;
                assert!(
                    transaction
                        .get(ContractDocumentGet {
                            id: "new".to_string(),
                        })
                        .await?
                        .is_none()
                );
                let handle = transaction.create(ContractDocument {
                    id: "new".to_string(),
                    value: "created".to_string(),
                })?;
                drop(handle);
                create_database
                    .put(
                        "contract",
                        "new",
                        &serde_json::to_vec(&ContractDocument {
                            id: "new".to_string(),
                            value: "concurrent-create".to_string(),
                        })
                        .unwrap(),
                    )
                    .await?;
                transaction.commit::<(), ()>(())
            }
        })
        .await;
    assert!(
        matches!(create_result, TrxResult::Conflict(details) if details.keys.iter().any(|conflict| conflict.expected_version.is_none() && conflict.actual_version == Some(0)))
    );

    let delete_database = create_db();
    delete_database
        .put(
            "contract",
            "delete",
            &serde_json::to_vec(&ContractDocument {
                id: "delete".to_string(),
                value: "before-delete".to_string(),
            })
            .unwrap(),
        )
        .await
        .unwrap();
    let delete_result = delete_database
        .trx(|transaction| {
            let delete_database = delete_database.clone();
            async move {
                let handle = transaction
                    .get(ContractDocumentGet {
                        id: "delete".to_string(),
                    })
                    .await?
                    .expect("document to delete");
                handle.delete();
                drop(handle);
                delete_database
                    .put(
                        "contract",
                        "delete",
                        &serde_json::to_vec(&ContractDocument {
                            id: "delete".to_string(),
                            value: "concurrent-delete".to_string(),
                        })
                        .unwrap(),
                    )
                    .await?;
                transaction.commit::<(), ()>(())
            }
        })
        .await;
    assert!(
        matches!(delete_result, TrxResult::Conflict(details) if details.keys.iter().any(|conflict| conflict.expected_version == Some(4) && conflict.actual_version == Some(5)))
    );
}

async fn run_optimistic_conflict_pair_contract(create_db: &impl Fn() -> Database) {
    let update_database = create_db();
    update_database
        .put("conflict", "update", b"v0")
        .await
        .unwrap();
    let conflict_key = vec![("conflict".to_string(), "update".to_string())];
    let (mut first_update, first_reads) = update_database
        .begin_immediate_with_reads(&conflict_key)
        .await
        .unwrap();
    let (mut second_update, second_reads) = update_database
        .begin_immediate_with_reads(&conflict_key)
        .await
        .unwrap();
    assert_eq!(
        first_reads[0].as_ref().map(|document| document.version),
        Some(0)
    );
    assert_eq!(
        second_reads[0].as_ref().map(|document| document.version),
        Some(0)
    );
    let first_update_outcome = first_update
        .apply_writes_and_commit(&[WriteOp::Update {
            pk: "conflict".to_string(),
            sk: "update".to_string(),
            expected_version: 0,
            data: b"first".to_vec(),
        }])
        .await
        .unwrap();
    let second_update_outcome = second_update
        .apply_writes_and_commit(&[WriteOp::Update {
            pk: "conflict".to_string(),
            sk: "update".to_string(),
            expected_version: 0,
            data: b"second".to_vec(),
        }])
        .await
        .unwrap();
    assert_eq!(first_update_outcome.affected_counts, vec![1]);
    assert!(first_update_outcome.conflict.is_none());
    assert_eq!(second_update_outcome.affected_counts, vec![0]);
    assert!(second_update_outcome.conflict.is_some());
    assert_eq!(
        update_database.get("conflict", "update").await.unwrap(),
        Some(Bytes::from_static(b"first"))
    );

    let create_database = create_db();
    let create_key = vec![("conflict".to_string(), "create".to_string())];
    let (mut first_create, first_create_reads) = create_database
        .begin_immediate_with_reads(&create_key)
        .await
        .unwrap();
    let (mut second_create, second_create_reads) = create_database
        .begin_immediate_with_reads(&create_key)
        .await
        .unwrap();
    assert!(first_create_reads[0].is_none());
    assert!(second_create_reads[0].is_none());
    let first_create_outcome = first_create
        .apply_writes_and_commit(&[WriteOp::Insert {
            pk: "conflict".to_string(),
            sk: "create".to_string(),
            data: b"first".to_vec(),
        }])
        .await
        .unwrap();
    let second_create_outcome = second_create
        .apply_writes_and_commit(&[WriteOp::Insert {
            pk: "conflict".to_string(),
            sk: "create".to_string(),
            data: b"second".to_vec(),
        }])
        .await
        .unwrap();
    assert_eq!(first_create_outcome.affected_counts, vec![1]);
    assert_eq!(second_create_outcome.affected_counts, vec![0]);
    assert!(second_create_outcome.conflict.is_some());

    let delete_database = create_db();
    delete_database
        .put("conflict", "delete", b"v0")
        .await
        .unwrap();
    let delete_key = vec![("conflict".to_string(), "delete".to_string())];
    let (mut first_delete, _) = delete_database
        .begin_immediate_with_reads(&delete_key)
        .await
        .unwrap();
    let (mut second_delete, _) = delete_database
        .begin_immediate_with_reads(&delete_key)
        .await
        .unwrap();
    let first_delete_outcome = first_delete
        .apply_writes_and_commit(&[WriteOp::Update {
            pk: "conflict".to_string(),
            sk: "delete".to_string(),
            expected_version: 0,
            data: b"updated".to_vec(),
        }])
        .await
        .unwrap();
    let second_delete_outcome = second_delete
        .apply_writes_and_commit(&[WriteOp::Delete {
            pk: "conflict".to_string(),
            sk: "delete".to_string(),
            expected_version: 0,
        }])
        .await
        .unwrap();
    assert_eq!(first_delete_outcome.affected_counts, vec![1]);
    assert_eq!(second_delete_outcome.affected_counts, vec![0]);
    assert!(second_delete_outcome.conflict.is_some());
}

async fn run_execute_ops_contract(create_db: &impl Fn() -> Database) {
    let database = create_db();
    database.put("ops", "existing", b"existing").await.unwrap();
    let results = database
        .execute_ops(vec![
            DbOp::Get {
                pk: "ops".to_string(),
                sk: "existing".to_string(),
            },
            DbOp::Query {
                pk: "ops".to_string(),
                after_sk: None,
                limit: Some(10),
            },
            DbOp::Put {
                pk: "ops".to_string(),
                sk: "created".to_string(),
                data: b"created".to_vec(),
            },
            DbOp::Delete {
                pk: "ops".to_string(),
                sk: "created".to_string(),
            },
        ])
        .await
        .unwrap();
    assert!(matches!(&results[0], DbResult::Single(Some(data)) if data.as_ref() == b"existing"));
    assert!(matches!(&results[1], DbResult::Multiple(items) if items.len() == 1));
    assert!(matches!(&results[2], DbResult::Done));
    assert!(matches!(&results[3], DbResult::Done));

    struct ContractRequest;

    impl DbRequest for ContractRequest {
        type Output = Option<Vec<u8>>;

        fn prepare(self) -> Prepared<Self::Output> {
            Prepared {
                ops: vec![DbOp::Get {
                    pk: "ops".to_string(),
                    sk: "existing".to_string(),
                }],
                parse: Box::new(|results| match results.next() {
                    Some(DbResult::Single(data)) => Ok(data.map(|bytes| bytes.to_vec())),
                    _ => Err(anyhow::anyhow!("unexpected execute_ops result")),
                }),
            }
        }
    }

    assert_eq!(
        ContractRequest.send_with(&database).await.unwrap(),
        Some(b"existing".to_vec())
    );
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::run_backend_contract;

    #[tokio::test]
    async fn memory_backend_contract() {
        run_backend_contract(crate::memory).await;
    }
}
