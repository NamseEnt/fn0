use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, Error as CoreError, PrimaryKey, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use serde_json::{Value, json};
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;

use crate::config::Options;

const RESPONSE_BUDGET_BYTES: usize = 64 * 1024 * 1024;
type ExpectedMutations = BTreeMap<(u64, usize), Option<Vec<u8>>>;
type WorkloadResult = Result<(Value, ExpectedMutations), String>;

struct Arrival {
    operation_number: u64,
    scheduled_at: Instant,
}

#[derive(Clone)]
struct Counters {
    scheduled: u64,
    admitted: u64,
    dropped: u64,
    completed: u64,
    successful: u64,
    rejected_conflicts: u64,
    errors: u64,
    timeouts: u64,
    queued: u64,
    in_flight: u64,
    in_flight_high_water: u64,
    queue_high_water: u64,
    operation_counts: [u64; 3],
    transaction_counts: [u64; 5],
    width_counts: [u64; 3],
    latency_histogram: Vec<u64>,
    maximum_latency_nanos: u64,
    operation_latency_histograms: [[u64; 64]; 3],
    operation_maximum_latency_nanos: [u64; 3],
    request_count: u64,
    request_fingerprint: u64,
    scheduled_fingerprint: u64,
    expected: BTreeMap<(usize, usize), Option<Vec<u8>>>,
    uncertain_mutation: bool,
}

impl Default for Counters {
    fn default() -> Self {
        Self {
            scheduled: 0,
            admitted: 0,
            dropped: 0,
            completed: 0,
            successful: 0,
            rejected_conflicts: 0,
            errors: 0,
            timeouts: 0,
            queued: 0,
            in_flight: 0,
            in_flight_high_water: 0,
            queue_high_water: 0,
            operation_counts: [0; 3],
            transaction_counts: [0; 5],
            width_counts: [0; 3],
            latency_histogram: vec![0; 64],
            maximum_latency_nanos: 0,
            operation_latency_histograms: [[0; 64]; 3],
            operation_maximum_latency_nanos: [0; 3],
            request_count: 0,
            request_fingerprint: 0xcbf2_9ce4_8422_2325,
            scheduled_fingerprint: 0xcbf2_9ce4_8422_2325,
            expected: BTreeMap::new(),
            uncertain_mutation: false,
        }
    }
}

struct PreparedRequest {
    request: Request,
    acknowledgements: Vec<(usize, Option<Vec<u8>>)>,
    operation_kind: usize,
    width: usize,
    deletes: u64,
    conditional: bool,
    fingerprint: u64,
}

#[derive(Clone, Copy)]
struct RequestAddress {
    tenant_number: usize,
    operation_number: u64,
    selection_offset: u64,
    write_ordinal: u64,
    mutation_ordinal: u64,
    lane_start: usize,
}

struct RequestFingerprintContext {
    address: RequestAddress,
    operation_kind: usize,
    width: usize,
    conditional: bool,
    deletes: u64,
    read_row: Option<usize>,
}

