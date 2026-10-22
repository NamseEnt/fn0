# Read-80/write-20 comparison

## Question and scope

Measure the existing DODB-contract RocksDB and B-link paths with 80% read API calls and 20% write API calls. This is a short product/engine workload comparison, not an engine optimization or a proof of maximum engine performance.

Reuse the exact implementation and frozen service executable from the accepted background-checkpoint matrix: source `d240416f0e0efafa7c08bb1e868387ebd0d894bc`, executable SHA256 `09bd505d77ce0bcbc61ca46f7389373e60bc1ab1bf6b80a90077bed328418e40`. The implementation checkout must remain clean. Record the separate controller commit and script hashes. Do not rebuild or change production code, defaults, durability, locks, admission, revision allocation, or cache configuration.

## Matrix

Use the authorized OCI A1 host `opc@217.142.246.204`, ZFS dataset `dodbbench/db` at `/bench/zfs/db`, synchronous WAL, 16 dedicated clients, 10,000 initial rows, 512-byte values, seed 20261013, overwrite slots 1024/client, reserved-range revisions, exclusive condition locks, and the existing width-1/4/8/16 write mixture. Reads retain the existing Get/Query16/preparatory-Get mixture. Only `--scenario rw20` changes to `--scenario rw80` relative to the preceding mixed-workload commands; new database/report paths are allocated separately.

Compare all five existing paths: minimal-native RocksDB, legacy sync 128 MiB, segmented sync 128 MiB, segmented background 32 MiB, and segmented background 128 MiB. Segmented WAL files remain 32 MiB; encoded-image and retained-WAL budgets remain 512 MiB.

First run an 8-second/1-second-warmup preflight on RocksDB and segmented sync/background 128 MiB. Then run each of the five paths three times with 10-second warmup and 30-second measurement, rotating path order between repetitions: 15 measured cases, approximately 12 minutes plus preparation and evidence collection. Preflight results are separate and never enter the headline medians.

## Acceptance and reporting

Require zero request errors/overloads, final-state and reopen validation, the full measurement window, exact latency count/sample-count/histogram agreement, matching fixed input-prefix fingerprints across paths, 16 callers, and a measured read fraction within 0.5 percentage points of 80%. Preserve raw output, exact commands, source/binary/controller provenance, resource samples, database hashes, and RocksDB logs. Preserve any failed attempt and its database; stop instead of tuning away a failure. Accepted cases are resumable and must not be rerun merely to repair reporting.

Retain the preceding guards: RSS below 3 GiB, database below 4 GiB, available memory at least 1500 MiB, free filesystem space at least 10 GiB, and 150-second child timeout. Require successful checkpoint completion/drain, no omitted events, no remaining tasks/images, and effective layout/execution matching the command. Record measured checkpoint counts and acknowledged overlap; do not force three checkpoints or lower a threshold to create checkpoint stress in this read-heavy workload. A cell with no checkpoint cannot establish checkpoint-overlap performance.

Report medians and repetition ranges for completed API calls/s, read calls/s, successful write transactions/s, workflow p50/p95/p99, and separate Get/Query16/write-success latency. Include RSS, WAL/database size, observed read fraction, checkpoint counts, and overlap. Throughput includes final client completion and checkpoint drain. Percentiles remain per-run percentiles followed by a median, not a pooled percentile.

No common complete-engine memory budget, steady-state conclusion, production adoption decision, or universal B-link/LSM ranking is implied. The RocksDB path directly uses TransactionDB while preserving conditions, revisions, and ABA; it is not raw WriteBatch. Save the result and evidence on the existing experiment branch and push task-owned files only.
