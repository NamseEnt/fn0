# A1 implementation and worker evidence

## Change summary

The primary retained the worker's original logs under [evidence/a1](evidence/a1). Absolute `/tmp` paths below describe the original execution locations. The later test-only decision-pin observation and final review are recorded in [A1-RESULTS.md](A1-RESULTS.md); the allocation-only outputs in this handback remain historical evidence.

Added the default-disabled `blink-writer-pin-reuse` storage feature. In planned execution, the feature releases the admission generation pin after admission finishes and before the page reuse decision. The optional parallel preparation path makes its reuse decision before creating its own immutable generation pin. The conservative active-reader guard remains in both paths.

`GenerationPin` owns an `Arc<PublishedGeneration>`. A published generation resolves page IDs through immutable catalog chunks to `PageCell` values that own `Arc<BlinkPage>`. Mutation writes through `WorkingBlinkState` overlays and publication replaces changed catalog cells/chunks. A reader that pins the old current generation after the reuse decision therefore retains its old immutable page objects; page ID reuse in the writer overlay does not alter those objects. The test `reader_arriving_after_reuse_decision_keeps_immutable_page_view` deterministically makes that interleaving and checks the old overflow value after the writer overlay reuses its page ID.

The 4,096-byte diagnostic uses a parallel-enabled store with two workers. Its single-leaf workload takes the existing conservative single-leaf fallback. This exercises parallel preparation and its internal pin lifetime without claiming parallel leaf execution.

## Deterministic overwrite diagnostic

Command, run once with default features and once with the opt-in feature:

```text
cargo test -p dodb-storage --lib writer_pin_reuse_bounds_overflow_churn_and_preserves_pinned_generations -- --nocapture
cargo test -p dodb-storage --lib --features blink-writer-pin-reuse writer_pin_reuse_bounds_overflow_churn_and_preserves_pinned_generations -- --nocapture
```

Fresh allocations are counted as the increase in page high-water between writes. Successful reuse is counted by intersecting the current key's overflow page IDs with the free page IDs present before that write. The reported unavailable reuse opportunities are inferred from a nonempty pre-write free list and no reused page; they are test observations, not production instrumentation or direct reuse-decision counters. The test asserts total fresh allocations equal the high-water increase and verifies invariants after each size case.

Default output:

```text
writer_pin_reuse_diagnostic value_size=512 initial_high_water=2 warmup_high_water=2 pinned_high_water=2 released_high_water=2 final_high_water=2 fresh_allocations_total=0 successful_reuses_total=0 no_reader_reuses=0 pinned_reuses=0 post_release_reuses=0 pinned_fresh_allocations=0 post_release_fresh_allocations=0 unavailable_reuse_opportunities=0 free_pages_final=0 active_pins_final=0 feature_enabled=false
writer_pin_reuse_diagnostic value_size=513 initial_high_water=3 warmup_high_water=11 pinned_high_water=27 released_high_water=59 final_high_water=91 fresh_allocations_total=88 successful_reuses_total=0 no_reader_reuses=0 pinned_reuses=0 post_release_reuses=0 pinned_fresh_allocations=8 post_release_fresh_allocations=64 unavailable_reuse_opportunities=87 free_pages_final=88 active_pins_final=0 feature_enabled=false
writer_pin_reuse_diagnostic value_size=4096 initial_high_water=4 warmup_high_water=20 pinned_high_water=52 released_high_water=116 final_high_water=180 fresh_allocations_total=176 successful_reuses_total=0 no_reader_reuses=0 pinned_reuses=0 post_release_reuses=0 pinned_fresh_allocations=16 post_release_fresh_allocations=128 unavailable_reuse_opportunities=87 free_pages_final=176 active_pins_final=0 feature_enabled=false
```

Opt-in output:

```text
writer_pin_reuse_diagnostic value_size=512 initial_high_water=2 warmup_high_water=2 pinned_high_water=2 released_high_water=2 final_high_water=2 fresh_allocations_total=0 successful_reuses_total=0 no_reader_reuses=0 pinned_reuses=0 post_release_reuses=0 pinned_fresh_allocations=0 post_release_fresh_allocations=0 unavailable_reuse_opportunities=0 free_pages_final=0 active_pins_final=0 feature_enabled=true
writer_pin_reuse_diagnostic value_size=513 initial_high_water=3 warmup_high_water=4 pinned_high_water=12 released_high_water=12 final_high_water=12 fresh_allocations_total=9 successful_reuses_total=79 no_reader_reuses=15 pinned_reuses=0 post_release_reuses=64 pinned_fresh_allocations=8 post_release_fresh_allocations=0 unavailable_reuse_opportunities=8 free_pages_final=9 active_pins_final=0 feature_enabled=true
writer_pin_reuse_diagnostic value_size=4096 initial_high_water=4 warmup_high_water=6 pinned_high_water=22 released_high_water=22 final_high_water=22 fresh_allocations_total=18 successful_reuses_total=158 no_reader_reuses=30 pinned_reuses=0 post_release_reuses=128 pinned_fresh_allocations=16 post_release_fresh_allocations=0 unavailable_reuse_opportunities=8 free_pages_final=18 active_pins_final=0 feature_enabled=true
```

