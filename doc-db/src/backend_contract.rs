use crate::{Database, ObservedDocument, TransactItem, memory, turso_with_config};
use anyhow::Result;
use bytes::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_TURSO_TEST_URL: &str = "http://127.0.0.1:18123";

pub(crate) async fn run_backend_contract<F>(
    db: Database,
    new_database: F,
    backend_name: &str,
) -> Result<()>
where
    F: Fn() -> Database,
{
    cleanup_backend_contract_data(&db).await?;
    let prefix = unique_prefix(backend_name);

    observed_read_contract(&db, &prefix).await?;
    mutation_revision_contract(&db, &prefix).await?;
    check_version_contract(&db, &prefix).await?;
    check_missing_contract(&db, &prefix).await?;
    insert_contract(&db, &prefix).await?;
    update_contract(&db, &prefix).await?;
    delete_contract(&db, &prefix).await?;
    multi_item_transaction_contract(&db, &prefix).await?;
    rollback_contracts(&db, &prefix).await?;
    empty_transaction_contract(&db).await?;
    query_contract(&db, &prefix).await?;
    scan_contract(&new_database(), &prefix).await?;
    cleanup_backend_contract_data(&db).await?;

    Ok(())
}

#[tokio::test]
async fn memory_backend_contract() -> Result<()> {
    run_backend_contract(memory(), memory, "memory").await
}

#[tokio::test]
async fn turso_backend_contract() -> Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let url =
        std::env::var("DOC_DB_TEST_URL").unwrap_or_else(|_| DEFAULT_TURSO_TEST_URL.to_string());
    let auth_token = std::env::var("DOC_DB_TEST_AUTH_TOKEN").unwrap_or_default();
    let new_url = url.clone();
    let new_auth_token = auth_token.clone();

    run_backend_contract(
        turso_with_config(url, auth_token),
        move || turso_with_config(new_url.clone(), new_auth_token.clone()),
        "turso",
    )
    .await
}

fn unique_prefix(backend_name: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    format!(
        "backend-contract/{backend_name}/{}/{}",
        std::process::id(),
        timestamp
    )
}

fn key(prefix: &str, name: &str) -> (String, String) {
    (format!("{prefix}/{name}"), "doc".to_string())
}

fn transact_key(pk: &str, sk: &str) -> (String, String) {
    (pk.to_string(), sk.to_string())
}

async fn cleanup_backend_contract_data(db: &Database) -> Result<()> {
    let items = db.scan(None, usize::MAX).await?;
    for (pk, sk, _) in items {
        if pk.starts_with("backend-contract/") {
            db.delete(&pk, &sk).await?;
        }
    }
    Ok(())
}

async fn observed_read_contract(db: &Database, prefix: &str) -> Result<()> {
    let (present_pk, present_sk) = key(prefix, "observed-present");
    let (missing_pk, missing_sk) = key(prefix, "observed-missing");
    let data = b"observed-value";

    db.put(&present_pk, &present_sk, data).await?;

    let (observed_data, version) = expect_present(db, &present_pk, &present_sk, data).await?;
    assert_eq!(observed_data.as_ref(), data);

    let observations = db
        .batch_get_observed(&[
            transact_key(&present_pk, &present_sk),
            transact_key(&missing_pk, &missing_sk),
        ])
        .await?;
    assert_eq!(observations.len(), 2);
    match &observations[0] {
        ObservedDocument::Present {
            data: batch_data,
            version: batch_version,
        } => {
            assert_eq!(batch_data.as_ref(), data);
            assert_eq!(*batch_version, version);
        }
        ObservedDocument::Missing => panic!("present observation was missing in batch"),
    }
    assert!(matches!(observations[1], ObservedDocument::Missing));

    expect_missing(db, &missing_pk, &missing_sk).await?;
    Ok(())
}

async fn mutation_revision_contract(db: &Database, prefix: &str) -> Result<()> {
    let (pk, sk) = key(prefix, "revision-change");

    db.put(&pk, &sk, b"before").await?;
    let (_, old_revision) = expect_present(db, &pk, &sk, b"before").await?;

    db.put(&pk, &sk, b"after").await?;
    let (_, new_revision) = expect_present(db, &pk, &sk, b"after").await?;
    assert_ne!(new_revision, old_revision);

    Ok(())
}

