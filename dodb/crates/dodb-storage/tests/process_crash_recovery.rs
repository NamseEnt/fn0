use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use dodb_core::{DocumentKey, Revision, TransactionMutation, TransactionRequest};
use dodb_storage::{BTreeStore, BlinkStore, DatabaseConfig, Document, ProductionFile};

const SEEDS: [u64; 3] = [0xd0db2026, 0xd0db2027, 0xd0db2028];
const PREFIX_TRANSACTIONS: usize = 128;
const SEED_ROWS: usize = 4096;
const CHILD_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Engine {
    MainBtree,
    PlannedBlink,
}

impl Engine {
    fn as_str(self) -> &'static str {
        match self {
            Self::MainBtree => "main-btree",
            Self::PlannedBlink => "planned-blink",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Boundary {
    Returned,
    BeforeWalAppend,
    BeforeWalSync,
    AfterWalSync,
    NormalExit,
}

impl Boundary {
    fn as_str(self) -> &'static str {
        match self {
            Self::Returned => "returned",
            Self::BeforeWalAppend => "before_wal_append",
            Self::BeforeWalSync => "before_wal_sync",
            Self::AfterWalSync => "after_wal_sync",
            Self::NormalExit => "normal_exit",
        }
    }

    fn is_crash(self) -> bool {
        self != Self::NormalExit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedRow {
    value: Vec<u8>,
    revision: Revision,
}

enum Store {
    Main(BTreeStore<ProductionFile, ProductionFile>),
    Planned(BlinkStore<ProductionFile, ProductionFile>),
}

impl Store {
    fn transact(
        &mut self,
        request: TransactionRequest,
    ) -> dodb_core::Result<dodb_core::TransactionResult> {
        match self {
            Self::Main(store) => store.transact(request),
            Self::Planned(store) => store.transact(request),
        }
    }

    fn check_invariants(&mut self) -> dodb_core::Result<dodb_storage::InvariantReport> {
        match self {
            Self::Main(store) => store.check_invariants(),
            Self::Planned(store) => store.check_invariants(),
        }
    }

    fn scan_all(&mut self) -> dodb_core::Result<Vec<Document>> {
        let mut cursor = None;
        let mut documents = Vec::new();
        loop {
            let batch = match self {
                Self::Main(store) => store.scan(cursor.as_ref(), 512)?,
                Self::Planned(store) => store.scan(cursor.as_ref(), 512)?,
            };
            if batch.is_empty() {
                return Ok(documents);
            }
            cursor = batch.last().map(|document| document.key.clone());
            documents.extend(batch);
        }
    }
}

#[test]
fn main_btree_and_planned_blink_recover_from_process_sigkill() {
    let artifact_root = std::env::var_os("DODB_CRASH_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("dodb-crash-recovery-{}", std::process::id()))
        });
    fs::create_dir_all(&artifact_root).expect("artifact root should exist");
    let case_log = artifact_root.join("case-results.csv");
    let response_log = artifact_root.join("successful-responses.tsv");
    let expectation_log = artifact_root.join("expected-final-states.tsv");
    fs::write(&case_log, "engine\twidth\tboundary\tseed\tstatus\tdetail\n").unwrap();
    fs::write(
        &response_log,
        "engine\twidth\tseed\tcase\ttransaction\trevision\n",
    )
    .unwrap();
    fs::write(
        &expectation_log,
        "engine\twidth\tseed\tcase\tkey_hex\tvalue_hex\trevision\n",
    )
    .unwrap();

    verify_checker_rejects_invalid_states();
    let mut passed = 0_usize;
    let mut failed = 0_usize;
    for engine in [Engine::MainBtree, Engine::PlannedBlink] {
        for width in [4_usize, 8] {
            for boundary in [
                Boundary::Returned,
                Boundary::BeforeWalAppend,
                Boundary::BeforeWalSync,
                Boundary::AfterWalSync,
            ] {
                for seed in SEEDS {
                    let case_name = format!(
                        "{}-w{width}-{}-{seed:08x}",
                        engine.as_str(),
                        boundary.as_str()
                    );
                    let case_dir = artifact_root.join(&case_name);
                    let result = run_case(
                        engine,
                        width,
                        boundary,
                        seed,
                        &case_dir,
                        &response_log,
                        &expectation_log,
                    );
                    match result {
                        Ok(detail) => {
                            passed += 1;
                            append_line(
                                &case_log,
                                format!(
                                    "{}\t{width}\t{}\t{seed:08x}\tPASS\t{detail}\n",
                                    engine.as_str(),
                                    boundary.as_str()
                                ),
                            );
                        }
                        Err(detail) => {
                            failed += 1;
                            append_line(
                                &case_log,
                                format!(
                                    "{}\t{width}\t{}\t{seed:08x}\tFAIL\t{detail}\n",
                                    engine.as_str(),
                                    boundary.as_str()
                                ),
                            );
                        }
                    }
                }
            }
            let seed = SEEDS[0];
            let case_name = format!("{}-w{width}-normal-{seed:08x}", engine.as_str());
            let case_dir = artifact_root.join(&case_name);
            let result = run_case(
                engine,
                width,
                Boundary::NormalExit,
                seed,
                &case_dir,
                &response_log,
                &expectation_log,
            );
            match result {
                Ok(detail) => {
                    passed += 1;
                    append_line(
                        &case_log,
                        format!(
                            "{}\t{width}\tnormal_exit\t{seed:08x}\tPASS\t{detail}\n",
                            engine.as_str()
                        ),
                    );
                }
                Err(detail) => {
                    failed += 1;
                    append_line(
                        &case_log,
                        format!(
                            "{}\t{width}\tnormal_exit\t{seed:08x}\tFAIL\t{detail}\n",
                            engine.as_str()
                        ),
                    );
                }
            }
        }
    }
    append_line(
        &case_log,
        format!("TOTAL\t52\t-\t-\t{passed}/52 PASS, {failed}/52 FAIL\tchecker_self_test=passed\n"),
    );
    println!(
        "PROCESS_CRASH_RECOVERY cases=52 passed={passed} failed={failed} artifact_root={}",
        artifact_root.display()
    );
    assert_eq!(
        passed, 52,
        "one or more process recovery cases failed; see case-results.csv"
    );
}

fn run_case(
    engine: Engine,
    width: usize,
    boundary: Boundary,
    seed: u64,
    case_dir: &Path,
    response_log: &Path,
    expectation_log: &Path,
) -> Result<String, String> {
    if case_dir.exists() {
        return Err(format!(
            "refusing to overwrite existing case path {}",
            case_dir.display()
        ));
    }
    fs::create_dir_all(case_dir).map_err(|error| error.to_string())?;
    let data_path = case_dir.join("target.db");
    let wal_path = case_dir.join("target.wal");
    let mut expected = BTreeMap::new();
    let mut store = open_store(engine, &data_path, &wal_path)
        .map_err(|error| format!("seed open failed: {error}"))?;
    let seed_mutations: Vec<_> = (0..SEED_ROWS)
        .map(|row_index| TransactionMutation::Put {
            key: row_key(row_index),
            value: value_bytes(seed, 0, row_index),
        })
        .collect();
    let seed_result = store
        .transact(TransactionRequest::new(Vec::new(), seed_mutations))
        .map_err(|error| format!("seed transaction failed: {error}"))?;
    let seed_revision = seed_result
        .revision
        .ok_or_else(|| "seed response has no revision".to_owned())?;
    for row_index in 0..SEED_ROWS {
        expected.insert(
            row_key(row_index),
            ExpectedRow {
                value: value_bytes(seed, 0, row_index),
                revision: seed_revision,
            },
        );
    }
    append_response(
        response_log,
        engine,
        width,
        seed,
        case_dir,
        "seed",
        seed_revision,
    );
    for transaction_index in 0..PREFIX_TRANSACTIONS {
        let mutations = prefix_mutations(seed, width, transaction_index);
        let result = store
            .transact(TransactionRequest::new(Vec::new(), mutations.clone()))
            .map_err(|error| format!("prefix transaction {transaction_index} failed: {error}"))?;
        let revision = result
            .revision
            .ok_or_else(|| format!("prefix transaction {transaction_index} has no revision"))?;
        for mutation in mutations {
            if let TransactionMutation::Put { key, value } = mutation {
                expected.insert(key, ExpectedRow { value, revision });
            }
        }
        append_response(
            response_log,
            engine,
            width,
            seed,
            case_dir,
            &format!("prefix-{transaction_index:03}"),
            revision,
        );
    }
    drop(store);

    let target = target_mutations(seed, width);
    let before_target = expected.clone();
    let target_dir = PathBuf::from(
        std::env::var_os("CARGO_BIN_EXE_process-crash-recovery-child")
            .ok_or_else(|| "Cargo did not expose child binary".to_owned())?,
    );
    let mut child = Command::new(target_dir)
        .arg(engine.as_str())
        .arg(&data_path)
        .arg(&wal_path)
        .arg(boundary.as_str())
        .arg(width.to_string())
        .arg(seed.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(
            fs::File::create(case_dir.join("child-stderr.log"))
                .map_err(|error| format!("child stderr log create failed: {error}"))?,
        ))
        .spawn()
        .map_err(|error| format!("child spawn failed: {error}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "child stdout unavailable".to_owned())?;
    let (line_sender, line_receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(&mut stdout);
        let mut line = String::new();
        let read_result = reader.read_line(&mut line);
        let _ = line_sender.send(read_result.map(|_| line));
    });
    let first_line = match line_receiver.recv_timeout(CHILD_TIMEOUT) {
        Ok(Ok(line)) => line.trim().to_owned(),
        Ok(Err(error)) => {
            let diagnostics = stop_child(&mut child);
            copy_pre_recovery(&data_path, &wal_path, case_dir).ok();
            return Err(format!("child stdout read failed: {error}; {diagnostics}"));
        }
        Err(error) => {
            let diagnostics = stop_child(&mut child);
            copy_pre_recovery(&data_path, &wal_path, case_dir).ok();
            return Err(format!(
                "child did not reach boundary before timeout: {error}; {diagnostics}"
            ));
        }
    };
    let hook_reached = first_line.starts_with("HOOK ");
    let success_revision = first_line
        .strip_prefix("SUCCESS ")
        .and_then(|value| value.parse::<u64>().ok());
    let handshake_ok = match boundary {
        Boundary::Returned => success_revision.is_some(),
        Boundary::NormalExit => success_revision.is_some(),
        Boundary::BeforeWalAppend | Boundary::BeforeWalSync | Boundary::AfterWalSync => {
            first_line == format!("HOOK {}", boundary.as_str())
        }
    };
    if !handshake_ok {
        let diagnostics = stop_child(&mut child);
        fs::write(
            case_dir.join("child-handshake.txt"),
            format!(
                "expected={}\nactual={}\n{}\n",
                boundary.as_str(),
                first_line,
                diagnostics
            ),
        )
        .ok();
        copy_pre_recovery(&data_path, &wal_path, case_dir).ok();
        return Err(format!(
            "expected explicit boundary handshake, got {first_line:?}; {diagnostics}"
        ));
    }
    if let Some(revision) = success_revision {
        append_response(
            response_log,
            engine,
            width,
            seed,
            case_dir,
            "target",
            Revision::new(revision),
        );
        for mutation in &target {
            if let TransactionMutation::Put { key, value } = mutation {
                expected.insert(
                    key.clone(),
                    ExpectedRow {
                        value: value.clone(),
                        revision: Revision::new(revision),
                    },
                );
            }
        }
    }
    if boundary.is_crash() {
        let status =
            kill_and_wait(&mut child).map_err(|error| format!("SIGKILL failed: {error}"))?;
        let signal = status.signal();
        fs::write(
            case_dir.join("child-handshake.txt"),
            format!(
                "{}\nchild_pid={}\nexit_signal={:?}\n",
                first_line,
                child.id(),
                signal
            ),
        )
        .ok();
        if signal != Some(9) {
            copy_pre_recovery(&data_path, &wal_path, case_dir).ok();
            return Err(format!("child did not terminate from SIGKILL: {status:?}"));
        }
    } else {
        let status = child
            .wait()
            .map_err(|error| format!("normal child wait failed: {error}"))?;
        fs::write(
            case_dir.join("child-handshake.txt"),
            format!("{}\nnormal_exit_status={status:?}\n", first_line),
        )
        .ok();
        if !status.success() {
            copy_pre_recovery(&data_path, &wal_path, case_dir).ok();
            return Err(format!(
                "normal control child exited unsuccessfully: {status:?}"
            ));
        }
    }
    copy_pre_recovery(&data_path, &wal_path, case_dir)
        .map_err(|error| format!("pre-recovery snapshot failed: {error}"))?;

    let prefix_max_revision = before_target
        .values()
        .map(|row| row.revision)
        .max()
        .unwrap_or(Revision::new(0));
    let mut expected_final = before_target.clone();
    let mut allow_target = false;
    let target_revision = success_revision.map(Revision::new);
    if let Some(revision) = target_revision {
        apply_target_expected(&mut expected_final, &target, revision);
        allow_target = true;
    }
    let recovered = read_actual(engine, &data_path, &wal_path)
        .map_err(|error| format!("first recovery open or scan failed: {error}"))?;
    append_line(
        &case_dir.join("recovery.log"),
        format!(
            "restart=1 rows={} max_revision={}\n",
            recovered.len(),
            recovered
                .values()
                .map(|row| row.revision.get())
                .max()
                .unwrap_or(0)
        ),
    );
    match boundary {
        Boundary::Returned | Boundary::NormalExit => {
            let revision = target_revision
                .ok_or_else(|| "successful target returned no revision".to_owned())?;
            if !target_is_fully_present(&recovered, &target, revision) {
                return Err(
                    "target transaction was not wholly recovered after successful return"
                        .to_owned(),
                );
            }
        }
        Boundary::BeforeWalAppend => {
            if !target_is_unchanged(&recovered, &target, &before_target) {
                return Err("target transaction appeared before WAL append".to_owned());
            }
        }
        Boundary::BeforeWalSync | Boundary::AfterWalSync => {
            if target_is_unchanged(&recovered, &target, &before_target) {
                if boundary == Boundary::AfterWalSync {
                    return Err("after_wal_sync transaction was not recovered".to_owned());
                }
            } else {
                let observed_revision = target
                    .first()
                    .and_then(|mutation| match mutation {
                        TransactionMutation::Put { key, .. } => {
                            recovered.get(key).map(|row| row.revision)
                        }
                        TransactionMutation::Delete { .. } => None,
                    })
                    .ok_or_else(|| "target revision could not be observed".to_owned())?;
                if observed_revision <= prefix_max_revision
                    || !target_is_fully_present(&recovered, &target, observed_revision)
                {
                    return Err("target transaction was only partially recovered or has an invalid revision".to_owned());
                }
                apply_target_expected(&mut expected_final, &target, observed_revision);
                allow_target = true;
            }
        }
    }
    validate_state(&expected_final, &recovered)
        .map_err(|error| format!("first recovery verification failed: {error}"))?;
    let recovered_max_revision = expected_final
        .values()
        .map(|row| row.revision)
        .max()
        .unwrap_or(Revision::new(0));
    let mut invariant_store = open_store(engine, &data_path, &wal_path)
        .map_err(|error| format!("recovery invariant reopen failed: {error}"))?;
    let invariant_report = invariant_store
        .check_invariants()
        .map_err(|error| format!("invariant check failed: {error}"))?;
    if !invariant_report.leaked_pages.is_empty()
        || invariant_report.max_revision != recovered_max_revision
    {
        return Err(format!(
            "recovery invariants mismatch: {:?}",
            invariant_report
        ));
    }
    write_expected(
        expectation_log,
        engine,
        width,
        seed,
        case_dir,
        &expected_final,
    );
    let additional_key = row_key(4095);
    let additional_value = value_bytes(seed, 999, width);
    let mut reopened = open_store(engine, &data_path, &wal_path)
        .map_err(|error| format!("second open failed: {error}"))?;
    let additional_result = reopened
        .transact(TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: additional_key.clone(),
                value: additional_value.clone(),
            }],
        ))
        .map_err(|error| format!("post-recovery write failed: {error}"))?;
    let additional_revision = additional_result
        .revision
        .ok_or_else(|| "post-recovery response has no revision".to_owned())?;
    append_response(
        response_log,
        engine,
        width,
        seed,
        case_dir,
        "post-recovery",
        additional_revision,
    );
    drop(reopened);
    let mut final_expected = expected_final;
    final_expected.insert(
        additional_key,
        ExpectedRow {
            value: additional_value,
            revision: additional_revision,
        },
    );
    let final_actual = read_actual(engine, &data_path, &wal_path)
        .map_err(|error| format!("post-recovery reopen verification failed: {error}"))?;
    validate_state(&final_expected, &final_actual)
        .map_err(|error| format!("post-recovery reopen verification failed: {error}"))?;
    append_line(
        &case_dir.join("recovery.log"),
        format!(
            "restart=2 rows={} max_revision={} post_recovery_write_revision={} verification=passed\n",
            final_actual.len(),
            final_actual
                .values()
                .map(|row| row.revision.get())
                .max()
                .unwrap_or(0),
            additional_revision.get()
        ),
    );
    Ok(format!(
        "hook_reached={hook_reached};sigkill={};recovered_rows={};target={}",
        boundary.is_crash(),
        recovered.len(),
        if allow_target { "full" } else { "absent" }
    ))
}

