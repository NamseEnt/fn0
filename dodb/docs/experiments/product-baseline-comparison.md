# dodb Product Workload External Baseline Benchmark

## Result

The current `parallel-blink` candidate has clear durable-write gains over
`main-btree`, including width 1 at 16 clients and widths 4/8 across the tested
client counts. It also has product-blocking resident read regressions: point
Get falls 19.6% at 4 clients and 22.5% at 16; Query limit 16 falls 24.7% and
25.2% at the same levels. The frozen policy in
[`fn0-product-workload.md`](fn0-product-workload.md) treats a sustained
regression above 10% in those paths as an adoption blocker. Write gains do not
cancel those regressions.

## Headline throughput

Values are median throughput across three runs. Get is operations/s, Query is
queries/s, durable writes and multi-key transactions are transactions/s, and
mixed cases are aggregate logical operations/s. `w4` and `w8` each count a
whole transaction as one transaction; mutation operations/s are transaction
throughput multiplied by the width. External comparisons retain the semantic
classes below and are not a single winner ranking.

| Workload | main | B-link | RocksDB | SQLite | Turso WAL | Turso MVCC | B-link/main |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Get c16 | 1,467,231 | 1,136,862 | 1,255,625 | 194,015 | 69,287 | 44,845 | 0.775x |
| Query16 c16 | 600,917 | 449,328 | 262,401 | 89,536 | 44,760 | 14,859 | 0.748x |
| Write w1 c16 | 4,238 | 7,181 | 8,028 | 973 | 971 | 697 | 1.694x |
| Tx w4 c16 | 2,758 | 4,833 | 6,401 | 773 | 722 | 486 | 1.753x |
| Tx w8 c16 | 1,794 | 3,142 | 3,976 | 592 | 512 | 465 | 1.752x |
| 95/5 c16 | 17,889 | 18,609 | 146,094 | 14,145 | 11,696 | 24,958 | 1.040x |
| 50/50 c16 | 4,135 | 7,494 | 15,887 | 1,993 | 1,766 | 1,522 | 1.813x |
| 20/80 c16 | 3,289 | 7,846 | 9,982 | 1,132 | 1,174 | 917 | 2.386x |

### Repetition and tail checks

Core c16 throughput was stable across the three repetitions for most rows. The
largest headline spread is B-link 95/5 at c16: minimum 42.3% below its median
and maximum 20.8% above it. Its 4% aggregate gain is within observed run noise
and is not a demonstrated improvement. Other headline B-link c16 min/max
spreads range from about 1% to 6.5%; raw minima and maxima are in the
[combined summary](results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-seedmatched-f893a005/summary.json).

The c16 p99 results make the trade-off visible:

| Workload | main p99 | B-link p99 | Change |
| --- | ---: | ---: | ---: |
| Get | 1.72 us | 2.20 us | +27.9% |
| Query16 | 3.08 us | 4.72 us | +53.2% |
| Write w1 | 6.80 ms | 3.69 ms | -45.8% |
| Tx w4 | 11.98 ms | 6.50 ms | -45.7% |
| Tx w8 | 19.28 ms | 9.13 ms | -52.7% |

For mixed cases, separate read and write tails avoid hiding one class inside an
aggregate percentile. At c16 in 95/5, B-link read p99 improves from 5.24 us to
4.04 us, while write p99 worsens from 3.15 ms to 4.95 ms. In 50/50, read p99
improves from 6.76 us to 4.88 us and write p99 from 9.85 ms to 6.01 ms. In
20/80, read p99 worsens from 7.84 us to 10.72 us while write p99 improves
from 12.93 ms to 5.39 ms. The 20/80 c4 throughput is also 36% below main;
at c16 it is 2.39x main. This concurrency sensitivity merits follow-up.

## A. Internal product comparison

The following ratios use medians from three repetitions. The product core is
c1/c4/c16; c64 is scaling evidence and does not waive a core regression.