fn roll_fingerprint(current: u64, value: u64) -> u64 {
    value.to_be_bytes().iter().fold(current, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

struct TenantInFlightGuard {
    counts: Arc<Mutex<BTreeMap<usize, u64>>>,
    tenant_number: usize,
}

impl Drop for TenantInFlightGuard {
    fn drop(&mut self) {
        if let Ok(mut counts) = self.counts.lock()
            && let Some(count) = counts.get_mut(&self.tenant_number)
        {
            *count = count.saturating_sub(1);
        }
    }
}

pub async fn run_fixed_per_tenant<S, F>(
    service: Arc<S>,
    options: &Options,
    on_offer_end: F,
) -> WorkloadResult
where
    S: DodbService + 'static,
    F: Fn() -> Value + Send + Sync + 'static,
{
    let started = Instant::now();
    let semaphore = Arc::new(Semaphore::new(options.max_in_flight));
    let observed_in_flight = Arc::new(AtomicU64::new(0));
    let mut tasks = JoinSet::new();
    let mut producers = JoinSet::new();
    let mut tenant_counter_handles = Vec::new();
    for tenant_number in 1..=options.active_tenants {
        let (sender, mut receiver) = mpsc::channel::<Arrival>(options.queue_bound);
        let counters = Arc::new(Mutex::new(Counters::default()));
        tenant_counter_handles.push((tenant_number, Arc::clone(&counters)));
        let producer_counters = Arc::clone(&counters);
        let queue_bound = options.queue_bound as u64;
        let duration_deadline = options
            .duration_ms
            .map(|duration| started + Duration::from_millis(duration));
        let operation_limit = options.operation_count.map(|limit| {
            let tenant_offset = (tenant_number - 1) as u64;
            if limit <= tenant_offset {
                0
            } else {
                (limit - tenant_offset).div_ceil(options.active_tenants as u64)
            }
        });
        let rate = options
            .per_tenant_rate
            .ok_or_else(|| "fixed-per-tenant topology requires a per-tenant rate".to_owned())?;
        producers.spawn(async move {
            let mut operation_number = 0_u64;
            loop {
                if operation_limit.is_some_and(|limit| operation_number >= limit) {
                    break;
                }
                let deadline_nanos =
                    u128::from(operation_number) * 1_000_000_000 / u128::from(rate);
                let Ok(deadline_nanos) = u64::try_from(deadline_nanos) else {
                    break;
                };
                let Some(scheduled_at) = started.checked_add(Duration::from_nanos(deadline_nanos))
                else {
                    break;
                };
                if duration_deadline.is_some_and(|deadline| scheduled_at >= deadline) {
                    break;
                }
                tokio::time::sleep_until(tokio::time::Instant::from_std(scheduled_at)).await;
                if let Ok(mut state) = producer_counters.lock() {
                    state.scheduled += 1;
                    state.scheduled_fingerprint =
                        roll_fingerprint(state.scheduled_fingerprint, operation_number);
                    if state.queued >= queue_bound {
                        state.dropped += 1;
                        state.scheduled_fingerprint =
                            roll_fingerprint(state.scheduled_fingerprint, 0);
                        operation_number += 1;
                        continue;
                    }
                    match sender.try_send(Arrival {
                        operation_number,
                        scheduled_at,
                    }) {
                        Ok(()) => {
                            state.admitted += 1;
                            state.queued += 1;
                            state.queue_high_water = state.queue_high_water.max(state.queued);
                            state.scheduled_fingerprint =
                                roll_fingerprint(state.scheduled_fingerprint, 1);
                        }
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            state.dropped += 1;
                            state.scheduled_fingerprint =
                                roll_fingerprint(state.scheduled_fingerprint, 0);
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            state.dropped += 1;
                            state.scheduled_fingerprint =
                                roll_fingerprint(state.scheduled_fingerprint, 0);
                            break;
                        }
                    }
                }
                operation_number += 1;
            }
            if let Some(deadline) = duration_deadline {
                tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
            }
        });
        let tenant_service = Arc::clone(&service);
        let consumer_counters = Arc::clone(&counters);
        let tenant_semaphore = Arc::clone(&semaphore);
        let tenant_observed_in_flight = Arc::clone(&observed_in_flight);
        let max_in_flight = options.max_in_flight as u64;
        let tenant_options = options.clone();
        tasks.spawn(async move {
            let mut expected = BTreeMap::new();
            let mut write_ordinal = 0_u64;
            let mut mutation_ordinal = 0_u64;
            while let Some(arrival) = receiver.recv().await {
                let prepared = prepare_request(
                    &tenant_options,
                    RequestAddress {
                        tenant_number,
                        operation_number: arrival.operation_number,
                        selection_offset: 0,
                        write_ordinal,
                        mutation_ordinal,
                        lane_start: 0,
                    },
                    &expected,
                );
                if prepared.operation_kind == 2 {
                    write_ordinal += 1;
                    mutation_ordinal += prepared.width as u64;
                }
                if let Ok(mut state) = consumer_counters.lock() {
                    state.request_count += 1;
                    state.request_fingerprint =
                        roll_fingerprint(state.request_fingerprint, prepared.fingerprint);
                }
                let is_mutation = prepared.request.is_mutation();
                let permit = tenant_semaphore
                    .acquire()
                    .await
                    .map_err(|error| error.to_string())?;
                let in_flight = max_in_flight - tenant_semaphore.available_permits() as u64;
                tenant_observed_in_flight.fetch_max(in_flight, Ordering::Relaxed);
                if let Ok(mut state) = consumer_counters.lock() {
                    state.queued = state.queued.saturating_sub(1);
                    state.in_flight += 1;
                    state.in_flight_high_water = state.in_flight_high_water.max(state.in_flight);
                }
                let response = tokio::time::timeout(
                    Duration::from_millis(tenant_options.request_timeout_ms),
                    tenant_service.execute(
                        TenantId::new(tenant_number as u64),
                        prepared.request,
                        ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
                    ),
                )
                .await;
                let mut uncertain = false;
                match response {
                    Ok(Ok(response))
                        if valid_response(
                            &response,
                            prepared.operation_kind,
                            &tenant_options,
                            tenant_number,
                            arrival.operation_number,
                        ) =>
                    {
                        for (row_number, value) in prepared.acknowledgements {
                            expected.insert((tenant_number, row_number), value);
                        }
                        if let Ok(mut state) = consumer_counters.lock() {
                            state.successful += 1;
                        }
                    }
                    Ok(Ok(_)) => {
                        uncertain = is_mutation;
                        if let Ok(mut state) = consumer_counters.lock() {
                            state.errors += 1;
                        }
                    }
                    Ok(Err(CoreError::Conflict(_))) => {
                        if let Ok(mut state) = consumer_counters.lock() {
                            state.rejected_conflicts += 1;
                        }
                    }
                    Ok(Err(_)) => {
                        uncertain = is_mutation;
                        if let Ok(mut state) = consumer_counters.lock() {
                            state.errors += 1;
                        }
                    }
                    Err(_) => {
                        uncertain = is_mutation;
                        if let Ok(mut state) = consumer_counters.lock() {
                            state.timeouts += 1;
                        }
                    }
                }
                let elapsed_nanos = Instant::now()
                    .saturating_duration_since(arrival.scheduled_at)
                    .as_nanos()
                    .min(u64::MAX as u128) as u64;
                let bucket = if elapsed_nanos == 0 {
                    0
                } else {
                    (u64::BITS - elapsed_nanos.leading_zeros()).min(63) as usize
                };
                if let Ok(mut state) = consumer_counters.lock() {
                    state.completed += 1;
                    state.in_flight = state.in_flight.saturating_sub(1);
                    state.latency_histogram[bucket] += 1;
                    state.maximum_latency_nanos = state.maximum_latency_nanos.max(elapsed_nanos);
                    state.operation_counts[prepared.operation_kind] += 1;
                    state.operation_latency_histograms[prepared.operation_kind][bucket] += 1;
                    state.operation_maximum_latency_nanos[prepared.operation_kind] = state
                        .operation_maximum_latency_nanos[prepared.operation_kind]
                        .max(elapsed_nanos);
                    if prepared.operation_kind == 2 {
                        state.transaction_counts[0] += 1;
                        state.transaction_counts[1] += prepared.width as u64;
                        state.transaction_counts[2] += prepared.deletes;
                        state.transaction_counts[if prepared.conditional { 3 } else { 4 }] += 1;
                        if let Some(width_index) = match prepared.width {
                            1 => Some(0),
                            4 => Some(1),
                            16 => Some(2),
                            _ => None,
                        } {
                            state.width_counts[width_index] += 1;
                        }
                    }
                    state.uncertain_mutation |= uncertain;
                }
                drop(permit);
            }
            let mut state = consumer_counters
                .lock()
                .map_err(|_| "tenant counter lock poisoned".to_owned())?
                .clone();
            state.expected = expected;
            Ok::<_, String>((tenant_number, state))
        });
    }
    while let Some(producer_result) = producers.join_next().await {
        if let Err(error) = producer_result {
            abort_and_reap(&mut producers).await;
            abort_and_reap(&mut tasks).await;
            return Err(format!("arrival producer failed: {error}"));
        }
    }
    let offer_boundary_accounting = tenant_counter_handles
        .iter()
        .filter_map(|(tenant_number, counters)| {
            counters.lock().ok().map(|state| {
                json!({
                    "tenant": tenant_number,
                    "scheduled": state.scheduled,
                    "admitted": state.admitted,
                    "dropped": state.dropped,
                    "completed": state.completed,
                    "queued": state.queued,
                    "in_flight": state.in_flight
                })
            })
        })
        .collect::<Vec<_>>();
    let completed_at_offer_end = offer_boundary_accounting
        .iter()
        .filter_map(|tenant| tenant["completed"].as_u64())
        .sum::<u64>();
    let offer_boundary = on_offer_end();
    let offer_window_elapsed_ms = started.elapsed().as_millis();
    let drained = tokio::time::timeout(Duration::from_millis(options.drain_timeout_ms), async {
        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            results.push(result.map_err(|error| error.to_string())??);
        }
        Ok::<_, String>(results)
    })
    .await;
    let mut results = match drained {
        Ok(Ok(results)) => results,
        Ok(Err(error)) => {
            abort_and_reap(&mut producers).await;
            abort_and_reap(&mut tasks).await;
            return Err(error);
        }
        Err(_) => {
            abort_and_reap(&mut producers).await;
            abort_and_reap(&mut tasks).await;
            let mut partial_totals = Counters::default();
            let partial_per_tenant = tenant_counter_handles
                .iter()
                .filter_map(|(tenant_number, counters)| {
                    counters.lock().ok().map(|state| {
                        partial_totals.scheduled += state.scheduled;
                        partial_totals.admitted += state.admitted;
                        partial_totals.dropped += state.dropped;
                        partial_totals.completed += state.completed;
                        partial_totals.successful += state.successful;
                        partial_totals.rejected_conflicts += state.rejected_conflicts;
                        partial_totals.errors += state.errors;
                        partial_totals.timeouts += state.timeouts;
                        json!({"tenant": tenant_number, "scheduled": state.scheduled, "admitted": state.admitted, "dropped": state.dropped, "completed": state.completed, "successful": state.successful, "rejected_conflicts": state.rejected_conflicts, "other_error": state.errors, "timed_out": state.timeouts, "queued": state.queued, "in_flight": state.in_flight, "request_count": state.request_count, "request_fingerprint": format!("fnv1a64:{:016x}", state.request_fingerprint), "scheduled_fingerprint": format!("fnv1a64:{:016x}", state.scheduled_fingerprint)})
                    })
                })
                .collect::<Vec<_>>();
            let partial_report = json!({
                "topology": "fixed-per-tenant",
                "status": "inconclusive_drain_timeout",
                "scheduled": partial_totals.scheduled,
                "admitted": partial_totals.admitted,
                "dropped": partial_totals.dropped,
                "completed": partial_totals.completed,
                "successful": partial_totals.successful,
                "rejected_conflicts": partial_totals.rejected_conflicts,
                "other_error": partial_totals.errors,
                "timed_out": partial_totals.timeouts,
                "valid_for_oracle": false,
                "cell_valid": false,
                "drain_timeout_ms": options.drain_timeout_ms,
                "offer_window_elapsed_ms": offer_window_elapsed_ms,
                "offer_boundary": offer_boundary,
                "offer_boundary_accounting": offer_boundary_accounting,
                "partial_per_tenant": partial_per_tenant,
                "failure_reason": "bounded drain expired; outstanding mutation outcomes are uncertain"
                ,"deterministic_fingerprints": deterministic_fingerprints(options, "fixed-per-tenant")
            });
            return Ok((partial_report, BTreeMap::new()));
        }
    };
    results.sort_by_key(|(tenant_number, _)| *tenant_number);
    let mut totals = Counters::default();
    let mut expected = BTreeMap::new();
    let mut per_tenant = Vec::new();
    for (tenant_number, state) in results {
        totals.scheduled += state.scheduled;
        totals.admitted += state.admitted;
        totals.dropped += state.dropped;
        totals.completed += state.completed;
        totals.successful += state.successful;
        totals.rejected_conflicts += state.rejected_conflicts;
        totals.errors += state.errors;
        totals.timeouts += state.timeouts;
        totals.request_count += state.request_count;
        totals.request_fingerprint =
            roll_fingerprint(totals.request_fingerprint, state.request_fingerprint);
        totals.scheduled_fingerprint =
            roll_fingerprint(totals.scheduled_fingerprint, state.scheduled_fingerprint);
        totals.queue_high_water += state.queue_high_water;
        totals.maximum_latency_nanos = totals
            .maximum_latency_nanos
            .max(state.maximum_latency_nanos);
        for (total, count) in totals
            .operation_counts
            .iter_mut()
            .zip(state.operation_counts)
        {
            *total += count;
        }
        for (total, count) in totals
            .transaction_counts
            .iter_mut()
            .zip(state.transaction_counts)
        {
            *total += count;
        }
        for (total, count) in totals.width_counts.iter_mut().zip(state.width_counts) {
            *total += count;
        }
        for (total, count) in totals
            .latency_histogram
            .iter_mut()
            .zip(&state.latency_histogram)
        {
            *total += count;
        }
        for (total_operation, tenant_operation) in totals
            .operation_latency_histograms
            .iter_mut()
            .zip(state.operation_latency_histograms)
        {
            for (total_bucket, tenant_bucket) in total_operation.iter_mut().zip(tenant_operation) {
                *total_bucket += tenant_bucket;
            }
        }
        for (total_maximum, tenant_maximum) in totals
            .operation_maximum_latency_nanos
            .iter_mut()
            .zip(state.operation_maximum_latency_nanos)
        {
            *total_maximum = (*total_maximum).max(tenant_maximum);
        }
        totals.uncertain_mutation |= state.uncertain_mutation;
        for ((expected_tenant, row_number), value) in &state.expected {
            expected.insert((*expected_tenant as u64, *row_number), value.clone());
        }
        per_tenant.push(json!({
            "tenant": tenant_number,
            "scheduled": state.scheduled,
            "admitted": state.admitted,
            "dropped": state.dropped,
            "completed": state.completed,
            "successful": state.successful,
            "rejected_conflicts": state.rejected_conflicts,
            "other_error": state.errors,
            "timed_out": state.timeouts,
            "queued_at_end": state.queued,
            "in_flight_at_end": state.in_flight,
            "in_flight_high_water": state.in_flight_high_water,
            "queue_high_water": state.queue_high_water,
            "request_count": state.request_count,
            "request_fingerprint": format!("fnv1a64:{:016x}", state.request_fingerprint),
            "scheduled_fingerprint": format!("fnv1a64:{:016x}", state.scheduled_fingerprint),
            "operation_counts": state.operation_counts,
            "transaction_counts": state.transaction_counts,
            "width_counts": state.width_counts,
            "latency_histogram": state.latency_histogram,
            "operation_latency_histograms": operation_latency_json(&state),
            "uncertain_mutation": state.uncertain_mutation
        }));
    }
    for tenant_number in 1..=options.active_tenants {
        if !per_tenant
            .iter()
            .any(|tenant| tenant["tenant"].as_u64() == Some(tenant_number as u64))
        {
            per_tenant.push(json!({
                "tenant": tenant_number,
                "scheduled": 0,
                "admitted": 0,
                "dropped": 0,
                "completed": 0,
                "successful": 0,
                "rejected_conflicts": 0,
                "other_error": 0,
                "timed_out": 0,
                "queued_at_end": 0,
                "in_flight_at_end": 0,
                "in_flight_high_water": 0,
                "queue_high_water": 0,
                "request_count": 0,
                "request_fingerprint": format!("fnv1a64:{:016x}", 0xcbf2_9ce4_8422_2325_u64),
                "scheduled_fingerprint": format!("fnv1a64:{:016x}", 0xcbf2_9ce4_8422_2325_u64),
                "operation_counts": [0, 0, 0],
                "transaction_counts": [0, 0, 0, 0, 0],
                "width_counts": [0, 0, 0],
                "latency_histogram": vec![0; 64],
                "operation_latency_histograms": operation_latency_json(&Counters::default()),
                "uncertain_mutation": false
            }));
        }
    }
    per_tenant.sort_by_key(|tenant| tenant["tenant"].as_u64().unwrap_or(0));
    let report = json!({
        "topology": "fixed-per-tenant",
        "deterministic_fingerprints": deterministic_fingerprints(options, "fixed-per-tenant"),
        "offered_schedule": "one independent absolute-deadline producer and sequential consumer lane per active tenant; bounded nonblocking channel admission",
        "scheduled": totals.scheduled,
        "admitted": totals.admitted,
        "dropped": totals.dropped,
        "completed": totals.completed,
        "completed_at_offer_end": completed_at_offer_end,
        "successful": totals.successful,
        "rejected_conflicts": totals.rejected_conflicts,
        "other_error": totals.errors,
        "timed_out": totals.timeouts,
        "queued_at_end": 0,
        "in_flight_at_end": 0,
        "configured_max_in_flight": options.max_in_flight,
        "observed_max_in_flight": observed_in_flight.load(Ordering::Relaxed),
        "queue_high_water_sum_across_tenants": totals.queue_high_water,
        "request_count": totals.request_count,
        "request_fingerprint_algorithm": "FNV-1a 64-bit over admitted requests in tenant order",
        "request_fingerprint": format!("fnv1a64:{:016x}", totals.request_fingerprint),
        "scheduled_fingerprint": format!("fnv1a64:{:016x}", totals.scheduled_fingerprint),
        "latency_histogram": {"unit": "nanoseconds; logarithmic power-of-two buckets; scheduled-to-completion", "bucket_counts": totals.latency_histogram, "observation_count": totals.completed, "maximum_nanos": totals.maximum_latency_nanos},
        "operation_latency_histograms": operation_latency_json(&totals),
        "latency_scope": "all admitted operations; request queue and service time included",
        "accounting": {"scheduled_equals_admitted_plus_dropped": totals.scheduled == totals.admitted + totals.dropped, "admitted_equals_completed_after_drain": totals.admitted == totals.completed, "completed_equals_success_plus_conflict_plus_error_and_timeout": totals.completed == totals.successful + totals.rejected_conflicts + totals.errors + totals.timeouts, "pending_equals_queued_plus_in_flight_at_offer_end": offer_boundary_accounting.iter().all(|tenant| tenant["queued"].as_u64().unwrap_or(0) + tenant["in_flight"].as_u64().unwrap_or(0) == tenant["admitted"].as_u64().unwrap_or(0).saturating_sub(tenant["completed"].as_u64().unwrap_or(0)) )},
        "valid_for_oracle": !totals.uncertain_mutation,
        "cell_valid": totals.errors == 0 && totals.rejected_conflicts == 0 && totals.timeouts == 0,
        "per_tenant": per_tenant
        ,"offer_window_elapsed_ms": offer_window_elapsed_ms,
        "offer_boundary": offer_boundary
        ,"offer_boundary_accounting": offer_boundary_accounting
    });
    Ok((report, expected))
}

