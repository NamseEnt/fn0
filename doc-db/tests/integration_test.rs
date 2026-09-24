use doc_db::{
    DbOp, DbRequest, DbResult, DocGet, DocKey, Document, Prepared, TrxResult, turso_with_config,
};
use serde::{Deserialize, Serialize};

forte_sdk::test_main!();

/// The `libsql-test` service in `docker-compose.yml`.
const DEFAULT_TEST_URL: &str = "http://127.0.0.1:18123";

fn create_test_db() -> doc_db::Database {
    let url = std::env::var("DOC_DB_TEST_URL").unwrap_or_else(|_| DEFAULT_TEST_URL.to_string());
    turso_with_config(url, String::new())
}

#[derive(Deserialize, Serialize)]
struct TrxDoc {
    id: String,
    value: i32,
}

impl Document for TrxDoc {
    fn key(&self) -> DocKey {
        DocKey::new("TrxIntegrationDoc", format!("id={}", self.id))
    }
}

struct TrxDocGet {
    id: &'static str,
}

impl DocGet for TrxDocGet {
    type Doc = TrxDoc;

    fn key(&self) -> DocKey {
        DocKey::new("TrxIntegrationDoc", format!("id={}", self.id))
    }
}

fn trx_get(id: &'static str) -> TrxDocGet {
    TrxDocGet { id }
}

struct RawGet {
    pk: String,
    sk: String,
}

impl DbRequest for RawGet {
    type Output = Option<Vec<u8>>;

    fn prepare(self) -> Prepared<Self::Output> {
        Prepared {
            ops: vec![DbOp::Get {
                pk: self.pk,
                sk: self.sk,
            }],
            parse: Box::new(|iter| match iter.next().unwrap() {
                DbResult::Single(value) => Ok(value.map(|bytes| bytes.to_vec())),
                _ => panic!("unexpected get result"),
            }),
        }
    }
}

struct RawQuery {
    pk: String,
    limit: Option<usize>,
}

impl DbRequest for RawQuery {
    type Output = Vec<(String, Vec<u8>)>;

    fn prepare(self) -> Prepared<Self::Output> {
        Prepared {
            ops: vec![DbOp::Query {
                pk: self.pk,
                after_sk: None,
                limit: self.limit,
            }],
            parse: Box::new(|iter| match iter.next().unwrap() {
                DbResult::Multiple(items) => Ok(items
                    .into_iter()
                    .map(|(sk, bytes)| (sk, bytes.to_vec()))
                    .collect()),
                _ => panic!("unexpected query result"),
            }),
        }
    }
}

struct RawPut {
    pk: String,
    sk: String,
    data: Vec<u8>,
}

impl DbRequest for RawPut {
    type Output = ();

    fn prepare(self) -> Prepared<Self::Output> {
        Prepared {
            ops: vec![DbOp::Put {
                pk: self.pk,
                sk: self.sk,
                data: self.data,
            }],
            parse: Box::new(|iter| match iter.next().unwrap() {
                DbResult::Done => Ok(()),
                _ => panic!("unexpected put result"),
            }),
        }
    }
}

struct RawDelete {
    pk: String,
    sk: String,
}

impl DbRequest for RawDelete {
    type Output = ();

    fn prepare(self) -> Prepared<Self::Output> {
        Prepared {
            ops: vec![DbOp::Delete {
                pk: self.pk,
                sk: self.sk,
            }],
            parse: Box::new(|iter| match iter.next().unwrap() {
                DbResult::Done => Ok(()),
                _ => panic!("unexpected delete result"),
            }),
        }
    }
}

async fn put_trx_doc(db: &doc_db::Database, id: &'static str, value: i32) {
    db.put(
        "TrxIntegrationDoc",
        &format!("id={id}"),
        &serde_json::to_vec(&TrxDoc {
            id: id.to_string(),
            value,
        })
        .unwrap(),
    )
    .await
    .unwrap();
}

async fn cleanup_trx_docs(db: &doc_db::Database, ids: &[&str]) {
    for id in ids {
        db.delete("TrxIntegrationDoc", &format!("id={id}"))
            .await
            .unwrap();
    }
}