async fn check_version_contract(db: &Database, prefix: &str) -> Result<()> {
    let (target_pk, target_sk) = key(prefix, "check-version-target");
    let (side_pk, side_sk) = key(prefix, "check-version-side");

    db.put(&target_pk, &target_sk, b"target").await?;
    let (_, current_revision) = expect_present(db, &target_pk, &target_sk, b"target").await?;

    let outcome = db
        .transact(&[
            TransactItem::CheckVersion {
                pk: target_pk.clone(),
                sk: target_sk.clone(),
                expected_version: current_revision,
            },
            TransactItem::Insert {
                pk: side_pk.clone(),
                sk: side_sk.clone(),
                data: b"side".to_vec(),
            },
        ])
        .await?;
    assert_success(outcome);
    expect_present(db, &target_pk, &target_sk, b"target").await?;
    expect_present(db, &side_pk, &side_sk, b"side").await?;

    let (stale_pk, stale_sk) = key(prefix, "check-version-stale");
    let (stale_side_pk, stale_side_sk) = key(prefix, "check-version-stale-side");
    db.put(&stale_pk, &stale_sk, b"old").await?;
    let (_, stale_revision) = expect_present(db, &stale_pk, &stale_sk, b"old").await?;
    db.put(&stale_pk, &stale_sk, b"external").await?;

    let outcome = db
        .transact(&[
            TransactItem::Insert {
                pk: stale_side_pk.clone(),
                sk: stale_side_sk.clone(),
                data: b"must-not-commit".to_vec(),
            },
            TransactItem::CheckVersion {
                pk: stale_pk.clone(),
                sk: stale_sk.clone(),
                expected_version: stale_revision,
            },
        ])
        .await?;
    assert_conflict(outcome, 1);
    expect_missing(db, &stale_side_pk, &stale_side_sk).await?;
    expect_present(db, &stale_pk, &stale_sk, b"external").await?;

    Ok(())
}

async fn check_missing_contract(db: &Database, prefix: &str) -> Result<()> {
    let (missing_pk, missing_sk) = key(prefix, "check-missing-success");
    let (success_side_pk, success_side_sk) = key(prefix, "check-missing-success-side");
    expect_missing(db, &missing_pk, &missing_sk).await?;

    let outcome = db
        .transact(&[
            TransactItem::CheckMissing {
                pk: missing_pk.clone(),
                sk: missing_sk.clone(),
            },
            TransactItem::Insert {
                pk: success_side_pk.clone(),
                sk: success_side_sk.clone(),
                data: b"side".to_vec(),
            },
        ])
        .await?;
    assert_success(outcome);
    expect_present(db, &success_side_pk, &success_side_sk, b"side").await?;

    let (existing_pk, existing_sk) = key(prefix, "check-missing-conflict");
    let (conflict_side_pk, conflict_side_sk) = key(prefix, "check-missing-conflict-side");
    db.put(&existing_pk, &existing_sk, b"existing").await?;

    let outcome = db
        .transact(&[
            TransactItem::Insert {
                pk: conflict_side_pk.clone(),
                sk: conflict_side_sk.clone(),
                data: b"must-not-commit".to_vec(),
            },
            TransactItem::CheckMissing {
                pk: existing_pk.clone(),
                sk: existing_sk.clone(),
            },
        ])
        .await?;
    assert_conflict(outcome, 1);
    expect_missing(db, &conflict_side_pk, &conflict_side_sk).await?;
    expect_present(db, &existing_pk, &existing_sk, b"existing").await?;

    Ok(())
}

async fn insert_contract(db: &Database, prefix: &str) -> Result<()> {
    let (insert_pk, insert_sk) = key(prefix, "insert-success");
    let outcome = db
        .transact(&[TransactItem::Insert {
            pk: insert_pk.clone(),
            sk: insert_sk.clone(),
            data: b"inserted".to_vec(),
        }])
        .await?;
    assert_success(outcome);
    expect_present(db, &insert_pk, &insert_sk, b"inserted").await?;

    let (existing_pk, existing_sk) = key(prefix, "insert-conflict");
    let (side_pk, side_sk) = key(prefix, "insert-conflict-side");
    db.put(&existing_pk, &existing_sk, b"original").await?;

    let outcome = db
        .transact(&[
            TransactItem::Insert {
                pk: side_pk.clone(),
                sk: side_sk.clone(),
                data: b"must-not-commit".to_vec(),
            },
            TransactItem::Insert {
                pk: existing_pk.clone(),
                sk: existing_sk.clone(),
                data: b"replacement".to_vec(),
            },
        ])
        .await?;
    assert_conflict(outcome, 1);
    expect_missing(db, &side_pk, &side_sk).await?;
    expect_present(db, &existing_pk, &existing_sk, b"original").await?;

    Ok(())
}

