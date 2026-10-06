# Payload, batching, and executor study

## Scope and source

This study tests the next decisions after the unified mixed benchmark at
`1a8cf817f0c725e928817e5e45cb046164f05325`. The benchmark changes are at
`76468ad448cb61050655d1b6499cf7dfa6f0448a`. The storage engine algorithms are
unchanged by that commit.

Accepted engine measurements use source
`d01da7ab3d5f6d5998c792cfff7539d977dab231`. This follow-up corrects the runner's
expected repetition number: each isolated phase0 process reports repetition
zero, whereas the external process records the matrix repetition. The first
48-run attempt stopped at that validation mismatch and remains quarantined
with `accepted=false`; its data is not used in the tables below.

The designated OCI A1 host has 2 OCPU and 12 GiB RAM. Measurements use the
`dodbbench/db` ZFS dataset with `sync=standard`, `recordsize=4K`, and compression
disabled. Native release builds use Rust 1.97.1 and `-C target-cpu=native`.
The RocksDB comparator uses version 11.8.1, source
`abeebd9630f11bd08c28b7bd43c7bdfc62050654`, with `WriteOptions.sync=true` and
`disableWAL=false`.

## Questions and controls

1. Does the high concurrency result persist when every 512-byte value changes?
   The legacy control repeatedly writes the same value. Both modes preserve
   the key schedule and request ratio; the changing mode also has additional
   value generation work.
2. How much does the collection policy affect B-link at low concurrency?
   Compare the same B-link engine with its current collection policy and a
   policy matching main's first yield, count and byte limits, FIFO pending
   request, and collection delay rules.
3. Does a background executor worker help at the same group size?
   Compare the same parallel B-link pipeline with one lane and two lanes.
   One lane uses the coordinator and zero background worker threads.
   The planned executor is a separate algorithm control, not a thread-only
   control.

The primary matrix is 50/50 mixed Point Get and width-one writes, 10,000
uniform keys, 16-byte keys, 512-byte values, and 64 MiB configured page cache.
It compares main, current B-link, B-link with matching collection policy, and
RocksDB at c4 and c64, in both value modes, with three repetitions. Each run
has 2 seconds of warmup and 5 seconds of measurement. It contains 48 runs.

The direct fixed-group write diagnostic compares planned B-link and parallel
B-link with one or two lanes at group sizes 4, 16, and 64. It uses changing
values, three repetitions, and real-sync and disabled-sync conditions.
Disabled-sync results are CPU and engine diagnostics and do not establish
durable performance.

## Interpretation limits

- Collection policy changes can affect waiting, batch size, backpressure, and
  scheduling together. A policy comparison does not isolate one yield's CPU
  cost.
- Changing values affect both redo bytes and generation CPU. Throughput alone
  does not identify a WAL or storage cause.
- Fixed groups bypass the benchmark coordinator. Their throughput includes
  request generation; group latency measures the synchronous apply call.
- The external mixed read loop generates a transaction value before reading
  its key. The dodb mixed read loop generates only the read request. This
  harness CPU asymmetry must be considered in RocksDB comparisons.
- Five-second runs do not establish sustained compaction, checkpoint, or
  recovery behavior. This study does not remove the existing pure-read
  regression against main or establish product readiness.
- `wal_sync_nanos` measures the `sync_data` call. Encoding and append timers
  are separate. The isolated payload probe can test storage behavior but
  cannot identify an internal ZFS mechanism.

## Mixed results

The [primary artifact](results/oci-a1-2ocpu-12g-zfs-payload-batching-d01da7ab/)
passes all 48 expected cells. Each raw row matches the source SHA, input
parameters, trace fingerprint, collection policy, and zero error/conflict
gates. Dodb overloads are also zero. RocksDB overloads are unavailable rather
than synthesized as zero; its reopened full-value hash verification passes.
Copied raw/log checksums and metadata were independently revalidated.

Throughput is aggregate logical operations per second, not write transactions
per second. Values are medians of three repetitions. Ratios are medians of
ratios with matching repetition seeds.

