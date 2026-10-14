# RocksDB Revision Range Reservation Experiment

## Result and scope

This is a one-run-per-condition service workload comparison between the
serialized revision counter and the opt-in 4,096-token range allocator. The
default remains `serialized`. The experiment changes only RocksDB revision
allocation; service requests, responses, user-key locks, workload generation,
and the eight-operation blocking limit remain unchanged. These short synthetic
runs do not establish a stable performance improvement or represent production
traffic.

The implementation verification SHA is
`d9c5e01e239abf5e203cbdc7379a0beb8c5345b8`. The same SHA was used for both
allocator modes and all ten measurements. The workload binary SHA-256 was
`1b362e58777b205be30008a163b850fbd96dd56db5f17ed1f43773fc82c5db81` before
and after every run. The later documentation commit is separate from this
implementation SHA.

## Allocation, identity, and recovery

The serialized mode is the existing behavior: after user conditions pass, a
user transaction locks and advances tenant metadata key `00 01` in the same
synced transaction as its mutations. In range mode, the user transaction first
locks every condition and mutation key in the existing canonical order and
checks its conditions. It then takes a short tenant-local allocator mutex. If
the in-memory range is empty, an internal TransactionDB transaction locks only
metadata key `00 01`, reserves the next 4,096 values, and commits with WAL
enabled and `sync=true`. The candidate values become usable only after that
commit succeeds. The mutex is released before constructing and syncing the
user transaction.

The counter contains the last committed revision in serialized mode and the
reservation high-water mark in range mode. There are no counter readers beyond
revision allocation. Existing eight-byte metadata is retained without adding a
format marker. A pre-existing serialized database reserves strictly above its
stored counter when range mode first opens it. Switching back to serialized
mode advances above the reserved high-water mark. Reopen abandons the unused
tail of a range; it is never reissued. This also makes gaps from failed or
rolled-back writes acceptable. No numeric continuity promise is made.

Allocation follows user-key lock acquisition and condition evaluation. A
same-key transaction cannot get a revision until the prior holder commits or
rolls back, including when the key appears only as a condition. All mutations
within one transaction retain the same revision. Condition-only requests still
return no commit identity and allocate no revision; rejected conditions do not
mutate data or consume a revision. Independent-key transactions can complete
in a different order from their allocated identities. Their operations
commute, and those identities define a valid logical replay order; response
completion and RocksDB sequence numbers are not used as identities. The
`Revision` and `Lsn` are distinct newtypes over `u64`; RocksDB continues to map
the same allocated integer to both fields, with `TransactionResult.commit_lsn`
typed as `Option<Lsn>` and condition-only success returning `None`. The service
contract requires identity and per-key equality, not a dense number sequence or
response completion order. The existing workload replays mutations by the
returned identity and checks the independent final state.

The reservation transaction never locks user keys, so it cannot form a lock
cycle with a transaction holding user keys while requesting a range. The
allocator mutex can cover the internal reservation sync, but is not held over
the user commit. If a reservation commit returns an error, that candidate range
is not installed and the current write fails. A later transaction can safely
retry by locking and reading the persisted counter; this experiment does not
guess an uncertain high-water mark. Overflow and malformed counter metadata
fail closed before any token is returned.

## Correctness checks

Local checks used the existing pinned RocksDB archive and passed:

- `cargo check --locked --offline -p dodb-rocksdb --bin service_workload --tests`
- `cargo fmt --package dodb-rocksdb -- --check`
- RocksDB unit tests: 9 passed, including barrier-synchronized concurrent
  assignment, small-range exhaustion, abandoned issued tokens, partial-range
  shutdown/reopen, existing serialized database transition, overflow, and
  malformed metadata failure.
- Shared transaction contract: passed on both RocksDB allocator modes, along
  with native and B-link.
- Shared read contract: 2 passed.
- Research metrics checks: 4 passed, including zero user-transaction counter
  accesses in range mode and separately counted user and reservation commits.

OCI Release focused checks passed 16/16: 9 unit tests, 2 read-contract tests,
4 metrics tests, and the shared transaction-contract test. The common
transaction contract creates separate databases for the two allocator modes.
The two OCI smoke runs also passed independent state validation and reopen
validation. All ten measured reports passed state validation, shutdown/reopen
validation, and exact paired first-100 workflow fingerprint comparison.

One initial local unit-test attempt failed four allocator cases because the
empty-range sentinel returned revision zero. The allocator now recognizes the
uninitialized state before assignment, and the complete local and OCI suites
passed afterward. OCI had no failed runs or retries. No process-kill or
power-loss experiment was added. Uncertain reservation-commit behavior was not
fault-injected; only overflow and corrupt metadata failures were directly
tested. A post-allocation user-transaction commit failure was also not
fault-injected; the focused check discards issued tokens and verifies they are
not returned after reopen.

## OCI workload results

Each workload condition ran once. Single-tenant profiles used 16 clients and
10,000 initial rows; MT-MIX used four clients on each of four tenants and
10,000 rows per tenant. All used 512-byte values, seed `20261013`, 1 second of
warmup, 5 seconds of admission, the existing deadline/drain/reopen checks, and
`--research-metrics 1` for both modes. Input generation fingerprints matched
between allocator modes for every scenario. The actual ratios below include
preparatory Gets. The success rate denominator is the existing measured
admission-plus-drain duration.