async fn update_contract(db: &Database, prefix: &str) -> Result<()> {
    let (pk, sk) = key(prefix, "update-success");
    db.put(&pk, &sk, b"before").await?;
    let (_, old_revision) = expect_present(db, &pk, &sk, b"before").await?;

    let outcome = db
        .transact(&[TransactItem::Update {
            pk: pk.clone(),
            sk: sk.clone(),
            expected_version: old_revision,
            data: b"after".to_vec(),
        }])
        .await?;
    assert_success(outcome);
    let (_, new_revision) = expect_present(db, &pk, &sk, b"after").await?;
    assert_ne!(new_revision, old_revision);

    let (stale_pk, stale_sk) = key(prefix, "update-stale");
    let (side_pk, side_sk) = key(prefix, "update-stale-side");
    db.put(&stale_pk, &stale_sk, b"old").await?;
    let (_, stale_revision) = expect_present(db, &stale_pk, &stale_sk, b"old").await?;
    db.put(&stale_pk, &stale_sk, b"external").await?;

    let outcome = db
        .transact(&[
            TransactItem::Insert {
                pk: side_pk.clone(),
                sk: side_sk.clone(),
                data: b"must-not-commit".to_vec(),
            },
            TransactItem::Update {
                pk: stale_pk.clone(),
                sk: stale_sk.clone(),
                expected_version: stale_revision,
                data: b"stale-write".to_vec(),
            },
        ])
        .await?;
    assert_conflict(outcome, 1);
    expect_missing(db, &side_pk, &side_sk).await?;
    expect_present(db, &stale_pk, &stale_sk, b"external").await?;

    Ok(())
}

async fn delete_contract(db: &Database, prefix: &str) -> Result<()> {
    let (pk, sk) = key(prefix, "delete-success");
    db.put(&pk, &sk, b"to-delete").await?;
    let (_, revision) = expect_present(db, &pk, &sk, b"to-delete").await?;

    let outcome = db
        .transact(&[TransactItem::Delete {
            pk: pk.clone(),
            sk: sk.clone(),
            expected_version: revision,
        }])
        .await?;
    assert_success(outcome);
    assert!(db.get(&pk, &sk).await?.is_none());
    expect_missing(db, &pk, &sk).await?;

    let (stale_pk, stale_sk) = key(prefix, "delete-stale");
    let (side_pk, side_sk) = key(prefix, "delete-stale-side");
    db.put(&stale_pk, &stale_sk, b"old").await?;
    let (_, stale_revision) = expect_present(db, &stale_pk, &stale_sk, b"old").await?;
    db.put(&stale_pk, &stale_sk, b"external").await?;

    let outcome = db
        .transact(&[
            TransactItem::Insert {
                pk: side_pk.clone(),
                sk: side_sk.clone(),
                data: b"must-not-commit".to_vec(),
            },
            TransactItem::Delete {
                pk: stale_pk.clone(),
                sk: stale_sk.clone(),
                expected_version: stale_revision,
            },
        ])
        .await?;
    assert_conflict(outcome, 1);
    expect_missing(db, &side_pk, &side_sk).await?;
    expect_present(db, &stale_pk, &stale_sk, b"external").await?;

    Ok(())
}

async fn multi_item_transaction_contract(db: &Database, prefix: &str) -> Result<()> {
    let (check_pk, check_sk) = key(prefix, "multi-check");
    let (update_pk, update_sk) = key(prefix, "multi-update");
    let (insert_pk, insert_sk) = key(prefix, "multi-insert");
    let (delete_pk, delete_sk) = key(prefix, "multi-delete");

    db.put(&check_pk, &check_sk, b"check").await?;
    db.put(&update_pk, &update_sk, b"before").await?;
    db.put(&delete_pk, &delete_sk, b"delete").await?;
    let (_, check_revision) = expect_present(db, &check_pk, &check_sk, b"check").await?;
    let (_, update_revision) = expect_present(db, &update_pk, &update_sk, b"before").await?;
    let (_, delete_revision) = expect_present(db, &delete_pk, &delete_sk, b"delete").await?;

    let outcome = db
        .transact(&[
            TransactItem::CheckVersion {
                pk: check_pk.clone(),
                sk: check_sk.clone(),
                expected_version: check_revision,
            },
            TransactItem::Update {
                pk: update_pk.clone(),
                sk: update_sk.clone(),
                expected_version: update_revision,
                data: b"after".to_vec(),
            },
            TransactItem::Insert {
                pk: insert_pk.clone(),
                sk: insert_sk.clone(),
                data: b"inserted".to_vec(),
            },
            TransactItem::Delete {
                pk: delete_pk.clone(),
                sk: delete_sk.clone(),
                expected_version: delete_revision,
            },
        ])
        .await?;
    assert_success(outcome);

    expect_present(db, &check_pk, &check_sk, b"check").await?;
    let (_, new_update_revision) = expect_present(db, &update_pk, &update_sk, b"after").await?;
    assert_ne!(new_update_revision, update_revision);
    expect_present(db, &insert_pk, &insert_sk, b"inserted").await?;
    expect_missing(db, &delete_pk, &delete_sk).await?;

    Ok(())
}

