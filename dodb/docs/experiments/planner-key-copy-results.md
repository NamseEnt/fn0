# B-link Planner Key Copy Cleanup

## Motivation and scope

`plan_batch()` materialized an encoded key in `PlannedMutation.encoded_key`,
`RouteHint.encoded_key`, and a transaction-level `encoded_keys` vector before
building `mutated_key_set`. Runtime routing and rerouting read the planned
mutation's encoded key and route hint's leaf ID. Runtime execution reads
`mutated_key_set`; it never reads either redundant field. This experiment
removes those two avoidable copies while preserving the existing key ownership
in `last_key_writer`, `PlannedMutation.encoded_key`, and `mutated_key_set`.

The base is `35adac516dc5a964cd6ec60eedaf2ef77f04ee56`. The implementation
candidate is `02ffdeb6a574ff35c247b2507c954f529218d3cd`.

## Workspace usage audit

The workspace-wide audit covered `RouteHint`, `route_hint.encoded_key`,
`PhysicalTransactionPlan`, `encoded_keys`, and `mutated_key_set`.

- `RouteHint.encoded_key` had one construction and no read. Runtime routing
  uses `PlannedMutation.encoded_key`; execution uses `route_hint.leaf_id`.
- `PhysicalTransactionPlan.encoded_keys` had one construction and no read. It
  only supplied clones for `mutated_key_set`.
- No other crate reads either field. `RouteHint` and
  `PhysicalTransactionPlan` are public and re-exported by `dodb-storage`, but
  the repository documents this B-link engine as experimental and defines no
  stable API compatibility promise for these planner structures.
- Other occurrences were type exports, constructors, tests, or experiment
  documentation. No external consumer is present in this workspace.

The ownership path is now:

```text
encoded key
  ├─ clone -> PlannedMutation.encoded_key
  └─ move  -> last_key_writer

PlannedMutation.encoded_key
  └─ clone into BTreeSet -> PhysicalTransactionPlan.mutated_key_set

RouteHint
  └─ leaf_id only
```

The `BTreeSet` continues to deduplicate repeated keys. Its contents and
`restamp()` behavior are unchanged. No routing, dependency, grouping, mutation
ordering, revision, WAL, recovery, or page encoding logic changed.

## Correctness

Added `planned_key_copy_cleanup_preserves_routes_dependencies_and_key_set`.
It directly checks the routed leaf IDs, same-key and condition-key
predecessors, same-leaf page and structural-route predecessors, absence of a
same-page dependency for a different leaf, duplicate-key set semantics, exact
multi-mutation key sets, and provisional revision preservation. Existing
`planned_stale_route_reroutes_after_a_split` confirms split/reroute execution
continues to use the planned mutation key with the leaf hint.