pub async fn run_fixed_total<S, F>(
    service: Arc<S>,
    options: &Options,
    on_offer_end: F,
) -> WorkloadResult
where
    S: DodbService + 'static,
    F: Fn() -> Value + Send + Sync + 'static,
{
    let start = Instant::now();
    let deadline = options
        .duration_ms
        .map(|duration| start + Duration::from_millis(duration));
    let semaphore = Arc::new(Semaphore::new(options.max_in_flight));
    let observed_in_flight = Arc::new(AtomicU64::new(0));
    let boundary_counters = Arc::new(Mutex::new(Counters::default()));
    let tenant_in_flight = Arc::new(Mutex::new(BTreeMap::<usize, u64>::new()));
    let accepting = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let mut tasks = JoinSet::new();
    for client_number in 0..options.client_count {
        let client = Arc::clone(&service);
        let semaphore = Arc::clone(&semaphore);
        let observed = Arc::clone(&observed_in_flight);
        let accepting_lane = Arc::clone(&accepting);
        let client_boundary_counters = Arc::clone(&boundary_counters);
        let tenant_in_flight_counts = Arc::clone(&tenant_in_flight);
        let options = options.clone();
        let operation_limit = options.operation_count.map(|limit| {
            let client_number = client_number as u64;
            if limit <= client_number {
                0
            } else {
                (limit - client_number).div_ceil(options.client_count as u64)
            }
        });
        tasks.spawn(async move {
            let mut state = Counters::default();
            let mut expected = BTreeMap::new();
            let mut write_ordinal = 0_u64;
            let mut mutation_ordinal = 0_u64;
            let mut tenant_counters = BTreeMap::<usize, Counters>::new();
            let mut operation_number = 0_u64;
            while operation_limit.is_none_or(|limit| operation_number < limit)
                && accepting_lane.load(Ordering::Relaxed)
                && deadline.is_none_or(|deadline| Instant::now() < deadline)
            {
                let scheduled_at = Instant::now();
                let tenant_number = operation_number as usize % options.active_tenants + 1;
                state.scheduled += 1;
                state.admitted += 1;
                if let Ok(mut boundary) = client_boundary_counters.lock() {
                    boundary.scheduled += 1;
                    boundary.admitted += 1;
                    boundary.queued += 1;
                }
                let lane_start = client_number * 16;
                let prepared = prepare_request(
                    &options,
                    RequestAddress {
                        tenant_number,
                        operation_number,
                        selection_offset: (client_number as u64 * 13) % 100,
                        write_ordinal,
                        mutation_ordinal,
                        lane_start,
                    },
                    &expected,
                );
                if prepared.operation_kind == 2 {
                    write_ordinal += 1;
                    mutation_ordinal += prepared.width as u64;
                }
                state.request_count += 1;
                state.request_fingerprint =
                    roll_fingerprint(state.request_fingerprint, prepared.fingerprint);
                state.scheduled_fingerprint =
                    roll_fingerprint(state.scheduled_fingerprint, operation_number);
                let tenant_request = tenant_counters.entry(tenant_number).or_default();
                tenant_request.request_count += 1;
                tenant_request.request_fingerprint =
                    roll_fingerprint(tenant_request.request_fingerprint, prepared.fingerprint);
                tenant_request.scheduled_fingerprint =
                    roll_fingerprint(tenant_request.scheduled_fingerprint, operation_number);
                let is_mutation = prepared.request.is_mutation();
                state.queued += 1;
                state.queue_high_water = state.queue_high_water.max(state.queued);
                let permit = semaphore
                    .acquire()
                    .await
                    .map_err(|error| error.to_string())?;
                let in_flight = options.max_in_flight as u64 - semaphore.available_permits() as u64;
                observed.fetch_max(in_flight, Ordering::Relaxed);
                let tenant_in_flight_now = if let Ok(mut counts) = tenant_in_flight_counts.lock() {
                    let count = counts.entry(tenant_number).or_default();
                    *count += 1;
                    *count
                } else {
                    1
                };
                let tenant_in_flight_guard = TenantInFlightGuard {
                    counts: Arc::clone(&tenant_in_flight_counts),
                    tenant_number,
                };
                state.in_flight += 1;
                state.queued = state.queued.saturating_sub(1);
                state.in_flight_high_water = state.in_flight_high_water.max(state.in_flight);
                if let Ok(mut boundary) = client_boundary_counters.lock() {
                    boundary.queued = boundary.queued.saturating_sub(1);
                    boundary.in_flight += 1;
                }
                let response = tokio::time::timeout(
                    Duration::from_millis(options.request_timeout_ms),
                    client.execute(
                        TenantId::new(tenant_number as u64),
                        prepared.request,
                        ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
                    ),
                )
                .await;
                let mut uncertain = false;
                let mut succeeded = false;
                let mut failed = false;
                let mut timed_out = false;
                let mut rejected_conflict = false;
                match response {
                    Ok(Ok(response))
                        if valid_response(
                            &response,
                            prepared.operation_kind,
                            &options,
                            tenant_number,
                            operation_number,
                        ) =>
                    {
                        succeeded = true;
                        for (row_number, value) in prepared.acknowledgements {
                            expected.insert((tenant_number, row_number), value);
                        }
                        state.successful += 1;
                    }
                    Ok(Ok(_)) => {
                        state.errors += 1;
                        failed = true;
                        uncertain = is_mutation;
                    }
                    Ok(Err(CoreError::Conflict(_))) => {
                        state.rejected_conflicts += 1;
                        rejected_conflict = true;
                    }
                    Ok(Err(_)) => {
                        state.errors += 1;
                        failed = true;
                        uncertain = is_mutation;
                    }
                    Err(_) => {
                        state.timeouts += 1;
                        timed_out = true;
                        uncertain = is_mutation;
                    }
                }
                if let Ok(mut boundary) = client_boundary_counters.lock() {
                    boundary.completed += 1;
                    boundary.successful += u64::from(succeeded);
                    boundary.rejected_conflicts += u64::from(rejected_conflict);
                    boundary.errors += u64::from(failed);
                    boundary.timeouts += u64::from(timed_out);
                    boundary.in_flight = boundary.in_flight.saturating_sub(1);
                }
                let elapsed = Instant::now()
                    .saturating_duration_since(scheduled_at)
                    .as_nanos()
                    .min(u64::MAX as u128) as u64;
                let bucket = if elapsed == 0 {
                    0
                } else {
                    (u64::BITS - elapsed.leading_zeros()).min(63) as usize
                };
                state.latency_histogram[bucket] += 1;
                state.maximum_latency_nanos = state.maximum_latency_nanos.max(elapsed);
                state.operation_latency_histograms[prepared.operation_kind][bucket] += 1;
                state.operation_maximum_latency_nanos[prepared.operation_kind] =
                    state.operation_maximum_latency_nanos[prepared.operation_kind].max(elapsed);
                state.completed += 1;
                state.in_flight = state.in_flight.saturating_sub(1);
                state.operation_counts[prepared.operation_kind] += 1;
                let tenant_state = tenant_counters.entry(tenant_number).or_default();
                tenant_state.scheduled += 1;
                tenant_state.admitted += 1;
                tenant_state.completed += 1;
                tenant_state.successful += u64::from(succeeded);
                tenant_state.rejected_conflicts += u64::from(rejected_conflict);
                tenant_state.errors += u64::from(failed);
                tenant_state.timeouts += u64::from(timed_out);
                tenant_state.operation_counts[prepared.operation_kind] += 1;
                tenant_state.latency_histogram[bucket] += 1;
                tenant_state.maximum_latency_nanos =
                    tenant_state.maximum_latency_nanos.max(elapsed);
                tenant_state.operation_latency_histograms[prepared.operation_kind][bucket] += 1;
                tenant_state.operation_maximum_latency_nanos[prepared.operation_kind] =
                    tenant_state.operation_maximum_latency_nanos[prepared.operation_kind]
                        .max(elapsed);
                tenant_state.in_flight_high_water =
                    tenant_state.in_flight_high_water.max(tenant_in_flight_now);
                tenant_state.uncertain_mutation |= uncertain;
                if prepared.operation_kind == 2 {
                    state.transaction_counts[0] += 1;
                    state.transaction_counts[1] += prepared.width as u64;
                    state.transaction_counts[2] += prepared.deletes;
                    state.transaction_counts[if prepared.conditional { 3 } else { 4 }] += 1;
                    if let Some(index) = match prepared.width {
                        1 => Some(0),
                        4 => Some(1),
                        16 => Some(2),
                        _ => None,
                    } {
                        state.width_counts[index] += 1;
                    }
                    tenant_state.transaction_counts[0] += 1;
                    tenant_state.transaction_counts[1] += prepared.width as u64;
                    tenant_state.transaction_counts[2] += prepared.deletes;
                    tenant_state.transaction_counts[if prepared.conditional { 3 } else { 4 }] += 1;
                    if let Some(width_index) = match prepared.width {
                        1 => Some(0),
                        4 => Some(1),
                        16 => Some(2),
                        _ => None,
                    } {
                        tenant_state.width_counts[width_index] += 1;
                    }
                }
                state.uncertain_mutation |= uncertain;
                drop(permit);
                drop(tenant_in_flight_guard);
                operation_number += 1;
            }
            state.expected = expected;
            Ok::<_, String>((client_number, state, tenant_counters))
        });
    }
    let results = if let Some(deadline) = deadline {
        tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
        accepting.store(false, Ordering::Relaxed);
        let boundary_accounting = boundary_counters
            .lock()
            .map(|state| {
                json!({
                    "scheduled": state.scheduled,
                    "admitted": state.admitted,
                    "completed": state.completed,
                    "successful": state.successful,
                    "rejected_conflicts": state.rejected_conflicts,
                    "other_error": state.errors,
                    "timed_out": state.timeouts,
                    "queued": state.queued,
                    "in_flight": state.in_flight
                })
            })
            .unwrap_or_else(
                |_| json!({"status": "unavailable", "reason": "counter lock poisoned"}),
            );
        let offer_boundary = on_offer_end();
        let offer_window_elapsed_ms = start.elapsed().as_millis();
        let drained =
            tokio::time::timeout(Duration::from_millis(options.drain_timeout_ms), async {
                let mut results = Vec::new();
                while let Some(result) = tasks.join_next().await {
                    results.push(result.map_err(|error| error.to_string())??);
                }
                Ok::<_, String>(results)
            })
            .await;
        let results = match drained {
            Ok(Ok(results)) => results,
            Ok(Err(error)) => {
                abort_and_reap(&mut tasks).await;
                return Err(error);
            }
            Err(_) => {
                abort_and_reap(&mut tasks).await;
                let partial_counters = boundary_counters
                    .lock()
                    .map(|state| json!({"scheduled": state.scheduled, "admitted": state.admitted, "completed": state.completed, "successful": state.successful, "other_error": state.errors, "timed_out": state.timeouts, "queued": state.queued, "in_flight": state.in_flight}))
                    .unwrap_or_else(|_| json!({"status": "unavailable", "reason": "counter lock poisoned"}));
                let partial_report = json!({
                    "topology": "fixed-total",
                    "status": "inconclusive_drain_timeout",
                    "partial_counters": partial_counters,
                    "valid_for_oracle": false,
                    "cell_valid": false,
                    "drain_timeout_ms": options.drain_timeout_ms,
                    "offer_window_elapsed_ms": offer_window_elapsed_ms,
                    "offer_boundary": offer_boundary,
                    "offer_boundary_accounting": boundary_accounting,
                    "failure_reason": "bounded drain expired; outstanding mutation outcomes are uncertain"
                    ,"deterministic_fingerprints": deterministic_fingerprints(options, "fixed-total")
                });
                return Ok((partial_report, BTreeMap::new()));
            }
        };
        (
            results,
            offer_boundary,
            boundary_accounting,
            offer_window_elapsed_ms,
        )
    } else {
        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            results.push(result.map_err(|error| error.to_string())??);
        }
        let boundary_accounting = boundary_counters
            .lock()
            .map(|state| {
                json!({
                    "scheduled": state.scheduled,
                    "admitted": state.admitted,
                    "completed": state.completed,
                    "queued": state.queued,
                    "in_flight": state.in_flight
                })
            })
            .unwrap_or_else(
                |_| json!({"status": "unavailable", "reason": "counter lock poisoned"}),
            );
        let offer_boundary = on_offer_end();
        (
            results,
            offer_boundary,
            boundary_accounting,
            start.elapsed().as_millis(),
        )
    };
    let (mut results, offer_boundary, offer_boundary_accounting, offer_window_elapsed_ms) = results;
    results.sort_by_key(|(client_number, _, _)| *client_number);
    let mut total = Counters::default();
    let mut expected = BTreeMap::new();
    let mut per_client = Vec::new();
    let mut per_tenant_counters = (1..=options.active_tenants)
        .map(|tenant_number| (tenant_number, Counters::default()))
        .collect::<BTreeMap<_, _>>();
    for (client_number, state, tenant_counters) in results {
        total.scheduled += state.scheduled;
        total.admitted += state.admitted;
        total.completed += state.completed;
        total.successful += state.successful;
        total.rejected_conflicts += state.rejected_conflicts;
        total.errors += state.errors;
        total.timeouts += state.timeouts;
        total.request_count += state.request_count;
        total.request_fingerprint =
            roll_fingerprint(total.request_fingerprint, state.request_fingerprint);
        total.scheduled_fingerprint =
            roll_fingerprint(total.scheduled_fingerprint, state.scheduled_fingerprint);
        total.maximum_latency_nanos = total.maximum_latency_nanos.max(state.maximum_latency_nanos);
        for (total, count) in total
            .operation_counts
            .iter_mut()
            .zip(state.operation_counts)
        {
            *total += count;
        }
        for (total, count) in total
            .transaction_counts
            .iter_mut()
            .zip(state.transaction_counts)
        {
            *total += count;
        }
        for (total, count) in total.width_counts.iter_mut().zip(state.width_counts) {
            *total += count;
        }
        for (total, count) in total
            .latency_histogram
            .iter_mut()
            .zip(&state.latency_histogram)
        {
            *total += count;
        }
        for (total_operation, lane_operation) in total
            .operation_latency_histograms
            .iter_mut()
            .zip(state.operation_latency_histograms)
        {
            for (total_bucket, lane_bucket) in total_operation.iter_mut().zip(lane_operation) {
                *total_bucket += lane_bucket;
            }
        }
        for (total_maximum, lane_maximum) in total
            .operation_maximum_latency_nanos
            .iter_mut()
            .zip(state.operation_maximum_latency_nanos)
        {
            *total_maximum = (*total_maximum).max(lane_maximum);
        }
        total.uncertain_mutation |= state.uncertain_mutation;
        for (tenant_number, tenant_state) in tenant_counters {
            let aggregate = per_tenant_counters.entry(tenant_number).or_default();
            aggregate.scheduled += tenant_state.scheduled;
            aggregate.admitted += tenant_state.admitted;
            aggregate.completed += tenant_state.completed;
            aggregate.successful += tenant_state.successful;
            aggregate.rejected_conflicts += tenant_state.rejected_conflicts;
            aggregate.errors += tenant_state.errors;
            aggregate.timeouts += tenant_state.timeouts;
            aggregate.request_count += tenant_state.request_count;
            aggregate.request_fingerprint = roll_fingerprint(
                aggregate.request_fingerprint,
                tenant_state.request_fingerprint,
            );
            aggregate.scheduled_fingerprint = roll_fingerprint(
                aggregate.scheduled_fingerprint,
                tenant_state.scheduled_fingerprint,
            );
            aggregate
                .operation_counts
                .iter_mut()
                .zip(tenant_state.operation_counts)
                .for_each(|(total, count)| *total += count);
            aggregate
                .latency_histogram
                .iter_mut()
                .zip(tenant_state.latency_histogram)
                .for_each(|(total, count)| *total += count);
            for (total_operation, lane_operation) in aggregate
                .operation_latency_histograms
                .iter_mut()
                .zip(tenant_state.operation_latency_histograms)
            {
                for (total_bucket, lane_bucket) in total_operation.iter_mut().zip(lane_operation) {
                    *total_bucket += lane_bucket;
                }
            }
            for (total_maximum, lane_maximum) in aggregate
                .operation_maximum_latency_nanos
                .iter_mut()
                .zip(tenant_state.operation_maximum_latency_nanos)
            {
                *total_maximum = (*total_maximum).max(lane_maximum);
            }
            aggregate
                .transaction_counts
                .iter_mut()
                .zip(tenant_state.transaction_counts)
                .for_each(|(total, count)| *total += count);
            aggregate
                .width_counts
                .iter_mut()
                .zip(tenant_state.width_counts)
                .for_each(|(total, count)| *total += count);
            aggregate.maximum_latency_nanos = aggregate
                .maximum_latency_nanos
                .max(tenant_state.maximum_latency_nanos);
            aggregate.in_flight_high_water = aggregate
                .in_flight_high_water
                .max(tenant_state.in_flight_high_water);
        }
        for ((tenant_number, row_number), value) in &state.expected {
            expected.insert((*tenant_number as u64, *row_number), value.clone());
        }
        per_client.push(json!({"client_lane": client_number, "scheduled": state.scheduled, "admitted": state.admitted, "completed": state.completed, "successful": state.successful, "rejected_conflicts": state.rejected_conflicts, "other_error": state.errors, "timed_out": state.timeouts, "in_flight_high_water": state.in_flight_high_water, "request_count": state.request_count, "request_fingerprint": format!("fnv1a64:{:016x}", state.request_fingerprint), "scheduled_fingerprint": format!("fnv1a64:{:016x}", state.scheduled_fingerprint), "operation_counts": state.operation_counts, "transaction_counts": state.transaction_counts, "width_counts": state.width_counts}));
    }
    let per_tenant = per_tenant_counters.into_iter().map(|(tenant_number, state)| json!({"tenant": tenant_number, "scheduled": state.scheduled, "admitted": state.admitted, "completed": state.completed, "successful": state.successful, "rejected_conflicts": state.rejected_conflicts, "other_error": state.errors, "timed_out": state.timeouts, "request_count": state.request_count, "request_fingerprint": format!("fnv1a64:{:016x}", state.request_fingerprint), "scheduled_fingerprint": format!("fnv1a64:{:016x}", state.scheduled_fingerprint), "operation_counts": state.operation_counts, "transaction_counts": state.transaction_counts, "width_counts": state.width_counts, "latency_histogram": state.latency_histogram, "operation_latency_histograms": operation_latency_json(&state), "maximum_latency_nanos": state.maximum_latency_nanos, "in_flight_high_water": state.in_flight_high_water})).collect::<Vec<_>>();
    let report = json!({
        "topology": "fixed-total",
        "deterministic_fingerprints": deterministic_fingerprints(options, "fixed-total"),
        "client_lane_execution": "one sequential request stream per client; streams execute concurrently and multiplex tenants round-robin; each lane owns 16 distinct mutation keys",
        "scheduled": total.scheduled,
        "admitted": total.admitted,
        "completed": total.completed,
        "completed_at_offer_end": offer_boundary_accounting["completed"],
        "successful": total.successful,
        "rejected_conflicts": total.rejected_conflicts,
        "request_count": total.request_count,
        "request_fingerprint_algorithm": "FNV-1a 64-bit over admitted requests in client-lane order",
        "request_fingerprint": format!("fnv1a64:{:016x}", total.request_fingerprint),
        "scheduled_fingerprint": format!("fnv1a64:{:016x}", total.scheduled_fingerprint),
        "other_error": total.errors,
        "timed_out": total.timeouts,
        "dropped": 0,
        "queued_at_end": 0,
        "in_flight_at_end": 0,
        "configured_max_in_flight": options.max_in_flight,
        "observed_max_in_flight": observed_in_flight.load(Ordering::Relaxed),
        "latency_histogram": {"unit": "nanoseconds; logarithmic power-of-two buckets", "bucket_counts": total.latency_histogram, "observation_count": total.completed, "maximum_nanos": total.maximum_latency_nanos},
        "operation_latency_histograms": operation_latency_json(&total),
        "latency_scope": "all admitted operations; semaphore wait and service time included",
        "operation_counts": total.operation_counts,
        "transaction_counts": {"transactions": total.transaction_counts[0], "mutations": total.transaction_counts[1], "delete_mutations": total.transaction_counts[2], "conditional_transactions": total.transaction_counts[3], "unconditional_transactions": total.transaction_counts[4]},
        "write_width_counts": total.width_counts,
        "accounting": {"scheduled_equals_admitted_plus_dropped": total.scheduled == total.admitted, "admitted_equals_completed_after_drain": total.admitted == total.completed, "completed_equals_success_plus_conflict_plus_error_and_timeout": total.completed == total.successful + total.rejected_conflicts + total.errors + total.timeouts, "pending_equals_queued_plus_in_flight_at_offer_end": offer_boundary_accounting["queued"].as_u64().unwrap_or(0) + offer_boundary_accounting["in_flight"].as_u64().unwrap_or(0) == offer_boundary_accounting["admitted"].as_u64().unwrap_or(0).saturating_sub(offer_boundary_accounting["completed"].as_u64().unwrap_or(0))},
        "valid_for_oracle": !total.uncertain_mutation,
        "cell_valid": total.errors == 0 && total.rejected_conflicts == 0 && total.timeouts == 0,
        "per_client_lane": per_client,
        "per_tenant": per_tenant,
        "offer_window_elapsed_ms": offer_window_elapsed_ms,
        "offer_boundary": offer_boundary
        ,"offer_boundary_accounting": offer_boundary_accounting
    });
    Ok((report, expected))
}

