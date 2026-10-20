# DODB RocksDB Baseline Results

## Selected performance comparison

The authoritative OCI comparison used performance source `ee6cd94f1a4e7b2bab7af603bc8983bbf8184542` and frozen release binaries. The 111-case 30-second matrix and the approved representative 21-case five-minute subset passed their duration, final/reopen, and cleanup checks. Values below are five-minute medians for the selected configurations, not an absolute maximum or universal winner.

| Workload and path | Median throughput | Scope |
| --- | ---: | --- |
| A: `rw20`, 16 clients, B-link | 4,338.07 transactions/s | Service API and validation |
| A: `rw20`, 16 clients, RocksDB service Shared-16 | 4,753.17 transactions/s | Actual DodbService path |
| A: `rw20`, 16 clients, synchronous RocksDB core, 16 callers and driver permits | 5,241.32 transactions/s | Full synchronous DODB semantics |
| E: unconditional width-16, raw RocksDB | 2,722.44 batches/s | Narrow reference; no DODB revisions, OCC, ABA, or condition checks |
| E: unconditional width-16, synchronous RocksDB core, 64 direct callers | 1,754.96 batches/s | Full DODB revision and final/reopen path; no conditional requests |
| E: unconditional width-16, RocksDB service Shared-32 | 1,565.34 batches/s | Actual service path; no conditional requests |
| E: unconditional width-16, B-link | 2,787.81 batches/s | Service path and final/reopen validation |

The results show that required API and service work affects each workload differently. Raw RocksDB is a narrower reference, not an equivalent DODB API implementation. The measured Shared mode is opt-in; Exclusive remains the production default. See [the full five-minute tables and limits](condition-shared-lock-20261019/engine-health/FIVE-MINUTE-RESULTS.md) and [the five-minute resource budget](condition-shared-lock-20261019/engine-health/FIVE-MINUTE-RESOURCE-BUDGET.md).

## Separate memory-only diagnostic

A seven-path memory diagnostic used a separate loopback parent/backend-worker process boundary and source `843f0ea1598a4fa87284d1e3ee421612f288b4c2`. It is not part of the throughput table. Its maximum one-second process-group RSS was 1,536.2 MiB (E/B-link); minimum host `MemAvailable` was 3.25 GiB; maximum live database was 1.42 GiB. Every final/reopen oracle passed. Each second bounded full Scan after a cache-warming Scan had zero `/dev/sda4` physical-read delta and zero backend Swap at its boundaries. Those observations apply only to the seven diagnostic scans and do not retroactively prove five-minute residency. The diagnostic recorded a 32 MiB native RocksDB block-cache capacity per database plus live-data/SST properties, while host ZFS ARC remained shared and cannot be counted as a process-dedicated cache.

The parent/backend split is process-level: backend RSS includes service adapter/runtime and loopback transport, not only native RocksDB allocations. RocksDB Get checks the encoded length, protocol value-size limit, and response-frame budget before copying a present value into the Rust result vector; the native read buffer is still allocated. These limits are preserved in the [final comparison audit](FINAL-AUDIT-DRAFT.md) and [memory-only diagnostic report](condition-shared-lock-20261019/engine-health/MEMORY-DIAGNOSTIC-RESULTS.md).

The selected 21 five-minute definitions, rather than the other 90 definitions in the original broad matrix, are the approved representative confirmation. Those other definitions were not measured at five minutes. The memory-exceed axis remains deferred by the user, and no production default was changed.