All requested gates passed after the focused test was added:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core` (6 passed)
- `cargo test -p dodb-storage` (114 unit tests passed; 8 recovery tests passed)
- `cargo test -p dodb-storage --bin phase0-bench` (10 passed)
- `cargo test --workspace` (all tests passed)
- `git diff --check`

## Local release smoke

Base and candidate ran on the same Apple M1 with 16 writers, zero readers,
width 1, `different-leaf-heavy`, 100,000 keys, cache 4,096, 16-byte keys,
64-byte values, group limit 64, group bytes 4,194,304, queue 256, zero
collection delay, disabled sync, two Tokio workers, one-second warmup and
duration, one repetition, and seed `0x3a042026`. Both reported zero errors and
zero overloads.

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput (mutations/s) | 75,605 | 77,377 | 1.023x |
| Planning (ns/mutation) | 1,230.3 | 1,149.0 | 0.934x |
| Route (ns/mutation) | 542.8 | 532.7 | 0.981x |
| Non-route planning (ns/mutation) | 687.5 | 616.3 | 0.896x |

Both planning gates improved by more than 5%, so the candidate proceeded to
OCI measurement.

## OCI environment and build

The same-host A/B ran on `opc@217.142.246.204`, Oracle Linux 9 AArch64,
Linux `6.12.0-206.104.4.4.el9uek.aarch64`, Rust `1.98.1`, and two logical
CPUs. The host reports 10 GiB memory available to Linux. The root XFS filesystem
is 30 GiB; the attached 200 GiB block device is not mounted as a benchmark
filesystem. Sync was disabled for these CPU planning measurements.

Both commits were built from detached clean worktrees with separate empty
target directories using:

```bash
cargo build --locked --release -p dodb-storage --bin phase0-bench
```

The repository's AArch64 config supplied `-C target-feature=+crc`; no
`RUSTFLAGS` override was used.

| Commit | `phase0-bench` SHA256 |
| --- | --- |
| Base `35adac516dc5a964cd6ec60eedaf2ef77f04ee56` | `f8d50a72bceb268729e19f2b2963459d0e0c7d06b551d00ff66d93b100c5ea8e` |
| Candidate `02ffdeb6a574ff35c247b2507c954f529218d3cd` | `8d38db5ee831c532cd65598157cdd331b49453f4027c88d5ef75722d0deeb424` |

## Width-1 primary workload

The width-1 workload used 16 writers, zero readers, width 1,
`different-leaf-heavy`, working set 100,000, cache 4,096, key size 16, value
size 64, group limit 64, group bytes 4,194,304, queue 256, zero collection
delay, disabled sync, two Tokio workers, one-second warmup, two-second duration,
three repetitions, and seed `0x3a042026`. Every repetition had zero errors and
zero overloads. Costs below are medians of per-mutation values computed from
`planning_nanos`, `planner_route_nanos`, and the existing component counters.

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput (mutations/s), median (min–max) | 47,019 (44,856–47,451) | 47,007 (46,461–47,661) | 1.000x |
| Planning (ns/mutation) | 2,579.6 | 2,288.6 | 0.894x |
| Route (ns/mutation) | 1,141.0 | 1,103.9 | 0.968x |
| Non-route planning (ns/mutation) | 1,438.6 | 1,192.7 | 0.829x |
| Routes/mutation | 1.000 | 1.000 | — |
| Page visits/route | 4.000 | 4.000 | — |
| Right-link hops/mutation | 0.000 | 0.000 | — |
| Physical execution (ns/mutation) | 4,023.5 | 4,056.5 | 1.008x |
| Page encode (ns/mutation) | 1,693.6 | 1,716.9 | 1.014x |
| WAL append (ns/mutation) | 4,345.9 | 4,631.9 | 1.066x |
| WAL group encode (ns/mutation) | 2,208.5 | 2,570.8 | 1.164x |

The non-route planning ratio is `0.829x`, total planning is `0.894x`, and
throughput is `1.000x`. WAL group encode increased 16.4%, crossing the
predeclared 15% unrelated-regression reporting threshold. Route cost, physical
execution, page encode, and WAL append stayed within that threshold. No
follow-up optimization was made for these unrelated costs.

## Width-16 control

The control used the same parameters except width 16, `uniform` distribution,
and seed `0x3a032026`. All repetitions had zero errors and zero overloads.

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput (mutations/s), median (min–max) | 38,418 (37,280–39,250) | 43,690 (41,241–45,001) | 1.137x |
| Throughput (transactions/s), median | 2,401 | 2,731 | 1.137x |
| Planning (ns/mutation) | 3,982.9 | 3,702.7 | 0.930x |
| Route (ns/mutation) | 1,699.3 | 1,609.9 | 0.947x |
| Non-route planning (ns/mutation) | 2,287.6 | 2,095.8 | 0.916x |

## Classification and remaining planner cost

Classification: **inconclusive** under the requested criteria. Non-route
planning improved to `0.829x` base and throughput remained at `1.000x` base,
meeting the inconclusive threshold. Width-16 throughput was `1.137x` base.
The useful-success threshold (`non-route <= 0.80x` and `total <= 0.90x`) was
not met because non-route planning remained above `0.80x`. This is not a
rejected candidate, so no revert was made.

At width 1, planning still costs 2.289 us/mutation: 1.104 us in routing and
1.193 us in non-route work. The remaining non-route time includes mutation
cloning, dependency and leaf-group bookkeeping, and the required ownership in
`last_key_writer`, `PlannedMutation.encoded_key`, and `mutated_key_set`; current
instrumentation does not split those costs further. This task did not alter
those representations.

## Artifacts and commits

Raw JSONL artifacts and their SHA256 values:

| Artifact | SHA256 |
| --- | --- |
| [`planned-base-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/planner-key-copies/planned-base-width1.jsonl) | `b77e76f8b9bb111e7059e6b19570343056ed96bd5a66ad42dbfd606c825884a1` |
| [`planned-candidate-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/planner-key-copies/planned-candidate-width1.jsonl) | `df0436330deb72bad5e45ddfee13eb73d359cb3f8910cec4ef78f3cef37e6575` |
| [`planned-base-width16.jsonl`](results/oci-a1-2ocpu-12g-200g/planner-key-copies/planned-base-width16.jsonl) | `05a16af9d6c76d2e6e63f7e4bab2fa8484e5cb59e751eb426c87fb45c0359c2d` |
| [`planned-candidate-width16.jsonl`](results/oci-a1-2ocpu-12g-200g/planner-key-copies/planned-candidate-width16.jsonl) | `8de2bafc4ddf075e25dc3ea69bcebb093558677a3a7ddd8e5945a56b1c3acb94` |

The same files and checksums are backed up at
`/home/opc/dodb-oci-artifacts-planner-key-copies-02ffdeb/`; repository and
backup checksums matched.

The implementation was committed and pushed as
`02ffdeb6a574ff35c247b2507c954f529218d3cd`. No revert commit was needed.