fn prepare_request(
    options: &Options,
    address: RequestAddress,
    expected: &BTreeMap<(usize, usize), Option<Vec<u8>>>,
) -> PreparedRequest {
    let RequestAddress {
        tenant_number,
        operation_number,
        selection_offset,
        write_ordinal,
        mutation_ordinal,
        lane_start,
    } = address;
    let selector = operation_number
        .wrapping_mul(37)
        .wrapping_add(selection_offset)
        % 100;
    let profile = options.profile.weights();
    let read_row = seeded_read_row(options, tenant_number, operation_number);
    let first_key = document_key(read_row);
    if selector < u64::from(profile[0]) {
        return PreparedRequest {
            request: Request::Get { key: first_key },
            acknowledgements: Vec::new(),
            operation_kind: 0,
            width: 0,
            deletes: 0,
            conditional: false,
            fingerprint: request_fingerprint(
                options,
                RequestFingerprintContext {
                    address,
                    operation_kind: 0,
                    width: 0,
                    conditional: false,
                    deletes: 0,
                    read_row: Some(read_row),
                },
                expected,
                &[],
            ),
        };
    }
    if selector < u64::from(profile[0] + profile[1]) {
        return PreparedRequest {
            request: Request::Query {
                pk: PrimaryKey::new(vec![0; 16]),
                exclusive_after_sk: None,
                limit: 16,
            },
            acknowledgements: Vec::new(),
            operation_kind: 1,
            width: 0,
            deletes: 0,
            conditional: false,
            fingerprint: request_fingerprint(
                options,
                RequestFingerprintContext {
                    address,
                    operation_kind: 1,
                    width: 0,
                    conditional: false,
                    deletes: 0,
                    read_row: None,
                },
                expected,
                &[],
            ),
        };
    }
    let width = options.width.for_operation(write_ordinal.wrapping_mul(37));
    let value = workload_value(options, operation_number ^ (selection_offset << 48));
    let conditional = write_ordinal % 2 == 0;
    let mut conditions = Vec::new();
    let mut mutations = Vec::new();
    let mut acknowledgements = Vec::new();
    let mut deletes = 0;
    for offset in 0..width {
        let row_number = options.initial_rows + lane_start + offset;
        let key = DocumentKey::new(vec![0; 16], (row_number as u128).to_be_bytes().to_vec());
        if conditional {
            conditions.push(
                if expected
                    .get(&(tenant_number, row_number))
                    .is_some_and(|state| state.is_some())
                {
                    TransactionCondition::Exists { key: key.clone() }
                } else {
                    TransactionCondition::NotExists { key: key.clone() }
                },
            );
        }
        if mutation_ordinal.wrapping_add(offset as u64) % 10 == 0 {
            mutations.push(TransactionMutation::Delete { key });
            acknowledgements.push((row_number, None));
            deletes += 1;
        } else {
            mutations.push(TransactionMutation::Put {
                key,
                value: value.clone(),
            });
            acknowledgements.push((row_number, Some(value.clone())));
        }
    }
    let fingerprint = request_fingerprint(
        options,
        RequestFingerprintContext {
            address,
            operation_kind: 2,
            width,
            conditional,
            deletes,
            read_row: None,
        },
        expected,
        &acknowledgements,
    );
    PreparedRequest {
        request: Request::Transact {
            request: TransactionRequest::new(conditions, mutations),
        },
        acknowledgements,
        operation_kind: 2,
        width,
        deletes,
        conditional,
        fingerprint,
    }
}