fn open_store(engine: Engine, data_path: &Path, wal_path: &Path) -> dodb_core::Result<Store> {
    match engine {
        Engine::MainBtree => Ok(Store::Main(BTreeStore::open_with_wal(
            ProductionFile::open(data_path)?,
            ProductionFile::open(wal_path)?,
            DatabaseConfig::default(),
        )?)),
        Engine::PlannedBlink => {
            let mut store = BlinkStore::open_with_wal(
                ProductionFile::open(data_path)?,
                ProductionFile::open(wal_path)?,
                DatabaseConfig::default(),
            )?;
            store.enable_planned_execution();
            Ok(Store::Planned(store))
        }
    }
}

fn read_actual(
    engine: Engine,
    data_path: &Path,
    wal_path: &Path,
) -> dodb_core::Result<BTreeMap<DocumentKey, ExpectedRow>> {
    let mut store = open_store(engine, data_path, wal_path)?;
    let invariants = store.check_invariants()?;
    if !invariants.leaked_pages.is_empty() {
        return Err(dodb_core::Error::invariant(format!(
            "leaked pages: {:?}",
            invariants.leaked_pages
        )));
    }
    let documents = store.scan_all()?;
    let actual: BTreeMap<_, _> = documents
        .into_iter()
        .map(|document| {
            (
                document.key,
                ExpectedRow {
                    value: document.value,
                    revision: document.revision,
                },
            )
        })
        .collect();
    let maximum_revision = actual
        .values()
        .map(|row| row.revision)
        .max()
        .unwrap_or(Revision::new(0));
    if invariants.max_revision != maximum_revision {
        return Err(dodb_core::Error::invariant(format!(
            "max revision mismatch: expected {:?}, got {:?}",
            maximum_revision, invariants.max_revision
        )));
    }
    Ok(actual)
}