async fn rollback_contracts(db: &Database, prefix: &str) -> Result<()> {
    middle_conflict_rolls_back_prior_writes(db, prefix).await?;
    late_conflict_rolls_back_prior_writes(db, prefix).await
}

async fn middle_conflict_rolls_back_prior_writes(db: &Database, prefix: &str) -> Result<()> {
    let (first_pk, first_sk) = key(prefix, "middle-first");
    let (conflict_pk, conflict_sk) = key(prefix, "middle-conflict");
    let (last_pk, last_sk) = key(prefix, "middle-last");

    db.put(&first_pk, &first_sk, b"first-before").await?;
    db.put(&conflict_pk, &conflict_sk, b"conflict-old").await?;
    let (_, first_revision) = expect_present(db, &first_pk, &first_sk, b"first-before").await?;
    let (_, conflict_revision) =
        expect_present(db, &conflict_pk, &conflict_sk, b"conflict-old").await?;
    db.put(&conflict_pk, &conflict_sk, b"conflict-external")
        .await?;

    let outcome = db
        .transact(&[
            TransactItem::Update {
                pk: first_pk.clone(),
                sk: first_sk.clone(),
                expected_version: first_revision,
                data: b"first-after".to_vec(),
            },
            TransactItem::CheckVersion {
                pk: conflict_pk.clone(),
                sk: conflict_sk.clone(),
                expected_version: conflict_revision,
            },
            TransactItem::Insert {
                pk: last_pk.clone(),
                sk: last_sk.clone(),
                data: b"must-not-commit".to_vec(),
            },
        ])
        .await?;
    assert_conflict(outcome, 1);
    expect_present(db, &first_pk, &first_sk, b"first-before").await?;
    expect_present(db, &conflict_pk, &conflict_sk, b"conflict-external").await?;
    expect_missing(db, &last_pk, &last_sk).await?;

    Ok(())
}

async fn late_conflict_rolls_back_prior_writes(db: &Database, prefix: &str) -> Result<()> {
    let (first_pk, first_sk) = key(prefix, "late-first");
    let (insert_pk, insert_sk) = key(prefix, "late-insert");
    let (delete_pk, delete_sk) = key(prefix, "late-delete");
    let (conflict_pk, conflict_sk) = key(prefix, "late-conflict");

    db.put(&first_pk, &first_sk, b"first-before").await?;
    db.put(&delete_pk, &delete_sk, b"delete-before").await?;
    db.put(&conflict_pk, &conflict_sk, b"conflict-old").await?;
    let (_, first_revision) = expect_present(db, &first_pk, &first_sk, b"first-before").await?;
    let (_, delete_revision) = expect_present(db, &delete_pk, &delete_sk, b"delete-before").await?;
    let (_, conflict_revision) =
        expect_present(db, &conflict_pk, &conflict_sk, b"conflict-old").await?;
    db.put(&conflict_pk, &conflict_sk, b"conflict-external")
        .await?;

    let outcome = db
        .transact(&[
            TransactItem::Update {
                pk: first_pk.clone(),
                sk: first_sk.clone(),
                expected_version: first_revision,
                data: b"first-after".to_vec(),
            },
            TransactItem::Insert {
                pk: insert_pk.clone(),
                sk: insert_sk.clone(),
                data: b"must-not-commit".to_vec(),
            },
            TransactItem::Delete {
                pk: delete_pk.clone(),
                sk: delete_sk.clone(),
                expected_version: delete_revision,
            },
            TransactItem::CheckVersion {
                pk: conflict_pk.clone(),
                sk: conflict_sk.clone(),
                expected_version: conflict_revision,
            },
        ])
        .await?;
    assert_conflict(outcome, 3);
    expect_present(db, &first_pk, &first_sk, b"first-before").await?;
    expect_missing(db, &insert_pk, &insert_sk).await?;
    expect_present(db, &delete_pk, &delete_sk, b"delete-before").await?;
    expect_present(db, &conflict_pk, &conflict_sk, b"conflict-external").await?;

    Ok(())
}

