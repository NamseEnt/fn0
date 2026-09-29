mod admin;
pub mod allocator_stats;
mod app_state;
mod backpressure;
pub mod bloom;
pub mod clock;
mod collect;
mod compaction_tier;
pub mod config;
mod disk;
// Shared with `benches/` and `bin/load`. One generator, so a load result and a
// bench result describe the same bytes; see the module doc.
pub mod corpus;
mod delete_requests;
mod flush;
pub mod gorilla;
pub mod histogram_chunk;
mod ingest;
pub mod journal;
pub mod label_name;
mod log_ingest;
pub mod log_scan;
pub mod logql;
pub mod malloc_tuning;
pub mod memory_budget;
pub mod memprof;
pub mod memtable;
mod merge;
mod metrics;
pub mod object_storage;
mod object_store_gc;
mod otlp_log;
mod otlp_tenant;
pub(crate) mod page_cache;
pub mod part;
pub mod part_registry;
pub mod query;
mod remote_lifecycle;
pub mod restore_meter;
mod retention;
mod router;
mod runtime_error;
pub mod series;
mod series_ingest;
mod series_merge;
mod series_part;
pub mod series_registry;
mod shutdown;
mod startup;
pub mod tenant;
pub mod tenant_policy;
mod tenant_quota;
pub mod test_support;
mod trace;
mod trace_ingest;
mod trace_merge;
mod trace_part;
pub mod trace_registry;

pub use app_state::{AppState, AppStateDependencies};
pub use router::build_router;
pub use startup::{recover, run};

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use std::sync::Arc;

    include!("tests/e2e.rs");
    include!("tests/shutdown_rehearsal.rs");
}