| Values | Clients | main | B-link current | B-link matching policy | RocksDB | Matching B-link/main |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Constant | 4 | 4,383 | 3,697 | 8,227 | 5,499 | 1.865x |
| Changing | 4 | 4,378 | 3,413 | 7,915 | 5,591 | 1.808x |
| Constant | 64 | 15,240 | 46,556 | 62,723 | 37,974 | 4.136x |
| Changing | 64 | 15,337 | 38,954 | 40,593 | 38,123 | 2.634x |

The c4 changing-value B-link policy comparison is particularly useful: the
engine and redo representation are the same, WAL bytes per write remain about
669–671, and mean sync duration remains about 0.83 ms. Transactions per group
rise from 1.58 to 3.90 and paired throughput rises 2.30x. The c4 regression under the
current adapter is therefore strongly affected by collection policy in this
workload. This result does not make a claim about every low concurrency
workload or a completed production coordinator change.

At c64, changing-value B-link with matching policy is 1.074x RocksDB
(paired range 1.058–1.077x), whereas constant-value B-link is 1.633x RocksDB.
At c4, changing-value B-link is 1.425x RocksDB (paired range 1.365–1.431x).
At c64, changing values reduce
matching-policy B-link throughput by 35%, and current-policy throughput by
16%. Main and RocksDB throughput vary much less between the two modes. This
demonstrates that constant-value workload shape materially favors the delta
path; it does not erase B-link's advantage over main under changing values.

### WAL and durability metrics

The following metrics are per-repetition ratios followed by their median.
`WAL/write` divides `wal_bytes_delta` by `successful_write_transactions`.
Main remains exactly 8,380 bytes per write in every accepted primary row.

| Values | Clients | B-link policy | WAL/write | Delta span bytes/write |
| --- | ---: | --- | ---: | ---: |
| Constant | 4 | Current | 395.7 B | 243.9 B |
| Constant | 4 | Matching | 256.0 B | 105.2 B |
| Constant | 64 | Current | 158.8 B | 8.8 B |
| Constant | 64 | Matching | 158.4 B | 8.4 B |
| Changing | 4 | Current | 670.7 B | 516.7 B |
| Changing | 4 | Matching | 669.4 B | 515.4 B |
| Changing | 64 | Current | 668.6 B | 514.6 B |
| Changing | 64 | Matching | 668.6 B | 514.6 B |

Constant-mode differences at c4 also reflect differing warmup coverage: a
faster policy changes more originally seeded values before measurement.
Changing mode avoids treating that near-no-op effect as evidence of smaller
groups inherently needing larger redo.

| Changing values | Transactions/group | Mean sync | Sync time / measured wall | Append time / measured wall |
| --- | ---: | ---: | ---: | ---: |
| main c4 | 3.79 | 1.481 ms | 85.6% | 5.0% |
| B-link current c4 | 1.58 | 0.831 ms | 89.6% | 3.1% |
| B-link matching c4 | 3.90 | 0.830 ms | 84.2% | 3.4% |
| main c64 | 57.60 | 5.063 ms | 67.9% | 10.2% |
| B-link current c64 | 32.73 | 1.162 ms | 69.2% | 5.5% |
| B-link matching c64 | 56.86 | 1.860 ms | 66.5% | 5.3% |

These are stopwatch sums divided by wall time, not mutually exclusive CPU
attribution percentages. Main's publication/preparation timers have different
scopes from B-link's physical and publication timers.

### Changing-value tails

| Clients | Engine | Read p99 | Write p99 |
| --- | --- | ---: | ---: |
| 4 | main | 8.76 us | 3.615 ms |
| 4 | B-link current | 8.60 us | 3.514 ms |
| 4 | B-link matching | 7.64 us | 1.954 ms |
| 4 | RocksDB | 14.52 us | 2.330 ms |
| 64 | main | 7.60 us | 17.105 ms |
| 64 | B-link current | 6.04 us | 6.144 ms |
| 64 | B-link matching | 6.80 us | 5.660 ms |
| 64 | RocksDB | 30.20 us | 6.676 ms |

The dodb policy comparison improves write tails much more than read tails.
It does not isolate publication lock contention. External latency includes
different harness and OS thread scheduling costs.

## Fixed-group executor results

The [direct fixed-group artifact](results/oci-a1-2ocpu-12g-zfs-fixed-group-d01da7ab/)
passes all 54 cases. Independent checks verify its manifest and file hashes,
source SHA, release build, exact group size, and one WAL sync call per group.
All transaction outcomes and the latest values of 16 sampled keys are checked
by the binary before a raw result is emitted.

