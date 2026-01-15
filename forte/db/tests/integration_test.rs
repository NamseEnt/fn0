use forte_db::{BatchOp, turso_with_config};

const LOCAL_LIBSQL_URL: &str = "http://127.0.0.1:8080";

fn create_test_db() -> forte_db::Database {
    turso_with_config(LOCAL_LIBSQL_URL.to_string(), String::new())
}

#[wstd::test]
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

#[wstd::test]
async fn test_get_nonexistent() {
    let db = create_test_db();

    // Table will be created automatically if not exists
    let result = db
        .get("nonexistent_pk", "nonexistent_sk")
        .await
        .expect("Query should succeed");

    assert!(result.is_none(), "Should return None for nonexistent key");
}

#[wstd::test]
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

#[wstd::test]
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

#[wstd::test]
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

#[wstd::test]
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

#[wstd::test]
async fn test_batch_put() {
    let db = create_test_db();

    let pk = "batch_test_pk";

    // Batch put multiple items
    let ops = vec![
        BatchOp::Put {
            pk,
            sk: "sk_1",
            data: b"data_1",
        },
        BatchOp::Put {
            pk,
            sk: "sk_2",
            data: b"data_2",
        },
        BatchOp::Put {
            pk,
            sk: "sk_3",
            data: b"data_3",
        },
    ];

    db.batch(&ops).await.expect("Batch put failed");

    // Verify all items were inserted
    let result = db.query(pk, None::<&str>, 10).await.expect("Query failed");
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].0, "sk_1");
    assert_eq!(result[1].0, "sk_2");
    assert_eq!(result[2].0, "sk_3");

    // Cleanup with batch delete
    let delete_ops = vec![
        BatchOp::Delete { pk, sk: "sk_1" },
        BatchOp::Delete { pk, sk: "sk_2" },
        BatchOp::Delete { pk, sk: "sk_3" },
    ];
    db.batch(&delete_ops).await.expect("Batch delete failed");

    // Verify all items were deleted
    let result = db.query(pk, None::<&str>, 10).await.expect("Query failed");
    assert_eq!(result.len(), 0);
}

#[wstd::test]
async fn test_batch_mixed_operations() {
    let db = create_test_db();

    let pk = "batch_mixed_pk";

    // First, put some initial data
    db.put(pk, "sk_to_keep", b"keep this")
        .await
        .expect("Put failed");
    db.put(pk, "sk_to_delete", b"delete this")
        .await
        .expect("Put failed");

    // Batch: add new item, update existing, delete another
    let ops = vec![
        BatchOp::Put {
            pk,
            sk: "sk_new",
            data: b"new data",
        },
        BatchOp::Put {
            pk,
            sk: "sk_to_keep",
            data: b"updated data",
        },
        BatchOp::Delete {
            pk,
            sk: "sk_to_delete",
        },
    ];

    db.batch(&ops).await.expect("Batch mixed failed");

    // Verify results
    let kept = db
        .get(pk, "sk_to_keep")
        .await
        .expect("Get failed")
        .expect("Should exist");
    assert_eq!(kept.as_ref(), b"updated data");

    let new_item = db
        .get(pk, "sk_new")
        .await
        .expect("Get failed")
        .expect("Should exist");
    assert_eq!(new_item.as_ref(), b"new data");

    let deleted = db.get(pk, "sk_to_delete").await.expect("Get failed");
    assert!(deleted.is_none(), "Should be deleted");

    // Cleanup
    db.batch(&[
        BatchOp::Delete {
            pk,
            sk: "sk_to_keep",
        },
        BatchOp::Delete { pk, sk: "sk_new" },
    ])
    .await
    .expect("Cleanup failed");
}

#[wstd::test]
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

#[wstd::test]
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

#[wstd::test]
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
