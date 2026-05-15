//! Doc-db handle for the worker-agent.
//!
//! The worker-agent reads/writes the same forte project DB that
//! fn0-control hosts on (project id `fn0-control`). Same pattern as
//! fn0-worker's `manifest_poller::build_database_from_env`, so
//! liveness state (`WorkerHostStatusDoc`) and rollout target
//! (`TargetFn0WorkerConfigDoc`) live in one DB that control can
//! query directly (e.g. `zombie_sweep`).

use doc_db::Database;

const CONTROL_PROJECT_ID: &str = "fn0-control";

pub fn build() -> Database {
    let group_token =
        std::env::var("TURSO_GROUP_TOKEN").expect("TURSO_GROUP_TOKEN must be set");
    let host_suffix =
        std::env::var("TURSO_DB_HOST_SUFFIX").expect("TURSO_DB_HOST_SUFFIX must be set");
    let url = format!("https://{CONTROL_PROJECT_ID}{host_suffix}");
    doc_db::turso_with_config(url, group_token)
}