| Scenario | Actual read/write serialized → ranges | Successful write tx/s serialized → ranges | Successful-write p95/p99 ms serialized → ranges | Workflow p95/p99 ms serialized → ranges |
| --- | ---: | ---: | ---: | ---: |
| RW95 | 95.03/4.97% → 95.03/4.97% | 885 → 787 (-11.2%) | 13.68/21.93 → 5.72/9.33 | 2.10/6.80 → 2.16/4.28 |
| RW80 | 80.63/19.37% → 80.17/19.83% | 925 → 2,231 (+141.3%) | 21.40/32.23 → 8.80/17.88 | 11.28/21.19 → 3.84/9.33 |
| RW50 | 49.23/50.77% → 49.67/50.33% | 979 → 2,523 (+157.7%) | 23.87/36.69 → 9.31/18.07 | 20.58/32.31 → 7.29/14.93 |
| RW20 | 19.56/80.44% → 20.12/79.88% | 980 → 2,601 (+165.5%) | 29.03/46.39 → 10.67/19.12 | 29.65/46.10 → 11.27/18.51 |
| MT-MIX | 71.25/28.75% → 69.08/30.92% | 2,042 → 2,842 (+39.2%) | 7.34/10.22 → 4.81/7.18 | 3.03/4.57 → 2.56/3.77 |

These are one-shot observations, not causal or statistical claims. RW95 had
lower successful-write throughput despite lower write tails. The range runs
admitted different numbers of requests because their execution rates differed;
actual mixes and all request counts remain in `condition-summary.csv`. The
driver's latency samples are bounded at 20,000 observations; successful-write
samples remained below that cap in these runs.

Condition rejection counts were 387/357 for RW95, 346/828 for RW80,
235/824 for RW50, 481/972 for RW20, and 777/1,090 for MT-MIX (serialized/range).
Natural conflicts were 0/0, 0/0, 0/0, 0/0, and 4/9 respectively. Other errors
and overloads were zero in every run. The MT-MIX conflict count change is small
and reflects a single run with more admitted writes, not a demonstrated
regression.

The serialized measurements observed one user transaction counter access per
successful write: 4,428, 4,630, 4,905, 4,911, and 10,217 by scenario. In range
mode, measured user transactions accessed the counter zero times. There were
1, 3, 3, 3, and 2 counter-only reservation transactions during measured
admission, plus 3 reservations per seeded tenant during setup (3 for each
single-tenant run and 12 for MT-MIX). Warmup made no additional reservations.
The measured reservation sync mean was 1.29, 1.62, 1.48, 1.50, and 2.15 ms by
scenario; approximate p95 values were 2.10, 4.19, 2.10, 2.10, and 4.19 ms.
Those measured refills run synchronously inside the service request and are
included in that request's latency. Per-request allocator mutex wait averaged
0.00039, 0.00092, 0.00101, 0.00105, and 0.00032 ms. During the measured phase,
the serialized counter lock-wait means were 2.99, 4.43, 4.58, 4.36, and 1.65 ms
per successful write; the range mode removed that user-transaction wait.

The fixed 10,000-row seed setup requires three durable reservations per tenant
in range mode. The existing workload deliberately excludes seed setup from
admission latency and throughput. Its full `seed_setup_ms` remains in every raw
report and `condition-summary.csv`; research histograms are disabled during
seed setup, so its three reservation sync durations per tenant are not isolated
from total seed time. The first 3,000? No: the range size is 4,096, so each
tenant's 10,000 seed writes reserve ranges of 4,096, 4,096, and 4,096 values,
leaving 2,288 unused tokens at setup completion.

Read semaphore wait means (serialized → ranges) were 0.195 → 0.200 ms for
RW95, 0.840 → 0.344 ms for RW80, 2.582 → 0.969 ms for RW50, 4.926 → 1.827 ms
for RW20, and 0.596 → 0.443 ms across MT-MIX tenants. User-key lock waits
remain because condition and mutation keys are still protected. No new
overload or infrastructure error became prominent. The table's one RW95
throughput decrease and the read wait changes do not support changing the
default.

The earlier revision-counter diagnostic was a separate SHA and one-run
attribution study. Its B-link results were not rerun and are not part of this
allocator comparison.

## Run accounting, provenance, and artifacts

| OCI phase | Planned | Executed | Passed | Failed | Not reached |
| --- | ---: | ---: | ---: | ---: | ---: |
| Focused Release tests | 16 | 16 | 16 | 0 | 0 |
| Allocator smoke | 2 | 2 | 2 | 0 | 0 |
| Performance matrix | 10 | 10 | 10 | 0 | 0 |

The workload build and focused-test helper runs reused the Rust 1.97.1 shared
Cargo target, `+crc` target setting, shared sccache, and existing RocksDB 11.8.1
archive. The host was `instance-20260923-1013`, ARM64 Linux 6.12, two CPUs, and
ZFS `/bench/zfs/db`. Full provenance and helper output are in
[`oci/host-and-source.txt`](oci/host-and-source.txt) and
`oci/evidence/build-runs/`. Per-run commands, logs, reports, status files, and
before/after binary hashes are under `oci/measure/` and `oci/smoke/`. Fresh
per-run databases remain on OCI at
`/bench/zfs/db/dodb-revision-range-runs/d9c5e01e/`; they were not copied into
this repository.

The exact implementation verification SHA used to build and measure was
`d9c5e01e239abf5e203cbdc7379a0beb8c5345b8`. The report and evidence are
committed separately. The serialized allocator remains the default; this
single-repetition result is not sufficient to promote range mode.