async fn empty_transaction_contract(db: &Database) -> Result<()> {
    let outcome = db.transact(&[]).await?;
    assert_success(outcome);
    Ok(())
}

async fn query_contract(db: &Database, prefix: &str) -> Result<()> {
    let pk = format!("{prefix}/query");
    for sk in ["c", "a", "b"] {
        db.put(&pk, sk, sk.as_bytes()).await?;
    }

    let all = db.query(&pk, None::<&str>, 10).await?;
    assert_eq!(query_keys(&all), ["a", "b", "c"]);

    let after_a = db.query(&pk, Some("a"), 10).await?;
    assert_eq!(query_keys(&after_a), ["b", "c"]);

    let limited = db.query(&pk, None::<&str>, 2).await?;
    assert_eq!(query_keys(&limited), ["a", "b"]);

    Ok(())
}

async fn scan_contract(db: &Database, prefix: &str) -> Result<()> {
    let scan_prefix = format!("{prefix}/scan");
    let entries = [
        (format!("{scan_prefix}/b"), "1", b"b1".as_slice()),
        (format!("{scan_prefix}/a"), "2", b"a2".as_slice()),
        (format!("{scan_prefix}/c"), "1", b"c1".as_slice()),
        (format!("{scan_prefix}/a"), "1", b"a1".as_slice()),
    ];
    for (pk, sk, data) in &entries {
        db.put(pk, sk, data).await?;
    }

    let start_cursor = (scan_prefix.clone(), String::new());
    let limited = db.scan(Some((&start_cursor.0, &start_cursor.1)), 2).await?;
    assert_eq!(
        scan_keys(&limited),
        [
            (format!("{scan_prefix}/a"), "1".to_string()),
            (format!("{scan_prefix}/a"), "2".to_string()),
        ]
    );

    let all = db
        .scan(Some((&start_cursor.0, &start_cursor.1)), usize::MAX)
        .await?;
    assert_eq!(
        selected_scan_keys(&all, &scan_prefix),
        [
            (format!("{scan_prefix}/a"), "1".to_string()),
            (format!("{scan_prefix}/a"), "2".to_string()),
            (format!("{scan_prefix}/b"), "1".to_string()),
            (format!("{scan_prefix}/c"), "1".to_string()),
        ]
    );

    let after_a2_pk = format!("{scan_prefix}/a");
    let after_a2_sk = "2".to_string();
    let after_a2 = db
        .scan(Some((&after_a2_pk, &after_a2_sk)), usize::MAX)
        .await?;
    assert_eq!(
        selected_scan_keys(&after_a2, &scan_prefix),
        [
            (format!("{scan_prefix}/b"), "1".to_string()),
            (format!("{scan_prefix}/c"), "1".to_string()),
        ]
    );

    Ok(())
}

async fn expect_present(
    db: &Database,
    pk: &str,
    sk: &str,
    expected_data: &[u8],
) -> Result<(Bytes, i64)> {
    match db.get_observed(pk, sk).await? {
        ObservedDocument::Present { data, version } => {
            assert_eq!(data.as_ref(), expected_data);
            Ok((data, version))
        }
        ObservedDocument::Missing => panic!("expected {pk}/{sk} to be present"),
    }
}

async fn expect_missing(db: &Database, pk: &str, sk: &str) -> Result<()> {
    assert!(matches!(
        db.get_observed(pk, sk).await?,
        ObservedDocument::Missing
    ));
    Ok(())
}

fn assert_success(outcome: crate::TransactOutcome) {
    assert!(
        outcome.conflict.is_none(),
        "expected transaction success, got a conflict"
    );
}

fn assert_conflict(outcome: crate::TransactOutcome, expected_step_index: usize) {
    let conflict = outcome.conflict.expect("expected transaction conflict");
    assert_eq!(conflict.step_index, expected_step_index);
}

fn query_keys(items: &[(String, Bytes)]) -> Vec<&str> {
    items.iter().map(|(sk, _)| sk.as_str()).collect()
}

fn scan_keys(items: &[(String, String, Bytes)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(pk, sk, _)| (pk.clone(), sk.clone()))
        .collect()
}

fn selected_scan_keys(items: &[(String, String, Bytes)], prefix: &str) -> Vec<(String, String)> {
    items
        .iter()
        .filter(|(pk, _, _)| pk.starts_with(prefix))
        .map(|(pk, sk, _)| (pk.clone(), sk.clone()))
        .collect()
}