#[forte_sdk::test]
async fn test_put_and_get() {
    let db = create_test_db();

    let test_pk = "test_pk";
    let test_sk = "test_sk";
    let test_data = b"hello world";

    // Put data (table will be created automatically if not exists)
    db.put(test_pk, test_sk, test_data)
        .await
        .expect("Failed to put data");

    // Get data
    let result = db.get(test_pk, test_sk).await.expect("Failed to get data");

    assert!(result.is_some(), "Should return inserted data");
    assert_eq!(result.unwrap().as_ref(), test_data);

    // Cleanup
    db.delete(test_pk, test_sk)
        .await
        .expect("Failed to delete data");
}

#[forte_sdk::test]
async fn test_get_nonexistent() {
    let db = create_test_db();

    // Table will be created automatically if not exists
    let result = db
        .get("nonexistent_pk", "nonexistent_sk")
        .await
        .expect("Query should succeed");

    assert!(result.is_none(), "Should return None for nonexistent key");
}

#[forte_sdk::test]
async fn test_update_existing() {
    let db = create_test_db();

    let test_pk = "update_test_pk";
    let test_sk = "update_test_sk";
    let initial_data = b"initial data";
    let updated_data = b"updated data";

    // Put initial data
    db.put(test_pk, test_sk, initial_data)
        .await
        .expect("Failed to put initial data");

    // Update with new data
    db.put(test_pk, test_sk, updated_data)
        .await
        .expect("Failed to update data");

    // Verify update
    let result = db.get(test_pk, test_sk).await.expect("Failed to get data");

    assert!(result.is_some(), "Should return updated data");
    assert_eq!(result.unwrap().as_ref(), updated_data);

    // Cleanup
    db.delete(test_pk, test_sk)
        .await
        .expect("Failed to delete data");
}

#[forte_sdk::test]
async fn test_delete() {
    let db = create_test_db();

    let test_pk = "delete_test_pk";
    let test_sk = "delete_test_sk";
    let test_data = b"to be deleted";

    // Put data
    db.put(test_pk, test_sk, test_data)
        .await
        .expect("Failed to put data");

    // Verify data exists
    let result = db.get(test_pk, test_sk).await.expect("Failed to get data");
    assert!(result.is_some(), "Data should exist before delete");

    // Delete data
    db.delete(test_pk, test_sk)
        .await
        .expect("Failed to delete data");

    // Verify data is gone
    let result = db.get(test_pk, test_sk).await.expect("Failed to get data");
    assert!(result.is_none(), "Data should be gone after delete");
}

#[forte_sdk::test]
async fn test_query() {
    let db = create_test_db();

    let pk = "query_test_pk";

    // Insert multiple items with same pk
    for i in 0..5 {
        let sk = format!("sk_{:02}", i);
        let data = format!("data_{}", i);
        db.put(pk, &sk, data.as_bytes())
            .await
            .expect("Failed to put data");
    }

    // Query first page (limit 2)
    let page1 = db.query(pk, None::<&str>, 2).await.expect("Query failed");
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0].0, "sk_00");
    assert_eq!(page1[1].0, "sk_01");

    // Query second page using last sk as cursor
    let page2 = db
        .query(pk, Some(&page1[1].0), 2)
        .await
        .expect("Query failed");
    assert_eq!(page2.len(), 2);
    assert_eq!(page2[0].0, "sk_02");
    assert_eq!(page2[1].0, "sk_03");

    // Query third page
    let page3 = db
        .query(pk, Some(&page2[1].0), 2)
        .await
        .expect("Query failed");
    assert_eq!(page3.len(), 1);
    assert_eq!(page3[0].0, "sk_04");

    // Cleanup
    for i in 0..5 {
        let sk = format!("sk_{:02}", i);
        db.delete(pk, &sk).await.expect("Failed to delete");
    }
}