fn validate_state(
    expected: &BTreeMap<DocumentKey, ExpectedRow>,
    actual: &BTreeMap<DocumentKey, ExpectedRow>,
) -> Result<(), String> {
    if expected.len() != actual.len() {
        return Err(format!(
            "row count mismatch: expected {}, got {}",
            expected.len(),
            actual.len()
        ));
    }
    for (key, expected_row) in expected {
        let actual_row = actual
            .get(key)
            .ok_or_else(|| format!("missing key {:?}", key))?;
        if actual_row.value != expected_row.value {
            return Err(format!("wrong value for key {:?}", key));
        }
        if actual_row.revision != expected_row.revision {
            return Err(format!(
                "wrong revision for key {:?}: expected {:?}, got {:?}",
                key, expected_row.revision, actual_row.revision
            ));
        }
    }
    if let Some(extra_key) = actual.keys().find(|key| !expected.contains_key(*key)) {
        return Err(format!("unexpected key {:?}", extra_key));
    }
    Ok(())
}

fn verify_checker_rejects_invalid_states() {
    let key_a = row_key(1);
    let key_b = row_key(2);
    let expected = BTreeMap::from([
        (
            key_a.clone(),
            ExpectedRow {
                value: value_bytes(SEEDS[0], 1, 0),
                revision: Revision::new(9),
            },
        ),
        (
            key_b.clone(),
            ExpectedRow {
                value: value_bytes(SEEDS[0], 2, 1),
                revision: Revision::new(10),
            },
        ),
    ]);
    assert!(
        validate_state(
            &expected,
            &BTreeMap::from([(key_a.clone(), expected[&key_a].clone())])
        )
        .is_err()
    );
    let mut wrong_value = expected.clone();
    wrong_value.get_mut(&key_a).unwrap().value[0] ^= 1;
    assert!(validate_state(&expected, &wrong_value).is_err());
    let mut wrong_revision = expected.clone();
    wrong_revision.get_mut(&key_b).unwrap().revision = Revision::new(9);
    assert!(validate_state(&expected, &wrong_revision).is_err());
    let mut partial_target = expected.clone();
    partial_target.get_mut(&key_a).unwrap().value[8] ^= 1;
    assert!(validate_state(&expected, &partial_target).is_err());
}

