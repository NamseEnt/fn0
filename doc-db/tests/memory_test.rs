use doc_db::{BatchOp, DbOp, DbRequest, DbResult, Prepared};

fn create_test_db() -> doc_db::Database {
    doc_db::memory()
}

// ============================================================
// In-memory backend tests
// ============================================================

#[forte_sdk::test]
async fn memory_put_and_get() {
    let db = create_test_db();
    db.put("pk", "sk", b"hello").await.unwrap();

    let result = db.get("pk", "sk").await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ref(), b"hello");
}

#[forte_sdk::test]
async fn memory_get_nonexistent() {
    let db = create_test_db();
    let result = db.get("missing", "missing").await.unwrap();
    assert!(result.is_none());
}

#[forte_sdk::test]
async fn memory_update_existing() {
    let db = create_test_db();
    db.put("pk", "sk", b"v1").await.unwrap();
    db.put("pk", "sk", b"v2").await.unwrap();

    let result = db.get("pk", "sk").await.unwrap().unwrap();
    assert_eq!(result.as_ref(), b"v2");
}

#[forte_sdk::test]
async fn memory_delete() {
    let db = create_test_db();
    db.put("pk", "sk", b"data").await.unwrap();
    db.delete("pk", "sk").await.unwrap();

    let result = db.get("pk", "sk").await.unwrap();
    assert!(result.is_none());
}

#[forte_sdk::test]
async fn memory_query_with_pagination() {
    let db = create_test_db();
    for i in 0..5 {
        db.put("pk", &format!("sk_{:02}", i), format!("d{}", i).as_bytes())
            .await
            .unwrap();
    }

    let page1 = db.query("pk", None::<&str>, 2).await.unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0].0, "sk_00");
    assert_eq!(page1[1].0, "sk_01");

    let page2 = db.query("pk", Some(&page1[1].0), 2).await.unwrap();
    assert_eq!(page2.len(), 2);
    assert_eq!(page2[0].0, "sk_02");

    let page3 = db.query("pk", Some(&page2[1].0), 2).await.unwrap();
    assert_eq!(page3.len(), 1);
    assert_eq!(page3[0].0, "sk_04");
}

#[forte_sdk::test]
async fn memory_scan() {
    let db = create_test_db();
    db.put("a", "1", b"a1").await.unwrap();
    db.put("a", "2", b"a2").await.unwrap();
    db.put("b", "1", b"b1").await.unwrap();

    let page1 = db.scan(None, 2).await.unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0].0, "a");
    assert_eq!(page1[0].1, "1");

    let page2 = db.scan(Some((&page1[1].0, &page1[1].1)), 2).await.unwrap();
    assert_eq!(page2.len(), 1);
    assert_eq!(page2[0].0, "b");
}

#[forte_sdk::test]
async fn memory_batch() {
    let db = create_test_db();
    db.batch(&[
        BatchOp::Put {
            pk: "pk",
            sk: "a",
            data: b"1",
        },
        BatchOp::Put {
            pk: "pk",
            sk: "b",
            data: b"2",
        },
        BatchOp::Put {
            pk: "pk",
            sk: "c",
            data: b"3",
        },
    ])
    .await
    .unwrap();

    let items = db.query("pk", None::<&str>, 10).await.unwrap();
    assert_eq!(items.len(), 3);

    db.batch(&[BatchOp::Delete { pk: "pk", sk: "b" }])
        .await
        .unwrap();

    let items = db.query("pk", None::<&str>, 10).await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].0, "a");
    assert_eq!(items[1].0, "c");
}

#[forte_sdk::test]
async fn memory_transaction_commit() {
    let db = create_test_db();

    let mut tx = db.transaction().await.unwrap();
    tx.put("pk", "sk", b"in_tx").await.unwrap();

    // Not visible outside tx yet (separate snapshot)
    let outside = db.get("pk", "sk").await.unwrap();
    assert!(outside.is_none());

    tx.commit().await.unwrap();

    // Now visible
    let result = db.get("pk", "sk").await.unwrap().unwrap();
    assert_eq!(result.as_ref(), b"in_tx");
}

#[forte_sdk::test]
async fn memory_transaction_rollback() {
    let db = create_test_db();
    db.put("pk", "sk", b"original").await.unwrap();

    let mut tx = db.transaction().await.unwrap();
    tx.put("pk", "sk", b"modified").await.unwrap();
    tx.rollback().await.unwrap();

    let result = db.get("pk", "sk").await.unwrap().unwrap();
    assert_eq!(result.as_ref(), b"original");
}

