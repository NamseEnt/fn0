#[path = "support/blink_checkpoint.rs"]
mod blink_checkpoint;
#[path = "support/tenant_density/config.rs"]
mod config;
#[path = "support/tenant_density/resources.rs"]
mod resources;
#[path = "support/tenant_density/scheduler.rs"]
mod scheduler;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use config::{Options, Topology};
use dodb_core::{
    DocumentKey, RevisionState, TenantId, TransactionCondition, TransactionMutation,
    TransactionRequest,
};
use dodb_rocksdb::memory_diagnostic::{MemoryDiagnosticSession, RemoteDodbService};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use serde_json::{Value, json};

const RESPONSE_BUDGET_BYTES: usize = 64 * 1024 * 1024;

fn filesystem_snapshot(path: &Path) -> Value {
    match resources::filesystem_snapshot(path) {
        Ok(snapshot) => snapshot,
        Err(error) => json!({"status": "unavailable", "reason": error}),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let options = Options::parse(&arguments).map_err(std::io::Error::other)?;
    let report_path = options.report_path.clone();
    let execution: Result<Value, Box<dyn std::error::Error>> = async {
    let source_revision = std::env::var("DODB_SOURCE_REVISION")?;
    let compiled_source_revision = env!("DODB_BUILD_GIT_COMMIT");
    if source_revision != compiled_source_revision {
        return Err(std::io::Error::other(format!(
            "requested source revision {source_revision} does not match compiled revision {compiled_source_revision}"
        )).into());
    }
    let session = MemoryDiagnosticSession::new(source_revision.clone()).map_err(std::io::Error::other)?;
    let worker_path = std::env::current_exe()?.with_file_name("memory_diagnostic_worker");
    let filesystem_before_setup = filesystem_snapshot(&options.data_dir);
    let setup_worker = spawn_worker(&session, &worker_path, &options, "setup").await?;
    let setup_service = RemoteDodbService::new(setup_worker.connection().clone());
    seed_population(&setup_service, &options).await?;
    setup_worker.shutdown().map_err(std::io::Error::other)?;
    let filesystem_after_setup = filesystem_snapshot(&options.data_dir);

    let measurement_worker = spawn_worker(&session, &worker_path, &options, "measurement").await?;
    let service = RemoteDodbService::new(measurement_worker.connection().clone());
    open_measurement_tenants(&service, &options).await?;
    session.set_phase("measurement-correctness-preflight");
    let preflight_mutations = correctness_preflight(&service, &options).await?;
    measurement_worker
        .start_measurement()
        .map_err(std::io::Error::other)?;
    session.set_phase("offered-load");
    let offer_process_before = session.process_boundary_snapshot();
    let offer_host_before = resources::host_device_snapshot();
    let offer_io_before = resources::process_io_from_boundary(&offer_process_before);
    let driver_io_before = blink_checkpoint::process_write_io();
    let started = Instant::now();
    let workload = run_workload(&service, &options, &session).await;
    let elapsed_after_drain = started.elapsed();
    session.set_phase("measurement-drain");
    measurement_worker
        .stop_measurement()
        .map_err(std::io::Error::other)?;
    measurement_worker
        .shutdown()
        .map_err(std::io::Error::other)?;
    let filesystem_after_measurement = filesystem_snapshot(&options.data_dir);
    let (workload, mut expected_mutations) = workload?;
    for (key, value) in preflight_mutations {
        if expected_mutations.insert(key, value).is_some() {
            return Err(std::io::Error::other("preflight arena overlaps acknowledged workload mutations").into());
        }
    }
    let offered_elapsed = workload["offer_window_elapsed_ms"]
        .as_u64()
        .map(Duration::from_millis)
        .unwrap_or(elapsed_after_drain);
    let offer_boundary = &workload["offer_boundary"];
    let boundary_fields_present = [
        "process_after",
        "host_device_after",
        "process_io_after",
        "driver_process_io_after",
    ]
    .iter()
    .all(|field| !offer_boundary[*field].is_null());
    let offer_process_after = offer_boundary["process_after"].clone();
    let offer_host_after = offer_boundary["host_device_after"].clone();
    let offer_io_after = offer_boundary["process_io_after"].clone();
    let driver_io_after = offer_boundary["driver_process_io_after"].clone();
    let oracle_is_valid = workload["valid_for_oracle"].as_bool().unwrap_or(false);
    let accounting_is_valid = workload["accounting"]
        .as_object()
        .is_some_and(|accounting| accounting.values().all(|value| value.as_bool() == Some(true)));
    let mut cell_is_valid = workload["cell_valid"].as_bool().unwrap_or(false)
        && boundary_fields_present
        && accounting_is_valid;
    let (oracle, reopened_oracle, oracle_filesystem_after, reopened_filesystem_after) = if oracle_is_valid {
        let oracle_worker = spawn_worker(&session, &worker_path, &options, "oracle").await?;
        let oracle_service = RemoteDodbService::new(oracle_worker.connection().clone());
        let oracle = validate_population(&oracle_service, &options, &expected_mutations).await?;
        oracle_worker.shutdown().map_err(std::io::Error::other)?;
        let filesystem_after_oracle = filesystem_snapshot(&options.data_dir);

        let reopened_worker = spawn_worker(&session, &worker_path, &options, "reopen-oracle").await?;
        let reopened_service = RemoteDodbService::new(reopened_worker.connection().clone());
        let reopened =
            validate_population(&reopened_service, &options, &expected_mutations).await?;
        reopened_worker.shutdown().map_err(std::io::Error::other)?;
        let filesystem_after_reopen_oracle = filesystem_snapshot(&options.data_dir);
        (oracle, reopened, filesystem_after_oracle, filesystem_after_reopen_oracle)
    } else {
        let skipped = json!({
            "passed": false,
            "status": "skipped",
            "reason": "a timed-out or failed mutation has uncertain outcome"
        });
        let skipped_filesystem = json!({"status": "skipped", "reason": "oracle skipped because mutation outcome is uncertain"});
        (skipped.clone(), skipped, skipped_filesystem.clone(), skipped_filesystem)
    };
    cell_is_valid &= oracle["passed"] == true && reopened_oracle["passed"] == true;

    let diagnostic = session.finish_sampling();
    let measurement_process_summary = resources::summarize_measurement_processes(
        &diagnostic,
        &offer_process_before,
        &offer_process_after,
    );
    Ok(json!({
        "status": if cell_is_valid { "completed" } else { "inconclusive" },
        "requested_source_revision": source_revision,
        "compiled_source_revision": compiled_source_revision,
        "effective_configuration": effective_configuration(&options),
        "measurement_worker_epoch": "measurement worker only; opened-idle tenants received Get; registered minus active and opened-idle were never opened in this epoch",
        "correctness_preflight_key_scope": {
            "tenant": 1,
            "rows": [options.initial_rows + options.client_count * 16 + 16, options.initial_rows + options.client_count * 16 + 17, options.initial_rows + options.client_count * 16 + 18],
            "disjoint_from_seeded_rows": true,
            "disjoint_from_fixed_total_mutation_lanes": true,
            "validated_in_post_measurement_full_population_oracle": oracle_is_valid
        },
        "offered_load_elapsed_ms": offered_elapsed.as_millis(),
        "offer_boundary_process_before": offer_process_before,
        "offer_boundary_process_after": offer_process_after,
        "offer_boundary_host_device_before": offer_host_before,
        "offer_boundary_host_device_after": offer_host_after,
        "offer_boundary_process_io_before": offer_io_before,
        "offer_boundary_process_io_after": offer_io_after,
        "driver_process_write_io": {"before_offer": driver_io_before, "after_offer": driver_io_after, "scope": "harness process cumulative counters; runtime and transport included"},
        "filesystem_snapshots": {
            "before_setup": filesystem_before_setup,
            "after_setup_worker_shutdown": filesystem_after_setup,
            "after_measurement_worker_shutdown": filesystem_after_measurement,
            "after_oracle_worker_shutdown": oracle_filesystem_after,
            "after_reopen_oracle_worker_shutdown": reopened_filesystem_after
        },
        "workload": workload,
        "post_measurement_full_population_oracle": oracle,
        "reopened_full_population_oracle": reopened_oracle,
        "memory_diagnostic": diagnostic,
        "measurement_worker_process_sampling": measurement_process_summary,
        "retained_heap_bytes": {"value": null, "status": "unsupported", "reason": "allocator retained heap is not instrumented"},
        "filesystem_write_amplification": {"value": null, "status": "unsupported", "reason": "allocated blocks are not device or ZFS write amplification"}
    }))
    }
    .await;
    match execution {
        Ok(report) => {
            let cell_is_valid = report["status"].as_str() == Some("completed")
                && report["post_measurement_full_population_oracle"]["passed"] == true
                && report["reopened_full_population_oracle"]["passed"] == true;
            std::fs::write(&report_path, serde_json::to_vec_pretty(&report)?)?;
            if cell_is_valid {
                Ok(())
            } else {
                Err(std::io::Error::other(
                    "cell report is inconclusive or failed; report preserved",
                )
                .into())
            }
        }
        Err(error) => {
            let failure_report = json!({
                "status": "failed_or_inconclusive",
                "failure_reason": error.to_string(),
                "data_directory_preserved": true,
                "partial_workload_counters": {"status": "unavailable_after_early_abort"}
            });
            std::fs::write(&report_path, serde_json::to_vec_pretty(&failure_report)?)?;
            Err(error)
        }
    }
}

async fn spawn_worker(
    session: &Arc<MemoryDiagnosticSession>,
    worker_path: &Path,
    options: &Options,
    phase: &str,
) -> Result<dodb_rocksdb::memory_diagnostic::MemoryDiagnosticWorker, Box<dyn std::error::Error>> {
    session.set_phase(phase);
    let mut worker_arguments = vec![
        "--backend".to_owned(),
        options.backend.as_str().to_owned(),
        "--revision-allocator".to_owned(),
        "reserved-ranges".to_owned(),
        "--condition-key-lock-mode".to_owned(),
        "exclusive".to_owned(),
        "--memory-blink-max-open-shards".to_owned(),
        options.registered_tenants.to_string(),
        "--memory-tenant-count".to_owned(),
        options.registered_tenants.to_string(),
    ];
    if options.backend == config::Backend::Blink {
        let checkpoint_configuration = options.blink_checkpoint.configuration();
        worker_arguments.extend([
            "--blink-wal-layout".to_owned(),
            checkpoint_configuration["layout"]
                .as_str()
                .unwrap_or("legacy")
                .to_owned(),
            "--blink-checkpoint-execution".to_owned(),
            checkpoint_configuration["execution"]
                .as_str()
                .unwrap_or("sync")
                .to_owned(),
        ]);
        if let Some(segment_bytes) = checkpoint_configuration["segment_bytes"].as_u64() {
            worker_arguments.extend([
                "--blink-wal-segment-bytes".to_owned(),
                segment_bytes.to_string(),
            ]);
        }
        if let dodb_server::BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } =
            options.checkpoint_policy
        {
            worker_arguments.extend([
                "--blink-checkpoint-wal-bytes".to_owned(),
                threshold_bytes.to_string(),
            ]);
        }
    }
    session
        .spawn_worker(
            worker_path,
            &worker_arguments,
            RESPONSE_BUDGET_BYTES,
            dodb_protocol::ProtocolLimits::default().max_mutations,
            options.max_in_flight,
            &options.data_dir,
            Duration::from_secs(30),
        )
        .await
        .map_err(|error| Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>)
}