#[forte_sdk::test]
async fn test_scan() {
    let db = create_test_db();

    // Insert items with different pks
    let items = [
        ("scan_pk_a", "sk_1"),
        ("scan_pk_a", "sk_2"),
        ("scan_pk_b", "sk_1"),
        ("scan_pk_b", "sk_2"),
        ("scan_pk_c", "sk_1"),
    ];

    for (pk, sk) in &items {
        let data = format!("{}:{}", pk, sk);
        db.put(pk, sk, data.as_bytes())
            .await
            .expect("Failed to put data");
    }

    // Scan first page (limit 2)
    let page1 = db.scan(None, 2).await.expect("Scan failed");
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0].0, "scan_pk_a");
    assert_eq!(page1[0].1, "sk_1");
    assert_eq!(page1[1].0, "scan_pk_a");
    assert_eq!(page1[1].1, "sk_2");

    // Scan second page using last (pk, sk) as cursor
    let page2 = db
        .scan(Some((&page1[1].0, &page1[1].1)), 2)
        .await
        .expect("Scan failed");
    assert_eq!(page2.len(), 2);
    assert_eq!(page2[0].0, "scan_pk_b");
    assert_eq!(page2[0].1, "sk_1");
    assert_eq!(page2[1].0, "scan_pk_b");
    assert_eq!(page2[1].1, "sk_2");

    // Scan third page
    let page3 = db
        .scan(Some((&page2[1].0, &page2[1].1)), 2)
        .await
        .expect("Scan failed");
    assert_eq!(page3.len(), 1);
    assert_eq!(page3[0].0, "scan_pk_c");
    assert_eq!(page3[0].1, "sk_1");

    // Cleanup
    for (pk, sk) in &items {
        db.delete(pk, sk).await.expect("Failed to delete");
    }
}

#[forte_sdk::test]
async fn test_unconditional_transaction() {
    let db = create_test_db();

    assert!(matches!(
        db.trx(|trx| async move {
            trx.create(TrxDoc {
                id: "one".to_string(),
                value: 1,
            })?;
            trx.create(TrxDoc {
                id: "two".to_string(),
                value: 2,
            })?;
            trx.create(TrxDoc {
                id: "three".to_string(),
                value: 3,
            })?;
            trx.commit::<(), ()>(())
        })
        .await,
        TrxResult::Committed(())
    ));

    let rows = db
        .query("TrxIntegrationDoc", None::<&str>, 10)
        .await
        .expect("Query failed");
    assert_eq!(rows.len(), 3);

    assert!(matches!(
        db.trx(|trx| async move {
            let doc = trx.get(trx_get("two")).await?.unwrap();
            doc.delete();
            trx.commit::<(), ()>(())
        })
        .await,
        TrxResult::Committed(())
    ));

    let rows = db
        .query("TrxIntegrationDoc", None::<&str>, 10)
        .await
        .expect("Query failed");
    assert_eq!(rows.len(), 2);
    cleanup_trx_docs(&db, &["one", "three"]).await;
}

#[forte_sdk::test]
async fn test_transaction_commit() {
    let db = create_test_db();

    let pk = "tx_commit_pk";

    // Start transaction
    let mut tx = db.transaction().await.expect("Transaction begin failed");

    // Put data in transaction
    tx.put(pk, "sk_1", b"data_1")
        .await
        .expect("Transaction put failed");
    tx.put(pk, "sk_2", b"data_2")
        .await
        .expect("Transaction put failed");

    // Data should be visible within transaction
    let result = tx.get(pk, "sk_1").await.expect("Transaction get failed");
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ref(), b"data_1");

    // Commit transaction
    tx.commit().await.expect("Transaction commit failed");

    // Data should be visible after commit
    let result = db.get(pk, "sk_1").await.expect("Get failed");
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ref(), b"data_1");

    let result = db.get(pk, "sk_2").await.expect("Get failed");
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ref(), b"data_2");

    // Cleanup
    db.delete(pk, "sk_1").await.expect("Delete failed");
    db.delete(pk, "sk_2").await.expect("Delete failed");
}

#[forte_sdk::test]
async fn test_transaction_rollback() {
    let db = create_test_db();

    let pk = "tx_rollback_pk";

    // Put initial data
    db.put(pk, "sk_existing", b"original")
        .await
        .expect("Put failed");

    // Start transaction
    let mut tx = db.transaction().await.expect("Transaction begin failed");

    // Modify data in transaction
    tx.put(pk, "sk_existing", b"modified")
        .await
        .expect("Transaction put failed");
    tx.put(pk, "sk_new", b"new data")
        .await
        .expect("Transaction put failed");

    // Data should be modified within transaction
    let result = tx
        .get(pk, "sk_existing")
        .await
        .expect("Transaction get failed");
    assert_eq!(result.unwrap().as_ref(), b"modified");

    // Rollback transaction
    tx.rollback().await.expect("Transaction rollback failed");

    // Original data should be restored
    let result = db.get(pk, "sk_existing").await.expect("Get failed");
    assert_eq!(result.unwrap().as_ref(), b"original");

    // New data should not exist
    let result = db.get(pk, "sk_new").await.expect("Get failed");
    assert!(result.is_none());

    // Cleanup
    db.delete(pk, "sk_existing").await.expect("Delete failed");
}

