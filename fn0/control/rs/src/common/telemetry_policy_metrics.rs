//! Low-cardinality control-plane metrics for telemetry policy convergence.
//!
//! Tenant ids and error text intentionally do not appear here. Detailed
//! diagnosis belongs in authenticated inspection of the outbox and exception
//! documents; these instruments are for fleet-level alerting.

use forte_sdk::metrics;

fn add_counter(name: &'static str, value: u64) {
    metrics::meter().u64_counter(name).build().add(value, &[]);
}

pub fn sync_attempt() {
    add_counter("fn0.telemetry.policy.sync.attempts", 1);
}

pub fn sync_success() {
    add_counter("fn0.telemetry.policy.sync.success", 1);
}

pub fn sync_failure() {
    add_counter("fn0.telemetry.policy.sync.failure", 1);
}

pub fn sync_stale() {
    add_counter("fn0.telemetry.policy.sync.stale", 1);
}

pub fn sync_conflict() {
    add_counter("fn0.telemetry.policy.sync.conflict", 1);
}

pub fn enqueue_correction() {
    add_counter("fn0.telemetry.policy.enqueue_corrections", 1);
}

pub fn migration_exceptions(value: u64) {
    metrics::meter()
        .u64_gauge("fn0.telemetry.policy.migration_exceptions")
        .build()
        .record(value, &[]);
}

pub fn record_state(
    pending: u64,
    oldest_pending_age_seconds: u64,
    revoke_pending: u64,
    oldest_revoke_pending_age_seconds: u64,
    migration_exceptions: u64,
) {
    metrics::meter()
        .u64_gauge("fn0.telemetry.policy.pending")
        .build()
        .record(pending, &[]);
    metrics::meter()
        .u64_gauge("fn0.telemetry.policy.oldest_pending_age_seconds")
        .build()
        .record(oldest_pending_age_seconds, &[]);
    metrics::meter()
        .u64_gauge("fn0.telemetry.policy.revoke_pending")
        .build()
        .record(revoke_pending, &[]);
    metrics::meter()
        .u64_gauge("fn0.telemetry.policy.oldest_revoke_pending_age_seconds")
        .build()
        .record(oldest_revoke_pending_age_seconds, &[]);
    metrics::meter()
        .u64_gauge("fn0.telemetry.policy.migration_exceptions")
        .build()
        .record(migration_exceptions, &[]);
}
