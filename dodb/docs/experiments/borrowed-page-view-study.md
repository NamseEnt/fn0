# Borrowed page view study

## Decision and scope

This experiment isolates page-level `Arc<BlinkPage>` cloning in generation-pinned
B-link reads. The opt-in `blink-borrowed-page-views` feature returns page views
whose lifetime is bounded by the `GenerationPin` borrow. The pin, all read
metrics, active-pin accounting, and page-reuse guard remain enabled. Parallel
write jobs still receive owned page references. The default feature set remains
the control.

The initial product-core matrix compares main B-tree, default B-link, and
borrowed-page B-link at c16 for Get and Query(16), with three paired repetitions:
18 full cases and six smoke cases. Both B-link variants use the same source,
compiler, read metrics, coordinator, and workload. Measurements use the canonical
OCI A1 host with two OCPU and ZFS, real sync, 10,000 keys, 16-byte keys,
512-byte values, two seconds of warmup, and five seconds of measurement.
The full measurement windows total 126 seconds, excluding build, initialization,
and teardown. Profiles are diagnostic and excluded from throughput ratios.

The candidate must preserve exact values, range ordering, overflow reads,
publication, page reuse, checkpoints, and recovery. Same-engine paired ratios
determine whether avoiding page clones improves reads. Product eligibility
separately requires Get and Query(16) to reach at least 90% of main; an improved
experimental candidate does not imply product eligibility.

The next independent architectural comparison is a main-path compact WAL
counterfactual with matching collection, batching, changing values, commit
framing, real durability, and recovery semantics.

## Local validation

The feature-on storage library suite passed 177 tests, with four pre-existing
ignored microbenchmarks. The new test checks unchanged page reference counts,
an old pinned overflow value through replacement, splits, and checkpoint,
exact Get/Query/Scan results, high-water and epoch rejection, page-reuse
eligibility, and reopen behavior. Existing concurrent publication, randomized
differential, overlay merge, torn-tail, and durability fault tests also run in
the feature-on suite.

The read matrix validator rejects a candidate with an absent or false borrowed
view flag, disabled read metrics, or inactive read counters. It also rejects a
borrowed-view binary mislabeled as the baseline. All eight matrix tests pass.

Strict Clippy is blocked by nine pre-existing diagnostics in `blink/mod.rs` and
`wal.rs`: seven argument-count findings and two nested condition findings.
The diagnostic source fragments were checked against the parent revision;
none originate in the candidate changes. No lint suppression was added.