| Workload | c1 | c4 | c16 | c64 |
| --- | ---: | ---: | ---: | ---: |
| Point Get | 1.072x | 0.804x | 0.775x | 0.781x |
| Query limit 16 | 1.006x | 0.753x | 0.748x | N/A |
| Durable write w1 | 1.197x | 0.963x | 1.694x | 2.560x |
| Durable transaction w4 | 1.209x | 1.194x | 1.753x | 2.053x |
| Durable transaction w8 | 2.037x | 1.329x | 1.752x | 2.313x |
| Read-heavy 95/5 | N/A | 1.204x | 1.040x | 0.601x |
| Balanced 50/50 | N/A | 0.936x | 1.813x | 2.597x |
| Write-heavy 20/80 | N/A | 0.640x | 2.386x | 3.642x |

The c4/c16 Get and Query regressions exceed the frozen 10% limit. Width 1 is
near parity at c4 and improves at c16. Widths 4 and 8 improve at each tested
concurrency. At c16, 95/5 aggregate throughput is statistically inconclusive
from its run spread, while its write p99 is worse. The c4 20/80 regression and
c64 95/5 regression are additional scaling concerns. On this matrix the
candidate is not ready to replace `main-btree` under the documented product
policy.

At c16 in the 250,000-row cache-pressure runs, B-link/main is 0.908x for Get,
0.787x for Query16, 2.71x for width-1 write, and 2.56x for 50/50 mixed. In
the focused 2 KiB overflow runs, it is 0.902x for Get and 0.748x for width-1
write. These diagnostic cases point to a separate cost in the read and
overflow write paths; they are not blended into the resident headline scores.

## B. External Pareto position

### RocksDB

RocksDB remains ahead of B-link on durable write throughput at c16: 1.12x for
w1, 1.32x for w4, and 1.27x for w8. B-link reaches 89%, 76%, and 79% of those
respective rates. RocksDB is also 1.10x B-link on point Get; B-link leads on
Query16 (1.71x). RocksDB is a write-optimized LSM reference, not a required
product winner. Its atomic WriteBatch does not implement dodb conditional or
revision-based transaction semantics.

For mixed c16, RocksDB reports 7.85x B-link throughput at 95/5, 2.12x at
50/50, and 1.27x at 20/80. Those are aggregate logical operation rates under
the same client mix; read/write tails and operation cost differ by adapter.

### SQLite and Turso WAL

At c16, B-link point Get throughput is 5.9x SQLite and 16.4x Turso WAL; for
Query16 it is 5.0x SQLite and 10.0x Turso WAL. SQLite remains a useful
conventional B-tree reference. Its single-writer behavior is part of these
results. Turso WAL is an embedded conventional-WAL reference, not SQLite
rebranded.

Turso WAL at c16 records 971 tx/s for w1, 722 for w4, and 512 for w8. It has
zero terminal errors in these rows, but retries 37,328, 37,384, and 37,169
times respectively; p99 is about 95–98 ms. Successful transaction throughput
therefore needs to be read with retry counts and tail latency, not in
isolation.

### Turso MVCC

Turso MVCC is an approximate concurrent-transaction reference. At c16 it
records 697 tx/s for w1, 486 tx/s for w4, and 465 tx/s for w8, compared with
B-link's 7,181, 4,833, and 3,142 tx/s. It records 24,958 aggregate ops/s for
95/5 with 29 conflicts/retries, 1,522 ops/s for 50/50, and 917 ops/s for
20/80. The headline rates, conflicts, retries, and errors use v0.8.2-pre.2.
Its MVCC snapshot and conflict behavior, group commit, and retry behavior
differ from dodb's transaction contract.

In the seed-matched v0.8.2-pre.2 P5 95/5 c4 case, MVCC produced 118
`database is locked` errors over three repetitions. The pinned adapter retries typed Busy and
BusySnapshot outcomes and conflict messages, but counts a generic `database
is locked` error as a failed operation. Turso's manual says concurrent
transactions that return `SQLITE_BUSY` must be rolled back and retried, but
these generic lock messages do not include a typed status in the harness
record. This P5 c4 row is therefore retained with its errors but excluded from
a fair throughput ranking. An adapter change to retry these generic lock
failures consistently with Turso's transaction contract remains a follow-up.

