# Durable dodb vs Turso vs RocksDB on OCI A1 with OpenZFS

## Provenance correction (added 2026-09-25)

Every dodb result labelled "Planned" or "dodb Planned" in this document is **actually experiment main-btree due missing --engine argument**. It is not the planned Blink engine.

- The core matrix reused `../oci-a1-2ocpu-12g-zfs-real-sync/raw/*-planned-*.jsonl`. All 42 of those rows record `"engine":"main-btree"`, because that runner never passed `--engine` and the `phase0-bench` default is `main-btree`.
- The two sustained runs (`raw/sustained-dodb-planned-*.jsonl`) were started by `scripts/run_crossdb_matrix.py` without `--engine`, and both rows also record `"engine":"main-btree"`.
- The following figures therefore describe main-btree built from the experiment source, not planned-blink: Planned / RocksDB 0.221×, Planned / Turso WAL 2.733×, Planned / Turso MVCC 4.532×, about 8.4 KB of WAL per transaction, 2,883 and 3,944 sustained tx/s, and WAL/RSS growth to 8–9 GB.
- The Turso and RocksDB results are unaffected, and so are the workload equivalence, syscall verification and harness evidence for those engines.
- The numbers below are kept unchanged. The true planned-blink durable baseline and its comparison with these same Turso/RocksDB rows is in `planned-blink-durable-baseline-results.md`.

This compares the durable dodb Planned result with Turso (WAL and MVCC with group commit) and RocksDB on the same host, pool and dataset, with every successful write returned only after its log was synchronized. It is a durable-write and concurrency comparison on a memory-resident working set. It is not a larger-than-memory benchmark.

## Environment

- OCI A1, 2 OCPU (2 logical CPUs, AArch64 Neoverse-N1), 12 GiB configured RAM (10,898 MiB visible), Oracle Linux Server 9.8, kernel `6.12.0-206.104.4.4.el9uek.aarch64`.
- OpenZFS 2.2.11, pool `dodbbench` (single vdev on the OCI boot block volume, `ashift=12`), dataset `/bench/zfs/db` with `sync=standard`, `recordsize=4K`, `compression=off`, `atime=off`. Unchanged from the real-sync run; `environment.txt` has the values read back after this run.
- Every database file, WAL, log and SST lived under `/bench/zfs/db/<engine>/` (the harness refuses any other data directory). Each run started from an empty directory, which was removed afterwards. Sources, builds and result files were on the root XFS volume; no database file was.
- Artifacts: `docs/experiments/results/oci-a1-2ocpu-12g-zfs-crossdb/` (harness sources in `harness/`, runner and analysis in `scripts/`, raw JSONL in `raw/`, generated tables in `tables.md` and `analysis.json`, checksums in `SHA256SUMS`).
- Measurement order: core matrix 12:55–13:22 UTC, sustained runs 13:23–13:51 UTC. Builds, probes and the verification tools ran only when no measurement was running.

## Durability contract

- A logical transaction counts as successful only when the database call returned success after its WAL or log was synchronized. No asynchronous or no-sync mode was measured.
- Width 1 is one mutation per logical transaction. Width 16 is one atomic transaction with exactly 16 mutations. The harness never merges independent transactions; each engine's own group commit was left on and used.
- Turso WAL: `PRAGMA journal_mode = wal`, `PRAGMA synchronous = FULL` (read back as 2), `BEGIN` + UPSERTs + `COMMIT`.
- Turso MVCC: `PRAGMA journal_mode = mvcc`, `PRAGMA synchronous = FULL` (read back as 2), `PRAGMA mvcc_group_commit = on` (read back as 1), `BEGIN CONCURRENT` + UPSERTs + `COMMIT`. The control run used `mvcc_group_commit = off` (read back as 0).
- RocksDB: one `WriteBatch` per logical transaction, `DB::Write` with `WriteOptions.sync = true`, `disableWAL = false`, default `use_fsync = false` (fdatasync). Primary is the default write path; `enable_pipelined_write = true` is reported separately.
- The journal mode, `synchronous` and `mvcc_group_commit` values were checked on every writer connection, and the run aborts if any value differs.

## Versions