fn request_fingerprint(
    options: &Options,
    context: RequestFingerprintContext,
    expected: &BTreeMap<(usize, usize), Option<Vec<u8>>>,
    acknowledgements: &[(usize, Option<Vec<u8>>)],
) -> u64 {
    let RequestFingerprintContext {
        address,
        operation_kind,
        width,
        conditional,
        deletes,
        read_row,
    } = context;
    let RequestAddress {
        tenant_number,
        operation_number,
        lane_start,
        ..
    } = address;
    let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    for value in [
        tenant_number as u64,
        operation_number,
        lane_start as u64,
        operation_kind as u64,
        width as u64,
        u64::from(conditional),
        deletes,
    ] {
        fingerprint = roll_fingerprint(fingerprint, value);
    }
    match read_row {
        Some(row_number) => {
            fingerprint = roll_fingerprint(fingerprint, row_number as u64);
            for byte in crate::seeded_value(options, tenant_number, row_number) {
                fingerprint = roll_fingerprint(fingerprint, u64::from(byte));
            }
        }
        None if operation_kind == 1 => {
            for row_number in 0..16 {
                fingerprint = roll_fingerprint(fingerprint, row_number as u64);
                for byte in crate::seeded_value(options, tenant_number, row_number) {
                    fingerprint = roll_fingerprint(fingerprint, u64::from(byte));
                }
            }
        }
        None => {}
    }
    for (row_number, value) in acknowledgements {
        fingerprint = roll_fingerprint(fingerprint, *row_number as u64);
        match value {
            Some(value) => {
                fingerprint = roll_fingerprint(fingerprint, 1);
                for byte in value {
                    fingerprint = roll_fingerprint(fingerprint, u64::from(*byte));
                }
            }
            None => fingerprint = roll_fingerprint(fingerprint, 0),
        }
        let condition_state = expected.get(&(tenant_number, *row_number));
        fingerprint = roll_fingerprint(
            fingerprint,
            u64::from(condition_state.is_some_and(Option::is_some)),
        );
    }
    fingerprint
}