async fn seed_population(
    service: &RemoteDodbService,
    options: &Options,
) -> Result<(), Box<dyn std::error::Error>> {
    for tenant_number in 1..=options.registered_tenants {
        let tenant = TenantId::new(tenant_number as u64);
        for first_row in (0..options.initial_rows).step_by(16) {
            let mutations = (first_row..(first_row + 16).min(options.initial_rows))
                .map(|row_number| TransactionMutation::Put {
                    key: document_key(row_number),
                    value: seeded_value(options, tenant_number, row_number),
                })
                .collect();
            let request = Request::Transact {
                request: TransactionRequest::new(Vec::new(), mutations),
            };
            expect_response(
                service
                    .execute(tenant, request, ExecutionBudget::new(RESPONSE_BUDGET_BYTES))
                    .await?,
                "seed transaction",
            )?;
        }
    }
    Ok(())
}

async fn open_measurement_tenants(
    service: &RemoteDodbService,
    options: &Options,
) -> Result<(), Box<dyn std::error::Error>> {
    for tenant_number in 1..=(options.active_tenants + options.opened_idle_tenants) {
        let request = Request::Get {
            key: document_key(0),
        };
        expect_response(
            service
                .execute(
                    TenantId::new(tenant_number as u64),
                    request,
                    ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
                )
                .await?,
            "measurement open Get",
        )?;
    }
    Ok(())
}