#[forte_sdk::test]
async fn test_transaction_delete() {
    let db = create_test_db();

    let pk = "tx_delete_pk";

    // Put initial data
    db.put(pk, "sk_to_delete", b"delete me")
        .await
        .expect("Put failed");

    // Start transaction and delete
    let mut tx = db.transaction().await.expect("Transaction begin failed");
    tx.delete(pk, "sk_to_delete")
        .await
        .expect("Transaction delete failed");

    // Commit
    tx.commit().await.expect("Transaction commit failed");

    // Data should be gone
    let result = db.get(pk, "sk_to_delete").await.expect("Get failed");
    assert!(result.is_none());
}

#[forte_sdk::test]
async fn test_send_with_single_get() {
    let db = create_test_db();
    let pk = "exec_ops_get_pk";
    let sk = "sk_1";

    db.put(pk, sk, b"hello").await.expect("Put failed");

    let result = (RawGet {
        pk: pk.to_string(),
        sk: sk.to_string(),
    })
    .send_with(&db)
    .await
    .expect("send_with failed");
    assert_eq!(result.as_deref(), Some(b"hello".as_slice()));

    db.delete(pk, sk).await.expect("Delete failed");
}

#[forte_sdk::test]
async fn test_send_with_vec_gets() {
    let db = create_test_db();
    let pk = "exec_ops_multi_pk";

    db.put(pk, "sk_a", b"aaa").await.expect("Put failed");
    db.put(pk, "sk_b", b"bbb").await.expect("Put failed");

    let results = vec![
        RawGet {
            pk: pk.to_string(),
            sk: "sk_a".to_string(),
        },
        RawGet {
            pk: pk.to_string(),
            sk: "sk_b".to_string(),
        },
        RawGet {
            pk: pk.to_string(),
            sk: "sk_nonexistent".to_string(),
        },
    ]
    .send_with(&db)
    .await
    .expect("send_with failed");

    assert_eq!(results[0].as_deref(), Some(b"aaa".as_slice()));
    assert_eq!(results[1].as_deref(), Some(b"bbb".as_slice()));
    assert!(results[2].is_none());

    db.delete(pk, "sk_a").await.expect("Delete failed");
    db.delete(pk, "sk_b").await.expect("Delete failed");
}

#[forte_sdk::test]
async fn test_send_with_single_query() {
    let db = create_test_db();
    let pk = "exec_ops_query_pk";

    for i in 0..5 {
        let sk = format!("sk_{:02}", i);
        db.put(pk, &sk, format!("data_{}", i).as_bytes())
            .await
            .expect("Put failed");
    }

    let results = (RawQuery {
        pk: pk.to_string(),
        limit: Some(3),
    })
    .send_with(&db)
    .await
    .expect("send_with failed");
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, "sk_00");
    assert_eq!(results[1].0, "sk_01");
    assert_eq!(results[2].0, "sk_02");

    for i in 0..5 {
        let sk = format!("sk_{:02}", i);
        db.delete(pk, &sk).await.expect("Delete failed");
    }
}

#[forte_sdk::test]
async fn test_send_with_tuple_put_and_delete() {
    let db = create_test_db();
    let pk = "exec_ops_put_del_pk";

    (
        RawPut {
            pk: pk.to_string(),
            sk: "sk_1".to_string(),
            data: b"data_1".to_vec(),
        },
        RawPut {
            pk: pk.to_string(),
            sk: "sk_2".to_string(),
            data: b"data_2".to_vec(),
        },
    )
        .send_with(&db)
        .await
        .expect("send_with put failed");

    let got = db.get(pk, "sk_1").await.expect("Get failed");
    assert_eq!(got.unwrap().as_ref(), b"data_1");
    let got = db.get(pk, "sk_2").await.expect("Get failed");
    assert_eq!(got.unwrap().as_ref(), b"data_2");

    (RawDelete {
        pk: pk.to_string(),
        sk: "sk_1".to_string(),
    })
    .send_with(&db)
    .await
    .expect("send_with delete failed");

    let got = db.get(pk, "sk_1").await.expect("Get failed");
    assert!(got.is_none());

    db.delete(pk, "sk_2").await.expect("Cleanup failed");
}

