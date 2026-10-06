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

The feature-on integration check across core, storage, testkit, service,
protocol, client, server, and soak passed 334 tests; the default storage check
passed 211 tests. Both had four pre-existing ignored microbenchmarks. The
OCI feature-on storage library suite also passed all 177 tests.

## Accepted OCI results

All accepted binaries and raw records use source
`b7c67715c2dc52d30fce43fcb5d885fedd070547`, Rust 1.97.1, and
`-C target-cpu=native`. The [independent validation record](results/borrowed-page-validation-b7c67715.json)
confirms 18 full cases, six smoke cases, four diagnostic profiles, source
identity, binary hashes, raw-file hashes, all read metrics enabled, zero active
pins at completion, and exact Query(16) row counts. The
[raw matrices and build evidence](results/oci-a1-2ocpu-12g-zfs-borrowed-page-b7c67715/)
include the command logs, environment, source provenance, and compressed perf
captures. Native executables remain at the recorded OCI paths; their hashes
are retained in the build evidence and matrix manifests.

| Operation | Borrowed / default B-link, paired median [min, max] | Borrowed / main, paired median [min, max] | Default B-link / main, paired median |
| --- | --- | --- | --- |
| Get | 1.104508 [1.061954, 1.156131] | 0.899577 [0.876926, 0.903067] | 0.814460 |
| Query(16) | 1.305054 [1.303157, 1.338802] | 0.978344 [0.975562, 1.002715] | 0.748964 |

Median read p99 for default B-link, borrowed B-link, and main was respectively
2.20 / 2.00 / 1.76 microseconds for Get and 4.84 / 3.28 / 3.12 microseconds
for Query(16). The read control uses the same async harness and two Tokio
workers for all engines, with the same seeds within each repetition.

The paired intervention improved both operations in all three pairs. It
substantially reduced the Query(16) gap while retaining every observational
counter. Get is on the 90% product-budget boundary: the measured paired median
is slightly below 0.90 and one repetition reaches 0.903. This initial matrix
does not establish that the Get gate passes. Keep the feature opt-in pending
the same-session main compact-WAL read comparison and longer Get windows.

The Get control profile attributes 19.51% of user-cycle samples to
`GenerationPin::owned_page`, including routing and entry lookup. The reports
retain raw Rust v0 symbols because the host perf version does not demangle
them. Profiles include initialization and warmup; their sample shares are
diagnostic and cannot be interpreted as additive shares of the throughput
gain or as proof of a particular cache-coherence mechanism.

Two build attempts were excluded before any measured case: the first exhausted
the root filesystem during linking, and the second was rejected because Python
bytecode caches made the source checkout unclean. The accepted retry moved the
build cache and temporary directory onto ZFS and disabled Python bytecode
generation. Task-related storage build caches were removed from the root disk;
older benchmark result artifacts were preserved. The retry completed with a
clean checkout before and after the study.
