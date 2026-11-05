#![recursion_limit = "256"]

#[path = "support/blink_checkpoint.rs"]
mod blink_checkpoint;
#[path = "support/tenant_density/config.rs"]
mod config;
#[path = "support/tenant_density/resources.rs"]
mod resources;
#[path = "support/tenant_density/scheduler.rs"]
mod scheduler;
#[path = "support/tenant_density/workload_failure.rs"]
mod workload_failure;

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
use sha2::{Digest, Sha256};

const RESPONSE_BUDGET_BYTES: usize = 64 * 1024 * 1024;

fn filesystem_snapshot(path: &Path) -> Value {
    match resources::filesystem_snapshot(path) {
        Ok(snapshot) => snapshot,
        Err(error) => json!({"status": "unavailable", "reason": error}),
    }
}

fn measurement_resource_samples(diagnostic: &Value) -> Value {
    let Some(measurement_report) = diagnostic
        .get("engine_health_reports")
        .and_then(Value::as_array)
        .and_then(|reports| {
            reports
                .iter()
                .find(|report| report["worker_phase"].as_str() == Some("measurement"))
        })
    else {
        return json!({"status": "unavailable", "reason": "measurement worker report is missing"});
    };
    let samples = measurement_report["periodic_pressure"]["samples"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let allocator_samples = samples
        .iter()
        .filter_map(|sample| sample["allocator_retained"]["retained_free_arena_bytes"].as_u64())
        .collect::<Vec<_>>();
    let reachable_samples = samples
        .iter()
        .filter_map(|sample| {
            sample["blink_paged_resources"]["disk_bytes"]["reachable_source_bytes"].as_u64()
        })
        .collect::<Vec<_>>();
    let minimum_allocator_retained_bytes = allocator_samples.iter().copied().min();
    let maximum_allocator_retained_bytes = allocator_samples.iter().copied().max();
    let minimum_reachable_source_bytes = reachable_samples.iter().copied().min();
    let maximum_reachable_source_bytes = reachable_samples.iter().copied().max();
    json!({
        "status": "sampled",
        "worker_phase": "measurement-including-drain",
        "periodic_sample_count": samples.len(),
        "allocator_retained": {
            "allocator": "glibc mallinfo2",
            "field": "fordblks",
            "sample_count": allocator_samples.len(),
            "missing_sample_count": samples.len().saturating_sub(allocator_samples.len()),
            "minimum_bytes": minimum_allocator_retained_bytes,
            "maximum_bytes": maximum_allocator_retained_bytes,
            "meaning": "free arena bytes retained by glibc; excludes allocated blocks and separately mmap-allocated blocks"
        },
        "reachable_source": {
            "sample_count": reachable_samples.len(),
            "missing_sample_count": samples.len().saturating_sub(reachable_samples.len()),
            "minimum_logical_file_bytes": minimum_reachable_source_bytes,
            "maximum_logical_file_bytes": maximum_reachable_source_bytes,
            "meaning": "unique reachable checkpoint pack and WAL segment logical lengths; not allocated filesystem blocks or device writes"
        },
    })
}

fn successful_checkpoint_cycles(session: &MemoryDiagnosticSession) -> u64 {
    session
        .health_reports_snapshot()
        .iter()
        .map(|report| {
            let measurement_start_ms =
                report["shared_resource_snapshots"]["measurement_started_unix_ms"]
                    .as_u64()
                    .unwrap_or(u64::MAX);
            let measurement_end_ms =
                report["shared_resource_snapshots"]["measurement_stopped_unix_ms"]
                    .as_u64()
                    .unwrap_or(0);
            report["blink_checkpoint_metrics"]["checkpoint_events"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|event| {
                    event["succeeded"] == true
                        && event["started_unix_nanos"]
                            .as_u64()
                            .is_some_and(|started| started / 1_000_000 >= measurement_start_ms)
                        && event["ended_unix_nanos"]
                            .as_u64()
                            .is_some_and(|ended| ended / 1_000_000 <= measurement_end_ms)
                })
                .count() as u64
        })
        .sum()
}

fn validate_long_reader_retention(
    diagnostic: &Value,
    required_duration_ms: u64,
    require_checkpoint_overlap: bool,
) -> Value {
    if required_duration_ms == 0 {
        return json!({ "status": "not_requested", "passed": true });
    }
    let mut retained_duration_ms = 0_u64;
    let mut successful_checkpoints_during_reader = 0_u64;
    let mut passed_event = false;
    let mut events = Vec::new();
    if let Some(reports) = diagnostic["engine_health_reports"].as_array() {
        for report in reports {
            let reader_events = report["long_reader_retention"]["events"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let checkpoint_events = report["blink_checkpoint_metrics"]["checkpoint_events"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for reader_event in reader_events {
                let start_ms = reader_event["started_unix_ms"].as_u64().unwrap_or(0);
                let end_ms = reader_event["released_unix_ms"].as_u64().unwrap_or(0);
                let held_ms = reader_event["retained_ms"].as_u64().unwrap_or(0);
                retained_duration_ms = retained_duration_ms.max(held_ms);
                let start_ns = u128::from(start_ms) * 1_000_000;
                let end_ns = u128::from(end_ms) * 1_000_000;
                let overlap_count = checkpoint_events
                    .iter()
                    .filter(|checkpoint| {
                        checkpoint["succeeded"] == true
                            && checkpoint["started_unix_nanos"]
                                .as_u64()
                                .is_some_and(|value| u128::from(value) >= start_ns)
                            && checkpoint["ended_unix_nanos"]
                                .as_u64()
                                .is_some_and(|value| u128::from(value) <= end_ns)
                    })
                    .count() as u64;
                successful_checkpoints_during_reader =
                    successful_checkpoints_during_reader.max(overlap_count);
                passed_event |= held_ms >= required_duration_ms
                    && (!require_checkpoint_overlap || overlap_count >= 1);
                events.push(json!({ "reader": reader_event, "successful_checkpoints_during_reader": overlap_count }));
            }
        }
    }
    let passed = passed_event;
    json!({
        "status": if passed { "passed" } else { "failed" },
        "passed": passed,
        "required_retained_duration_ms": required_duration_ms,
        "maximum_observed_retained_duration_ms": retained_duration_ms,
        "minimum_successful_checkpoints_during_reader": u64::from(require_checkpoint_overlap),
        "maximum_successful_checkpoints_during_reader": successful_checkpoints_during_reader,
        "events": events,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let options = Options::parse(&arguments).map_err(std::io::Error::other)?;
    let report_path = options.report_path.clone();
    let mut pressure_monitor = options
        .memory_budget
        .clone()
        .map(dodb_rocksdb::memory_pressure::PressureMonitor::new)
        .transpose()
        .map_err(std::io::Error::other)?;
    let session =
        MemoryDiagnosticSession::new(std::env::var("DODB_SOURCE_REVISION").unwrap_or_default())
            .map_err(std::io::Error::other)?;
    let mut retained_workload = None;
    let execution: Result<Value, Box<dyn std::error::Error>> = async {
    let source_revision = std::env::var("DODB_SOURCE_REVISION")?;
    let compiled_source_revision = env!("DODB_BUILD_GIT_COMMIT");
    if source_revision != compiled_source_revision {
        return Err(std::io::Error::other(format!(
            "requested source revision {source_revision} does not match compiled revision {compiled_source_revision}"
        )).into());
    }
    let worker_path = std::env::current_exe()?.with_file_name("memory_diagnostic_worker");
    let filesystem_before_setup = filesystem_snapshot(&options.data_dir);
    let setup_worker = spawn_worker(&session, &worker_path, &options, pressure_monitor.as_ref(), "setup").await?;
    let setup_service = RemoteDodbService::new(setup_worker.connection().clone());
    let seed_manifest_sha256 = seed_population(&setup_service, &options).await?;
    setup_worker.shutdown().map_err(std::io::Error::other)?;
    let filesystem_after_setup = filesystem_snapshot(&options.data_dir);

    let measurement_worker = spawn_worker(&session, &worker_path, &options, pressure_monitor.as_ref(), "measurement").await?;
    let service = RemoteDodbService::new(measurement_worker.connection().clone());
    open_measurement_tenants(&service, &options).await?;
    session.set_phase("measurement-correctness-preflight");
    let preflight_mutations = correctness_preflight(&service, &options).await?;
    let mut workload_seed_expected = preflight_mutations.clone();
    let mut phase_reports = serde_json::Map::new();
    if options.warmup_duration_ms > 0 {
        let mut warmup_options = options.clone();
        warmup_options.duration_ms = Some(options.warmup_duration_ms);
        warmup_options.operation_count = None;
        warmup_options.mutation_key_mode = config::MutationKeyMode::Growing;
        warmup_options.mutation_key_epoch = 5_000;
        session.set_phase("warm-up");
        let (warmup_report, warmup_mutations) = run_workload_seeded(
            &service,
            &warmup_options,
            &session,
            &workload_seed_expected,
        )
        .await?;
        if warmup_report["cell_valid"] != true {
            return Err(std::io::Error::other("warm-up workload was not operationally valid").into());
        }
        workload_seed_expected.extend(warmup_mutations);
        phase_reports.insert("warmup".to_owned(), warmup_report);
    }
    measurement_worker
        .start_measurement()
        .map_err(std::io::Error::other)?;
    session.set_phase("offered-load");
    if let Some(monitor) = &pressure_monitor { monitor.boundary("offer-start"); }
    let offer_process_before = session.process_boundary_snapshot();
    let offer_host_before = resources::host_device_snapshot();
    let offer_io_before = resources::process_io_from_boundary(&offer_process_before);
    let driver_io_before = blink_checkpoint::process_write_io();
    let started = Instant::now();
    if options.long_reader_retention_ms > 0 {
        measurement_worker
            .hold_reader(1)
            .map_err(std::io::Error::other)?;
    }
    let workload = run_slo_windowed_workload_seeded(
        &service,
        &measurement_worker,
        &options,
        &session,
        &workload_seed_expected,
    )
    .await;
    retained_workload = workload
        .as_ref()
        .ok()
        .map(|(workload, _)| workload.clone());
    let elapsed_after_drain = started.elapsed();
    if options.long_reader_retention_ms > 0 {
        measurement_worker
            .release_readers()
            .map_err(std::io::Error::other)?;
    }
    if let Some(monitor) = &pressure_monitor { monitor.boundary("after-drain"); }
    session.set_phase("measurement-drain");
    measurement_worker
        .stop_measurement()
        .map_err(std::io::Error::other)?;
    measurement_worker
        .shutdown()
        .map_err(std::io::Error::other)?;
    let filesystem_after_measurement = filesystem_snapshot(&options.data_dir);
    let (mut workload, mut expected_mutations) = workload?;
    let mut checkpoint_extension_reports = Vec::new();
    let mut measured_duration_ms = options.duration_ms.unwrap_or(0);
    let mut checkpoint_cycles = successful_checkpoint_cycles(&session);
    if options.backend == config::Backend::Blink
        && options.slo_window_ms == Some(300_000)
        && measured_duration_ms >= 1_800_000
    {
        while workload["cell_valid"] == true
            && checkpoint_cycles < 20
            && measured_duration_ms < 7_200_000
        {
            let extension_ms = 900_000_u64.min(7_200_000 - measured_duration_ms);
            let extension_phase = format!("measurement-extension-{}", measured_duration_ms / 60_000);
            let extension_worker = spawn_worker(
                &session,
                &worker_path,
                &options,
                pressure_monitor.as_ref(),
                &extension_phase,
            )
            .await?;
            let extension_service = RemoteDodbService::new(extension_worker.connection().clone());
            open_measurement_tenants(&extension_service, &options).await?;
            extension_worker
                .start_measurement()
                .map_err(std::io::Error::other)?;
            let mut extension_options = options.clone();
            extension_options.duration_ms = Some(extension_ms);
            extension_options.operation_count = None;
            extension_options.mutation_key_epoch = options
                .mutation_key_epoch
                .saturating_add(measured_duration_ms / 300_000);
            session.set_phase("checkpoint-cycle-extension");
            let (extension_report, extension_mutations) =
                run_slo_windowed_workload_seeded(
                    &extension_service,
                    &extension_worker,
                    &extension_options,
                    &session,
                    &expected_mutations,
                )
                    .await?;
            extension_worker
                .stop_measurement()
                .map_err(std::io::Error::other)?;
            extension_worker
                .shutdown()
                .map_err(std::io::Error::other)?;
            expected_mutations.extend(extension_mutations);
            measured_duration_ms = measured_duration_ms.saturating_add(extension_ms);
            checkpoint_cycles = successful_checkpoint_cycles(&session);
            checkpoint_extension_reports.push(json!({
                "phase": extension_phase,
                "duration_ms": extension_ms,
                "workload": extension_report,
                "successful_checkpoint_cycles_cumulative": checkpoint_cycles,
            }));
            if extension_report["cell_valid"] != true {
                workload["cell_valid"] = json!(false);
                break;
            }
        }
        workload["checkpoint_cycle_gate_passed"] = json!(checkpoint_cycles >= 20);
        workload["successful_checkpoint_cycles"] = json!(checkpoint_cycles);
        workload["measured_duration_ms_including_extensions"] = json!(measured_duration_ms);
        if checkpoint_cycles < 20 {
            workload["cell_valid"] = json!(false);
            workload["checkpoint_cycle_gate_status"] = json!("failed_at_120_minute_limit");
        } else {
            workload["checkpoint_cycle_gate_status"] = json!("passed");
        }
    }
    phase_reports.insert("measurement".to_owned(), workload.clone());
    if !checkpoint_extension_reports.is_empty() {
        phase_reports.insert(
            "checkpoint_cycle_extensions".to_owned(),
            Value::Array(checkpoint_extension_reports),
        );
    }
    if options.recovery_duration_ms > 0
        && workload["valid_for_oracle"] == true
        && workload["slo_stop_reason"] == "duration_completed"
    {
        let recovery_worker = spawn_worker(&session, &worker_path, &options, pressure_monitor.as_ref(), "recovery").await?;
        let recovery_service = RemoteDodbService::new(recovery_worker.connection().clone());
        open_measurement_tenants(&recovery_service, &options).await?;
        recovery_worker
            .start_measurement()
            .map_err(std::io::Error::other)?;
        let mut recovery_options = options.clone();
        recovery_options.duration_ms = Some(options.recovery_duration_ms);
        recovery_options.operation_count = None;
        recovery_options.mutation_key_epoch = options.mutation_key_epoch.saturating_add(1_000);
        if options.long_reader_retention_ms > 0 {
            recovery_worker
                .hold_reader(1)
                .map_err(std::io::Error::other)?;
        }
        session.set_phase("recovery-offered-load");
        let (recovery_report, recovery_mutations) = run_slo_windowed_workload_seeded(
            &recovery_service,
            &recovery_worker,
            &recovery_options,
            &session,
            &expected_mutations,
        )
        .await?;
        if options.long_reader_retention_ms > 0 {
            recovery_worker
                .release_readers()
                .map_err(std::io::Error::other)?;
        }
        recovery_worker
            .stop_measurement()
            .map_err(std::io::Error::other)?;
        recovery_worker.shutdown().map_err(std::io::Error::other)?;
        workload["cell_valid"] = json!(workload["cell_valid"] == true && recovery_report["cell_valid"] == true);
        workload["valid_for_oracle"] = json!(workload["valid_for_oracle"] == true && recovery_report["valid_for_oracle"] == true);
        for (key, value) in recovery_mutations {
            expected_mutations.insert(key, value);
        }
        phase_reports.insert("recovery_after_reopen".to_owned(), recovery_report);
        workload["phase_workloads"] = Value::Object(phase_reports.clone());
    } else {
        workload["phase_workloads"] = Value::Object(phase_reports.clone());
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
        let oracle_worker = spawn_worker(&session, &worker_path, &options, pressure_monitor.as_ref(), "oracle").await?;
        let oracle_service = RemoteDodbService::new(oracle_worker.connection().clone());
        let oracle = validate_population(&oracle_service, &options, &expected_mutations).await?;
        oracle_worker.shutdown().map_err(std::io::Error::other)?;
        let filesystem_after_oracle = filesystem_snapshot(&options.data_dir);

        let reopened_worker = spawn_worker(&session, &worker_path, &options, pressure_monitor.as_ref(), "reopen-oracle").await?;
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
    let measurement_health = diagnostic["engine_health_reports"]
        .as_array()
        .and_then(|reports| {
            reports
                .iter()
                .find(|report| report["worker_phase"].as_str() == Some("measurement"))
        });
    let paged_resource_report = measurement_health
        .map(|report| &report["blink_paged_resources_before_shutdown"]);
    let paged_route_verified = if options.blink_demand_paging {
        paged_resource_report.is_some_and(|resources| {
            resources["status"].as_str() == Some("sampled")
                && resources["source_manager_totals"]["paged_route_enabled"] == true
                && resources["source_manager_totals"]["loaded_paged_tenant_count"].as_u64()
                    == Some((options.active_tenants + options.opened_idle_tenants) as u64)
                && resources["applied_paging_config"]["paged_writer_enabled"] == true
                && resources["applied_paging_config"]["page_loader"]["maximum_cache_bytes"]
                    .as_u64()
                    == Some(options.blink_paged_cache_bytes)
                && resources["format_identities"]["version_index"] == "DDBVIDX1/v2"
                && resources["format_identities"]["delta_chain_sidecar"] == "DDBVCHN1/v1"
        })
    } else {
        paged_resource_report.is_none_or(|resources| resources["status"].as_str() == Some("resident"))
    };
    cell_is_valid &= paged_route_verified;
    let long_reader_retention = validate_long_reader_retention(
        &diagnostic,
        options.long_reader_retention_ms,
        options.backend == config::Backend::Blink,
    );
    cell_is_valid &= long_reader_retention["passed"] == true;
    let measurement_process_summary = resources::summarize_measurement_processes(
        &diagnostic,
        &offer_process_before,
        &offer_process_after,
    );
    let measurement_resource_report = measurement_resource_samples(&diagnostic);
    Ok(json!({
        "status": if cell_is_valid { "completed" } else { "inconclusive" },
        "seed_manifest_sha256": seed_manifest_sha256,
        "requested_source_revision": source_revision,
        "compiled_source_revision": compiled_source_revision,
        "effective_configuration": effective_configuration(&options),
        "measurement_worker_epoch": "measurement worker only; opened-idle tenants received Get; registered minus active and opened-idle were never opened in this epoch",
        "measurement_paged_route_verification": {
            "requested": options.blink_demand_paging,
            "verified": paged_route_verified,
            "measurement_worker_snapshot": paged_resource_report.cloned().unwrap_or(Value::Null),
        },
        "long_reader_retention": long_reader_retention,
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
        "measurement_periodic_resources": measurement_resource_report,
        "retained_heap_bytes": measurement_resource_report["allocator_retained"].clone(),
        "filesystem_write_amplification": {"value": null, "status": "unsupported", "reason": "allocated blocks are not device or ZFS write amplification"}
    }))
    }
    .await;
    let pressure_report = pressure_monitor
        .as_mut()
        .map_or_else(|| json!({"enabled": false}), |monitor| monitor.finish());
    match execution {
        Ok(mut report) => {
            report["backend_memory_pressure"] = pressure_report;
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
                "effective_configuration": effective_configuration(&options),
                "backend_memory_pressure": pressure_report,
                "memory_diagnostic": session.finish_sampling(),
                "partial_workload_counters": workload_failure::failure_metadata(retained_workload)
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
    pressure_monitor: Option<&dodb_rocksdb::memory_pressure::PressureMonitor>,
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
        "--memory-worker-phase".to_owned(),
        phase.to_owned(),
    ];
    if let Some(monitor) = pressure_monitor {
        worker_arguments.extend(
            monitor
                .worker_arguments(phase)
                .map_err(std::io::Error::other)?,
        );
    }
    if let Some(resources) = options.shared_resources {
        worker_arguments.extend(resources.arguments());
    }
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
        if options.blink_demand_paging {
            worker_arguments.extend([
                "--memory-blink-demand-paging".to_owned(),
                "on".to_owned(),
                "--memory-blink-paged-cache-bytes".to_owned(),
                options.blink_paged_cache_bytes.to_string(),
                "--memory-blink-memory-limit-bytes".to_owned(),
                options.blink_memory_limit_bytes.to_string(),
                "--memory-blink-memory-per-tenant-bytes".to_owned(),
                options.blink_memory_per_tenant_bytes.to_string(),
                "--memory-blink-disk-limit-bytes".to_owned(),
                options.blink_disk_limit_bytes.to_string(),
                "--memory-blink-disk-per-tenant-bytes".to_owned(),
                options.blink_disk_per_tenant_bytes.to_string(),
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
) -> Result<String, Box<dyn std::error::Error>> {
    let mut seed_manifest = Sha256::new();
    for tenant_number in 1..=options.registered_tenants {
        let tenant = TenantId::new(tenant_number as u64);
        for first_row in (0..options.initial_rows).step_by(16) {
            let mutations = (first_row..(first_row + 16).min(options.initial_rows))
                .map(|row_number| {
                    let value = seeded_value(options, tenant_number, row_number);
                    seed_manifest.update((tenant_number as u64).to_be_bytes());
                    seed_manifest.update((row_number as u64).to_be_bytes());
                    seed_manifest.update((value.len() as u64).to_be_bytes());
                    seed_manifest.update(&value);
                    TransactionMutation::Put {
                        key: document_key(row_number),
                        value,
                    }
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
    Ok(format!("{:x}", seed_manifest.finalize()))
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

async fn run_workload_seeded(
    service: &RemoteDodbService,
    options: &Options,
    session: &Arc<MemoryDiagnosticSession>,
    initial_expected: &BTreeMap<(u64, usize), Option<Vec<u8>>>,
) -> Result<(Value, BTreeMap<(u64, usize), Option<Vec<u8>>>), Box<dyn std::error::Error>> {
    let scheduler_service = Arc::new(RemoteDodbService::new(service.connection().clone()));
    let callback_session = Arc::clone(session);
    let pressure_root = options
        .memory_budget
        .as_ref()
        .map(|budget| budget.cgroup_root.clone());
    let on_offer_end = move || {
        callback_session.set_phase("measurement-drain");
        let process_after = callback_session.process_boundary_snapshot();
        json!({
            "process_after": process_after,
            "host_device_after": resources::host_device_snapshot(),
            "process_io_after": resources::process_io_from_boundary(&process_after),
            "driver_process_io_after": blink_checkpoint::process_write_io(),
            "backend_memory_pressure_after": pressure_root.as_ref().map(|root| json!({
                "cell": dodb_rocksdb::memory_pressure::group_snapshot(root),
                "measurement": dodb_rocksdb::memory_pressure::group_snapshot(&root.join("measurement"))
            }))
        })
    };
    if options.topology == Topology::FixedPerTenant {
        return scheduler::run_fixed_per_tenant_seeded(
            scheduler_service,
            options,
            initial_expected,
            on_offer_end,
        )
        .await
        .map_err(|error| Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>);
    }
    return scheduler::run_fixed_total(scheduler_service, options, on_offer_end)
        .await
        .map_err(|error| Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>);
}

async fn run_slo_windowed_workload_seeded(
    service: &RemoteDodbService,
    worker: &dodb_rocksdb::memory_diagnostic::MemoryDiagnosticWorker,
    options: &Options,
    session: &Arc<MemoryDiagnosticSession>,
    initial_expected: &BTreeMap<(u64, usize), Option<Vec<u8>>>,
) -> Result<(Value, BTreeMap<(u64, usize), Option<Vec<u8>>>), Box<dyn std::error::Error>> {
    let (Some(total_duration_ms), Some(window_duration_ms)) =
        (options.duration_ms, options.slo_window_ms)
    else {
        return Ok(run_workload_seeded(service, options, session, initial_expected).await?);
    };
    let mut elapsed_ms = 0_u64;
    let mut consecutive_failed_windows = 0_u8;
    let mut windows = Vec::new();
    let mut maintenance_debt_windows = Vec::new();
    let mut expected_mutations = initial_expected.clone();
    let mut final_report = Value::Null;
    let mut stop_reason = "duration_completed";
    while elapsed_ms < total_duration_ms {
        let window_ms = window_duration_ms.min(total_duration_ms - elapsed_ms);
        let mut window_options = options.clone();
        window_options.duration_ms = Some(window_ms);
        window_options.operation_count = None;
        window_options.mutation_key_epoch = options
            .mutation_key_epoch
            .saturating_add(windows.len() as u64);
        let (report, mutations) =
            run_workload_with_stop_monitor(service, &window_options, session, &expected_mutations)
                .await?;
        expected_mutations.extend(mutations);
        let score = score_slo_window(&report);
        let checkpoint_metrics = worker
            .checkpoint_metrics_snapshot()
            .unwrap_or_else(|error| json!({ "status": "unavailable", "reason": error }));
        let current_debt = checkpoint_metrics["checkpoint_debt_bytes_current"].as_u64();
        if let Some(debt) = current_debt {
            maintenance_debt_windows.push(debt);
        }
        consecutive_failed_windows = if score["passed"] == true {
            0
        } else {
            consecutive_failed_windows.saturating_add(1)
        };
        final_report = report.clone();
        windows.push(json!({
            "window_index": windows.len(),
            "start_elapsed_ms": elapsed_ms,
            "duration_ms": window_ms,
            "score": score,
            "checkpoint_metrics": checkpoint_metrics,
            "workload": report,
        }));
        elapsed_ms = elapsed_ms.saturating_add(window_ms);
        if consecutive_failed_windows >= 2 {
            stop_reason = "two_consecutive_failed_slo_windows";
            break;
        }
        if maintenance_debt_windows.len() >= 3 {
            let debt_window_count = maintenance_debt_windows.len();
            if maintenance_debt_windows[debt_window_count - 3]
                < maintenance_debt_windows[debt_window_count - 2]
                && maintenance_debt_windows[debt_window_count - 2]
                    < maintenance_debt_windows[debt_window_count - 1]
            {
                stop_reason = "maintenance_debt_increased_across_three_windows";
                break;
            }
        }
    }
    let every_window_slo_passed = windows
        .iter()
        .all(|window| window["score"]["passed"] == true);
    let every_window_valid = windows
        .iter()
        .all(|window| window["workload"]["cell_valid"] == true);
    final_report["slo_window_duration_ms"] = json!(window_duration_ms);
    final_report["slo_windows"] = Value::Array(windows);
    final_report["slo_stop_reason"] = json!(stop_reason);
    final_report["slo_passed"] = json!(every_window_slo_passed);
    final_report["maintenance_debt_windows"] = json!(maintenance_debt_windows);
    final_report["cell_valid"] = json!(
        final_report["cell_valid"] == true
            && every_window_valid
            && stop_reason == "duration_completed"
    );
    Ok((final_report, expected_mutations))
}

async fn run_workload_with_stop_monitor(
    service: &RemoteDodbService,
    options: &Options,
    session: &Arc<MemoryDiagnosticSession>,
    initial_expected: &BTreeMap<(u64, usize), Option<Vec<u8>>>,
) -> Result<(Value, BTreeMap<(u64, usize), Option<Vec<u8>>>), Box<dyn std::error::Error>> {
    let initial_memory_snapshot = options
        .memory_budget
        .as_ref()
        .map(|budget| dodb_rocksdb::memory_pressure::group_snapshot(&budget.cgroup_root));
    let workload = run_workload_seeded(service, options, session, initial_expected);
    tokio::pin!(workload);
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            result = &mut workload => return Ok(result?),
            _ = interval.tick() => {
                let available_bytes = resources::filesystem_available_bytes(&options.data_dir)
                    .map_err(|error| std::io::Error::other(format!("could not verify live ZFS reserve: {error}")))?;
                if available_bytes < 10 * 1024 * 1024 * 1024 {
                    return Err(std::io::Error::other(format!("live ZFS reserve crossed 10 GiB: {available_bytes} bytes remain")).into());
                }
                if let (Some(budget), Some(initial)) = (&options.memory_budget, &initial_memory_snapshot) {
                    let current = dodb_rocksdb::memory_pressure::group_snapshot(&budget.cgroup_root);
                    if let Some(event) = cgroup_stop_event(initial, &current) {
                        return Err(std::io::Error::other(format!("cgroup stop condition triggered: {event}")).into());
                    }
                }
            }
        }
    }
}

fn cgroup_stop_event(initial: &Value, current: &Value) -> Option<&'static str> {
    for (counter, reason) in [
        ("high", "memory.high stall"),
        ("max", "memory.max limit event"),
        ("oom", "cgroup OOM"),
        ("oom_kill", "cgroup OOM kill"),
    ] {
        let Some(initial_count) = initial["memory.events"][counter].as_u64() else {
            return Some("cgroup event counters unavailable");
        };
        let Some(current_count) = current["memory.events"][counter].as_u64() else {
            return Some("cgroup event counters unavailable");
        };
        if current_count > initial_count {
            return Some(reason);
        }
    }
    None
}

fn score_slo_window(report: &Value) -> Value {
    let mut read_counts = vec![0_u64; 64];
    for operation_name in ["Get", "Query"] {
        if let Some(buckets) =
            report["operation_latency_histograms"][operation_name]["bucket_counts"].as_array()
        {
            for (bucket_index, bucket_count) in buckets.iter().enumerate().take(read_counts.len()) {
                read_counts[bucket_index] =
                    read_counts[bucket_index].saturating_add(bucket_count.as_u64().unwrap_or(0));
            }
        }
    }
    let write_counts = report["operation_latency_histograms"]["Write"]["bucket_counts"]
        .as_array()
        .map(|buckets| {
            buckets
                .iter()
                .map(|count| count.as_u64().unwrap_or(0))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let read_p99_upper_bound_ms = percentile_bucket_ms(&read_counts, 0.99);
    let write_p99_upper_bound_ms = percentile_bucket_ms(&write_counts, 0.99);
    let scheduled = report["scheduled"].as_u64().unwrap_or(0);
    let rejected_or_failed = ["rejected_conflicts", "other_error", "timed_out", "dropped"]
        .iter()
        .map(|field| report[*field].as_u64().unwrap_or(0))
        .sum::<u64>();
    let rejection_rate = if scheduled == 0 {
        1.0
    } else {
        rejected_or_failed as f64 / scheduled as f64
    };
    let passed = read_p99_upper_bound_ms <= 50.0
        && write_p99_upper_bound_ms <= 100.0
        && rejection_rate <= 0.001;
    json!({
        "passed": passed,
        "read_p99_upper_bound_ms": read_p99_upper_bound_ms,
        "write_p99_upper_bound_ms": write_p99_upper_bound_ms,
        "rejection_rate": rejection_rate,
        "rejected_or_failed_requests": rejected_or_failed,
        "scheduled_requests": scheduled
    })
}

fn percentile_bucket_ms(bucket_counts: &[u64], percentile: f64) -> f64 {
    let observation_count = bucket_counts.iter().sum::<u64>();
    if observation_count == 0 {
        return f64::INFINITY;
    }
    let target = (observation_count as f64 * percentile).ceil() as u64;
    let mut cumulative = 0_u64;
    for (bucket_index, bucket_count) in bucket_counts.iter().copied().enumerate() {
        cumulative = cumulative.saturating_add(bucket_count);
        if cumulative >= target {
            return 2_f64.powi(bucket_index.min(63) as i32) / 1_000_000.0;
        }
    }
    f64::INFINITY
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
        "rocksdb_shared_resources": options.shared_resources.map(|resources| resources.report()),
        "backend_memory_budget": options.memory_budget,
        "backend": options.backend.as_str(), "registered_tenants": options.registered_tenants,
        "seed": options.seed, "duration_ms": options.duration_ms, "warmup_duration_ms": options.warmup_duration_ms,
        "recovery_duration_ms": options.recovery_duration_ms, "slo_window_ms": options.slo_window_ms, "operation_count": options.operation_count,
        "long_reader_retention_ms": options.long_reader_retention_ms,
        "mutation_key_mode": options.mutation_key_mode.as_str(),
        "request_timeout_ms": options.request_timeout_ms, "drain_timeout_ms": options.drain_timeout_ms,
        "active_tenants": options.active_tenants, "opened_idle_tenants": options.opened_idle_tenants,
        "measurement_never_opened_tenants": options.registered_tenants - options.active_tenants - options.opened_idle_tenants,
        "client_count": options.client_count,
        "client_count_scope": if options.topology == config::Topology::FixedPerTenant { "per_tenant" } else { "global" },
        "topology": options.topology.as_str(),
        "tenant_arrival_phase_selected": options.tenant_arrival_phase.as_str(),
        "tenant_arrival_phase_applies": options.topology == config::Topology::FixedPerTenant,
        "tenant_arrival_phase": if options.topology == config::Topology::FixedPerTenant { json!(options.tenant_arrival_phase.as_str()) } else { Value::Null },
        "profile": options.profile.as_str(), "profile_weights_get_query_write": options.profile.weights(),
        "width": match options.width { config::Width::Mixed => json!("mixed 80/15/5 for 1/4/16"), config::Width::Fixed(width) => json!(width) }, "query_limit": 16, "primary_key_bytes": 16, "sort_key_bytes": 16,
        "value_bytes": options.value_bytes, "initial_rows_per_tenant": options.initial_rows,
        "queue_bound": options.queue_bound, "queue_bound_scope": "per_client",
        "max_in_flight": options.max_in_flight, "max_in_flight_scope": "per_client",
        "per_tenant_rate": options.per_tenant_rate,
        "blink_layout": options.blink_checkpoint.configuration(),
        "blink_demand_paging_requested": options.blink_demand_paging,
        "blink_paged_cache_bytes_requested": options.blink_demand_paging.then_some(options.blink_paged_cache_bytes),
        "blink_memory_limit_bytes_requested": options.blink_demand_paging.then_some(options.blink_memory_limit_bytes),
        "blink_memory_per_tenant_bytes_requested": options.blink_demand_paging.then_some(options.blink_memory_per_tenant_bytes),
        "blink_disk_limit_bytes_requested": options.blink_demand_paging.then_some(options.blink_disk_limit_bytes),
        "blink_disk_per_tenant_bytes_requested": options.blink_demand_paging.then_some(options.blink_disk_per_tenant_bytes),
        "checkpoint_policy": format!("{:?}", options.checkpoint_policy),
        "minimal_native_revisions": "reserved-ranges", "minimal_native_condition_locks": "exclusive"
    })
}

#[cfg(test)]
mod tests {
    use super::{effective_configuration, score_slo_window, validate_long_reader_retention};
    use crate::config::{
        Backend, MutationKeyMode, Options, Profile, TenantArrivalPhase, Topology, Width,
    };

    fn options(topology: Topology) -> Options {
        Options {
            backend: Backend::Blink,
            shared_resources: None,
            memory_budget: None,
            data_dir: "/tmp/density-effective-config-data".into(),
            report_path: "/tmp/density-effective-config-report.json".into(),
            seed: 1,
            registered_tenants: 4,
            active_tenants: 4,
            opened_idle_tenants: 0,
            client_count: 1,
            duration_ms: Some(1_000),
            warmup_duration_ms: 0,
            recovery_duration_ms: 0,
            slo_window_ms: None,
            long_reader_retention_ms: 0,
            mutation_key_mode: MutationKeyMode::Fixed,
            mutation_key_epoch: 0,
            operation_count: None,
            initial_rows: 256,
            value_bytes: 512,
            topology,
            tenant_arrival_phase: TenantArrivalPhase::Synchronized,
            per_tenant_rate: (topology == Topology::FixedPerTenant).then_some(50),
            queue_bound: 8,
            max_in_flight: 8,
            request_timeout_ms: 1_000,
            drain_timeout_ms: 1_000,
            profile: Profile::Mixed,
            width: Width::Mixed,
            checkpoint_policy: dodb_server::BlinkCheckpointPolicy::Disabled,
            blink_checkpoint: crate::blink_checkpoint::Options::default(),
            blink_demand_paging: false,
            blink_paged_cache_bytes: 2 * 1024 * 1024,
            blink_memory_limit_bytes: 512 * 1024 * 1024,
            blink_memory_per_tenant_bytes: 64 * 1024 * 1024,
            blink_disk_limit_bytes: 4 * 1024 * 1024 * 1024,
            blink_disk_per_tenant_bytes: 512 * 1024 * 1024,
        }
    }

    #[test]
    fn effective_configuration_marks_fixed_total_phase_as_not_applicable() {
        let fixed_total = effective_configuration(&options(Topology::FixedTotal));
        assert_eq!(fixed_total["tenant_arrival_phase_selected"], "synchronized");
        assert_eq!(fixed_total["tenant_arrival_phase_applies"], false);
        assert!(fixed_total["tenant_arrival_phase"].is_null());

        let fixed_per_tenant = effective_configuration(&options(Topology::FixedPerTenant));
        assert_eq!(
            fixed_per_tenant["tenant_arrival_phase_selected"],
            "synchronized"
        );
        assert_eq!(fixed_per_tenant["tenant_arrival_phase_applies"], true);
        assert_eq!(fixed_per_tenant["tenant_arrival_phase"], "synchronized");
    }

    #[test]
    fn slo_window_score_enforces_latency_and_rejection_thresholds() {
        let mut read_buckets = vec![0_u64; 64];
        read_buckets[15] = 100;
        let mut write_buckets = vec![0_u64; 64];
        write_buckets[16] = 100;
        let passing = serde_json::json!({
            "operation_latency_histograms": {
                "Get": { "bucket_counts": read_buckets },
                "Query": { "bucket_counts": vec![0_u64; 64] },
                "Write": { "bucket_counts": write_buckets }
            },
            "scheduled": 1000,
            "rejected_conflicts": 0,
            "other_error": 0,
            "timed_out": 0,
            "dropped": 0
        });
        assert_eq!(score_slo_window(&passing)["passed"], true);

        let mut failing = passing.clone();
        failing["operation_latency_histograms"]["Write"]["bucket_counts"][17] = 1.into();
        assert_eq!(score_slo_window(&failing)["passed"], false);
        failing["operation_latency_histograms"]["Write"]["bucket_counts"][17] = 0.into();
        failing["other_error"] = 2.into();
        assert_eq!(score_slo_window(&failing)["passed"], false);
    }

    #[test]
    fn long_reader_gate_requires_duration_and_backend_specific_overlap() {
        let diagnostic = serde_json::json!({
            "engine_health_reports": [{
                "long_reader_retention": { "events": [{
                    "tenant": 1,
                    "started_unix_ms": 1000,
                    "released_unix_ms": 3000,
                    "retained_ms": 2000
                }] },
                "blink_checkpoint_metrics": { "checkpoint_events": [{
                    "succeeded": true,
                    "started_unix_nanos": 1500000000_u64,
                    "ended_unix_nanos": 2000000000_u64
                }] }
            }]
        });
        assert_eq!(
            validate_long_reader_retention(&diagnostic, 1000, true)["passed"],
            true
        );
        assert_eq!(
            validate_long_reader_retention(&diagnostic, 2001, true)["passed"],
            false
        );
        let no_checkpoint = serde_json::json!({
            "engine_health_reports": [{
                "long_reader_retention": { "events": [{
                    "started_unix_ms": 1000,
                    "released_unix_ms": 3000,
                    "retained_ms": 2000
                }] },
                "blink_checkpoint_metrics": { "checkpoint_events": [] }
            }]
        });
        assert_eq!(
            validate_long_reader_retention(&no_checkpoint, 2000, false)["passed"],
            true
        );
    }
}
