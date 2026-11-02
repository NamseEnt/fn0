# T01 Shared Resource Ledger

Status: partial implementation. The opt-in storage feature and synchronous global/per-tenant byte ledger are present. Nine focused unit tests pass for exact limits, over-limit rejection, tenant limits, shared-manager contention, category transfer, final shared-owner release, progress-pool isolation, checked overflow, and unwind cleanup.

Command: `cargo test -p dodb-storage --features blink-demand-paging blink::resource::tests --lib` (9 passed).

Strict Clippy command: `cargo clippy -p dodb-storage --features blink-demand-paging --lib -- -D warnings` failed on twelve existing findings in `blink/mod.rs` and `wal.rs`. No finding pointed to `blink/resource.rs`. The findings include argument-count, collapsible-if, needless-option-as-deref, and redundant-slicing lints. No lint suppression was added.

The module is not wired into server admission or page ownership. Byte capacities are aggregate global/per-tenant limits with per-category accounting; per-category hard ceilings and async wait/cancellation admission are not implemented. T01 is therefore not accepted as complete. T02 remains blocked on the T01 acceptance gap only after the ledger's remaining requirements are either implemented or explicitly amended; current work continues in dependency order.