The superseded v0.8.0-pre.13 run recorded 457 `database is locked` errors for
95/5 c4 over three repetitions, and an additional same-binary run reproduced
lock errors. That finding is retained in the historical raw records but is
excluded from the current-version summary.

## C. Architecture stress

P9 remains an architecture-stress control, not a product headline. At 64
writers and width 16 with 64 B values, B-link/main throughput ratios are
2.78x for different-leaf-heavy, 3.37x for same-leaf-heavy, and 4.68x for
uniform. All use real sync. This supports the batching/parallel execution
hypothesis under that stress shape; it does not override the Get/Query
regressions above.

## Comparator semantics and exact configuration

The product candidate is the current `parallel-blink` selector. Other selectors
such as planned/versioned experiments are not included as additional product
candidates. The overlay candidate is excluded because its synchronous
materialization result failed the previously recorded product guardrail. The
policy update commit is `524d0d99375c649ea92ac273f160e724f011a4f7`.

| Comparator | Version and setup | Comparison class |
| --- | --- | --- |
| dodb `main-btree` | Experiment source SHA below; unconditional transactional mutation API; real sync; 64 MiB page cache. | Semantically comparable to B-link. |
| dodb `parallel-blink` | Active B-link product candidate; same binary and workload adapter as main selector; real sync; 64 MiB page cache. | Semantically comparable to main. |
| RocksDB | 11.8.1, commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; WAL on; `WriteOptions.sync=true`; one atomic WriteBatch per transaction; pipelined writes off; default-style options with 64 MiB block cache and 64 MiB write buffer. | Storage-engine reference only. No dodb OCC conditions. |
| SQLite | SQLite 3.50.2 via rusqlite 0.37.0 bundled build; WAL; `synchronous=FULL`; 64 MiB page cache; BLOB primary key in a `WITHOUT ROWID` table. | Approximately comparable for point operations and simple transactions; one writer. |
| Turso WAL | Embedded Turso v0.8.2-pre.2, commit `418ef1c3e8e4975597be5882ebcf1d03c3ce83aa`; WAL mode; `synchronous=FULL`; 64 MiB cache; ordinary `BEGIN`. | Approximately comparable for simple reads and unconditional transactions; serialized WAL writes. |
| Turso MVCC | Same embedded Turso build; experimental MVCC mode; `synchronous=FULL`; `BEGIN CONCURRENT`; group commit enabled; 64 MiB cache. | Approximately comparable for concurrent transactions; snapshot isolation and retry/conflict behavior differ. |

All databases used local embedded paths on the same OCI ZFS dataset. Values
were durable on successful return: dodb real sync, SQLite/Turso WAL FULL, and
RocksDB synced WAL WriteBatch. No sync-disabled run is included.