async fn correctness_preflight(
    service: &RemoteDodbService,
    options: &Options,
) -> Result<BTreeMap<(u64, usize), Option<Vec<u8>>>, Box<dyn std::error::Error>> {
    let tenant = TenantId::new(1);
    let preflight_start = options
        .initial_rows
        .checked_add(
            options
                .client_count
                .checked_mul(16)
                .ok_or_else(|| std::io::Error::other("preflight key range overflow"))?,
        )
        .ok_or_else(|| std::io::Error::other("preflight key range overflow"))?;
    let first_row = preflight_start + 16;
    let second_row = preflight_start + 17;
    let third_row = preflight_start + 18;
    let first_key = document_key(first_row);
    let second_key = document_key(second_row);
    let initial_value = workload_value(options, 0xc011_5eed);
    transact(
        service,
        tenant,
        vec![TransactionCondition::NotExists {
            key: first_key.clone(),
        }],
        vec![TransactionMutation::Put {
            key: first_key.clone(),
            value: initial_value.clone(),
        }],
    )
    .await?;
    match transact(
        service,
        tenant,
        vec![TransactionCondition::NotExists {
            key: first_key.clone(),
        }],
        vec![TransactionMutation::Put {
            key: first_key.clone(),
            value: vec![1],
        }],
    )
    .await
    {
        Err(error)
            if matches!(
                error.downcast_ref::<dodb_core::Error>(),
                Some(dodb_core::Error::Conflict(_))
            ) => {}
        Err(error) => {
            return Err(std::io::Error::other(format!(
                "NotExists preflight failed for a non-conflict reason: {error}"
            ))
            .into());
        }
        Ok(()) => {
            return Err(std::io::Error::other(
                "NotExists rejection preflight unexpectedly committed",
            )
            .into());
        }
    }
    let first_state = get_state(service, tenant, first_key.clone()).await?;
    let first_revision = match &first_state {
        RevisionState::Present { value, revision } if value == &initial_value => *revision,
        _ => {
            return Err(std::io::Error::other("conditional write preflight value mismatch").into());
        }
    };
    let atomic_value = workload_value(options, 0x00a7_001c);
    let atomic_result = service
        .execute(
            tenant,
            Request::Transact {
                request: TransactionRequest::new(
                    vec![
                        TransactionCondition::Exists {
                            key: first_key.clone(),
                        },
                        TransactionCondition::Exists {
                            key: second_key.clone(),
                        },
                    ],
                    vec![
                        TransactionMutation::Put {
                            key: first_key.clone(),
                            value: atomic_value,
                        },
                        TransactionMutation::Put {
                            key: second_key.clone(),
                            value: vec![2],
                        },
                    ],
                ),
            },
            ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
        )
        .await;
    match atomic_result {
        Err(dodb_core::Error::Conflict(_)) => {}
        Err(error) => {
            return Err(std::io::Error::other(format!(
                "multi-key atomicity preflight failed for a non-conflict reason: {error}"
            ))
            .into());
        }
        Ok(_) => {
            return Err(std::io::Error::other(
                "multi-key atomicity preflight unexpectedly committed",
            )
            .into());
        }
    }
    if get_state(service, tenant, first_key.clone()).await? != first_state
        || !get_state(service, tenant, second_key.clone())
            .await?
            .is_missing()
    {
        return Err(std::io::Error::other("multi-key rejection changed one of its keys").into());
    }
    let revised_value = workload_value(options, 0x5a1e_0001);
    transact(
        service,
        tenant,
        vec![TransactionCondition::RevisionEquals {
            key: first_key.clone(),
            expected_revision: first_revision,
        }],
        vec![TransactionMutation::Put {
            key: first_key.clone(),
            value: revised_value.clone(),
        }],
    )
    .await?;
    match transact(
        service,
        tenant,
        vec![TransactionCondition::RevisionEquals {
            key: first_key.clone(),
            expected_revision: first_revision,
        }],
        vec![TransactionMutation::Put {
            key: first_key.clone(),
            value: vec![3],
        }],
    )
    .await
    {
        Err(error)
            if matches!(
                error.downcast_ref::<dodb_core::Error>(),
                Some(dodb_core::Error::Conflict(_))
            ) => {}
        Err(error) => {
            return Err(std::io::Error::other(format!(
                "stale revision preflight failed for a non-conflict reason: {error}"
            ))
            .into());
        }
        Ok(()) => {
            return Err(
                std::io::Error::other("stale revision preflight unexpectedly committed").into(),
            );
        }
    }
    let latest_revision = match get_state(service, tenant, first_key.clone()).await? {
        RevisionState::Present { value, revision } if value == revised_value => revision,
        _ => {
            return Err(
                std::io::Error::other("revision condition preflight value mismatch").into(),
            );
        }
    };
    if latest_revision <= first_revision {
        return Err(std::io::Error::other(
            "successful revision-conditional write did not advance revision",
        )
        .into());
    }
    expect_response(
        service
            .execute(
                tenant,
                Request::Delete {
                    key: first_key.clone(),
                },
                ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
            )
            .await?,
        "preflight delete",
    )?;
    if !get_state(service, tenant, first_key.clone())
        .await?
        .is_missing()
    {
        return Err(std::io::Error::other("delete preflight did not produce missing state").into());
    }
    let reinsert_value = workload_value(options, 0x7e1a_5e47);
    transact(
        service,
        tenant,
        vec![TransactionCondition::NotExists {
            key: first_key.clone(),
        }],
        vec![TransactionMutation::Put {
            key: first_key,
            value: reinsert_value.clone(),
        }],
    )
    .await?;
    match get_state(service, tenant, document_key(first_row)).await? {
        RevisionState::Present { value, revision }
            if value == reinsert_value && revision > latest_revision => {}
        _ => {
            return Err(std::io::Error::other(
                "tombstone reinsert preflight did not restore a newer value",
            )
            .into());
        }
    }
    let mut expected = BTreeMap::new();
    expected.insert((tenant.get(), first_row), Some(reinsert_value));
    expected.insert((tenant.get(), second_row), None);
    expected.insert((tenant.get(), third_row), None);
    Ok(expected)
}

