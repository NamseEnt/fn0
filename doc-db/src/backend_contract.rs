use crate::{
    Database, DbOp, DbResult, DocDbRevision, ObservedDocument, TransactCondition, TransactMutation,
    TransactRequest, memory, turso_with_config,
};
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
    run_backend_contract_inner(db, new_database, backend_name, false).await
}

pub(crate) async fn run_revisioned_missing_backend_contract<F>(
    db: Database,
    new_database: F,
    backend_name: &str,
) -> Result<()>
where
    F: Fn() -> Database,
{
    run_backend_contract_inner(db, new_database, backend_name, true).await
}

async fn run_backend_contract_inner<F>(
    db: Database,
    new_database: F,
    backend_name: &str,
    exact_missing_revisions: bool,
) -> Result<()>
where
    F: Fn() -> Database,
{
    cleanup_backend_contract_data(&db).await?;
    let prefix = unique_prefix(backend_name);

    observed_read_contract(&db, &prefix, exact_missing_revisions).await?;
    mutation_revision_contract(&db, &prefix).await?;
    check_version_contract(&db, &prefix, exact_missing_revisions).await?;
    condition_only_contract(&db, &prefix).await?;
    check_missing_contract(&db, &prefix, exact_missing_revisions).await?;
    insert_contract(&db, &prefix, exact_missing_revisions).await?;
    update_contract(&db, &prefix, exact_missing_revisions).await?;
    delete_contract(&db, &prefix, exact_missing_revisions).await?;
    multi_item_transaction_contract(&db, &prefix, exact_missing_revisions).await?;
    rollback_contracts(&db, &prefix, exact_missing_revisions).await?;
    empty_transaction_contract(&db).await?;
    query_contract(&db, &prefix).await?;
    scan_contract(&new_database(), &prefix).await?;
    execute_ops_order_contract(&db, &prefix).await?;
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

fn transact_request(
    conditions: Vec<TransactCondition>,
    mutations: Vec<TransactMutation>,
) -> TransactRequest {
    TransactRequest {
        conditions,
        mutations,
    }
}

fn revision_equals(pk: &str, sk: &str, revision: DocDbRevision) -> TransactCondition {
    TransactCondition::RevisionEquals {
        pk: pk.to_string(),
        sk: sk.to_string(),
        expected_revision: revision,
    }
}

fn not_exists(pk: &str, sk: &str) -> TransactCondition {
    TransactCondition::NotExists {
        pk: pk.to_string(),
        sk: sk.to_string(),
    }
}

fn put(pk: &str, sk: &str, data: &[u8]) -> TransactMutation {
    TransactMutation::Put {
        pk: pk.to_string(),
        sk: sk.to_string(),
        data: data.to_vec(),
    }
}

fn delete(pk: &str, sk: &str) -> TransactMutation {
    TransactMutation::Delete {
        pk: pk.to_string(),
        sk: sk.to_string(),
    }
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

async fn observed_read_contract(
    db: &Database,
    prefix: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
    let (present_pk, present_sk) = key(prefix, "observed-present");
    let (missing_pk, missing_sk) = key(prefix, "observed-missing");
    let data = b"observed-value";

    db.put(&present_pk, &present_sk, data).await?;

    let (observed_data, revision) = expect_present(db, &present_pk, &present_sk, data).await?;
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
            revision: batch_revision,
        } => {
            assert_eq!(batch_data.as_ref(), data);
            assert_eq!(*batch_revision, revision);
        }
        ObservedDocument::Missing { .. } => panic!("present observation was missing in batch"),
    }
    assert!(matches!(
        observations[1],
        ObservedDocument::Missing { revision }
            if revision.is_none() != exact_missing_revisions
    ));

    expect_missing(db, &missing_pk, &missing_sk, exact_missing_revisions).await?;
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

async fn check_version_contract(
    db: &Database,
    prefix: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
    let (target_pk, target_sk) = key(prefix, "check-version-target");
    let (side_pk, side_sk) = key(prefix, "check-version-side");

    db.put(&target_pk, &target_sk, b"target").await?;
    let (_, current_revision) = expect_present(db, &target_pk, &target_sk, b"target").await?;

    let outcome = db
        .transact(&transact_request(
            vec![revision_equals(&target_pk, &target_sk, current_revision)],
            vec![put(&side_pk, &side_sk, b"side")],
        ))
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
        .transact(&transact_request(
            vec![revision_equals(&stale_pk, &stale_sk, stale_revision)],
            vec![put(&stale_side_pk, &stale_side_sk, b"must-not-commit")],
        ))
        .await?;
    assert_conflict(outcome, 0);
    expect_missing(db, &stale_side_pk, &stale_side_sk, exact_missing_revisions).await?;
    expect_present(db, &stale_pk, &stale_sk, b"external").await?;

    Ok(())
}

async fn check_missing_contract(
    db: &Database,
    prefix: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
    let (missing_pk, missing_sk) = key(prefix, "check-missing-success");
    let (success_side_pk, success_side_sk) = key(prefix, "check-missing-success-side");
    expect_missing(db, &missing_pk, &missing_sk, exact_missing_revisions).await?;

    let outcome = db
        .transact(&transact_request(
            vec![not_exists(&missing_pk, &missing_sk)],
            vec![put(&success_side_pk, &success_side_sk, b"side")],
        ))
        .await?;
    assert_success(outcome);
    expect_present(db, &success_side_pk, &success_side_sk, b"side").await?;

    let (existing_pk, existing_sk) = key(prefix, "check-missing-conflict");
    let (conflict_side_pk, conflict_side_sk) = key(prefix, "check-missing-conflict-side");
    db.put(&existing_pk, &existing_sk, b"existing").await?;

    let outcome = db
        .transact(&transact_request(
            vec![not_exists(&existing_pk, &existing_sk)],
            vec![put(
                &conflict_side_pk,
                &conflict_side_sk,
                b"must-not-commit",
            )],
        ))
        .await?;
    assert_conflict(outcome, 0);
    expect_missing(
        db,
        &conflict_side_pk,
        &conflict_side_sk,
        exact_missing_revisions,
    )
    .await?;
    expect_present(db, &existing_pk, &existing_sk, b"existing").await?;

    Ok(())
}

async fn condition_only_contract(db: &Database, prefix: &str) -> Result<()> {
    let (pk, sk) = key(prefix, "condition-only");
    db.put(&pk, &sk, b"before").await?;
    let (_, revision) = expect_present(db, &pk, &sk, b"before").await?;

    let outcome = db
        .transact(&transact_request(
            vec![revision_equals(&pk, &sk, revision)],
            vec![],
        ))
        .await?;
    assert_success(outcome);
    expect_present(db, &pk, &sk, b"before").await?;

    db.put(&pk, &sk, b"external").await?;
    let outcome = db
        .transact(&transact_request(
            vec![revision_equals(&pk, &sk, revision)],
            vec![],
        ))
        .await?;
    assert_conflict(outcome, 0);
    expect_present(db, &pk, &sk, b"external").await?;
    Ok(())
}

async fn insert_contract(db: &Database, prefix: &str, exact_missing_revisions: bool) -> Result<()> {
    let (insert_pk, insert_sk) = key(prefix, "insert-success");
    let outcome = db
        .transact(&transact_request(
            vec![not_exists(&insert_pk, &insert_sk)],
            vec![put(&insert_pk, &insert_sk, b"inserted")],
        ))
        .await?;
    assert_success(outcome);
    expect_present(db, &insert_pk, &insert_sk, b"inserted").await?;

    let (existing_pk, existing_sk) = key(prefix, "insert-conflict");
    let (side_pk, side_sk) = key(prefix, "insert-conflict-side");
    db.put(&existing_pk, &existing_sk, b"original").await?;

    let outcome = db
        .transact(&transact_request(
            vec![
                not_exists(&side_pk, &side_sk),
                not_exists(&existing_pk, &existing_sk),
            ],
            vec![
                put(&side_pk, &side_sk, b"must-not-commit"),
                put(&existing_pk, &existing_sk, b"replacement"),
            ],
        ))
        .await?;
    assert_conflict(outcome, 1);
    expect_missing(db, &side_pk, &side_sk, exact_missing_revisions).await?;
    expect_present(db, &existing_pk, &existing_sk, b"original").await?;

    Ok(())
}

async fn update_contract(db: &Database, prefix: &str, exact_missing_revisions: bool) -> Result<()> {
    let (pk, sk) = key(prefix, "update-success");
    db.put(&pk, &sk, b"before").await?;
    let (_, old_revision) = expect_present(db, &pk, &sk, b"before").await?;

    let outcome = db
        .transact(&transact_request(
            vec![revision_equals(&pk, &sk, old_revision)],
            vec![put(&pk, &sk, b"after")],
        ))
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
        .transact(&transact_request(
            vec![revision_equals(&stale_pk, &stale_sk, stale_revision)],
            vec![
                put(&side_pk, &side_sk, b"must-not-commit"),
                put(&stale_pk, &stale_sk, b"stale-write"),
            ],
        ))
        .await?;
    assert_conflict(outcome, 0);
    expect_missing(db, &side_pk, &side_sk, exact_missing_revisions).await?;
    expect_present(db, &stale_pk, &stale_sk, b"external").await?;

    Ok(())
}

async fn delete_contract(db: &Database, prefix: &str, exact_missing_revisions: bool) -> Result<()> {
    let (pk, sk) = key(prefix, "delete-success");
    db.put(&pk, &sk, b"to-delete").await?;
    let (_, revision) = expect_present(db, &pk, &sk, b"to-delete").await?;

    let outcome = db
        .transact(&transact_request(
            vec![revision_equals(&pk, &sk, revision)],
            vec![delete(&pk, &sk)],
        ))
        .await?;
    assert_success(outcome);
    assert!(db.get(&pk, &sk).await?.is_none());
    expect_missing(db, &pk, &sk, exact_missing_revisions).await?;

    let (stale_pk, stale_sk) = key(prefix, "delete-stale");
    let (side_pk, side_sk) = key(prefix, "delete-stale-side");
    db.put(&stale_pk, &stale_sk, b"old").await?;
    let (_, stale_revision) = expect_present(db, &stale_pk, &stale_sk, b"old").await?;
    db.put(&stale_pk, &stale_sk, b"external").await?;

    let outcome = db
        .transact(&transact_request(
            vec![revision_equals(&stale_pk, &stale_sk, stale_revision)],
            vec![
                put(&side_pk, &side_sk, b"must-not-commit"),
                delete(&stale_pk, &stale_sk),
            ],
        ))
        .await?;
    assert_conflict(outcome, 0);
    expect_missing(db, &side_pk, &side_sk, exact_missing_revisions).await?;
    expect_present(db, &stale_pk, &stale_sk, b"external").await?;

    Ok(())
}

async fn multi_item_transaction_contract(
    db: &Database,
    prefix: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
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
        .transact(&transact_request(
            vec![
                revision_equals(&check_pk, &check_sk, check_revision),
                revision_equals(&update_pk, &update_sk, update_revision),
                not_exists(&insert_pk, &insert_sk),
                revision_equals(&delete_pk, &delete_sk, delete_revision),
            ],
            vec![
                put(&update_pk, &update_sk, b"after"),
                put(&insert_pk, &insert_sk, b"inserted"),
                delete(&delete_pk, &delete_sk),
            ],
        ))
        .await?;
    assert_success(outcome);

    expect_present(db, &check_pk, &check_sk, b"check").await?;
    let (_, new_update_revision) = expect_present(db, &update_pk, &update_sk, b"after").await?;
    assert_ne!(new_update_revision, update_revision);
    expect_present(db, &insert_pk, &insert_sk, b"inserted").await?;
    expect_missing(db, &delete_pk, &delete_sk, exact_missing_revisions).await?;

    Ok(())
}

async fn rollback_contracts(
    db: &Database,
    prefix: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
    middle_conflict_rolls_back_prior_writes(db, prefix, exact_missing_revisions).await?;
    late_conflict_rolls_back_prior_writes(db, prefix, exact_missing_revisions).await
}

async fn middle_conflict_rolls_back_prior_writes(
    db: &Database,
    prefix: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
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
        .transact(&transact_request(
            vec![
                revision_equals(&first_pk, &first_sk, first_revision),
                not_exists(&last_pk, &last_sk),
                revision_equals(&conflict_pk, &conflict_sk, conflict_revision),
            ],
            vec![
                put(&first_pk, &first_sk, b"first-after"),
                put(&last_pk, &last_sk, b"must-not-commit"),
            ],
        ))
        .await?;
    assert_conflict(outcome, 2);
    expect_present(db, &first_pk, &first_sk, b"first-before").await?;
    expect_present(db, &conflict_pk, &conflict_sk, b"conflict-external").await?;
    expect_missing(db, &last_pk, &last_sk, exact_missing_revisions).await?;

    Ok(())
}

async fn late_conflict_rolls_back_prior_writes(
    db: &Database,
    prefix: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
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
        .transact(&transact_request(
            vec![
                revision_equals(&first_pk, &first_sk, first_revision),
                not_exists(&insert_pk, &insert_sk),
                revision_equals(&delete_pk, &delete_sk, delete_revision),
                revision_equals(&conflict_pk, &conflict_sk, conflict_revision),
            ],
            vec![
                put(&first_pk, &first_sk, b"first-after"),
                put(&insert_pk, &insert_sk, b"must-not-commit"),
                delete(&delete_pk, &delete_sk),
            ],
        ))
        .await?;
    assert_conflict(outcome, 3);
    expect_present(db, &first_pk, &first_sk, b"first-before").await?;
    expect_missing(db, &insert_pk, &insert_sk, exact_missing_revisions).await?;
    expect_present(db, &delete_pk, &delete_sk, b"delete-before").await?;
    expect_present(db, &conflict_pk, &conflict_sk, b"conflict-external").await?;

    Ok(())
}

async fn empty_transaction_contract(db: &Database) -> Result<()> {
    let outcome = db.transact(&transact_request(vec![], vec![])).await?;
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

async fn execute_ops_order_contract(db: &Database, prefix: &str) -> Result<()> {
    let (pk, sk) = key(prefix, "execute-ops-order");
    let results = db
        .execute_ops(vec![
            DbOp::Put {
                pk: pk.clone(),
                sk: sk.clone(),
                data: b"inserted".to_vec(),
            },
            DbOp::Get {
                pk: pk.clone(),
                sk: sk.clone(),
            },
        ])
        .await?;
    assert!(matches!(
        results.as_slice(),
        [DbResult::Done, DbResult::Single(Some(data))] if data.as_ref() == b"inserted"
    ));

    let results = db
        .execute_ops(vec![
            DbOp::Put {
                pk: pk.clone(),
                sk: sk.clone(),
                data: b"deleted".to_vec(),
            },
            DbOp::Delete {
                pk: pk.clone(),
                sk: sk.clone(),
            },
            DbOp::Get { pk, sk },
        ])
        .await?;
    assert!(matches!(
        results.as_slice(),
        [DbResult::Done, DbResult::Done, DbResult::Single(None)]
    ));
    Ok(())
}

async fn expect_present(
    db: &Database,
    pk: &str,
    sk: &str,
    expected_data: &[u8],
) -> Result<(Bytes, DocDbRevision)> {
    match db.get_observed(pk, sk).await? {
        ObservedDocument::Present { data, revision } => {
            assert_eq!(data.as_ref(), expected_data);
            Ok((data, revision))
        }
        ObservedDocument::Missing { .. } => panic!("expected {pk}/{sk} to be present"),
    }
}

async fn expect_missing(
    db: &Database,
    pk: &str,
    sk: &str,
    exact_missing_revisions: bool,
) -> Result<()> {
    let ObservedDocument::Missing { revision } = db.get_observed(pk, sk).await? else {
        panic!("expected {pk}/{sk} to be missing")
    };
    assert_eq!(revision.is_some(), exact_missing_revisions);
    Ok(())
}

fn assert_success(outcome: crate::TransactOutcome) {
    assert!(
        outcome.conflict.is_none(),
        "expected transaction success, got a conflict"
    );
}

fn assert_conflict(outcome: crate::TransactOutcome, expected_condition_index: usize) {
    let conflict = outcome.conflict.expect("expected transaction conflict");
    assert_eq!(conflict.condition_index, expected_condition_index);
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