fn valid_response(
    response: &Response,
    operation_kind: usize,
    options: &Options,
    tenant_number: usize,
    operation_number: u64,
) -> bool {
    match (operation_kind, response) {
        (0, Response::Get(state)) => {
            let row_number = seeded_read_row(options, tenant_number, operation_number);
            matches!(state.value(), Some(value) if *value == crate::seeded_value(options, tenant_number, row_number))
        }
        (1, Response::Query(rows)) => {
            rows.len() == 16
                && rows.iter().enumerate().all(|(row_number, row)| {
                    row.key == document_key(row_number)
                        && row.value == crate::seeded_value(options, tenant_number, row_number)
                })
        }
        (2, Response::Transact(_)) => true,
        _ => false,
    }
}

fn seeded_read_row(options: &Options, tenant_number: usize, operation_number: u64) -> usize {
    ((u128::from(operation_number) * 0x9e37 + tenant_number as u128) % options.initial_rows as u128)
        as usize
}

fn document_key(row_number: usize) -> DocumentKey {
    DocumentKey::new(vec![0; 16], (row_number as u128).to_be_bytes().to_vec())
}

fn workload_value(options: &Options, operation_number: u64) -> Vec<u8> {
    let mut value = Vec::with_capacity(options.value_bytes);
    let mut state = options.seed ^ operation_number ^ 0xa5a5_5a5a_d3c4_b2e1;
    while value.len() < options.value_bytes {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        value.extend_from_slice(&state.to_be_bytes());
    }
    value.truncate(options.value_bytes);
    value
}