#[forte_sdk::test]
async fn test_send_with_heterogeneous_independent_operations() {
    let db = create_test_db();
    let pk = "exec_ops_mixed_pk";

    db.put(pk, "sk_existing", b"old_data")
        .await
        .expect("Put failed");

    let (existing, (), items) = (
        RawGet {
            pk: pk.to_string(),
            sk: "sk_existing".to_string(),
        },
        RawPut {
            pk: pk.to_string(),
            sk: "sk_new".to_string(),
            data: b"new_data".to_vec(),
        },
        RawQuery {
            pk: pk.to_string(),
            limit: None,
        },
    )
        .send_with(&db)
        .await
        .expect("send_with mixed failed");

    assert_eq!(existing.as_deref(), Some(b"old_data".as_slice()));
    assert!(
        items
            .iter()
            .any(|(sk, data)| sk == "sk_existing" && data.as_slice() == b"old_data")
    );
    assert_eq!(
        db.get(pk, "sk_new").await.expect("Get failed").as_deref(),
        Some(b"new_data".as_slice())
    );

    db.delete(pk, "sk_existing").await.expect("Cleanup failed");
    db.delete(pk, "sk_new").await.expect("Cleanup failed");
}

#[forte_sdk::test]
async fn test_execute_raw_transactional_committed_with_row_counters() {
    let db = create_test_db();
    let pk = "raw_trx_pk";

    db.put(pk, "sk_1", b"data_1").await.expect("Put failed");
    db.put(pk, "sk_2", b"data_2").await.expect("Put failed");

    let outcome = db
        .execute_raw_transactional(&[
            doc_db::RawStatement {
                sql: "SELECT sk, data FROM docs WHERE pk = ? ORDER BY sk".to_string(),
                args: vec![doc_db::text_value(pk)],
            },
            doc_db::RawStatement {
                sql: "UPDATE docs SET data = ?, version = version + 1 WHERE pk = ? AND sk = ?"
                    .to_string(),
                args: vec![
                    doc_db::Value::Blob {
                        value: b"data_1_migrated".to_vec().into(),
                    },
                    doc_db::text_value(pk),
                    doc_db::text_value("sk_1"),
                ],
            },
        ])
        .await
        .expect("execute_raw_transactional failed");

    let statement_results = match outcome {
        doc_db::RawTransactionOutcome::Committed { statement_results } => statement_results,
        doc_db::RawTransactionOutcome::RolledBack {
            failed_statement_index,
            error_message,
        } => panic!("Unexpected rollback at {failed_statement_index}: {error_message}"),
    };

    assert_eq!(statement_results.len(), 2);

    let select_result = &statement_results[0];
    assert_eq!(select_result.column_names, vec!["sk", "data"]);
    assert_eq!(select_result.rows.len(), 2);
    assert!(select_result.rows_read >= 2);
    assert_eq!(select_result.rows_written, 0);

    let update_result = &statement_results[1];
    assert_eq!(update_result.affected_row_count, 1);
    assert!(update_result.rows_written >= 1);

    let migrated = db.get(pk, "sk_1").await.expect("Get failed");
    assert_eq!(migrated.unwrap().as_ref(), b"data_1_migrated");

    db.delete(pk, "sk_1").await.expect("Cleanup failed");
    db.delete(pk, "sk_2").await.expect("Cleanup failed");
}