| Item | Value |
|---|---|
| dodb Planned | production source `0d310da32af5dfaa9df6469788998dacccebb841` (no crate change up to `fe16a0f5929a9d2c9bdf6d9d5a1869330d119bd0`); binary SHA256 `79122436e330be122059f8a4bf5908406e55b74596bfab2ee3cf745dd17fb6cb` |
| Turso | tag `v0.8.0-pre.13` (tag object `900afeaae972952aaa8c21c72aeb4e5466a43727`), commit `64b8ef5742fc18937f9c89806c81e3f6475dc7a3` |
| RocksDB | tag `v11.8.1`, commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654` |
| Rust | `rustc 1.98.1 (48a229cea 2026-09-01)`, `cargo 1.98.1` (same toolchain as the dodb builds) |
| C++ | `gcc (GCC) 11.5.0 20240719 (Red Hat 11.5.0-14.0.1)` |

- Turso: when the benchmark started, the latest stable release was `v0.7.2` (2026-07-30), and that tag has no `mvcc_group_commit` pragma, so it cannot run the required MVCC group-commit comparator. The newest release was the pre-release `v0.8.0-pre.13` (published 2026-09-25 08:17 UTC); that tag was pinned. `main` was not used.
- Turso MVCC status in the upstream docs at this tag: the manual's limitations section says "MVCC is a supported journal mode", while the journal-mode table describes `mvcc` as "**Note:** the feature is not production ready so do not use it for critical data right now." The docs call `mvcc_group_commit` "Off by default" and say it lets concurrent MVCC commits batch their logical-log writes and share a single fsync. The Turso MVCC numbers below are an experimental-feature result on a pre-release. Quotes are in `turso-version.txt`.
- RocksDB `11.8.1` was the GitHub "Latest" release (2026-08-07) when the benchmark started.

## Build provenance

- Turso: the `turso` Rust crate (`bindings/rust`) from the pinned tree, linked into the benchmark process as a path dependency. There is one process per run, with one persistent connection per writer thread, and no CLI process is started. Release profile copies upstream's (`opt-level=3`, `codegen-units=4`, `lto=thin`, `panic=abort`), with `RUSTFLAGS=-C target-cpu=native`, default crate features (mimalloc global allocator, fts), and the default I/O backend (`PlatformIO`, the syscall backend on Linux). Binary SHA256 and harness source hashes are in `turso-binary-proof.txt`; the full verbose build is `turso-build.log`.
- RocksDB: `make static_lib DEBUG_LEVEL=0 PORTABLE=0` (AArch64 `-march=armv8-a+crc+crypto -O2 -DNDEBUG`). The first attempt stopped on a `-Werror` unused-parameter warning in `util/compression.cc` (the code path used when snappy is absent). The final library comes from `make clean` and a full rebuild with the official `DISABLE_WARNING_AS_ERROR=1` switch and otherwise unchanged flags. No RocksDB source was changed. Only zlib and zstd were detected, and none were used because compression is off. `librocksdb.a` SHA256 `b0d15c3de73506ee286e667a2356ddedf72b974a3a7bec71d9ad568dec624ece`. The benchmark reaches the C++ API through a small in-process shim (`harness/rocksdb-bench/shim/rocksdb_shim.cc`, `-O2 -fno-rtti`, same `-march`). Because the static library is not PIC, the harness binary is linked as a non-PIE executable (`-C relocation-model=static`). Everything is in `rocksdb-build.log` (it keeps the failed first attempt and its 254 `-Werror` compile lines) and `rocksdb-binary-proof.txt`.
- dodb: the core matrix reuses the existing real-sync raw results in `../oci-a1-2ocpu-12g-zfs-real-sync/raw/*-planned-*.jsonl` and does not rerun dodb. The crossdb runs use the same seeds, working set, key and value size, fresh database per run, 2 s warmup and 5 s measurement, so no dodb confirmation rerun was needed. The sustained test needed 10-second windows, which `phase0-bench` does not export. It ran a separate build of the same Planned source with `dodb-sustained-window.patch`: 51 lines added to the benchmark binary `phase0-bench.rs` only (per-transaction finish time and latency, bucketed into `window_NN_*` fields). The storage library is unchanged: `diff -rq` against the Planned tree reports only that file. Binary SHA256 `fba986d4dea7d306b4fb814e4cd82c468f427572362e95a9c1e06e5525304999`; see `dodb-binary-proof.txt` and `dodb-window-build.log`.
- No production dodb source was changed.

## Syscall durability verification

Each probe opened a fresh database under `/bench/zfs/db`, committed three width-1 transactions and one width-16 transaction, closed cleanly, reopened, and checked all 19 values and the row count. Marker `faccessat` calls were placed just before each commit call and just after it returned. The traces used `strace -f -tt -y -e trace=fsync,fdatasync,msync,sync_file_range,faccessat,faccessat2`.

| Engine | Verdict | Sync seen between commit start and return | Reopen check |
|---|---|---|---|
| Turso WAL | VERIFIED | 4/4 commits: `fsync` returned 0 on `kv.db-wal` | 19/19 values, 19 rows |
| Turso MVCC (group commit on) | VERIFIED | 4/4 commits: `fsync` returned 0 on `kv.db-log` | 19/19 values, 19 rows |
| RocksDB | VERIFIED | 4/4 commits: `fdatasync` returned 0 on `000004.log` (first commit also `fsync` on the directory) | 19/19 values, 19 rows |

- On Linux, Turso's sync is `libc::fsync` (`core/io/unix.rs`; `FullFsync` has no effect off Apple platforms).
- The probes use one writer, which exercises the solo commit path. The multiwriter group-commit path was checked separately (see Group-commit observations).
- dodb's durability evidence is unchanged from the real-sync run (`../oci-a1-2ocpu-12g-zfs-real-sync/sync-strace.txt`; `fdatasync` on the WAL before responses, plus positive WAL sync counters in every run).
- Files: `turso-wal-strace.txt`, `turso-mvcc-strace.txt`, `rocksdb-strace.txt` (verdict, each transaction, and the ordered marker/sync lines), the `*.raw` traces, and the `probe-*.jsonl` records.
- No mode was marked DURABILITY CONTRACT NOT VERIFIED.

## Workload equivalence

- `harness/common/workload.rs` is a port of the dodb `phase0-bench` generator: same `splitmix64` writer state (`seed ^ writer_id * 0x9e3779b97f4a7c15`), the same index rules for uniform, compact locality (`same-leaf-heavy`) and spread locality (`different-leaf-heavy`), the same duplicate-key fallback, the same value bytes (`(operation + offset) & 0xff`, 64 bytes), and the same seed rows (all working-set keys, value `index & 0xff`).
- The DB key is the exact 16 logical key bytes, pk (8) followed by sk (8). dodb stores the same pair with its own internal length-prefixed encoding.
- Seeds match the dodb run: invocation seed `979000000 + scenario_index × 1009 + repetition_index`; warmup writer seed `seed ^ 0xaaaa0000 ^ 0x10000000`; measured writer seed `seed ^ 0xbbbb0000 ^ 0x10000000`. The analysis found 0 scenarios whose seed set differs from the dodb Planned run.
- `harness/workload-equivalence` builds the original generator by cutting its source out of `phase0-bench.rs` at the Planned commit (SHA256 `678e8c0b…`) and compiling it against `dodb-core`. Two checks:
  - Synthetic (`workload-equivalence-synthetic.txt`): for every scenario (14 core + 2 sustained), every repetition, both phases and every writer, the first 2,000 transactions from the original and the shared generator hash the same. That is 3,372 writer streams with 0 mismatches. The seed rows for all three distributions at 100,000 and 1,000,000 rows are identical.
  - Actual runs (`workload-equivalence-raw.txt`): each run records, per writer and phase, the number of generated transactions and a hash of exactly that sequence. The original generator recomputed all of them: 186 records, 13,008 writer streams, 13,170,086 generated transactions, 0 failures.
- Retried Turso transactions reuse the same generated mutations; the generator only moves forward when a new logical transaction starts.

## Harness and run policy

- One OS thread per writer. Each writer keeps its own Turso connection with prepared `BEGIN`/UPSERT/`COMMIT` statements and a single-threaded Tokio runtime, the same model as upstream `perf/throughput`. RocksDB writers are threads that call `DB::Write` on one shared DB. dodb (from the earlier run) used its async coordinator with 2 Tokio workers.
- Lifecycle per run: fresh directory → open → seed all working-set rows (untimed; 1,000-row transactions) → 2 s warmup → 5 s measurement → clean close → reopen → verification → remove the directory. Throughput is successful transactions divided by the measured wall time, including completions after the deadline, as in dodb.
- Retry policy, fixed before any run: a retryable error (Turso `Busy`, `BusySnapshot`, or a write-write conflict) triggers `ROLLBACK`, a sleep of `min(100 µs × 2^(n-1), 10 ms)` after failed attempt n, and another attempt, up to 16 attempts. After 16 the logical transaction is counted as abandoned. Non-retryable errors are counted and not retried. Turso `busy_timeout` stays at its default of 0, so every BUSY reaches the harness and is counted. Latency is measured from the first attempt to the final outcome.
- UPSERT: `INSERT INTO kv (k, v) VALUES (?1, ?2) ON CONFLICT (k) DO UPDATE SET v = excluded.v` on `CREATE TABLE kv (k BLOB PRIMARY KEY, v BLOB NOT NULL)`, with no secondary index and no `WITHOUT ROWID`.
- Memory settings, fixed before measuring: Turso `PRAGMA cache_size = -32768` (32 MiB) on every connection, which holds the whole seeded 100k-row database (about 12 MB). RocksDB 64 MiB LRU block cache, 4 KiB blocks, no compression; everything else is default (64 MiB write buffer, 2 write buffers, L0 triggers 4/20/36, `max_background_jobs = 2`). No setting was changed after results were seen.
- Run order: each scenario and repetition ran its engine list rotated by `scenario_index + repetition_index`, so no engine always ran first. The list was the three primary engines plus RocksDB pipelined, plus Turso MVCC GC-off for the four chosen scenarios. The sustained order rotated between the 16- and 64-writer groups. See `run-order.txt`.
- The MVCC GC-off control was fixed in advance to the four uniform multiwriter scenarios (0, 3, 6, 9). RocksDB pipelined ran the full matrix.

## Core 14-scenario matrix

Names: "compact locality" is dodb's `same-leaf-heavy` (all writers cycle over the same 64 keys), and "spread locality" is `different-leaf-heavy` (each writer walks its own range, offset by 1,009 keys). dodb values come from the reused real-sync run. Every value is the mean of three repetitions.

### Successful logical tx/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,534 | 1,116 | 573 | 10,034 | 142 | 10,011 |
| 1 | 16 | 1 | compact-locality | 3,851 | 1,493 | 372 | 10,913 | — | 9,910 |
| 2 | 16 | 1 | spread-locality | 4,005 | 1,584 | 3,354 | 10,614 | — | 9,875 |
| 3 | 16 | 16 | uniform | 499 | 320 | 476 | 5,723 | 125 | 7,631 |
| 4 | 16 | 16 | compact-locality | 2,149 | 997 | 441 | 6,827 | — | 7,918 |
| 5 | 16 | 16 | spread-locality | 1,871 | 1,001 | 330 | 7,126 | — | 7,986 |
| 6 | 64 | 1 | uniform | 5,289 | 941 | 324 | 26,254 | 57 | 30,302 |
| 7 | 64 | 1 | compact-locality | 8,116 | 2,160 | 269 | 27,378 | — | 29,707 |
| 8 | 64 | 1 | spread-locality | 6,768 | 1,334 | 7,231 | 26,852 | — | 29,888 |
| 9 | 64 | 16 | uniform | 612 | 280 | 289 | 9,573 | 54 | 8,923 |
| 10 | 64 | 16 | compact-locality | 2,747 | 911 | 225 | 11,601 | — | 10,789 |
| 11 | 64 | 16 | spread-locality | 1,980 | 1,045 | 695 | 11,351 | — | 11,205 |
| 12 | 1 | 1 | uniform | 1,083 | 1,197 | 1,291 | 1,512 | — | 1,563 |
| 13 | 1 | 16 | uniform | 280 | 311 | 877 | 1,353 | — | 1,345 |

### Successful mutation ops/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,534 | 1,116 | 573 | 10,034 | 142 | 10,011 |
| 1 | 16 | 1 | compact-locality | 3,851 | 1,493 | 372 | 10,913 | — | 9,910 |
| 2 | 16 | 1 | spread-locality | 4,005 | 1,584 | 3,354 | 10,614 | — | 9,875 |
| 3 | 16 | 16 | uniform | 7,984 | 5,125 | 7,624 | 91,576 | 2,007 | 122,090 |
| 4 | 16 | 16 | compact-locality | 34,377 | 15,951 | 7,063 | 109,224 | — | 126,695 |
| 5 | 16 | 16 | spread-locality | 29,930 | 16,011 | 5,281 | 114,010 | — | 127,780 |
| 6 | 64 | 1 | uniform | 5,289 | 941 | 324 | 26,254 | 57 | 30,302 |
| 7 | 64 | 1 | compact-locality | 8,116 | 2,160 | 269 | 27,378 | — | 29,707 |
| 8 | 64 | 1 | spread-locality | 6,768 | 1,334 | 7,231 | 26,852 | — | 29,888 |
| 9 | 64 | 16 | uniform | 9,795 | 4,486 | 4,623 | 153,162 | 867 | 142,770 |
| 10 | 64 | 16 | compact-locality | 43,955 | 14,569 | 3,601 | 185,614 | — | 172,618 |
| 11 | 64 | 16 | spread-locality | 31,678 | 16,716 | 11,114 | 181,616 | — | 179,274 |
| 12 | 1 | 1 | uniform | 1,083 | 1,197 | 1,291 | 1,512 | — | 1,563 |
| 13 | 1 | 16 | uniform | 4,477 | 4,970 | 14,038 | 21,645 | — | 21,520 |

### Attempted logical tx/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,535 | 1,263 | 573 | 10,034 | 142 | 10,011 |
| 1 | 16 | 1 | compact-locality | 3,852 | 1,640 | 381 | 10,913 | — | 9,910 |
| 2 | 16 | 1 | spread-locality | 4,005 | 1,732 | 3,354 | 10,614 | — | 9,875 |
| 3 | 16 | 16 | uniform | 499 | 463 | 476 | 5,723 | 126 | 7,631 |
| 4 | 16 | 16 | compact-locality | 2,149 | 1,136 | 486 | 6,827 | — | 7,918 |
| 5 | 16 | 16 | spread-locality | 1,871 | 1,137 | 330 | 7,126 | — | 7,986 |
| 6 | 64 | 1 | uniform | 5,289 | 1,535 | 324 | 26,254 | 57 | 30,302 |
| 7 | 64 | 1 | compact-locality | 8,118 | 2,740 | 433 | 27,378 | — | 29,707 |
| 8 | 64 | 1 | spread-locality | 6,769 | 1,927 | 7,231 | 26,852 | — | 29,888 |
| 9 | 64 | 16 | uniform | 612 | 896 | 290 | 9,573 | 54 | 8,923 |
| 10 | 64 | 16 | compact-locality | 2,747 | 1,490 | 493 | 11,601 | — | 10,789 |
| 11 | 64 | 16 | spread-locality | 1,980 | 1,617 | 695 | 11,351 | — | 11,205 |
| 12 | 1 | 1 | uniform | 1,083 | 1,197 | 1,291 | 1,512 | — | 1,563 |
| 13 | 1 | 16 | uniform | 280 | 311 | 877 | 1,353 | — | 1,345 |

- The single-writer rows (12, 13) are controls and stay out of every geometric mean.

## Latency

### Latency of successful transactions, p50 / p95 / p99 µs (mean of repetition percentiles)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 4,314 / 6,090 / 8,065 | 829 / 1,555 / 48,046 | 23,952 / 59,237 / 88,039 | 1,656 / 2,013 / 2,331 | 51,965 / 400,175 / 661,079 | 1,579 / 1,884 / 2,168 |
| 1 | 16 | 1 | compact-locality | 3,564 / 9,682 / 12,236 | 768 / 1,147 / 30,630 | 7,899 / 163,714 / 227,857 | 1,545 / 1,922 / 2,313 | — | 1,586 / 1,944 / 2,686 |
| 2 | 16 | 1 | spread-locality | 3,908 / 4,664 / 6,760 | 754 / 1,178 / 24,187 | 2,374 / 8,221 / 80,461 | 1,582 / 1,962 / 2,245 | — | 1,602 / 1,920 / 2,252 |
| 3 | 16 | 16 | uniform | 30,684 / 40,533 / 63,042 | 2,741 / 37,414 / 87,973 | 30,477 / 72,275 / 107,066 | 2,632 / 3,908 / 6,714 | 54,071 / 490,292 / 731,617 | 1,876 / 3,080 / 4,732 |
| 4 | 16 | 16 | compact-locality | 6,830 / 11,178 / 12,596 | 965 / 2,512 / 71,837 | 11,875 / 97,405 / 134,945 | 2,268 / 2,870 / 4,129 | — | 1,893 / 2,726 / 4,005 |
| 5 | 16 | 16 | spread-locality | 7,990 / 11,766 / 14,604 | 992 / 5,731 / 72,678 | 10,526 / 219,905 / 877,354 | 2,191 / 2,740 / 3,624 | — | 1,841 / 2,715 / 4,173 |
| 6 | 64 | 1 | uniform | 11,377 / 17,060 / 22,323 | 1,011 / 58,996 / 94,820 | 155,328 / 497,345 / 822,699 | 2,388 / 3,267 / 4,103 | 421,045 / 3,516,015 / 4,798,295 | 1,994 / 3,075 / 3,845 |
| 7 | 64 | 1 | compact-locality | 7,348 / 11,547 / 14,263 | 265 / 27,797 / 85,115 | 9,809 / 364,096 / 554,756 | 2,268 / 3,096 / 3,886 | — | 1,882 / 2,970 / 13,369 |
| 8 | 64 | 1 | spread-locality | 8,897 / 13,078 / 15,495 | 924 / 45,034 / 86,120 | 7,187 / 17,931 / 26,110 | 2,339 / 3,195 / 3,837 | — | 2,004 / 3,066 / 3,756 |
| 9 | 64 | 16 | uniform | 103,147 / 119,823 / 216,045 | 3,557 / 78,361 / 99,846 | 167,801 / 537,277 / 811,649 | 6,360 / 9,734 / 14,653 | 678,595 / 3,746,325 / 5,106,663 | 6,772 / 10,556 / 14,284 |
| 10 | 64 | 16 | compact-locality | 21,931 / 31,209 / 40,224 | 1,114 / 68,647 / 95,433 | 53,287 / 189,457 / 251,714 | 5,252 / 8,092 / 10,869 | — | 5,645 / 8,612 / 11,468 |
| 11 | 64 | 16 | spread-locality | 31,262 / 40,014 / 44,266 | 1,112 / 65,319 / 95,387 | 62,818 / 216,997 / 346,464 | 5,252 / 8,211 / 12,421 | — | 5,411 / 8,389 / 12,041 |
| 12 | 1 | 1 | uniform | 867 / 1,171 / 1,740 | 801 / 1,019 / 1,279 | 749 / 974 / 1,242 | 643 / 823 / 968 | — | 622 / 797 / 943 |
| 13 | 1 | 16 | uniform | 3,272 / 4,483 / 5,598 | 2,719 / 3,616 / 23,873 | 996 / 1,215 / 1,479 | 699 / 925 / 1,135 | — | 701 / 914 / 1,109 |

### CPU (% of one core, mean) and sampled peak RSS (MiB, max of repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 44% / 788 | 23% / 117 | 184% / 249 | 31% / 53 | 193% / 187 | 28% / 52 |
| 1 | 16 | 1 | compact-locality | 24% / 404 | 20% / 100 | 184% / 192 | 31% / 47 | — | 25% / 46 |
| 2 | 16 | 1 | spread-locality | 35% / 384 | 19% / 111 | 136% / 213 | 31% / 50 | — | 26% / 49 |
| 3 | 16 | 16 | uniform | 76% / 864 | 43% / 131 | 191% / 345 | 62% / 140 | 192% / 268 | 70% / 171 |
| 4 | 16 | 16 | compact-locality | 61% / 344 | 30% / 98 | 169% / 204 | 49% / 92 | — | 53% / 114 |
| 5 | 16 | 16 | spread-locality | 67% / 324 | 31% / 127 | 188% / 228 | 51% / 153 | — | 55% / 179 |
| 6 | 64 | 1 | uniform | 63% / 889 | 43% / 295 | 195% / 473 | 66% / 97 | 195% / 477 | 73% / 103 |
| 7 | 64 | 1 | compact-locality | 45% / 616 | 41% / 294 | 178% / 477 | 62% / 84 | — | 64% / 97 |
| 8 | 64 | 1 | spread-locality | 55% / 560 | 39% / 295 | 144% / 557 | 64% / 96 | — | 69% / 101 |
| 9 | 64 | 16 | uniform | 77% / 952 | 61% / 355 | 191% / 643 | 90% / 258 | 195% / 507 | 91% / 252 |
| 10 | 64 | 16 | compact-locality | 76% / 388 | 49% / 295 | 179% / 486 | 79% / 129 | — | 80% / 117 |
| 11 | 64 | 16 | spread-locality | 83% / 359 | 52% / 416 | 185% / 527 | 77% / 193 | — | 80% / 182 |
| 12 | 1 | 1 | uniform | 21% / 633 | 16% / 45 | 14% / 121 | 7% / 37 | — | 7% / 36 |
| 13 | 1 | 16 | uniform | 43% / 718 | 30% / 47 | 36% / 146 | 16% / 64 | — | 16% / 64 |

- CPU is the process's user+system CPU time during the measured interval, divided by the wall time (100% = one of the two cores). Peak RSS is sampled every 100 ms by the runner, as in the dodb run.
- Turso MVCC used 136–196% CPU in every multiwriter scenario, which means both cores were saturated. Its throughput here is limited by CPU on this 2-OCPU host, not by fsync.
- Turso WAL's low p50 and high p99 come from lock hand-off: a writer that gets the write lock commits in about one fsync, while writers that get BUSY back off and retry, and their successful attempts carry the waiting time.

## Conflict, busy and error behaviour

### Busy / conflict / retry / abandoned / error counts (sum of 3 measured intervals)

| # | Writers | Width | Distribution | Engine | Attempted tx | Committed tx | Busy | Busy snapshot | Conflicts | Retries | Abandoned | Errors | Verified |
|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | Turso WAL | 19,103 | 16,890 | 39,650 | 0 | 0 | 37,437 | 2,213 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-on | 9,110 | 9,110 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | RocksDB | 150,550 | 150,550 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-off | 2,174 | 2,174 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | RocksDB pipelined | 150,204 | 150,204 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | Turso WAL | 24,736 | 22,513 | 39,833 | 0 | 0 | 37,610 | 2,223 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | Turso MVCC GC-on | 5,863 | 5,716 | 7 | 0 | 4,357 | 4,217 | 147 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | RocksDB | 163,735 | 163,735 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | RocksDB pipelined | 148,704 | 148,704 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | Turso WAL | 26,142 | 23,921 | 39,769 | 0 | 0 | 37,548 | 2,221 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | Turso MVCC GC-on | 50,712 | 50,712 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | RocksDB | 159,249 | 159,249 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | RocksDB pipelined | 148,168 | 148,168 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | Turso WAL | 7,028 | 4,866 | 38,398 | 0 | 0 | 36,236 | 2,162 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-on | 7,156 | 7,156 | 0 | 0 | 417 | 417 | 0 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | RocksDB | 85,881 | 85,881 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-off | 1,928 | 1,926 | 0 | 0 | 34 | 32 | 2 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | RocksDB pipelined | 114,492 | 114,492 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | Turso WAL | 17,156 | 15,054 | 40,405 | 0 | 0 | 38,303 | 2,102 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | Turso MVCC GC-on | 7,344 | 6,674 | 0 | 4 | 31,316 | 30,650 | 670 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | RocksDB | 102,437 | 102,437 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | RocksDB pipelined | 118,815 | 118,815 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | Turso WAL | 17,158 | 15,107 | 40,989 | 0 | 0 | 38,938 | 2,051 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | Turso MVCC GC-on | 5,428 | 5,428 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | RocksDB | 106,917 | 106,917 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | RocksDB pipelined | 119,821 | 119,821 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | Turso WAL | 23,385 | 14,339 | 166,073 | 0 | 0 | 157,027 | 9,046 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-on | 4,883 | 4,883 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | RocksDB | 393,982 | 393,982 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-off | 1,006 | 1,006 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | RocksDB pipelined | 454,680 | 454,680 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | Turso WAL | 41,666 | 32,854 | 169,367 | 0 | 0 | 160,555 | 8,812 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | Turso MVCC GC-on | 7,226 | 4,484 | 0 | 74 | 49,342 | 46,674 | 2,742 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | RocksDB | 410,812 | 410,812 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | RocksDB pipelined | 445,739 | 445,739 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | Turso WAL | 29,327 | 20,306 | 167,377 | 0 | 0 | 158,356 | 9,021 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | Turso MVCC GC-on | 115,225 | 115,225 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | RocksDB | 403,339 | 403,339 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | RocksDB pipelined | 448,460 | 448,460 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | Turso WAL | 13,693 | 4,285 | 160,061 | 0 | 0 | 150,653 | 9,408 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-on | 4,541 | 4,527 | 0 | 0 | 977 | 963 | 14 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | RocksDB | 143,706 | 143,706 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-off | 934 | 932 | 0 | 0 | 32 | 30 | 2 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | RocksDB pipelined | 133,943 | 133,943 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | Turso WAL | 22,693 | 13,868 | 168,979 | 0 | 0 | 160,154 | 8,825 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | Turso MVCC GC-on | 7,571 | 3,453 | 0 | 30 | 86,392 | 82,304 | 4,118 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | RocksDB | 174,138 | 174,138 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | RocksDB pipelined | 161,970 | 161,970 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | Turso WAL | 24,665 | 15,936 | 169,551 | 0 | 0 | 160,822 | 8,729 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | Turso MVCC GC-on | 11,461 | 11,461 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | RocksDB | 170,488 | 170,488 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | RocksDB pipelined | 168,243 | 168,243 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | Turso WAL | 17,961 | 17,961 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | Turso MVCC GC-on | 19,373 | 19,373 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | RocksDB | 22,676 | 22,676 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | RocksDB pipelined | 23,441 | 23,441 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | Turso WAL | 4,689 | 4,689 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | Turso MVCC GC-on | 13,162 | 13,162 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | RocksDB | 20,295 | 20,295 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | RocksDB pipelined | 20,177 | 20,177 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |

- Turso WAL allows one writer at a time. With 16 writers, 8.5–12.3% of logical transactions were abandoned after 16 attempts at width 1 and 12.0–30.8% at width 16; with 64 writers, 21.1–38.7% at width 1 and 35.4–68.7% at width 16. Every BUSY was `database is locked`.
- Turso MVCC reported write-write conflicts (`Write-write conflict`, surfaced by the Rust binding as a generic error and classified from its message) only when writers touched the same keys: compact locality (all writers cycle over 64 keys) and uniform width 16. The spread and uniform width-1 scenarios had no conflicts. A few `BusySnapshot` / "Commit dependency aborted" errors happened in compact-locality runs and were retried. Up to 54% of compact-locality width-16 transactions at 64 writers were abandoned.
- RocksDB had no busy, conflict, retry or error in any run; `DB::Write` has no row-level conflicts.
- No run recorded a non-retryable error.

## Cross-database geometric means

Ratio = dodb Planned successful mutation ops/s ÷ comparator successful mutation ops/s. Above 1 means Planned was faster.

### Planned / comparator mutation throughput per multiwriter scenario

| # | Writers | Width | Distribution | Planned / Turso WAL | Planned / Turso MVCC GC-on | Planned / RocksDB | Planned / Turso MVCC GC-off | Planned / RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3.166x | 6.168x | 0.352x | 24.844x | 0.353x |
| 1 | 16 | 1 | compact-locality | 2.580x | 10.358x | 0.353x | — | 0.389x |
| 2 | 16 | 1 | spread-locality | 2.528x | 1.194x | 0.377x | — | 0.406x |
| 3 | 16 | 16 | uniform | 1.558x | 1.047x | 0.087x | 3.978x | 0.065x |
| 4 | 16 | 16 | compact-locality | 2.155x | 4.867x | 0.315x | — | 0.271x |
| 5 | 16 | 16 | spread-locality | 1.869x | 5.668x | 0.263x | — | 0.234x |
| 6 | 64 | 1 | uniform | 5.619x | 16.330x | 0.201x | 93.384x | 0.175x |
| 7 | 64 | 1 | compact-locality | 3.757x | 30.199x | 0.296x | — | 0.273x |
| 8 | 64 | 1 | spread-locality | 5.073x | 0.936x | 0.252x | — | 0.226x |
| 9 | 64 | 16 | uniform | 2.183x | 2.119x | 0.064x | 11.292x | 0.069x |
| 10 | 64 | 16 | compact-locality | 3.017x | 12.206x | 0.237x | — | 0.255x |
| 11 | 64 | 16 | spread-locality | 1.895x | 2.850x | 0.174x | — | 0.177x |

### Geometric means of Planned / comparator (primary comparators)

| Group | Planned / Turso WAL | Planned / Turso MVCC GC-on | Planned / RocksDB |
|---|---:|---:|---:|
| overall (12) | 2.733x (n=12) | 4.532x (n=12) | 0.221x (n=12) |
| width 1 | 3.610x (n=6) | 5.725x (n=6) | 0.298x (n=6) |
| width 16 | 2.069x (n=6) | 3.587x (n=6) | 0.163x (n=6) |
| writers 16 | 2.250x (n=6) | 3.608x (n=6) | 0.264x (n=6) |
| writers 64 | 3.319x (n=6) | 5.693x (n=6) | 0.185x (n=6) |
| uniform | 2.789x (n=4) | 3.867x (n=4) | 0.141x (n=4) |
| compact locality | 2.818x (n=4) | 11.676x (n=4) | 0.297x (n=4) |
| spread locality | 2.596x (n=4) | 2.061x (n=4) | 0.257x (n=4) |

### Secondary geometric means (not part of the primary headline)

| Group | Planned / RocksDB pipelined | Planned / Turso MVCC GC-off | Turso MVCC GC-on / GC-off | RocksDB default / pipelined |
|---|---:|---:|---:|---:|
| overall (12) | 0.211x (n=12) | 17.967x (n=4) | 4.647x (n=4) | 0.955x (n=12) |
| width 1 | 0.290x (n=6) | 48.167x (n=2) | 4.799x (n=2) | 0.973x (n=6) |
| width 16 | 0.153x (n=6) | 6.702x (n=2) | 4.499x (n=2) | 0.936x (n=6) |
| writers 16 | 0.248x (n=6) | 9.941x (n=2) | 3.911x (n=2) | 0.939x (n=6) |
| writers 64 | 0.179x (n=6) | 32.473x (n=2) | 5.521x (n=2) | 0.971x (n=6) |
| uniform | 0.129x (n=4) | 17.967x (n=4) | 4.647x (n=4) | 0.914x (n=4) |
| compact locality | 0.293x (n=4) | n/a (n=0) | n/a (n=0) | 0.985x (n=4) |
| spread locality | 0.248x (n=4) | n/a (n=0) | n/a (n=0) | 0.967x (n=4) |

- Headline over the 12 multiwriter scenarios: Planned / Turso WAL **2.733×**, Planned / Turso MVCC group commit **4.532×**, Planned / RocksDB **0.221×**. RocksDB delivered about 4.5× Planned's durable mutation throughput overall (geometric mean). Its lead was largest in uniform width 16 (about 11.5× at 16 writers and 15.6× at 64 writers) and smallest in spread-locality width 1 at 16 writers (about 2.7×).
- The GC-off and pipelined columns are secondary and stay out of the headline.

## Group-commit observations

### RocksDB write-path counters in the measured interval (mean of repetitions)

| # | Writers | Width | Distribution | Variant | WAL writes | WAL syncs | Writes/sync | Ingest MB | Flushes | Compactions | Compaction read MiB | Compaction write MiB | Stall s / delay+stop count | Max L0 files | Max pending compaction MiB |
|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | RocksDB | 49,667 | 5,769 | 8.70 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 0 | 16 | 1 | uniform | RocksDB pipelined | 49,667 | 6,206 | 8.07 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 1 | 16 | 1 | compact-locality | RocksDB | 54,000 | 6,041 | 9.03 | 4.4 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 1 | 16 | 1 | compact-locality | RocksDB pipelined | 49,000 | 6,106 | 8.12 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 2 | 16 | 1 | spread-locality | RocksDB | 52,667 | 5,910 | 8.98 | 4.3 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 2 | 16 | 1 | spread-locality | RocksDB pipelined | 48,667 | 6,117 | 8.07 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 3 | 16 | 16 | uniform | RocksDB | 28,000 | 3,437 | 8.33 | 36.3 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 3 | 16 | 16 | uniform | RocksDB pipelined | 37,667 | 4,776 | 7.99 | 48.4 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 4 | 16 | 16 | compact-locality | RocksDB | 33,333 | 4,100 | 8.33 | 43.3 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 4 | 16 | 16 | compact-locality | RocksDB pipelined | 39,333 | 4,911 | 8.07 | 50.2 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 5 | 16 | 16 | spread-locality | RocksDB | 35,000 | 4,319 | 8.25 | 45.2 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 5 | 16 | 16 | spread-locality | RocksDB pipelined | 39,333 | 4,940 | 8.09 | 50.6 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 6 | 64 | 1 | uniform | RocksDB | 131,000 | 3,954 | 33.21 | 10.4 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 6 | 64 | 1 | uniform | RocksDB pipelined | 150,667 | 4,833 | 31.37 | 12.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 7 | 64 | 1 | compact-locality | RocksDB | 136,333 | 4,081 | 33.56 | 10.9 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 7 | 64 | 1 | compact-locality | RocksDB pipelined | 148,333 | 4,728 | 31.46 | 11.8 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 8 | 64 | 1 | spread-locality | RocksDB | 133,667 | 4,033 | 33.34 | 10.7 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 8 | 64 | 1 | spread-locality | RocksDB pipelined | 148,667 | 4,802 | 31.14 | 11.9 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 9 | 64 | 16 | uniform | RocksDB | 47,333 | 1,457 | 32.87 | 60.7 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 9 | 64 | 16 | uniform | RocksDB pipelined | 44,000 | 2,396 | 18.94 | 56.6 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 10 | 64 | 16 | compact-locality | RocksDB | 57,333 | 1,787 | 32.49 | 73.5 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 10 | 64 | 16 | compact-locality | RocksDB pipelined | 53,667 | 2,368 | 22.83 | 68.4 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 11 | 64 | 16 | spread-locality | RocksDB | 56,333 | 1,762 | 32.26 | 72.0 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 11 | 64 | 16 | spread-locality | RocksDB pipelined | 55,667 | 2,365 | 23.72 | 71.1 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 12 | 1 | 1 | uniform | RocksDB | 7,559 | 7,559 | 1.00 | 0.7 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 12 | 1 | 1 | uniform | RocksDB pipelined | 7,814 | 7,814 | 1.00 | 0.7 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 13 | 1 | 16 | uniform | RocksDB | 6,765 | 6,765 | 1.00 | 8.6 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 13 | 1 | 16 | uniform | RocksDB pipelined | 6,726 | 6,726 | 1.00 | 8.6 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |

- RocksDB: in the measured interval, WAL writes per sync were about 8.3–9.0 with 16 writers and about 31–34 with 64 writers at width 1 (the "Interval" lines of `rocksdb.dbstats`, read at the start and end of the measured interval). At 16 writers and width 1 that was about 5.8k WAL syncs in about 5 s, roughly 0.9 ms per sync if they ran back to back. The 5 s runs ingested only about 4–74 MB, so one or two memtable flushes and no compaction happened in the width-16 runs, and no stall in any run.
- dodb Planned (from the real-sync run): 15.6–15.9 transactions per sync with 16 writers and 63.2–63.5 with 64 writers, so its batching per sync is larger than RocksDB's. But each dodb transaction writes about 8,380 WAL bytes (page images; 2,899,471,620 bytes for 345,999 transactions in the sustained run), while RocksDB wrote about 80 bytes per width-1 transaction (4.0 MB for about 50k writes). At width 1, mean dodb WAL sync time was 2.7 ms at 16 writers and 5.1 ms at 64 writers in the core matrix (2.8 and 5.6 ms in the sustained runs). RocksDB's syncs were much shorter and more frequent, which is where most of the throughput gap comes from on this storage path.
- Turso MVCC group commit: GC-on / GC-off mutation throughput was 4.647× (geometric mean over the four uniform multiwriter scenarios; 4.0× at 16 writers width 1, 5.7× at 64 writers width 1). A separate strace count (`group-commit-strace.txt`; 16 writers, width 1, 1 s + 3 s, only for counting because strace slows syscalls) saw 321 fsync calls for 1,040 commits with GC-on (about 3.2 commits per fsync) and 446 fsyncs for 388 commits with GC-off. So group commit really shares fsyncs among concurrent commits, and it helps a lot. MVCC throughput still stayed below Turso WAL in 8 of the 12 multiwriter scenarios because MVCC saturated both CPUs.
- Turso WAL does not group commits across connections: strace counted 3,913 fsyncs for 3,842 commits.
- RocksDB pipelined vs default: default / pipelined = 0.955× (geometric mean, 12 scenarios). Pipelined write was faster at 16 writers width 16 (for example 7,631 vs 5,723 tx/s uniform) and at 64 writers width 1, and slightly slower at 64 writers width 16.

### Turso files at the end of the measured interval (mean bytes) and RSS growth

| # | Writers | Width | Distribution | Engine | kv.db | kv.db-wal | kv.db-log | RSS growth after seed (MiB) |
|---:|---:|---:|---|---|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | Turso WAL | 11,952,128 | 36,592,499 | n/a | 70.0 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-on | 10,700,117 | 0 | 1,485,617 | 85.9 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-off | 8,048,640 | 0 | 4,072,826 | 45.7 |
| 1 | 16 | 1 | compact-locality | Turso WAL | 11,972,608 | 6,462,939 | n/a | 66.6 |
| 1 | 16 | 1 | compact-locality | Turso MVCC GC-on | 9,418,069 | 0 | 2,867,171 | 46.2 |
| 2 | 16 | 1 | spread-locality | Turso WAL | 11,972,608 | 32,184,099 | n/a | 68.1 |
| 2 | 16 | 1 | spread-locality | Turso MVCC GC-on | 10,697,387 | 0 | 3,906,527 | 74.7 |
| 3 | 16 | 16 | uniform | Turso WAL | 11,952,128 | 154,527,499 | n/a | 76.2 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-on | 9,374,379 | 0 | 7,355,081 | 202.2 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-off | 12,025,856 | 0 | 989,472 | 137.7 |
| 4 | 16 | 16 | compact-locality | Turso WAL | 11,972,608 | 19,557,672 | n/a | 41.8 |
| 4 | 16 | 16 | compact-locality | Turso MVCC GC-on | 11,976,704 | 0 | 3,788,504 | 51.6 |
| 5 | 16 | 16 | spread-locality | Turso WAL | 11,972,608 | 38,453,365 | n/a | 46.6 |
| 5 | 16 | 16 | spread-locality | Turso MVCC GC-on | 11,976,704 | 0 | 2,784,322 | 90.6 |
| 6 | 64 | 1 | uniform | Turso WAL | 11,952,128 | 31,737,765 | n/a | 161.5 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-on | 8,048,640 | 0 | 4,237,062 | 259.1 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-off | 8,048,640 | 0 | 4,009,597 | 185.2 |
| 7 | 64 | 1 | compact-locality | Turso WAL | 11,972,608 | 11,960,392 | n/a | 210.4 |
| 7 | 64 | 1 | compact-locality | Turso MVCC GC-on | 11,976,704 | 0 | 0 | 210.6 |
| 8 | 64 | 1 | spread-locality | Turso WAL | 11,972,608 | 31,128,005 | n/a | 211.4 |
| 8 | 64 | 1 | spread-locality | Turso MVCC GC-on | 10,697,387 | 0 | 3,600,404 | 253.1 |
| 9 | 64 | 16 | uniform | Turso WAL | 11,952,128 | 137,036,725 | n/a | 298.3 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-on | 12,025,856 | 0 | 1,684,234 | 408.6 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-off | 12,025,856 | 0 | 477,621 | 215.3 |
| 10 | 64 | 16 | compact-locality | Turso WAL | 11,972,608 | 25,653,899 | n/a | 163.9 |
| 10 | 64 | 16 | compact-locality | Turso MVCC GC-on | 11,976,704 | 0 | 1,877,976 | 121.1 |
| 11 | 64 | 16 | spread-locality | Turso WAL | 11,972,608 | 34,555,845 | n/a | 275.3 |
| 11 | 64 | 16 | spread-locality | Turso MVCC GC-on | 11,976,704 | 0 | 0 | 247.1 |
| 12 | 1 | 1 | uniform | Turso WAL | 11,952,128 | 1,163,245 | n/a | 11.4 |
| 12 | 1 | 1 | uniform | Turso MVCC GC-on | 12,025,856 | 0 | 892,753 | 23.6 |
| 13 | 1 | 16 | uniform | Turso WAL | 11,952,128 | 2,906,005 | n/a | 12.2 |
| 13 | 1 | 16 | uniform | Turso MVCC GC-on | 12,025,856 | 0 | 919,364 | 27.7 |

- Turso exposes no checkpoint counter through the Rust binding. File sizes are the only signal: the WAL-mode `kv.db-wal` reached 6–155 MB within 7 s (no checkpoint completed while writers kept the WAL busy), while the MVCC log stayed at 0–7 MB (the MVCC auto-checkpoint threshold was read back as 4,120,000).

## 120-second sustained results

Workload: working set 1,000,000, 16 and 64 writers, width 1, uniform, 10 s warmup, 120 s measurement, real durability, one run per engine, fresh database, with the engine order rotated between the two groups. This is still memory-resident: 1M × about 80 logical bytes is far below RAM.

| Writers | Engine | Successful tx/s | Attempted tx/s | p50 / p95 / p99 µs | CPU % one core | Peak RSS MiB | Max disk MiB | Busy | Conflicts | Abandoned | Errors | Verified |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 16 | dodb Planned | 2,883 | 2,883 | 5,082 / 8,470 / 10,259 | 52 | 8,626 | 7,775.9 | 0 | 0 | 0 | 0 | reopen not run by phase0-bench; WAL sync counters positive |
| 64 | dodb Planned | 3,944 | 3,944 | 15,130 / 23,035 / 30,477 | 69 | 9,694 | 9,011.1 | 0 | 0 | 0 | 0 | reopen not run by phase0-bench; WAL sync counters positive |
| 16 | RocksDB | 10,367 | 10,367 | 1,611 / 1,984 / 2,331 | 35 | 421 | 163.7 | 0 | 0 | 0 | 0 | True |
| 64 | RocksDB | 25,768 | 25,768 | 2,367 / 3,385 / 4,915 | 66 | 669 | 189.9 | 0 | 0 | 0 | 0 | True |
| 16 | Turso MVCC GC-on | 543 | 543 | 27,707 / 68,005 / 103,130 | 191 | 746 | 124.9 | 0 | 0 | 0 | 0 | True |
| 64 | Turso MVCC GC-on | 275 | 275 | 192,001 / 625,084 / 956,006 | 196 | 886 | 120.6 | 0 | 16 | 1 | 0 | True |
| 16 | Turso WAL | 1,026 | 1,172 | 894 / 1,923 / 54,856 | 27 | 200 | 634.9 | 315,054 | 0 | 17,539 | 0 | True |
| 64 | Turso WAL | 889 | 1,496 | 1,046 / 55,122 / 95,063 | 45 | 433 | 571.4 | 1,312,384 | 0 | 72,919 | 0 | True |

### 10-second windows, 16 writers: successful tx/s (p95 / p99 µs)

| Window | dodb Planned | RocksDB | Turso MVCC GC-on | Turso WAL |
|---:|---:|---:|---:|---:|
| 0-10 s | 3,041 (7,937 / 9,786) | 10,037 (1,985 / 2,453) | 489 (77,489 / 111,964) | 1,036 (1,879 / 55,093) |
| 10-20 s | 2,985 (8,209 / 9,965) | 10,812 (1,933 / 2,308) | 538 (64,651 / 96,002) | 1,041 (1,946 / 54,751) |
| 20-30 s | 2,918 (8,427 / 10,571) | 10,921 (1,929 / 2,260) | 502 (75,006 / 119,641) | 1,033 (1,843 / 44,775) |
| 30-40 s | 2,895 (8,193 / 9,473) | 9,978 (2,085 / 2,444) | 499 (72,271 / 108,016) | 1,032 (2,063 / 55,123) |
| 40-50 s | 2,846 (8,766 / 10,623) | 10,656 (1,961 / 2,324) | 516 (72,005 / 99,888) | 1,051 (1,598 / 44,679) |
| 50-60 s | 2,954 (8,166 / 9,968) | 10,587 (1,973 / 2,274) | 514 (79,983 / 132,175) | 1,007 (1,874 / 44,699) |
| 60-70 s | 2,863 (8,836 / 11,340) | 9,887 (2,000 / 2,340) | 577 (63,983 / 95,996) | 1,025 (1,962 / 54,845) |
| 70-80 s | 2,897 (8,380 / 10,195) | 10,719 (1,945 / 2,309) | 570 (64,061 / 95,984) | 1,026 (2,140 / 64,859) |
| 80-90 s | 2,917 (7,967 / 9,940) | 10,173 (1,987 / 2,308) | 586 (63,861 / 91,993) | 1,031 (1,919 / 64,692) |
| 90-100 s | 2,824 (8,226 / 10,014) | 9,828 (2,017 / 2,318) | 569 (63,671 / 95,987) | 1,015 (1,976 / 54,855) |
| 100-110 s | 2,738 (9,050 / 10,294) | 10,429 (1,995 / 2,321) | 556 (65,724 / 100,398) | 1,019 (1,834 / 45,152) |
| 110-120 s | 2,719 (9,098 / 10,930) | 10,382 (1,980 / 2,275) | 601 (60,790 / 95,485) | 994 (2,035 / 64,562) |

### 10-second windows, 64 writers: successful tx/s (p95 / p99 µs)

| Window | dodb Planned | RocksDB | Turso MVCC GC-on | Turso WAL |
|---:|---:|---:|---:|---:|
| 0-10 s | 4,182 (21,987 / 30,818) | 24,232 (4,469 / 6,428) | 259 (600,007 / 920,009) | 878 (54,711 / 87,217) |
| 10-20 s | 4,530 (20,347 / 28,214) | 25,813 (3,485 / 5,382) | 260 (567,728 / 771,983) | 883 (55,078 / 94,160) |
| 20-30 s | 4,211 (21,016 / 26,033) | 26,121 (3,341 / 4,921) | 264 (663,992 / 1,096,530) | 891 (55,109 / 95,096) |
| 30-40 s | 3,976 (23,138 / 28,806) | 26,453 (3,261 / 3,993) | 288 (576,358 / 868,403) | 904 (54,921 / 94,924) |
| 40-50 s | 3,807 (25,825 / 31,221) | 23,427 (3,779 / 7,527) | 274 (624,038 / 828,014) | 906 (55,231 / 95,038) |
| 50-60 s | 3,862 (23,082 / 26,526) | 26,416 (3,223 / 4,117) | 284 (543,986 / 759,987) | 908 (62,247 / 95,284) |
| 60-70 s | 3,779 (23,995 / 35,587) | 25,928 (3,316 / 5,021) | 271 (725,021 / 948,591) | 852 (55,118 / 94,916) |
| 70-80 s | 3,769 (24,282 / 36,684) | 26,638 (3,164 / 4,069) | 287 (551,998 / 778,100) | 877 (55,286 / 94,906) |
| 80-90 s | 3,876 (23,678 / 28,714) | 26,388 (3,283 / 4,039) | 287 (599,996 / 988,003) | 884 (55,076 / 95,096) |
| 90-100 s | 3,764 (23,174 / 32,174) | 26,043 (3,310 / 4,247) | 260 (688,014 / 1,088,009) | 896 (54,855 / 92,825) |
| 100-110 s | 3,773 (23,675 / 34,375) | 26,619 (3,222 / 3,926) | 274 (747,963 / 1,056,013) | 894 (54,879 / 95,169) |
| 110-120 s | 3,978 (21,367 / 44,725) | 25,136 (3,299 / 4,379) | 288 (587,819 / 1,227,874) | 898 (55,033 / 95,143) |

- The windows are 10-second buckets by completion time from the start of measurement. The few completions after the 120 s deadline (in-flight at the deadline) are counted in the totals but not shown as a window.
- Peak RSS and maximum disk come from the runner's once-per-second sampling of the whole process lifetime, including the untimed seeding of 1M rows (`raw/sustained-*.process-monitor.jsonl`).
- dodb Planned stayed steady over 120 s (2,719–3,041 tx/s at 16 writers; 3,764–4,530 tx/s at 64 writers) with zero errors, but its files and memory grew without a ceiling in the window. The WAL grew by about 8.4 KB per transaction (2.90 GB in the 16-writer measured interval, 3.98 GB at 64 writers). The data directory reached 7.8 / 9.0 GB and RSS reached 8.6 / 9.7 GiB, which is close to the 10.9 GiB visible RAM. At 16 writers the sampled RSS rose from 0.33 GiB after 1 s to 4.3 GiB after 61 s and 8.8 GiB after 211 s, together with file size. A longer run at this rate would exhaust memory on this host. This is recorded as a result and was not tuned.
- RocksDB held 9.8k–11.2k tx/s (16 writers) and 23.4k–26.6k tx/s (64 writers) in every window, with p99 at or below 7.5 ms in all windows. Turso MVCC held 489–601 tx/s and 259–288 tx/s with p99 of 0.1 s and about 1 s. Turso WAL held about 1.0k and about 0.9k tx/s, with 17,539 and 72,919 abandoned logical transactions.

## RocksDB compaction and stall behaviour

### RocksDB sustained 16 writers: flush, compaction and stall behaviour

- WAL writes 1,244,000, WAL syncs 139,000, writes per sync 8.95, ingest 100.1 MB, commit groups 139,000.
- cfstats deltas: compaction read 0.0 MiB, flush plus compaction write 73.3 MiB, delays 0, stops 0.
- Flushes 2, compactions 0, compaction read 0.0 MiB, compaction write 0.0 MiB.
- Stall time from dbstats 0.000 s; stall-condition transitions 0 []; samples with write stopped 0, with a delayed write rate 0.
- Max L0 files 3, L0 files at end 3, max pending compaction bytes 0.0 MiB.
- Event timeline (seconds from measured start): 8.2 flush (511186 entries), 68.2 flush (465000 entries)

### RocksDB sustained 64 writers: flush, compaction and stall behaviour

- WAL writes 3,092,000, WAL syncs 93,000, writes per sync 33.19, ingest 245.8 MB, commit groups 93,000.
- cfstats deltas: compaction read 152.7 MiB, flush plus compaction write 249.9 MiB, delays 0, stops 0.
- Flushes 5, compactions 1, compaction read 152.7 MiB, compaction write 75.0 MiB.
- Stall time from dbstats 0.000 s; stall-condition transitions 0 []; samples with write stopped 0, with a delayed write rate 0.
- Max L0 files 4, L0 files at end 3, max pending compaction bytes 152.7 MiB.
- Event timeline (seconds from measured start): 22.8 flush (465252 entries), 46.8 flush (465296 entries), 48.8 compaction L0->L6 (152.7->75.0 MiB, 2004 ms), 71.8 flush (464768 entries), 94.8 flush (464890 entries), 118.8 flush (465036 entries)

- 16 writers, 120 s: 2 memtable flushes (at 8 s and 68 s), no compaction, no stall. L0 reached at most 3 files, and pending compaction bytes stayed 0.
- 64 writers, 120 s: 5 flushes (about every 24 s), one L0→L6 compaction at 48.8 s (152.7 MiB read, 75.0 MiB written, 2.0 s). No write stall, delay or stop (`total-delays = 0`, `total-stops = 0`, stall-condition listener silent, `is-write-stopped` never set). L0 reached at most 4 files, and pending compaction bytes peaked at 152.7 MiB just before that compaction. The 40–50 s window, which holds the flush and the compaction start, had the lowest RocksDB throughput (23,427 tx/s) and the highest p99 (7.5 ms).
- The whole sustained run wrote 100 MB (16 writers) and 246 MB (64 writers) of user data into a 64 MiB-memtable LSM, so it saw only the first levels of compaction. A true steady-state or larger-than-memory LSM comparison is left for a later phase.

## Correctness

- Every run verified after a clean close and reopen. The sampled keys were about 1% of the working set, picked from the seed, plus every key of each writer's last committed transaction. Each value had to be 64 identical bytes equal to the last committed write of one of the writers, or the seed value if no committed write touched the key. The row count had to equal the working set plus the committed keys outside it, so an aborted transaction's insert would have been caught.

### Post-reopen verification of the core matrix

| Engine | Runs passed | Sampled keys checked | Sampled keys with committed writes |
|---|---:|---:|---:|
| RocksDB | 42/42 | 50,342 | 27,585 |
| RocksDB pipelined | 42/42 | 50,341 | 27,839 |
| Turso MVCC GC-on | 42/42 | 50,350 | 17,234 |
| Turso MVCC GC-off | 12/12 | 15,832 | 4,710 |
| Turso WAL | 42/42 | 50,380 | 14,940 |

- Sustained runs: RocksDB, Turso MVCC and Turso WAL passed the same post-reopen check (9,917 or more sampled keys each). The dodb sustained run used `phase0-bench`, which does not reopen. Its correctness evidence is the zero error, conflict and overload counts plus positive WAL sync counters, and the real-sync run's close/reopen check.
- Width-16 atomicity probe (`atomicity.jsonl`): 16 writers × 200 transactions, each writing all of the same 16 keys with a value tag unique to the transaction, then close and reopen. For every engine all 16 keys held the same tag, that tag belonged to a committed transaction and not to an abandoned one, and the table had exactly 16 rows.

| Engine | Committed | Abandoned after retries | Busy | Conflicts | Passed |
|---|---:|---:|---:|---:|---|
| Turso WAL | 2,900 | 300 | 5,762 | 0 | yes |
| Turso MVCC GC-on | 2,881 | 319 | 25 | 12,455 | yes |
| Turso MVCC GC-off | 2,862 | 338 | 23 | 12,423 | yes |
| RocksDB | 3,200 | 0 | 0 | 0 | yes |
| RocksDB pipelined | 3,200 | 0 | 0 | 0 | yes |

- No partial width-16 commit was seen. Turso retries were included in the expected final state: the expected values track only committed attempts.

## Limitations

- One host, one OCI boot-volume-backed ZFS vdev, 2 OCPUs. Turso MVCC, and partly RocksDB at 64 writers, were limited by CPU on this host, so a larger machine could change the ranking between Turso modes and the size of every gap.
- The working set is memory-resident (100k and 1M rows). These results say nothing about larger-than-memory behaviour or long-run LSM steady state.
- The dodb core numbers come from the earlier real-sync run on the same host and dataset (about 11:00–12:10 UTC, same day) and were not measured in the same rotation as the external engines. Lifecycle, durations, seeds and workload are the same, but the process model differs: an async coordinator with 2 Tokio workers vs one OS thread per writer.
- Turso results depend on the fixed retry policy (16 attempts, exponential backoff capped at 10 ms, `busy_timeout = 0`). A different policy would change the attempted and abandoned counts and the latency tails. Successful-throughput ratios use committed transactions only.
- Turso is a pre-release (`v0.8.0-pre.13`), and its docs call MVCC not production ready. The Rust binding returns write-write conflicts as a generic error; the harness classifies them by message text.
- Turso's page cache is per connection. The 32 MiB setting was applied to every connection, so the memory footprints are not equal across engines.
- The dodb sustained run used a benchmark-binary-only window patch, and phase0-bench does not reopen and verify after the run.
- The RocksDB statistics come from `rocksdb.dbstats` / `rocksdb.cfstats` properties and an event listener. `Statistics` tickers were not enabled, so nothing was added to the write path.
- No crash or fault-injection test was run. Durability is supported by syscall order and clean reopen, not by power-loss testing.

## Conclusion

- **dodb real durable result**: on this host, Planned delivered durable throughput above both Turso modes in the 12 multiwriter scenarios (geometric mean 2.733× over Turso WAL and 4.532× over Turso MVCC group commit). It was far below RocksDB (0.221×, meaning RocksDB was about 4.5× faster). The main visible cause is WAL volume per transaction: about 8.4 KB of page images for dodb against tens of bytes for RocksDB, which makes each shared sync slower even though dodb batches more transactions per sync. In the 120 s test Planned kept steady throughput, but its WAL and RSS grew without a ceiling to about 8–9 GB.
- **Turso WAL result**: durable (fsync on the WAL before every commit returns) but single-writer. 16 and 64 concurrent writers produced heavy BUSY traffic and abandoned transactions under the fixed retry policy. Committed throughput stayed at 0.9–2.2k tx/s at width 1 and 0.3–1.0k tx/s at width 16, whether there were 16 or 64 writers.
- **Turso MVCC experimental result**: durable (fsync on the MVCC log before commit returns). Group commit really shares fsyncs and gave about 4.6× over group commit off, but MVCC used both CPUs and landed below Turso WAL in 8 of 12 multiwriter scenarios. Conflicts appeared only where writers shared keys.
- **RocksDB result**: durable (fdatasync on the WAL for every synced write) and the fastest engine in every multiwriter scenario and in both 120 s sustained runs. It had no write stall, a single compaction at 64 writers, and flat 10-second windows. The 120 s test covers only the early part of LSM life (at most 4 L0 files, one compaction), so it does not settle steady-state LSM behaviour.
- These results are for a memory-resident, write-only, durable workload on one small host. They do not support a claim that dodb beats LSM engines in general, and on this workload RocksDB was clearly faster.