fn deterministic_fingerprints(options: &Options, topology: &str) -> Value {
    let input_identity = format!(
        "seed={};registered={};active={};idle={};rows={};value_bytes={}",
        options.seed,
        options.registered_tenants,
        options.active_tenants,
        options.opened_idle_tenants,
        options.initial_rows,
        options.value_bytes
    );
    let schedule_identity = format!(
        "hash=fnv1a64;topology={topology};profile={};width={:?};clients={};rate={:?};duration={:?};operations={:?};selector=op*37+client_offset mod 100;write_width_selector=write_ordinal*37 mod 100;conditional=even_write_ordinal;delete=mutation_ordinal mod 10",
        options.profile.as_str(),
        options.width,
        options.client_count,
        options.per_tenant_rate,
        options.duration_ms,
        options.operation_count
    );
    json!({
        "input_fingerprint": stable_fingerprint(input_identity.as_bytes()),
        "input_identity": input_identity,
        "schedule_fingerprint": stable_fingerprint(schedule_identity.as_bytes()),
        "schedule_identity": schedule_identity,
        "scope": "deterministic input and schedule recipe; observed operation and width counts describe the realized prefix"
    })
}

fn operation_latency_json(counters: &Counters) -> Value {
    let mut operations = serde_json::Map::new();
    for (operation_index, operation_name) in ["Get", "Query", "Write"].iter().enumerate() {
        let observation_count = counters.operation_latency_histograms[operation_index]
            .iter()
            .sum::<u64>();
        operations.insert(
            (*operation_name).to_owned(),
            json!({
                "bucket_counts": counters.operation_latency_histograms[operation_index].to_vec(),
                "observation_count": observation_count,
                "maximum_nanos": counters.operation_maximum_latency_nanos[operation_index]
            }),
        );
    }
    Value::Object(operations)
}

fn stable_fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

async fn abort_and_reap<T: 'static>(tasks: &mut JoinSet<T>) {
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
}

#[cfg(test)]
mod tests {
    use super::{
        RequestAddress, deterministic_fingerprints, prepare_request, run_fixed_per_tenant,
        run_fixed_total,
    };
    use crate::config::{Options, Profile, Width};
    use dodb_core::TenantId;
    use dodb_service::{DodbService, ExecutionBudget, Request, Response};
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    fn test_options() -> Options {
        Options {
            backend: crate::config::Backend::Blink,
            shared_resources: None,
            memory_budget: None,
            data_dir: "/tmp/density-test-data".into(),
            report_path: "/tmp/density-test-report.json".into(),
            seed: 7,
            registered_tenants: 512,
            active_tenants: 16,
            opened_idle_tenants: 8,
            client_count: 4,
            duration_ms: None,
            operation_count: Some(20_000),
            initial_rows: 256,
            value_bytes: 512,
            topology: crate::config::Topology::FixedTotal,
            per_tenant_rate: None,
            queue_bound: 32,
            max_in_flight: 8,
            request_timeout_ms: 1000,
            drain_timeout_ms: 1000,
            profile: Profile::ReadHeavy,
            width: Width::Mixed,
            checkpoint_policy: dodb_server::BlinkCheckpointPolicy::Disabled,
            blink_checkpoint: crate::blink_checkpoint::Options::default(),
        }
    }

    #[test]
    fn write_schedule_keeps_profile_width_condition_and_delete_selectors_independent() {
        let options = test_options();
        let expected = BTreeMap::new();
        let mut operation_counts = [0_u64; 3];
        let mut width_counts = [0_u64; 3];
        let mut conditional_count = 0_u64;
        let mut delete_count = 0_u64;
        let mut mutation_count = 0_u64;
        let mut write_ordinal = 0_u64;
        let mut mutation_ordinal = 0_u64;
        for operation_number in 0..20_000 {
            let request = prepare_request(
                &options,
                RequestAddress {
                    tenant_number: 1,
                    operation_number,
                    selection_offset: 0,
                    write_ordinal,
                    mutation_ordinal,
                    lane_start: 0,
                },
                &expected,
            );
            operation_counts[request.operation_kind] += 1;
            if request.operation_kind == 2 {
                write_ordinal += 1;
                mutation_ordinal += request.width as u64;
                conditional_count += u64::from(request.conditional);
                delete_count += request.deletes;
                mutation_count += request.width as u64;
                width_counts[match request.width {
                    1 => 0,
                    4 => 1,
                    16 => 2,
                    _ => panic!("unexpected width"),
                }] += 1;
            }
        }
        assert!((operation_counts[0] as f64 / 20_000.0 - 0.80).abs() < 0.02);
        assert!((operation_counts[1] as f64 / 20_000.0 - 0.15).abs() < 0.02);
        assert!((operation_counts[2] as f64 / 20_000.0 - 0.05).abs() < 0.02);
        assert!((width_counts[0] as f64 / write_ordinal as f64 - 0.80).abs() < 0.03);
        assert!((width_counts[1] as f64 / write_ordinal as f64 - 0.15).abs() < 0.03);
        assert!((width_counts[2] as f64 / write_ordinal as f64 - 0.05).abs() < 0.03);
        assert!((conditional_count as f64 / write_ordinal as f64 - 0.5).abs() < 0.01);
        assert!((delete_count as f64 / mutation_count as f64 - 0.1).abs() < 0.02);
    }

