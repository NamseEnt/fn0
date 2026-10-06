# Read metrics and worker policy study

## Scope and evidence

This study records two controlled comparisons on accepted source
`cb55d6fe8325137a96e1c9615c64cb1942e328fb`:

1. Durable mixed 50/50 Point Get and width-one writes with changing 512-byte
   values, comparing one lane, two lanes, a 32-operation background-dispatch
   threshold, and RocksDB.
2. Pure Get and Query(16) reads, comparing main B-tree, B-link with read
   observational metrics enabled or disabled, and RocksDB. A c1 follow-up
   compares the two B-link metric modes only.

Both used 10,000 uniform keys, 16-byte keys, 512-byte values, real sync, three
repetitions, 2 seconds of warmup, and 5 seconds of measurement. Mixed used
width one and 50/50 reads/writes at c4 and c64. Pure reads used Get and
Query(16) at 16 readers; the follow-up used one reader.

The accepted run set contains 60 full cases: 24 mixed, 24 c16 read, and 12 c1 read cases, plus 16 accepted smoke cases. The independent validator checked each case, source identity, raw gates, and artifact hashes. It also checked all 18 Query(16) raw records across c16 and c1 (12 + 6) for the expected returned row count. The [validation record](results/read-metrics-worker-policy-validation-cb55d6fe.json) contains paired ratios and case counts.

Runs used the OCI A1 ARM64 host with two OCPU, a Neoverse-N1 CPU, and 12 GiB
RAM. The data lived on the `dodbbench/db` ZFS dataset with `sync=standard`,
`recordsize=4K`, and compression disabled. Native builds used Rust 1.97.1 and
`-C target-cpu=native`. RocksDB was version 11.8.1 with synchronous writes and
WAL enabled. The [build provenance](results/oci-a1-2ocpu-12g-zfs-worker-policy-cb55d6fe/build-proof/provenance.txt)
records the compiler, commands, clean checkout, and binary evidence.

The durable mixed artifact is the
[worker-policy matrix](results/oci-a1-2ocpu-12g-zfs-worker-policy-cb55d6fe/results/payload-batching-summary.json).
The pure-read artifacts are the
[c16 matrix](results/oci-a1-2ocpu-12g-zfs-read-metrics-cb55d6fe/results/read-metrics-summary.json)
and the [c1 follow-up](results/oci-a1-2ocpu-12g-zfs-read-metrics-c1-cb55d6fe/results/read-metrics-summary.json).
Raw `git_commit` values identify the checked-out source at runtime; they are not embedded build IDs. Separate [c16 build provenance](results/oci-a1-2ocpu-12g-zfs-read-metrics-cb55d6fe/build-proof/provenance.txt) records the clean checkout, pinned compiler, commands, and exact binary hashes.

## Facts

### Durable mixed worker comparison

All rows below are aggregate logical operations per second. Each cell shows
the three raw repetition rates, their median and range, the median write p99,
and background-worker dispatches per parallel group for each repetition.
The comparison is paired by seed. It is not a write-transactions-per-second
table.