async fn transact(
    service: &RemoteDodbService,
    tenant: TenantId,
    conditions: Vec<TransactionCondition>,
    mutations: Vec<TransactionMutation>,
) -> Result<(), Box<dyn std::error::Error>> {
    expect_response(
        service
            .execute(
                tenant,
                Request::Transact {
                    request: TransactionRequest::new(conditions, mutations),
                },
                ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
            )
            .await?,
        "correctness preflight transaction",
    )
}

async fn get_state(
    service: &RemoteDodbService,
    tenant: TenantId,
    key: DocumentKey,
) -> Result<RevisionState, Box<dyn std::error::Error>> {
    match service
        .execute(
            tenant,
            Request::Get { key },
            ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
        )
        .await?
    {
        Response::Get(state) => Ok(state),
        response => {
            Err(std::io::Error::other(format!("unexpected preflight response {response:?}")).into())
        }
    }
}

async fn run_workload(
    service: &RemoteDodbService,
    options: &Options,
    session: &Arc<MemoryDiagnosticSession>,
) -> Result<(Value, BTreeMap<(u64, usize), Option<Vec<u8>>>), Box<dyn std::error::Error>> {
    let scheduler_service = Arc::new(RemoteDodbService::new(service.connection().clone()));
    let callback_session = Arc::clone(session);
    let on_offer_end = move || {
        callback_session.set_phase("measurement-drain");
        let process_after = callback_session.process_boundary_snapshot();
        json!({
            "process_after": process_after,
            "host_device_after": resources::host_device_snapshot(),
            "process_io_after": resources::process_io_from_boundary(&process_after),
            "driver_process_io_after": blink_checkpoint::process_write_io()
        })
    };
    if options.topology == Topology::FixedPerTenant {
        return scheduler::run_fixed_per_tenant(scheduler_service, options, on_offer_end)
            .await
            .map_err(|error| Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>);
    }
    return scheduler::run_fixed_total(scheduler_service, options, on_offer_end)
        .await
        .map_err(|error| Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>);
}