These rates are write transactions per second. One lane and two lanes use
the same parallel physical execution pipeline. The final column is the median
of the paired two-lane/one-lane throughput ratios.

| Group | Sync | Planned | One lane | Two lanes | Two/one lanes |
| ---: | --- | ---: | ---: | ---: | ---: |
| 4 | Real | 4,627 | 4,520 | 4,214 | 0.956x |
| 16 | Real | 13,237 | 14,098 | 13,134 | 0.932x |
| 64 | Real | 20,909 | 22,008 | 22,465 | 1.016x |
| 4 | Disabled | 53,189 | 60,044 | 43,289 | 0.717x |
| 16 | Disabled | 59,967 | 70,409 | 58,405 | 0.834x |
| 64 | Disabled | 65,995 | 77,985 | 73,436 | 0.989x |

Real-sync group-four runs have substantial variation: the paired ratio range
is 0.822–1.013x. Group-sixteen real-sync ratios are all below one
(0.896–0.943x). Group-sixty-four real-sync ratios are all slightly above one
(1.011–1.037x). Disabled-sync ratios are below one in every pair at all three
group sizes, but the measurements still include buffered WAL writes and
filesystem behavior; they are not CPU-only measurements.

The one-lane optimized pipeline outperforms the planned control without sync
at every group size. That comparison includes an algorithm change. The
worker-only comparison shows that a background worker is expensive for small
groups and delivers only a small durable gain for group 64 on this two-core
host. It does not support attributing the mixed high concurrency gain to
executor thread parallelism.

## Isolated payload I/O

The [payload I/O artifact](results/oci-a1-2ocpu-12g-zfs-wal-sync-payload-d01da7ab/)
contains 36 cases: six sizes, append and fixed-length ring modes, and three
repetitions. Each case has 20 warmup calls and 100 measured calls. Source and
script hashes are stable; the source is clean. Each manifest checksum was
independently verified. Payload generation is outside timing.

Sizes represent 669-byte B-link writes and 8,380-byte main writes multiplied
by groups of 4, 16, and 64. They are controlled representative sizes, not
replays of exact engine WAL buffers. The table gives the median of each
repetition's mean syscall duration.

| Payload | Append fdatasync | Ring fdatasync | Append fstat + pwrite | Ring fstat + pwrite |
| ---: | ---: | ---: | ---: | ---: |
| 2,676 B | 0.734 ms | 1.364 ms | 0.026 ms | 0.043 ms |
| 10,704 B | 0.836 ms | 0.999 ms | 0.045 ms | 0.035 ms |
| 33,520 B | 1.514 ms | 2.102 ms | 0.069 ms | 0.070 ms |
| 42,816 B | 2.080 ms | 2.184 ms | 0.086 ms | 0.076 ms |
| 134,080 B | 3.024 ms | 3.000 ms | 0.168 ms | 0.151 ms |
| 536,320 B | 5.193 ms | 5.283 ms | 0.531 ms | 0.459 ms |

The size intervention supports payload size contributing to the sync cost on
this host. A group with 536,320 bytes remains expensive when logical file
length is fixed. File growth alone therefore does not explain the large
sync-duration difference. Write syscalls also get more expensive, but their
timer is much smaller than fdatasync in these cases and is not included in
the engine's `wal_sync_nanos` metric.

Ring mode prewrites 64 MiB of zeros and fixes logical file size. It does not
hold ZFS physical allocation or copy-on-write costs constant. Small ring
sizes are not monotonically ordered by sync duration. This experiment does
not identify ZIL, txg, record alignment, page-cache state, device scheduling,
or another internal mechanism as the cause. No full-system I/O trace or
matched allocated-block experiment was performed.

## Staged c16 mixed control

The [c16 artifact](results/oci-a1-2ocpu-12g-zfs-payload-batching-c16-d01da7ab/)
passes all 12 cases, including independent copied-artifact validation. It
uses changing values and matches collection policy for both B-link variants.