| Clients | Variant | Aggregate ops/s, repetitions 1 / 2 / 3 | Median [min, max] | Write p99 median, us | Background dispatches / group, repetitions 1 / 2 / 3 |
| ---: | --- | ---: | ---: | ---: | ---: |
| 4 | One lane | 8,650.618 / 8,489.951 / 8,589.762 | 8,589.762 [8,489.951, 8,650.618] | 1,840.456 | 0 / 0 / 0 |
| 4 | Two lanes | 8,317.018 / 7,767.639 / 8,314.341 | 8,314.341 [7,767.639, 8,317.018] | 1,866.096 | 1 / 1 / 1 |
| 4 | Adaptive threshold 32 | 8,554.840 / 8,352.826 / 8,168.817 | 8,352.826 [8,168.817, 8,554.840] | 1,895.977 | 0 / 0 / 0 |
| 4 | RocksDB | 5,631.251 / 5,695.850 / 5,695.430 | 5,695.430 [5,631.251, 5,695.850] | 2,344.940 | N/A |
| 64 | One lane | 39,290.515 / 39,064.844 / 38,345.462 | 39,064.844 [38,345.462, 39,290.515] | 5,835.850 | 0 / 0 / 0 |
| 64 | Two lanes | 40,582.144 / 41,181.142 / 40,246.340 | 40,582.144 [40,246.340, 41,181.142] | 5,720.369 | 1 / 1 / 1 |
| 64 | Adaptive threshold 32 | 39,682.692 / 40,715.147 / 40,842.828 | 40,715.147 [39,682.692, 40,842.828] | 5,558.528 | 0.897 / 0.893 / 0.898 |
| 64 | RocksDB | 38,116.660 / 38,025.092 / 37,229.074 | 38,025.092 [37,229.074, 38,116.660] | 6,653.258 | N/A |

The paired lane and adaptive ratios are:

| Clients | Numerator / denominator | Paired ratios, repetitions 1 / 2 / 3 | Median [min, max] |
| ---: | --- | ---: | ---: |
| 4 | One lane / two lanes | 1.040110 / 1.092990 / 1.033126 | 1.040110 [1.033126, 1.092990] |
| 4 | Adaptive 32 / two lanes | 1.028595 / 1.075336 / 0.982497 | 1.028595 [0.982497, 1.075336] |
| 4 | Adaptive 32 / one lane | 0.988928 / 0.983848 / 0.950995 | 0.983848 [0.950995, 0.988928] |
| 64 | Two lanes / one lane | 1.032874 / 1.054174 / 1.049572 | 1.049572 [1.032874, 1.054174] |
| 64 | Adaptive 32 / two lanes | 0.977836 / 0.988684 / 1.014821 | 0.988684 [0.977836, 1.014821] |
| 64 | Adaptive 32 / one lane | 1.009981 / 1.042245 / 1.065128 | 1.042245 [1.009981, 1.065128] |

Against RocksDB, the median paired ratios for one lane, two lanes, and
adaptive 32 are respectively 1.508185, 1.459827, and 1.466476 at c4; at c64
they are 1.029987, 1.081046, and 1.070744. These are descriptive comparisons
between different engines and harness execution models.