    #[test]
    fn fingerprints_are_deterministic_and_include_schedule_inputs() {
        let options = test_options();
        let first = deterministic_fingerprints(&options, "fixed-total");
        let second = deterministic_fingerprints(&options, "fixed-total");
        assert_eq!(first, second);
        let changed = Options {
            profile: Profile::Mixed,
            ..options
        };
        assert_ne!(
            first["schedule_fingerprint"],
            deterministic_fingerprints(&changed, "fixed-total")["schedule_fingerprint"]
        );
    }

    struct DelayedService {
        options: Options,
        delay: Duration,
        conflict_transactions: bool,
        active: Arc<AtomicU64>,
        active_high_water: Arc<AtomicU64>,
    }

    struct ActiveRequest(Arc<AtomicU64>);

    impl Drop for ActiveRequest {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::Relaxed);
        }
    }

    impl DodbService for DelayedService {
        fn execute<'service>(
            &'service self,
            tenant: TenantId,
            request: Request,
            _budget: ExecutionBudget,
        ) -> dodb_service::ServiceFuture<'service> {
            let options = self.options.clone();
            let delay = self.delay;
            let conflict_transactions = self.conflict_transactions;
            let active = Arc::clone(&self.active);
            let high_water = Arc::clone(&self.active_high_water);
            Box::pin(async move {
                let running = active.fetch_add(1, Ordering::Relaxed) + 1;
                high_water.fetch_max(running, Ordering::Relaxed);
                let _active_request = ActiveRequest(Arc::clone(&active));
                tokio::time::sleep(delay).await;
                match request {
                    Request::Get { key } => {
                        let row_number =
                            u128::from_be_bytes(key.sk.as_bytes().try_into().unwrap()) as usize;
                        Ok(Response::Get(dodb_core::RevisionState::present(
                            crate::seeded_value(&options, tenant.get() as usize, row_number),
                            dodb_core::Revision::new(1),
                        )))
                    }
                    Request::Query { .. } => Ok(Response::Query(
                        (0..16)
                            .map(|row_number| dodb_service::Document {
                                key: super::document_key(row_number),
                                value: crate::seeded_value(
                                    &options,
                                    tenant.get() as usize,
                                    row_number,
                                ),
                                revision: dodb_core::Revision::new(1),
                            })
                            .collect(),
                    )),
                    Request::Transact { .. } if conflict_transactions => {
                        Err(dodb_core::Error::conflict(dodb_core::TransactionConflict {
                            key: super::document_key(0),
                            expected: dodb_core::ConditionExpectation::NotExists,
                            actual: dodb_core::ObservedState::missing(dodb_core::Revision::new(0)),
                        }))
                    }
                    Request::Transact { .. } => Ok(Response::Transact(
                        dodb_service::TransactionOutcome::conditions_satisfied(),
                    )),
                    _ => Err(dodb_core::Error::InvalidRequest(
                        "unexpected fake-service request".to_owned(),
                    )),
                }
            })
        }
    }

    fn delayed_service(options: &Options, delay: Duration) -> Arc<DelayedService> {
        Arc::new(DelayedService {
            options: options.clone(),
            delay,
            conflict_transactions: false,
            active: Arc::new(AtomicU64::new(0)),
            active_high_water: Arc::new(AtomicU64::new(0)),
        })
    }

    #[tokio::test]
    async fn fixed_per_tenant_accounts_queue_drops_and_scheduled_latency() {
        let mut options = test_options();
        options.topology = crate::config::Topology::FixedPerTenant;
        options.active_tenants = 1;
        options.per_tenant_rate = Some(100);
        options.duration_ms = Some(120);
        options.operation_count = None;
        options.queue_bound = 1;
        options.max_in_flight = 1;
        options.request_timeout_ms = 1000;
        options.drain_timeout_ms = 3000;
        let service = delayed_service(&options, Duration::from_millis(80));
        let (report, _) =
            run_fixed_per_tenant(Arc::clone(&service), &options, || json!({"captured": true}))
                .await
                .unwrap();
        assert!(report["dropped"].as_u64().unwrap() > 0);
        assert!(report["accounting"]["scheduled_equals_admitted_plus_dropped"] == true);
        assert!(report["accounting"]["pending_equals_queued_plus_in_flight_at_offer_end"] == true);
        assert!(
            report["latency_histogram"]["maximum_nanos"]
                .as_u64()
                .unwrap()
                >= 80_000_000
        );
        assert!(report["offer_window_elapsed_ms"].as_u64().unwrap() >= 120);
    }

    #[tokio::test]
    async fn fixed_total_runs_concurrent_lanes_and_timeout_marks_mutation_uncertain() {
        let mut options = test_options();
        options.client_count = 4;
        options.max_in_flight = 4;
        options.operation_count = Some(24);
        options.profile = Profile::WriteHeavy;
        options.request_timeout_ms = 1000;
        options.drain_timeout_ms = 3000;
        let service = delayed_service(&options, Duration::from_millis(5));
        let (report, _) =
            run_fixed_total(Arc::clone(&service), &options, || json!({"captured": true}))
                .await
                .unwrap();
        assert!(report["observed_max_in_flight"].as_u64().unwrap() > 1);
        assert_eq!(
            report["per_tenant"].as_array().unwrap().len(),
            options.active_tenants
        );
        for (operation_name, operation_index) in [("Get", 0), ("Query", 1), ("Write", 2)] {
            assert_eq!(
                report["operation_latency_histograms"][operation_name]["observation_count"],
                report["operation_counts"][operation_index]
            );
        }
        let per_tenant_completed = report["per_tenant"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tenant| tenant["completed"].as_u64().unwrap())
            .sum::<u64>();
        assert_eq!(per_tenant_completed, report["completed"].as_u64().unwrap());
        assert_eq!(
            report["accounting"]["completed_equals_success_plus_conflict_plus_error_and_timeout"],
            true
        );
        assert!(
            report["accounting"]
                .as_object()
                .unwrap()
                .values()
                .all(|value| value == true)
        );

        options.client_count = 1;
        options.operation_count = Some(2);
        options.request_timeout_ms = 1000;
        let conflict_service = Arc::new(DelayedService {
            options: options.clone(),
            delay: Duration::from_millis(1),
            conflict_transactions: true,
            active: Arc::new(AtomicU64::new(0)),
            active_high_water: Arc::new(AtomicU64::new(0)),
        });
        let (conflict_report, _) =
            run_fixed_total(conflict_service, &options, || json!({"captured": true}))
                .await
                .unwrap();
        assert_eq!(conflict_report["rejected_conflicts"], 1);
        assert_eq!(conflict_report["other_error"], 0);
        assert_eq!(conflict_report["valid_for_oracle"], true);
        assert_eq!(conflict_report["cell_valid"], false);

        options.duration_ms = None;
        options.operation_count = Some(3);
        options.request_timeout_ms = 1;
        options.drain_timeout_ms = 3000;
        let service = delayed_service(&options, Duration::from_millis(50));
        let (report, _) = run_fixed_total(service, &options, || json!({"captured": true}))
            .await
            .unwrap();
        assert!(report["valid_for_oracle"] == false);
        assert!(report["cell_valid"] == false);
    }

    #[tokio::test]
    async fn fixed_total_drain_timeout_cancels_and_reaps_service_futures() {
        let mut options = test_options();
        options.client_count = 2;
        options.max_in_flight = 2;
        options.duration_ms = Some(10);
        options.operation_count = None;
        options.request_timeout_ms = 5000;
        options.drain_timeout_ms = 5;
        let service = delayed_service(&options, Duration::from_secs(1));
        let active = Arc::clone(&service.active);
        let result = run_fixed_total(service, &options, || json!({"captured": true})).await;
        let (report, _) = result.unwrap();
        assert_eq!(report["status"], "inconclusive_drain_timeout");
        assert!(report["partial_counters"]["in_flight"].as_u64().unwrap() > 0);
        tokio::task::yield_now().await;
        assert_eq!(active.load(Ordering::Relaxed), 0);
    }
}