async fn validate_population(
    service: &RemoteDodbService,
    options: &Options,
    expected_mutations: &BTreeMap<(u64, usize), Option<Vec<u8>>>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut verified_rows = 0_u64;
    for tenant_number in 1..=options.registered_tenants {
        let tenant = TenantId::new(tenant_number as u64);
        for row_number in 0..options.initial_rows {
            let state = service
                .execute(
                    tenant,
                    Request::Get {
                        key: document_key(row_number),
                    },
                    ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
                )
                .await?;
            match state {
                Response::Get(RevisionState::Present { value, .. })
                    if value == seeded_value(options, tenant_number, row_number) =>
                {
                    verified_rows += 1
                }
                _ => {
                    return Err(Box::new(std::io::Error::other(format!(
                        "oracle mismatch tenant={tenant_number} row={row_number}"
                    ))));
                }
            }
        }
    }
    for ((tenant_number, row_number), expected_value) in expected_mutations {
        let response = service
            .execute(
                TenantId::new(*tenant_number),
                Request::Get {
                    key: document_key(*row_number),
                },
                ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
            )
            .await?;
        let matches = match (expected_value, response) {
            (Some(expected), Response::Get(RevisionState::Present { value, .. })) => {
                *expected == value
            }
            (None, Response::Get(RevisionState::Missing { .. })) => true,
            _ => false,
        };
        if !matches {
            return Err(Box::new(std::io::Error::other(format!(
                "mutation oracle mismatch tenant={tenant_number} row={row_number}"
            ))));
        }
    }
    Ok(
        json!({"scope": "all registered tenants plus every acknowledged mutated key; outside measurement worker epoch", "verified_seed_rows": verified_rows, "verified_mutated_keys": expected_mutations.len(), "passed": true}),
    )
}