The workload does 16 writes without external readers, eight writes with an old generation pinned, then 64 writes after releasing it. For 513- and 4,096-byte values the candidate reuses overflow pages without an external reader, denies reuse while the external reader is pinned, and resumes reuse after release without further high-water growth. The 512-byte case is the inline-value boundary control. Page counts and high-water are logical allocator observations; `page_count * 4096` would be normalized page-image bytes, not retained heap, RSS, filesystem allocation, or disk returned.

## Correctness checks

`planned_reuse_preserves_ordered_group_revisions_and_reopen` checks a multi-transaction group that deletes an overflow value and conditionally writes a new overflow value under `NotExists`, then checks increasing revisions, the final value, and WAL reopen state.

`reader_arriving_after_reuse_decision_keeps_immutable_page_view` pins a generation after the reuse decision and applies delete plus overflow put in a working overlay using the already-made reuse decision. It verifies that the old pinned generation still reads its original value while the working overlay reuses page capacity.

`process_kill_recovers_incomplete_transaction_then_write_then_second_crash` runs child service processes for legacy WAL and segmented durable bases. Each child first acknowledges four 4,096-byte overwrites, pauses at `before_commit_record` during an incomplete transaction, then is killed with signal 9. A second child reopens and validates the acknowledged value/revision, acknowledges another write, pauses before a new commit, and is killed with signal 9. The final service reopen validates the second acknowledged value/revision. The segmented-bases child also verifies a `.pages` base pack exists. ChildGuard kills and reaps child processes if a wait or assertion fails.

## Checks and outcomes

- `cargo fmt --manifest-path dodb/crates/dodb-storage/Cargo.toml -- --check`: passed.
- `cargo fmt --manifest-path dodb/crates/dodb-server/Cargo.toml -- --check`: passed.
- `cargo test -p dodb-storage --lib`: passed, 205 passed, 0 failed, 4 ignored.
- `cargo test -p dodb-storage --lib --features blink-writer-pin-reuse`: passed, 205 passed, 0 failed, 4 ignored.
- `cargo test -p dodb-server --test blink_writer_pin_reuse`: passed, 2 passed, 0 failed.
- `cargo test -p dodb-server --test blink_writer_pin_reuse --features dodb-storage/blink-writer-pin-reuse`: passed, 2 passed, 0 failed.
- `cargo test -p dodb-server --test blink_wal_checkpoint --test durable_base_checkpoint --test background_checkpoint --features dodb-storage/blink-writer-pin-reuse`: passed; 8, 6, and 7 tests respectively, all passed.
- `cargo clippy -p dodb-storage -p dodb-server --all-targets --features dodb-storage/blink-writer-pin-reuse -- -D warnings`: failed on 23 existing warnings-as-errors in storage source and tests. No A1 implementation or new test location was reported. The exact output is preserved at `/tmp/a1-clippy.log`.
- The same Clippy command without `-D warnings` completed successfully. It reported existing warnings in storage/server code; no A1 source or new integration-test location was reported. Output is at `/tmp/a1-clippy-advisory.log`.
- `cargo fmt --all -- --check` could not complete because Cargo metadata reports `vendor/deno/ext/crypto/Cargo.toml` is inside the workspace but not a workspace member. The package-specific formatting checks above passed.

The deterministic allocator and recovery checks do not establish OCI throughput, RSS savings, disk space returned, or active-tenant density. No performance benchmark was run.

## Rejected retry and subsequent pin observation

`/tmp/a1-storage-default-final.log` was interrupted with Ctrl-C (exit 130) before the fault matrix finished. It contains no final test result and is not a passing-suite artifact. It is retained with the other original logs. The full default and candidate suite results above refer only to their two original successful logs.

The primary then requested an actual decision-pin observation to close the accounting gap. The storage library's test-only `last_reuse_decision_pins` atomic records the active pin count read by `can_reuse_pages`; production builds do not contain the field or recording store. The same three-size diagnostic asserts and reports baseline pin counts 1/2/1 and candidate counts 0/1/0 for no external reader, an external old generation, and after release. The 4096-byte case exercises parallel preparation's single-leaf serial fallback. Both focused diagnostics and storage formatting pass after this addition. The outputs are [default](evidence/a1/A1-diagnostic-pins-default.log.gz) and [candidate](evidence/a1/A1-diagnostic-pins-feature.log.gz).