fn target_mutations(seed: u64, width: usize) -> Vec<TransactionMutation> {
    (0..width)
        .map(|mutation_index| {
            let row_index = (mutation_index * 503 + 113) % SEED_ROWS;
            TransactionMutation::Put {
                key: row_key(row_index),
                value: value_bytes(seed, PREFIX_TRANSACTIONS, mutation_index),
            }
        })
        .collect()
}

fn apply_target_expected(
    expected: &mut BTreeMap<DocumentKey, ExpectedRow>,
    target: &[TransactionMutation],
    revision: Revision,
) {
    for mutation in target {
        if let TransactionMutation::Put { key, value } = mutation {
            expected.insert(
                key.clone(),
                ExpectedRow {
                    value: value.clone(),
                    revision,
                },
            );
        }
    }
}

fn target_is_fully_present(
    actual: &BTreeMap<DocumentKey, ExpectedRow>,
    target: &[TransactionMutation],
    revision: Revision,
) -> bool {
    target.iter().all(|mutation| match mutation {
        TransactionMutation::Put { key, value } => actual
            .get(key)
            .is_some_and(|row| row.value == *value && row.revision == revision),
        TransactionMutation::Delete { .. } => false,
    })
}

fn target_is_unchanged(
    actual: &BTreeMap<DocumentKey, ExpectedRow>,
    target: &[TransactionMutation],
    before: &BTreeMap<DocumentKey, ExpectedRow>,
) -> bool {
    target.iter().all(|mutation| match mutation {
        TransactionMutation::Put { key, .. } => actual.get(key) == before.get(key),
        TransactionMutation::Delete { .. } => false,
    })
}