#[forte_sdk::test]
async fn test_execute_raw_transactional_rolls_back_on_statement_error() {
    let db = create_test_db();
    let pk = "raw_trx_rollback_pk";

    db.put(pk, "sk_1", b"original").await.expect("Put failed");

    let outcome = db
        .execute_raw_transactional(&[
            doc_db::RawStatement {
                sql: "UPDATE docs SET data = ?, version = version + 1 WHERE pk = ? AND sk = ?"
                    .to_string(),
                args: vec![
                    doc_db::Value::Blob {
                        value: b"should_not_persist".to_vec().into(),
                    },
                    doc_db::text_value(pk),
                    doc_db::text_value("sk_1"),
                ],
            },
            doc_db::RawStatement {
                sql: "SELECT broken_column FROM no_such_table_for_rollback".to_string(),
                args: vec![],
            },
        ])
        .await
        .expect("execute_raw_transactional failed");

    match outcome {
        doc_db::RawTransactionOutcome::RolledBack {
            failed_statement_index,
            error_message,
        } => {
            assert_eq!(failed_statement_index, 1);
            assert!(!error_message.is_empty());
        }
        doc_db::RawTransactionOutcome::Committed { .. } => panic!("Expected rollback"),
    }

    let unchanged = db.get(pk, "sk_1").await.expect("Get failed");
    assert_eq!(unchanged.unwrap().as_ref(), b"original");

    db.delete(pk, "sk_1").await.expect("Cleanup failed");
}

#[forte_sdk::test]
async fn test_trx_read_only_key_conflict_is_atomic() {
    let db = create_test_db();
    cleanup_trx_docs(
        &db,
        &[
            "read-a",
            "read-b",
            "written",
            "atomic-a",
            "atomic-b",
            "missing",
            "create-conflict",
            "read-without-lock",
            "outside",
            "read-only",
        ],
    )
    .await;
    put_trx_doc(&db, "read-a", 1).await;
    put_trx_doc(&db, "read-b", 1).await;

    let result = db
        .trx(|trx| {
            let db = db.clone();
            async move {
                let (a, b) = trx.get((trx_get("read-a"), trx_get("read-b"))).await?;
                let mut a = a.unwrap();
                let b = b.unwrap();
                a.value = 2;
                drop(a);
                drop(b);
                put_trx_doc(&db, "read-b", 9).await;
                trx.commit::<(), ()>(())
            }
        })
        .await;

    assert!(matches!(result, TrxResult::Conflict(_)));
    let a = db
        .get("TrxIntegrationDoc", "id=read-a")
        .await
        .unwrap()
        .unwrap();
    let b = db
        .get("TrxIntegrationDoc", "id=read-b")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(serde_json::from_slice::<TrxDoc>(&a).unwrap().value, 1);
    assert_eq!(serde_json::from_slice::<TrxDoc>(&b).unwrap().value, 9);
    cleanup_trx_docs(&db, &["read-a", "read-b"]).await;
}

#[forte_sdk::test]
async fn test_trx_written_key_conflict_preserves_external_write() {
    let db = create_test_db();
    cleanup_trx_docs(&db, &["written"]).await;
    put_trx_doc(&db, "written", 1).await;

    let result = db
        .trx(|trx| {
            let db = db.clone();
            async move {
                let doc = trx.get(trx_get("written")).await?.unwrap();
                let mut doc = doc;
                doc.value = 2;
                drop(doc);
                put_trx_doc(&db, "written", 9).await;
                trx.commit::<(), ()>(())
            }
        })
        .await;

    assert!(matches!(result, TrxResult::Conflict(_)));
    let value = db
        .get("TrxIntegrationDoc", "id=written")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(serde_json::from_slice::<TrxDoc>(&value).unwrap().value, 9);
    cleanup_trx_docs(&db, &["written"]).await;
}

#[forte_sdk::test]
async fn test_trx_conflict_does_not_partially_commit_writes() {
    let db = create_test_db();
    cleanup_trx_docs(&db, &["atomic-a", "atomic-b"]).await;
    put_trx_doc(&db, "atomic-a", 1).await;
    put_trx_doc(&db, "atomic-b", 1).await;

    let result = db
        .trx(|trx| {
            let db = db.clone();
            async move {
                let (a, b) = trx.get((trx_get("atomic-a"), trx_get("atomic-b"))).await?;
                let mut a = a.unwrap();
                let mut b = b.unwrap();
                a.value = 2;
                b.value = 2;
                drop(a);
                drop(b);
                put_trx_doc(&db, "atomic-b", 9).await;
                trx.commit::<(), ()>(())
            }
        })
        .await;

    assert!(matches!(result, TrxResult::Conflict(_)));
    let a = db
        .get("TrxIntegrationDoc", "id=atomic-a")
        .await
        .unwrap()
        .unwrap();
    let b = db
        .get("TrxIntegrationDoc", "id=atomic-b")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(serde_json::from_slice::<TrxDoc>(&a).unwrap().value, 1);
    assert_eq!(serde_json::from_slice::<TrxDoc>(&b).unwrap().value, 9);
    cleanup_trx_docs(&db, &["atomic-a", "atomic-b"]).await;
}