#[forte_sdk::test]
async fn memory_execute_ops() {
    let db = create_test_db();
    db.put("pk", "a", b"aa").await.unwrap();

    let results = db
        .execute_ops(vec![
            DbOp::Get {
                pk: "pk".into(),
                sk: "a".into(),
            },
            DbOp::Get {
                pk: "pk".into(),
                sk: "missing".into(),
            },
            DbOp::Put {
                pk: "pk".into(),
                sk: "b".into(),
                data: b"bb".to_vec(),
            },
        ])
        .await
        .unwrap();

    assert_eq!(results.len(), 3);
    match &results[0] {
        DbResult::Single(Some(d)) => assert_eq!(d.as_ref(), b"aa"),
        _ => panic!("expected Single(Some)"),
    }
    match &results[1] {
        DbResult::Single(None) => {}
        _ => panic!("expected Single(None)"),
    }
    assert!(matches!(&results[2], DbResult::Done));

    // Verify the put worked
    let result = db.get("pk", "b").await.unwrap().unwrap();
    assert_eq!(result.as_ref(), b"bb");
}

#[forte_sdk::test]
async fn memory_send_with() {
    let db = create_test_db();
    db.put("TestDoc", "id=hello", b"{}").await.unwrap();

    // Test DbRequest::send_with using a simple custom request
    struct MyGet;
    impl DbRequest for MyGet {
        type Output = Option<Vec<u8>>;
        fn prepare(self) -> Prepared<Self::Output> {
            Prepared {
                ops: vec![DbOp::Get {
                    pk: "TestDoc".into(),
                    sk: "id=hello".into(),
                }],
                parse: Box::new(|iter| match iter.next().unwrap() {
                    DbResult::Single(opt) => Ok(opt.map(|b| b.to_vec())),
                    _ => panic!("unexpected"),
                }),
            }
        }
    }

    let result = MyGet.send_with(&db).await.unwrap();
    assert!(result.is_some());
}

// ============================================================
// Per-instance isolation test
// ============================================================

#[forte_sdk::test]
async fn memory_instances_are_isolated() {
    let db1 = create_test_db();
    let db2 = create_test_db();

    db1.put("pk", "sk", b"db1_data").await.unwrap();

    let result = db2.get("pk", "sk").await.unwrap();
    assert!(result.is_none(), "db2 should not see db1's data");
}

// ============================================================
// Mocking tests
// ============================================================

#[forte_sdk::test]
async fn mock_get_returns_data() {
    let db = create_test_db();

    db.mock_get("pk", "sk").returns(b"mocked_data".to_vec());

    let result = db.get("pk", "sk").await.unwrap().unwrap();
    assert_eq!(result.as_ref(), b"mocked_data");
}

#[forte_sdk::test]
async fn mock_get_returns_none() {
    let db = create_test_db();

    // Put real data
    db.put("pk", "sk", b"real").await.unwrap();

    // Mock overrides it
    db.mock_get("pk", "sk").returns_none();

    let result = db.get("pk", "sk").await.unwrap();
    assert!(result.is_none());

    // Mock consumed - second call hits real data
    let result = db.get("pk", "sk").await.unwrap().unwrap();
    assert_eq!(result.as_ref(), b"real");
}

#[forte_sdk::test]
async fn mock_get_returns_error() {
    let db = create_test_db();

    db.mock_get("pk", "sk").returns_err("network timeout");

    let err = db.get("pk", "sk").await.unwrap_err();
    assert!(err.to_string().contains("network timeout"));
}

#[forte_sdk::test]
async fn mock_put_returns_error() {
    let db = create_test_db();

    db.mock_put("pk", "sk").returns_err("disk full");

    let err = db.put("pk", "sk", b"data").await.unwrap_err();
    assert!(err.to_string().contains("disk full"));

    // Data should not have been written
    let result = db.get("pk", "sk").await.unwrap();
    assert!(result.is_none());
}

#[forte_sdk::test]
async fn mock_delete_returns_error() {
    let db = create_test_db();
    db.put("pk", "sk", b"data").await.unwrap();

    db.mock_delete("pk", "sk").returns_err("permission denied");

    let err = db.delete("pk", "sk").await.unwrap_err();
    assert!(err.to_string().contains("permission denied"));

    // Data should still exist
    let result = db.get("pk", "sk").await.unwrap();
    assert!(result.is_some());
}

#[forte_sdk::test]
async fn mock_is_one_shot() {
    let db = create_test_db();
    db.put("pk", "sk", b"real").await.unwrap();

    db.mock_get("pk", "sk").returns_err("fail once");

    // First call: mock fires
    assert!(db.get("pk", "sk").await.is_err());

    // Second call: falls through to real backend
    let result = db.get("pk", "sk").await.unwrap().unwrap();
    assert_eq!(result.as_ref(), b"real");
}

#[forte_sdk::test]
async fn mock_clear() {
    let db = create_test_db();

    db.mock_get("pk", "sk").returns_err("fail");
    db.clear_mocks();

    // Mock cleared, should not fail
    let result = db.get("pk", "sk").await.unwrap();
    assert!(result.is_none());
}

#[forte_sdk::test]
async fn mock_multiple_rules() {
    let db = create_test_db();

    db.mock_get("pk", "a").returns(b"mock_a".to_vec());
    db.mock_get("pk", "b").returns_err("fail_b");

    let a = db.get("pk", "a").await.unwrap().unwrap();
    assert_eq!(a.as_ref(), b"mock_a");

    let err = db.get("pk", "b").await.unwrap_err();
    assert!(err.to_string().contains("fail_b"));
}
