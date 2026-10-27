# Query, Get, Put, and Conditional Put Comparison

## Question and workload

Measure whether B-link or the DODB-contract RocksDB adapter handles an ordered
sort-key query mixed with point reads and writes more effectively. This is a
single-tenant Product/Engine comparison. It does not measure a maximum-tuned
RocksDB configuration or decide engine adoption.

Each deterministic schedule contains 1,000 top-level service calls:

| Operation | Share | Semantics |
| --- | ---: | --- |
| Query16 | 40% | Ordered Query with a limit of 16 adjacent sort-key rows; partition and cursor advance through the seeded key range. The current 10,000-row key layout has ten sort keys per partition, so a request can return at most ten rows. |
| Get | 30% | Point read of an existing seeded row. |
| Put | 15% | Unconditional overwrite of an existing row with a changing 512-byte value. |
| Conditional Put | 15% | One-key transaction with an Exists condition and a Put to an existing row. |

All operations use the same tenant, deterministic seed, initial rows, client
count, and fixed per-client schedule on both backends. The conditional request
must succeed. Query rows are checked for count, order, partition membership,
and value length. Concurrent writes update a disjoint seeded-key range from
the point-read range. Every latency category is recorded separately, including Query16,
ordinary Get, Put, conditional Put, and aggregate workflow.

## Conditions

- B-link and `minimal-native` RocksDB from the same frozen source revision and
  optimized binaries built for the authorized OCI AArch64 host.
- Sixteen dedicated client workers; 10,000 seeded rows; 512-byte values;
  seed `20261013`; 10-second warmup; real synchronous WAL.
- Keep each backend's normal service semantics, revision metadata, exclusive
  conditional key locking, and final/reopen state validation.
- Do not change admission, WAL/checkpoint policy, RocksDB tuning, or workload
  between backends. Run unprofiled.
- Run one eight-second correctness/resource preflight per backend, then three
  300-second repetitions per backend. Rotate backend order by repetition and
  compare fixed schedule fingerprints, not throughput-dependent operation
  totals.

## Acceptance and reporting

Reject cells with unexpected errors, failed conditions, incomplete latency
counts, query-order/value mismatches, fingerprint mismatch, an operation share
more than one percentage point from the target, incomplete final or reopen
validation, or resource guard violations. B-link sustained cells require at
least ten completed checkpoints and acknowledged-write overlap. Preserve all
failed attempts.
Use the existing 3 GiB process-group RSS, 4 GiB per-database, 1,500 MiB host
memory, and 10 GiB free benchmark-filesystem guards. Stop before running a cell
if the benchmark filesystem has less than 10 GiB free. Preserve each completed
database in a verified lossless archive before reclaiming its task-owned
database directory; never overlap archiving with measurement.

Report per-run values and medians/ranges for total API throughput; successful
operations by type; Query16, Get, Put, conditional Put, and workflow p50/p95/p99;
process-group RSS; database size; and reopen/oracle duration. State whether the
result is preliminary or sustained. No defaults change as part of this test.