fn prefix_mutations(seed: u64, width: usize, transaction_index: usize) -> Vec<TransactionMutation> {
    (0..width)
        .map(|mutation_index| {
            let row_index = (transaction_index * 37 + mutation_index * 503 + 71) % SEED_ROWS;
            TransactionMutation::Put {
                key: row_key(row_index),
                value: value_bytes(seed, transaction_index + 1, mutation_index),
            }
        })
        .collect()
}

fn row_key(row_index: usize) -> DocumentKey {
    let partition = (row_index as u64).to_be_bytes();
    let sort = (row_index as u64 ^ 0xd0db_2026).to_be_bytes();
    DocumentKey::new(partition.to_vec(), sort.to_vec())
}

fn value_bytes(seed: u64, transaction_index: usize, mutation_index: usize) -> Vec<u8> {
    let mut value = vec![0_u8; 64];
    value[..8].copy_from_slice(&seed.to_be_bytes());
    value[8..16].copy_from_slice(&(transaction_index as u64).to_be_bytes());
    value[16..24].copy_from_slice(&(mutation_index as u64).to_be_bytes());
    for (byte_index, byte) in value[24..].iter_mut().enumerate() {
        *byte = seed
            .wrapping_add((transaction_index as u64).wrapping_mul(31))
            .wrapping_add((mutation_index as u64).wrapping_mul(17))
            .wrapping_add(byte_index as u64) as u8;
    }
    value
}