Turso's `v0.8.2-pre.2` was the newest published pre-release available on
2026-10-06 ([release](https://github.com/tursodatabase/turso/releases/tag/v0.8.2-pre.2)).
The project documentation describes MVCC as experimental, documents
`BEGIN CONCURRENT`, snapshot conflict detection, and MVCC group commit; the
benchmark enables group commit explicitly because it is off by default
([PRAGMA reference](https://github.com/tursodatabase/turso/blob/v0.8.2-pre.2/docs/sql-reference/pragmas.mdx),
[transaction manual](https://github.com/tursodatabase/turso/blob/v0.8.2-pre.2/docs/manual.md)).

## Source, host, and method

| Item | Recorded value |
| --- | --- |
| Experiment branch/source | `experiment/b-link-batched-engine-monorepo`, `f893a0052963bc103533e276ece687b0ecaa6d1c` |
| Current main at P1–P4 rerun | `60203a8802517dbe4c7522c3fe390231ece9ae90` |
| Main snapshot recorded by P5–P9 run | `d5e41bb823b55d68bad6ab5ba73b0b5113223d8b`; `dodb/` is unchanged between this snapshot and current main. |
| OCI host | `instance-20260923-1013`, AArch64 Neoverse-N1, 2 OCPU, 12 GB shape (11,159,744 KiB reported), Oracle Linux 9.8, kernel `6.12.0-206.104.4.4.el9uek.aarch64` |
| Rust | `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| Filesystem/device | ZFS dataset `dodbbench/db` on `sda4`, mounted at `/bench/zfs/db` with `rw,noatime,seclabel,xattr,noacl,casesensitive`; `sync=standard`, `recordsize=4K`, compression off, atime off. Pool was ONLINE with no known data errors. |
| Cache/working set | dodb cache 16,384 x 4 KiB = 64 MiB; resident set 10,000 rows; cache-pressure set 250,000 rows. |
| Keys/values | 16-byte encoded key; primary value 512 B; diagnostic 64 B; focused 2 KiB overflow case. |
| Measurement | 2 s warmup, 5 s measured, 3 repetitions, fixed per-case seeds, rotated engine order, fresh database per run. |

At artifact verification after measurement, ZFS reported 141 GiB available and
7.73 GiB used. This is the post-run filesystem state, not a per-run database
size measurement.

The 40 executed cases are: P1 Get c1/c4/c16/c64; P2 Query16 c1/c4/c16;
P3 width-1 write c1/c4/c16/c64; P4 widths 4 and 8 at c1/c4/c16/c64 plus
different-leaf c64 for each width; P5 95/5, P6 50/50, and P7 20/80 at
c4/c16/c64; P8 hotspot 50/50 c16; P9 width-16 c64 with uniform,
different-leaf-heavy, and same-leaf-heavy distributions; cache-pressure Get,
Query16, width-1 write, and 50/50 mixed at c16; focused 2 KiB Get and width-1
write at c16. P9 values are 64 B; other core and focused cases use 512 B or
2 KiB as shown in the case name and raw record.

The current-version matrix comprises 40 workload cases x 6 engines x 3
repetitions = 720 selected individual executions. Exact P1–P4 dodb/SQLite/RocksDB cases
were rerun from source SHA `f893...` after discovering that the earlier P1–P4
artifacts came from `e5ce853...`. P5–P9, cache-pressure, and focused 2 KiB
dodb/SQLite/RocksDB cases already used `f893...`. All Turso cases were rerun
with v0.8.2-pre.2 after finding the newer release. The final Turso set uses
seed-matched P1–P4 records plus a same-case-order rerun of the remaining 19
cases; a first all-case run assigned different seeds to those latter cases,
and its mismatched rows are excluded. The earlier v0.8.0-pre.13 Turso rows are
also excluded. Every included row has three repetitions, source SHA `f893...`,
and binary hashes linked to its run-order start events.

The OCI experiment checkout was detached at the exact `f893...` source commit
for the P1–P4 and current Turso runs. The P1–P4 environment therefore records
an empty branch name; the local canonical branch and its remote-tracking ref
both pointed to `f893...`. The P5–P9 run recorded the branch name directly.

The detailed CSV has one row per case and engine. It includes throughput and
range, transaction and mutation rates, p50/p95/p99, separate read/write tails,
machine CPU, errors, conflicts, retries, overloads, sync/WAL counters where
the adapter reports them, source SHA, binary SHA, and raw artifact paths. A
missing external sync counter is blank/unavailable; it is not reported as
zero.

The rebuilt P1–P4 dodb and SQLite executables have different hashes from the
earlier P5–P9 executables despite sharing the same source SHA. Each run's hash
is preserved in its run-order record and summary. Binary hashes by segment are
listed in `summary.json`; Turso and RocksDB hashes are consistent across both
segments. This build-byte difference is a provenance limitation for direct
whole-matrix binary identity, though the source SHA and Rust version match.

| Comparator | P1–P4 raw run | P5–P9 raw run | Current Turso rerun |
| --- | --- | --- | --- |
| dodb `phase0-bench` | `9dbba45e244b4c823afbfe1b7c1ba0a4c501010de7ad51bacab31cdf9d702bcb` | `5fe27a8338fb6881fc5ffd6e06b7165e11a2cb6d1ad93d25cf380ead800eda23` | N/A |
| SQLite adapter | `161b202850e2e9921a7403d73f69407e495e0afbc0943b9180c7c3ed7a694239` | `afb56e097ae2a02d4b7e05feffbd25ad721255ec62c27e6da2f527336ce8b8cd` | N/A |
| Turso adapter | `a53bf88f27888fe0181789c01996f16a327144ff78fa8fd9de159a9a0d1e75c4` (superseded) | same (superseded) | `82e7966ab5a33d19d12280253c3b8251cd426218272650b41fdf5d7e798a28ab` |
| RocksDB adapter | `8f94708c5b330828f330f0a2aed9e519e37bb3e68f49c378839e653783dd20be` | same | N/A |

The current Turso adapter was built with Rust 1.97.1 from the pinned tag and
full commit above, default Cargo features, release profile, and
`RUSTFLAGS="-C target-cpu=native"`. Its harness embeds both the tag and exact
source commit in every JSONL result. The RocksDB shim used RocksDB 11.8.1 and
the adapter's recorded ARMv8 CRC/crypto C++ build flags.

The current Turso build command was:

```sh
git -C /tmp/turso-crossdb checkout --detach v0.8.2-pre.2
CROSSDB_TURSO_TAG=v0.8.2-pre.2 CROSSDB_TURSO_COMMIT=418ef1c3e8e4975597be5882ebcf1d03c3ce83aa RUSTFLAGS="-C target-cpu=native" CARGO_TARGET_DIR=/bench/zfs/db/product-build/turso-0.8.2-pre.2-target rustup run 1.97.1 cargo build --release --manifest-path /home/opc/fn0-product-f893a005/dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-crossdb/harness/turso-bench/Cargo.toml
```

The dodb `main-btree` selector is built from the experiment source tree to use
the same benchmark adapter as B-link. It is not a bit-for-bit build of
production `main`; the product source contains the B-link/WAL experiment
changes. Current `main`'s `dodb/` subtree matches the previously recorded main
snapshot, but this harness comparison should still be read as an in-tree
production-engine baseline.

## Reproduction and raw artifacts

The runner is
[`run_product_matrix.py`](results/oci-a1-2ocpu-12g-zfs-crossdb/scripts/run_product_matrix.py).
The checked-in summarizer maps each raw record to the binary hash for that
case/engine/repetition in `run-order.jsonl`:
[`summarize_product_matrix.py`](results/oci-a1-2ocpu-12g-zfs-crossdb/scripts/summarize_product_matrix.py).

The canonical run used these binary paths on OCI:

```sh
export BENCH_DATA_ROOT=/bench/zfs/db/product-data-turso-0.8.2-pre.2-f893a005
export PHASE0_BINARY=/bench/zfs/db/product-build/dodb-product-target/release/phase0-bench
export SQLITE_BINARY=/bench/zfs/db/product-build/sqlite-product-target/release/sqlite-bench
export TURSO_BINARY=/bench/zfs/db/product-build/turso-0.8.2-pre.2-target/release/turso-bench
export ROCKSDB_BINARY=/bench/zfs/db/product-build/rocksdb-product-target/release/rocksdb-bench
```

Run from the repository root. The committed runner's `--case-filter` accepts
comma-separated case names; each executed command and seed are also stored in
`run-order.jsonl`.

```sh
python3 dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-crossdb/scripts/run_product_matrix.py --results dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-product-baseline --phase all --duration-ms 5000 --warmup-ms 2000 --repetitions 3 --engine-filter all
```

The included `v0.8.2-pre.2` Turso rows come from two seed-aligned batches:
P1–P4 use the corresponding cases from the all-case Turso run, while P5–P9,
cache-pressure, and focused 2 KiB cases were rerun with the same filtered case
order as the dodb continuation run. An initial all-case Turso run assigned
different scenario indices and seeds to that second group; those rows are
excluded from the final summary. The exact commands and seeds for included
rows are preserved in the two current-version `run-order.jsonl` files.

Regenerate the combined median and detailed tables by excluding the superseded
Turso rows and replacing them with the current-version runs:

```sh
python3 dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-crossdb/scripts/summarize_product_matrix.py \
  --results dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-product-baseline-f893a005-p1-p4 \
  --results dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-product-baseline-continuation-f893a005 \
  --replacement-results dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-p1-p4-f893a005 \
  --replacement-results dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-tail-seedmatch-f893a005 \
  --replace-engine turso-wal --replace-engine turso-mvcc \
  --output dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-seedmatched-f893a005/summary.json \
  --csv-output dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-seedmatched-f893a005/product-baseline-results.csv
```

The P1–P4 exact rerun, its 378 JSONL records and logs, environment, run order,
and hash manifest are in
[`product-baseline-f893a005-p1-p4`](results/oci-a1-2ocpu-12g-zfs-product-baseline-f893a005-p1-p4).
The remaining exact-source results (342 JSONL records and logs) are in
[`product-baseline-continuation-f893a005`](results/oci-a1-2ocpu-12g-zfs-product-baseline-continuation-f893a005).
The selected current Turso data is split into the seed-aligned P1–P4 batch
(126 JSONL records and logs) at
[`product-baseline-turso-0.8.2-pre.2-p1-p4-f893a005`](results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-p1-p4-f893a005)
and the seed-matched 19-case rerun (114 JSONL records and logs) at
[`product-baseline-turso-0.8.2-pre.2-tail-seedmatch-f893a005`](results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-tail-seedmatch-f893a005).
The combined [`summary.json`](results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-seedmatched-f893a005/summary.json)
contains 240 case/engine rows; the
[`CSV`](results/oci-a1-2ocpu-12g-zfs-product-baseline-turso-0.8.2-pre.2-seedmatched-f893a005/product-baseline-results.csv)
contains the same metrics as a flat table. Raw folders have SHA-256 manifests;
`summary-sha256.txt` covers the generated JSON and CSV. The
supplemental Turso MVCC lock reproduction is in
[`product-validation-turso-errors-f893a005`](results/oci-a1-2ocpu-12g-zfs-product-validation-turso-errors-f893a005)
and is not counted as another canonical matrix row.

## Limitations and next investigation

- This is a single-tenant, mostly resident primary matrix. Cache-pressure runs
  cover representative Get, Query, width-1 write, and 50/50 mixed cases, not
  every workload/concurrency pair.
- The current Turso release was run after the other comparator batches, so its
  WAL/MVCC order rotates within Turso but is not interleaved with all six
  engines. The 19-case seed-alignment rerun also occurred after the first
  current-version Turso run. Hardware, ZFS settings, source SHA, and duration
  match; time-based host/storage drift remains possible.
- The raw harness did not retain a comparable on-disk database byte/page count
  for dodb. Its cache size and seeded/working-set rows are recorded. External
  adapters expose differing file-size snapshots in raw JSON, but those are not
  equivalent page-count metrics, so no cross-engine database-size claim is
  made here.
- 2 KiB values were included as a focused overflow case; there is no full
  8/32 KiB matrix. Multi-tenant fairness, burst/open-loop offered load, and
  larger-than-cache write-tail behavior remain unmeasured.
- CPU is recorded in each adapter's native measurement scope; it is not
  equivalent thread attribution across databases. WAL byte and physical sync
  counters are also not exposed uniformly, so raw per-engine instrumentation
  must be used rather than treating absent counters as zero.
- The product comparison uses unconditional mutations. Conditional OCC,
  `RevisionEquals`, and insert-if-absent behavior are not represented by the
  RocksDB batch adapter.
- The largest observed internal gap is the B-link read path and its latency at
  c4/c16. The next optimization investigation should profile Get/Query tree
  traversal and page/version indirection. A separate follow-up should examine
  95/5 write-tail batching and the 20/80 c4 scaling dip. Do not change the
  product guardrail based on these measurements.