| Engine | Aggregate ops/s median | Minimum | Maximum | Paired ratio/main |
| --- | ---: | ---: | ---: | ---: |
| main | 8,368 | 8,225 | 8,427 | 1.000x |
| B-link one lane | 22,581 | 21,667 | 22,969 | 2.680x |
| B-link two lanes | 22,367 | 22,172 | 23,048 | 2.719x |
| RocksDB | 15,952 | 14,687 | 15,966 | 1.893x |

Paired ratios can differ from ratios of displayed medians. B-link two lanes
versus one lane is 1.021x, with a 0.974–1.023x paired range. This is a small,
mixed-direction difference in the actual queued mixed workload. Both versions
beat RocksDB in every pair: one lane is 1.439x RocksDB (1.416–1.475x range),
and two lanes are 1.445x RocksDB (1.401–1.510x range). Worker threads are
therefore not necessary to obtain the c16 durable advantage observed here.

Write p99 medians are 7.183 ms for main, 2.726 ms for one lane, 2.757 ms for
two lanes, and 3.750 ms for RocksDB. Read p99 medians are 7.96, 6.84, 6.68,
and 12.12 us respectively. These are harness latency measurements, with the
external read-path asymmetry described above.

## Decision and next investments

The results support retaining B-link as the experimental candidate. They do
not support discarding it because of c4, nor attributing its advantage to
background worker parallelism. With changed payloads and matched collection
policy it beats main and RocksDB in the tested 50/50 c4/c16/c64 cases. This is
not an adoption gate for other ratios, transaction semantics, sustained
writeback, crash recovery, or the existing pure-read regression.

The current evidence ranks these investments:

1. Correct collection and compact redo/durability behavior. The c4 policy
   control has a large effect; changing-value redo remains about 12.5x smaller
   than main and the storage size intervention supports a sync-cost effect.
   These effects interact and are not additive percentage contributions.
2. The optimized physical pipeline, including its single-lane form. Its
   algorithm control is faster without sync, while worker-only gains are small
   or negative on this host.
3. Additional executor threads. Use them only after a group-size and end-to-end
   admission experiment establishes a benefit. A threshold that selects the
   planned algorithm would change more than worker count and is not a valid
   worker-only policy control.

The next most valuable product experiment is a compact pure-Get/Query16
profile and observational-counter ablation. `GenerationPublisher::pin` and
`BlinkReadHandle::get` update several shared atomics, including a zero-valued
right-link correction increment. Separately test observational accounting
while preserving generation pin lifetime, the active-pin accounting used by
`can_reuse_pages`, and publication/durability semantics. An improvement here
would identify removable instrumentation overhead; no improvement would
direct the investigation toward traversal, immutable maps, and pin ownership.
This is a hypothesis, not a verified cause of the prior read regression.

A second experiment should replay representative changing-value mixes and
transaction widths with matching collection policy, retaining a single-lane
pipeline control and measuring sustained checkpoint/compaction behavior.
It should include actual crash-and-reopen verification for both dodb and
RocksDB rather than infer all durability semantics from sync options.

A compact-redo main prototype is the architecture counterfactual if a choice
between trees is required. It must cover leaf data and recoverable superblock
metadata; removing only a main leaf image leaves roughly one 4,156-byte
superblock frame per transaction. Current controls do not establish what that
prototype would achieve, so replacing B-link with main now would be premature.

## Reproduction and artifact handling

The runner commands, binary hashes, input metadata, and run order are in each
artifact. [Build evidence](results/payload-batching-d01da7ab-build-evidence/)
records the pinned compiler, effective build commands, static RocksDB archive
hash, and all three release binary hashes. Both source commits produce the
same binary hashes because the second commit changes only runner validation.

Copied mixed artifacts preserve the original absolute run paths. To validate
them elsewhere, use the postprocessing helper's `recorded_results_root`
argument with `/bench/zfs/db/experiment-results/<artifact-name>`. This remaps
only artifact lookup paths and preserves original command records and hashes.
The relocated validation regression and seven other helper tests pass.

The [independent validation record](results/payload-batching-validation-d01da7ab.json)
records the checks performed on the downloaded artifacts.

The complete experiment includes 60 real-sync mixed runs, 54 direct fixed-group
runs, and 36 isolated I/O cases, plus separate accepted smoke runs. The failed
first matrix remains explicitly rejected. No default collection policy or
production storage engine code was changed by this study.