fn append_response(
    path: &Path,
    engine: Engine,
    width: usize,
    seed: u64,
    case_dir: &Path,
    transaction: &str,
    revision: Revision,
) {
    append_line(
        path,
        format!(
            "{}\t{width}\t{seed:08x}\t{}\t{transaction}\t{}\n",
            engine.as_str(),
            case_dir.file_name().unwrap().to_string_lossy(),
            revision.get()
        ),
    );
}

fn write_expected(
    path: &Path,
    engine: Engine,
    width: usize,
    seed: u64,
    case_dir: &Path,
    expected: &BTreeMap<DocumentKey, ExpectedRow>,
) {
    let mut output = String::new();
    for (key, row) in expected {
        output.push_str(&format!(
            "{}\t{width}\t{seed:08x}\t{}\t{}\t{}\t{}\n",
            engine.as_str(),
            case_dir.file_name().unwrap().to_string_lossy(),
            hex(&key.encode()),
            hex(&row.value),
            row.revision.get()
        ));
    }
    append_line(path, output);
}

fn copy_pre_recovery(data_path: &Path, wal_path: &Path, case_dir: &Path) -> std::io::Result<()> {
    let snapshot_dir = case_dir.join("pre-recovery");
    fs::create_dir_all(&snapshot_dir)?;
    if data_path.exists() {
        fs::copy(data_path, snapshot_dir.join("target.db"))?;
    }
    if wal_path.exists() {
        fs::copy(wal_path, snapshot_dir.join("target.wal"))?;
    }
    Ok(())
}

fn stop_child(child: &mut Child) -> String {
    let pid = child.id();
    let result = kill_and_wait(child);
    format!("child_pid={pid};termination={result:?}")
}

fn kill_and_wait(child: &mut Child) -> std::io::Result<std::process::ExitStatus> {
    if let Some(status) = child.try_wait()? {
        return Ok(status);
    }
    child.kill()?;
    child.wait()
}

fn append_line(path: &Path, text: String) {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("artifact log should open");
    file.write_all(text.as_bytes())
        .expect("artifact log should write");
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}
