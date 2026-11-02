# T01 Shared Resource Ledger

Status: accepted for the T01 storage-ledger scope at implementation commit `6d7cd1c1c67da0f490cdeb8d58068a7b1612328b`. The opt-in storage feature and synchronous global/per-tenant byte ledger are present. Thirteen focused unit tests pass for aggregate and category limits, exact-fit/over-limit, tenant limits, shared-manager contention, category transfer, final shared-owner release, progress-pool isolation, checked overflow, bounded wait, waiter-count rejection, timeout, cancellation, and unwind cleanup.

Command: `cargo test -p dodb-storage --features blink-demand-paging blink::resource::tests --lib` (13 passed).

Formatting: `rustfmt --check --edition 2024 --config skip_children=true dodb/crates/dodb-storage/src/blink/resource.rs` passed.

Strict Clippy command: `cargo clippy -p dodb-storage --features blink-demand-paging --lib -- -D warnings` failed on twelve existing findings in `blink/mod.rs` and `wal.rs`. No finding pointed to `blink/resource.rs`. The findings include argument-count, collapsible-if, needless-option-as-deref, and redundant-slicing lints. No lint suppression was added.

The module is not wired into server admission or page ownership; server integration is T12. T01 does not establish bounded process memory or runtime resource control until later tasks connect the ledger to real owners. T02 is now the next dependency task.