The new `parallel_background_worker_dispatches_delta` counter records actual background submissions. At c4 the two-lane variant dispatched one background job per group, while one lane and adaptive 32 dispatched none. At c64 the two-lane variant dispatched one per group; adaptive 32 dispatched 1,582 / 1,610 / 1,623 jobs across 1,764 / 1,803 / 1,807 groups; one lane dispatched none. The older `parallel_worker_dispatches_delta` includes the coordinator lane and must not be read as a background-worker count. The threshold gates dispatch after planning by total planned mutations in [BlinkStore](../../crates/dodb-storage/src/blink/mod.rs#L2938); it preserves the existing parallel preparation path.

### Pure reads

Each c16 row shows three raw rates in millions of operations per second, the
median and range, and median read p99. Phase0 used 16 asynchronous reader tasks
on two Tokio workers. RocksDB used 16 OS threads. The paired metrics-on/off
comparison uses the same engine, seeds, and workload settings.

| Operation | Variant | Mops/s, repetitions 1 / 2 / 3 | Median [min, max] | Read p99 median, us |
| --- | --- | ---: | ---: | ---: |
| Get | Main B-tree | 1.390572 / 1.328200 / 1.336268 | 1.336268 [1.328200, 1.390572] | 1.800 |
| Get | B-link metrics on | 1.103794 / 1.099156 / 1.044233 | 1.099156 [1.044233, 1.103794] | 2.200 |
| Get | B-link metrics off | 1.208570 / 1.186913 / 1.200739 | 1.200739 [1.186913, 1.208570] | 2.040 |
| Get | RocksDB | 1.329748 / 1.350910 / 1.337493 | 1.337493 [1.329748, 1.350910] | 2.080 |
| Query(16) | Main B-tree | 0.587825 / 0.590001 / 0.582347 | 0.587825 [0.582347, 0.590001] | 3.081 |
| Query(16) | B-link metrics on | 0.433423 / 0.448420 / 0.436400 | 0.436400 [0.433423, 0.448420] | 4.880 |
| Query(16) | B-link metrics off | 0.452856 / 0.446611 / 0.456477 | 0.452856 [0.446611, 0.456477] | 4.560 |
| Query(16) | RocksDB | 0.278936 / 0.281654 / 0.282476 | 0.281654 [0.278936, 0.282476] | 7.240 |

Paired metric-mode ratios (`off / on`) were 1.094923 / 1.079840 / 1.149876
for c16 Get, with median 1.094923 and range 1.079840–1.149876. For c16
Query(16), they were 1.044836 / 0.995964 / 1.046007, with median 1.044836
and range 0.995964–1.046007.

The c1 follow-up used the same phase0 engine and two Tokio workers. Its paired
`off / on` Get ratios were 1.022068 / 1.003494 / 1.007491, median 1.007491
and range 1.003494–1.022068. Query(16) ratios were 1.008436 / 1.001227 /
1.009083, median 1.008436 and range 1.001227–1.009083. c1 runs used about
99–100% of one core. c16 generally used about 195–197% across two cores; the
third metrics-on Get run measured 177.77%.

The c16 paired ratios against main B-tree and RocksDB are:

| Operation | Comparison | Paired ratio median [min, max] |
| --- | --- | ---: |
| Get | B-link metrics on / main | 0.793770 [0.781455, 0.827553] |
| Get | B-link metrics off / main | 0.893625 [0.869117, 0.898576] |
| Get | B-link metrics on / RocksDB | 0.813641 [0.780739, 0.830078] |
| Get | B-link metrics off / RocksDB | 0.897753 [0.878602, 0.908871] |
| Query(16) | B-link metrics on / main | 0.749380 [0.737334, 0.760033] |
| Query(16) | B-link metrics off / main | 0.770393 [0.756966, 0.783857] |
| Query(16) | B-link metrics on / RocksDB | 1.553846 [1.544911, 1.592096] |
| Query(16) | B-link metrics off / RocksDB | 1.615988 [1.585670, 1.623515] |

## Hypotheses

The direct lane control shows a concurrency-dependent result. At c4, one lane
was faster than two lanes in all three pairs. At c64, two lanes were faster
than one lane in all three pairs. Adaptive 32 did not consistently beat two
lanes at either client count. Its dispatch counter confirms that it suppressed
background dispatch at c4 and reduced, but did not eliminate, background
dispatch at c64.

This does not isolate the cost of a thread by itself. Lane count changes the
natural chunk size and group count: the c64 one-lane runs formed 1,698–1,720
groups, while two-lane runs formed 1,761–1,820. The measurements support a
conditional executor effect, not a general worker-thread speedup claim.

Metrics-off improved c16 Get throughput by a median 9.49% relative to metrics
on, while the c1 median difference was 0.75%. Query(16) differences were
smaller and less consistent. This supports an instrumentation cost that grows
under higher read concurrency. It does not prove cache-line contention or
explain the entire B-link-to-main gap. Throughput percentages cannot be
allocated among counters, pinning, page access, range materialization, or
other work from these matrices.

The c16 read gap to main remains after metrics are disabled: B-link metrics-off
was 0.893625 of main for Get and 0.770393 for Query(16). The Get result makes
instrumentation a material part of the measured difference, but not the whole
difference. The Query(16) gap is largely present in both metrics modes.

## Corrections

The no-sync diagnostics in the earlier
[payload and executor study](payload-batching-executor-study.md) do not isolate
CPU or tree-structure cost. Disabling sync still leaves WAL encoding, CRC
validation, append and write calls, page-cache changes, and ZFS work. A B-link
win in that condition cannot be assigned to the tree algorithm or parallel
physical execution alone.

The main-B-tree raw result of 8,380 WAL bytes per successful write is explained for this workload by one changed 4 KiB leaf image plus one superblock image. Each image payload is 8 bytes of page identity plus 4,096 bytes; each WAL frame adds a 48-byte header and 4-byte trailer. The commit frame adds 48 + 16 + 4 bytes. Thus `2 * (8 + 4096 + 48 + 4) + (48 + 16 + 4) = 8380` bytes. The [B-tree WAL path](../../crates/dodb-storage/src/btree/mod.rs#L796) appends the superblock image for each commit; [WAL framing](../../crates/dodb-storage/src/wal.rs#L25) and [page size](../../crates/dodb-storage/src/page.rs#L3) define the sizes. The benchmark changes one leaf in this seeded workload; 8,380 bytes is not a universal per-write size.

Changing-value B-link runs recorded about 669 bytes per write. The page-delta payload starts with an 18-byte page/base-LSN/span-count header, then each span adds a 4-byte offset-and-length header plus changed bytes. The enclosing page record adds 52 bytes and the commit adds 68 bytes. Superblock-image elision is a separate B-link optimization. Canonical span construction merges unchanged gaps of at most four bytes in [wal.rs](../../crates/dodb-storage/src/wal.rs#L3446). The earlier [payload study](payload-batching-executor-study.md) recorded approximately 158 bytes for its constant-value workload; that value does not describe a changed 512-byte value.

Cross-engine p99 values do not have identical timing boundaries. [Phase0](../../crates/dodb-storage/src/bin/phase0-bench.rs#L2938) creates the read request before starting its timer. The [external driver](../../experiments/mixed-harness/harness/common/driver.rs#L459) starts timing before selecting and generating the read key. Same-engine metrics-on/off and worker comparisons retain the same harness boundary; cross-engine p99 is descriptive only.

## Confirmed

The feature-off build removes observational increments for generation-pin count, maximum concurrent pins, read-operation count, and right-link corrections. It leaves the active-pin increment/decrement, the `Arc` generation pin, its lifetime, and the page-reuse guard intact, as shown in [GenerationPublisher::pin](../../crates/dodb-storage/src/blink/mod.rs#L1181), [BlinkReadHandle](../../crates/dodb-storage/src/blink/mod.rs#L1532), and [GenerationPin::drop](../../crates/dodb-storage/src/blink/mod.rs#L1519). The feature-gated test checks a Get's full value, query and scan results, active-pin accounting, page-reuse eligibility, and store invariants. The [worker-threshold test](../../crates/dodb-storage/src/blink/mod.rs#L10173) checks the 31/32-operation dispatch boundary and identical WAL output.

The accepted RocksDB runs reopened the database and passed sampled-key full-value hash-and-length verification. Dodb's phase0 Get success counter treats any `Ok(BatchResponse::Get(_))` as success; its pure-read raw counts do not compare every returned value with an expected value. The [targeted feature-off core test](../../crates/dodb-storage/src/blink/mod.rs#L14013) checks a Get's full value, query and scan results, and pin behavior. This study did not run a full crash and recovery test. The accepted mixed driver records RocksDB's reopen verification at [driver.rs](../../experiments/mixed-harness/harness/common/driver.rs#L1018).

The accepted evidence is limited to the accepted cb55d6fe artifacts linked
above. The older 527cbad3 mixed smoke is not used in the tables. The 55367600
mixed smoke was rejected because its RocksDB background-dispatch requirement
was incorrectly treated as missing; the 527cbad3 read smoke was rejected
because the runner expected `suite=read` while phase0 emitted
`suite=read-scaling`. Those raw files remain quarantined. A warm build made
with Rust 1.98.1 is also excluded; all reported binaries are covered by the
pinned 1.97.1 build evidence.

## Contribution ranking

The durable-mixed evidence ranks the current explanations in this order:

1. **Compact WAL records and superblock elision.** The accepted changing-value
   rows show about 669 bytes per B-link write versus 8,380 bytes for main in
   this one-leaf workload. The prior isolated payload-size intervention found
   that larger payloads increased real-sync time on this ZFS host. This supports
   payload cost as a contributor, not a throughput percentage or ZFS mechanism
   ([isolated payload results](payload-batching-executor-study.md#isolated-payload-io)).
2. **Collection and batching policy.** In the prior accepted c4 changing-value
   comparison, matching main's collection policy raised B-link throughput about
   2.30x and transactions per group from 1.58 to 3.90. It is the strongest
   direct coordinator-policy effect in the paired mixed evidence
   ([mixed results](payload-batching-executor-study.md#mixed-results)).
3. **The optimized parallel physical pipeline.** The prior one-lane fixed-group
   control exceeded planned execution in no-sync runs, but this changes the
   pipeline as well as the executor and retains WAL work; it is not a CPU-only
   or tree-only comparison
   ([fixed-group results](payload-batching-executor-study.md#fixed-group-executor-results)).
4. **Background lane count.** This study measures about a 4% one-lane advantage
   at c4 and a 5% two-lane advantage at c64. Group chunking changes with lane
   count, so the thread's independent contribution is not identified.

Publication-lock interference has not been isolated or ranked. These factors
cannot be converted into additive shares of total throughput.

Pure reads are a separate result. Metrics-off recovers a median 9.49% of c16
Get throughput and 4.48% for Query(16); at c1 the changes are below 1%. After
metrics are off, B-link remains at 89.36% of main for Get and 77.04% for
Query(16). The remaining causes, particularly for Query(16), are unknown.
The data do not support expecting counter sharding alone to recover the
remaining Query gap; that intervention is unmeasured. Cross-engine rates remain
descriptive because key streams and execution models differ.

## Architecture decision

Keep B-link in the experimental branch. Keep the default background threshold
at zero and keep read observational metrics enabled by default. These results
do not justify a production worker-default change, a production rollout, or a
main-B-tree rollback decision.

The adaptive threshold retains the parallel pipeline and only withholds
background-worker dispatch when a planned group's total mutation count is
below 32. It does not route such groups to the serial planned fallback. The
default threshold remains zero, so existing default dispatch behavior is
unchanged.

## Next experiments

1. Profile Get and Query(16), then test a borrowed page view while retaining `GenerationPin`, active-pin accounting, every read metric, and the page-reuse guard. `ReadPageSource::page()` returns `Arc<BlinkPage>` and the generation catalog clones an `Arc` per lookup; the pin retains the immutable page cells that own those pages ([trait and implementation](../../crates/dodb-storage/src/blink/mod.rs#L4215), [PageCell](../../crates/dodb-storage/src/blink/mod.rs#L896)). Validate exact values, range results, publication, and recovery semantics. If a safe view improves reads without failing these gates, keep B-link as the optimization candidate; otherwise leave the read path unchanged.
2. Add a main-path compact-redo counterfactual with the same coordinator, changes, commit framing, real-sync boundary, and recovery semantics. If main matches or exceeds B-link in mixed throughput while retaining its read lead, prefer main. If B-link still wins after WAL parity and passes read gates, retain it for its additional value.
3. Only after core read and write gates pass, run selected longer RocksDB and mixed-workload checks. Continued correctness and performance gates would support a product-readiness review; a failed core gate keeps the engine experimental.

## Outcomes

This study establishes accepted real-sync results for the stated workload, including a same-engine read-metrics control and a lane-count control. It does not establish a single cause for the main/B-link read gap or mixed throughput. No production change, default adjustment, or crash-recovery claim follows from these measurements.