#[forte_sdk::test]
async fn test_trx_missing_read_conflicts_when_key_is_inserted() {
    let db = create_test_db();
    cleanup_trx_docs(&db, &["missing"]).await;

    let result = db
        .trx(|trx| {
            let db = db.clone();
            async move {
                db.delete("TrxIntegrationDoc", "id=missing").await?;
                assert!(trx.get(trx_get("missing")).await?.is_none());
                put_trx_doc(&db, "missing", 9).await;
                trx.commit::<(), ()>(())
            }
        })
        .await;

    assert!(matches!(result, TrxResult::Conflict(_)));
    assert!(
        db.get("TrxIntegrationDoc", "id=missing")
            .await
            .unwrap()
            .is_some()
    );
    cleanup_trx_docs(&db, &["missing"]).await;
}

#[forte_sdk::test]
async fn test_trx_missing_read_then_create_conflicts() {
    let db = create_test_db();
    cleanup_trx_docs(&db, &["create-conflict"]).await;

    let result = db
        .trx(|trx| {
            let db = db.clone();
            async move {
                db.delete("TrxIntegrationDoc", "id=create-conflict").await?;
                assert!(trx.get(trx_get("create-conflict")).await?.is_none());
                let created = trx.create(TrxDoc {
                    id: "create-conflict".to_string(),
                    value: 2,
                })?;
                drop(created);
                put_trx_doc(&db, "create-conflict", 9).await;
                trx.commit::<(), ()>(())
            }
        })
        .await;

    assert!(matches!(result, TrxResult::Conflict(_)));
    let value = db
        .get("TrxIntegrationDoc", "id=create-conflict")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(serde_json::from_slice::<TrxDoc>(&value).unwrap().value, 9);
    cleanup_trx_docs(&db, &["create-conflict"]).await;
}

#[forte_sdk::test]
async fn test_trx_closure_can_use_database_before_commit() {
    let db = create_test_db();
    cleanup_trx_docs(&db, &["read-without-lock", "outside"]).await;
    put_trx_doc(&db, "read-without-lock", 1).await;

    let result = db
        .trx(|trx| {
            let db = db.clone();
            async move {
                let doc = trx.get(trx_get("read-without-lock")).await?;
                drop(doc);
                put_trx_doc(&db, "outside", 7).await;
                trx.commit::<(), ()>(())
            }
        })
        .await;

    assert!(matches!(result, TrxResult::Committed(())));
    assert!(
        db.get("TrxIntegrationDoc", "id=outside")
            .await
            .unwrap()
            .is_some()
    );
    cleanup_trx_docs(&db, &["read-without-lock", "outside"]).await;
}

#[forte_sdk::test]
async fn test_trx_read_only_commit_validates_without_writes() {
    let db = create_test_db();
    cleanup_trx_docs(&db, &["read-only"]).await;
    put_trx_doc(&db, "read-only", 1).await;

    let before_rows = db
        .execute_raw(
            "SELECT data, version FROM docs WHERE pk = ? AND sk = ?",
            vec![
                doc_db::text_value("TrxIntegrationDoc"),
                doc_db::text_value("id=read-only"),
            ],
            true,
        )
        .await
        .unwrap();
    let before_version = match &before_rows[0][1] {
        doc_db::Value::Integer { value } => *value,
        _ => panic!("expected integer version"),
    };

    let result = db
        .trx(|trx| async move {
            let doc = trx.get(trx_get("read-only")).await?;
            drop(doc);
            trx.commit::<(), ()>(())
        })
        .await;

    assert!(matches!(result, TrxResult::Committed(())));
    let rows = db
        .execute_raw(
            "SELECT data, version FROM docs WHERE pk = ? AND sk = ?",
            vec![
                doc_db::text_value("TrxIntegrationDoc"),
                doc_db::text_value("id=read-only"),
            ],
            true,
        )
        .await
        .unwrap();
    match rows[0][1] {
        doc_db::Value::Integer { value } => assert_eq!(value, before_version),
        _ => panic!("expected integer version"),
    }
    cleanup_trx_docs(&db, &["read-only"]).await;
}