fn document_key(row_number: usize) -> DocumentKey {
    DocumentKey::new(vec![0; 16], (row_number as u128).to_be_bytes().to_vec())
}

fn seeded_value(options: &Options, tenant_number: usize, row_number: usize) -> Vec<u8> {
    make_value(
        options.value_bytes,
        options.seed ^ ((tenant_number as u64) << 32) ^ row_number as u64,
    )
}

fn workload_value(options: &Options, operation_number: u64) -> Vec<u8> {
    make_value(
        options.value_bytes,
        options.seed ^ operation_number ^ 0xa5a5_5a5a_d3c4_b2e1,
    )
}

fn make_value(length: usize, seed: u64) -> Vec<u8> {
    let mut value = Vec::with_capacity(length);
    let mut state = seed;
    while value.len() < length {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        value.extend_from_slice(&state.to_be_bytes());
    }
    value.truncate(length);
    value
}

fn expect_response(response: Response, context: &str) -> Result<(), Box<dyn std::error::Error>> {
    match response {
        Response::Transact(_) | Response::Get(_) | Response::Query(_) | Response::Delete(_) => {
            Ok(())
        }
        other => Err(Box::new(std::io::Error::other(format!(
            "unexpected response for {context}: {other:?}"
        )))),
    }
}

fn effective_configuration(options: &Options) -> Value {
    json!({
        "backend": options.backend.as_str(), "registered_tenants": options.registered_tenants,
        "seed": options.seed, "duration_ms": options.duration_ms, "operation_count": options.operation_count,
        "request_timeout_ms": options.request_timeout_ms, "drain_timeout_ms": options.drain_timeout_ms,
        "active_tenants": options.active_tenants, "opened_idle_tenants": options.opened_idle_tenants,
        "measurement_never_opened_tenants": options.registered_tenants - options.active_tenants - options.opened_idle_tenants,
        "client_count": options.client_count, "topology": options.topology.as_str(),
        "profile": options.profile.as_str(), "profile_weights_get_query_write": options.profile.weights(),
        "width": match options.width { config::Width::Mixed => json!("mixed 80/15/5 for 1/4/16"), config::Width::Fixed(width) => json!(width) }, "query_limit": 16, "primary_key_bytes": 16, "sort_key_bytes": 16,
        "value_bytes": options.value_bytes, "initial_rows_per_tenant": options.initial_rows,
        "queue_bound": options.queue_bound, "max_in_flight": options.max_in_flight,
        "per_tenant_rate": options.per_tenant_rate,
        "blink_layout": options.blink_checkpoint.configuration(),
        "checkpoint_policy": format!("{:?}", options.checkpoint_policy),
        "minimal_native_revisions": "reserved-ranges", "minimal_native_condition_locks": "exclusive"
    })
}
