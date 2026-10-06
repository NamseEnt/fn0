# Main B-tree compact WAL counterfactual

## Scope

This is an explicit experimental main B-tree constructor, not a change to the
default engine or production opening path. `open_with_compact_wal` uses WAL v3
page deltas and elides superblock records when root, allocator, and high-water
metadata remain unchanged. The benchmark selects it with `--main-compact-wal`.
Main's coordinator, collection policy, page layout, committed read view, and
real-sync boundary remain the same.

Each WAL history starts with full data images and a metadata anchor. Following
a checkpoint, the next accepted transaction emits an anchor even when a failed
WAL reset leaves the preceding history in place. Metadata-changing transactions
still emit superblocks. Recovery validates and materializes committed deltas,
then reconstructs trailing metadata-stable generations from the latest
superblock record and the number of subsequent committed transactions. Torn
tails, uncommitted deltas, and non-prefix transactions are excluded by the
existing commit digest and framing rules.

The compact constructor retains the same baseline data-page and superblock
codecs. Compact WAL files must be opened through the explicit constructor;
the default WAL path does not support replaying page deltas. This experiment
does not authorize a production format migration.

## OCI matrix

The mixed comparison uses c4 and c64, changing 512-byte values, uniform 10,000
keys, width one, 50/50 Point Get and writes, real sync, and three paired
repetitions. Its four variants are main with full-image WAL, main with compact
WAL, B-link with main's collection policy, and that B-link with borrowed page
views. All variants retain read observational metrics. This is 24 full cases
and eight smoke cases.

The read comparison uses c16 Get and Query(16), main with full-image WAL,
main with compact WAL, and borrowed-page B-link. Three repetitions with two
seconds of warmup and ten seconds of measurement provide 18 full cases and
six smoke cases. The longer Get windows revisit the 0.899577 boundary observed
in the initial borrowed-page study. The combined full measurement windows
total 384 seconds, excluding builds, initialization, and teardown.

All runs use the designated OCI A1 host with two OCPU, ZFS `dodbbench/db`,
`sync=standard`, `recordsize=4K`, compression off, Rust 1.97.1, and native ARM64
code generation. Source, binary, command, seeds, throughput, latency, CPU,
WAL bytes, and group sizes are retained in the raw evidence. RSS was not
emitted by these phase0 records; memory acceptance remains unverified.

## Correctness gates

Four storage tests cover stable grouped deltas, exact values and revisions,
superblock elision, checkpoint/reset anchors, repeat reopen, splits, overflow,
tombstones, failed conditions, condition-only transactions, ranges, and every
truncation of a final compact transaction.

Three crash-file tests distinguish volatile and durable bytes. They cover
twelve WAL/publication crash points, every persisted byte-prefix of a two-
transaction group under an uncertain sync error, and twelve checkpoint crash
points followed by another durable commit. The checkpoint test found and
fixed the missing-anchor case following a durable checkpoint with a failed
WAL reset. A failed sync never turns an incomplete transaction into success.

The default storage and testkit release suites passed 269 tests, with four
pre-existing ignored microbenchmarks. All 19 read/mixed matrix tests passed,
and the generated 24-case mixed matrix was checked against the independent
configuration validator before scheduling OCI runs. Clippy reported no new
diagnostic source fragments relative to the parent revision; existing
diagnostics still prevent a strict warning-free check.

The feature-on integration check across eight related packages passed 341
tests, with four pre-existing ignored microbenchmarks. On OCI, all 181
feature-on storage library tests and all three compact-WAL crash tests passed.
A subsequent result-only commit removes an unnecessary `mut` from a test;
the targeted test passed again and no measured runtime code was changed.
The [local validation logs](results/oci-a1-2ocpu-12g-zfs-main-compact-99924ae7/local-validation/)
retain these suite results and the Clippy diagnostics with SHA256 checksums.

## Accepted results and provenance

All 42 full cases and 14 smoke cases completed at source
`99924ae7040065f75066da0b033ac4a81149b1f1`. The checkout was clean at that
revision before and after building and measuring. The
[independent validation record](results/main-compact-wal-validation-99924ae7.json)
checks the four matrix manifests, source, native build/test evidence, binary
hashes, raw-file hashes, paired seeds and mixed trace prefixes, feature flags,
real sync, zero semantic errors/conflicts/overloads, latency fields, read
counters, returned Query(16) row counts, and actual WAL record types.
The [raw matrices and build evidence](results/oci-a1-2ocpu-12g-zfs-main-compact-99924ae7/)
retain every command and run log. Native executables remain at the recorded
OCI paths; hashes are retained in both build evidence and run manifests.
Together with the initial borrowed-page study, this continuation completed
60 additional full cases, 20 smoke cases, and four diagnostic profiles.
These counts exclude the preceding read-metrics/worker-policy study.

Ratios below are medians of the three same-seed pairwise ratios, not ratios
of unpaired throughput medians. Bracketed bounds are observed min/max,
not confidence intervals.

| Mixed clients | Compact main / full-image main | Default-read B-link / compact main | Borrowed-page B-link / compact main | Borrowed / default-read B-link |
| --- | --- | --- | --- | --- |
| c4 | 1.669144 [1.606615, 1.825581] | 1.121512 [0.968456, 1.154196] | 1.048560 [1.013305, 1.122342] | 0.972402 [0.934952, 1.046310] |
| c64 | 2.003129 [1.952995, 2.061185] | 1.333779 [1.304873, 1.348956] | 1.334076 [1.288796, 1.371890] | 1.000223 [0.987679, 1.017002] |

| Variant | c4 aggregate ops/s | c4 write p99, ms | c4 WAL bytes/write | c64 aggregate ops/s | c64 write p99, ms | c64 WAL bytes/write |
| --- | --- | --- | --- | --- | --- | --- |
| Main full-image WAL | 4,336 | 3.597 | 8,380.00 | 14,923 | 19.703 | 8,380.00 |
| Main compact WAL | 7,237 | 2.158 | 669.35 | 29,893 | 8.005 | 668.59 |
| Default-read B-link | 8,076 | 1.943 | 669.36 | 39,413 | 5.858 | 668.59 |
| Borrowed-page B-link | 7,751 | 1.996 | 669.34 | 39,880 | 5.931 | 668.60 |

These cells are medians within each variant. Compact main and both B-link
variants emit deltas without full data images or superblocks during the
measurement window. Their WAL bytes per successful write are effectively
matched. Average transactions/group are 3.79 / 3.89 / 3.91 at c4 and
56.77 / 56.62 / 56.05 at c64 for compact main / default-read B-link /
borrowed-page B-link. The collection policy and durability match; achieved
group sizes are observed rather than forced to an identical value.

CPU use, as percent of one core, is 23.79 / 22.39 / 21.99 at c4 and
55.72 / 48.95 / 48.54 at c64 for those same variants. This matrix isolates
the main WAL change; it does not isolate the mechanism behind B-link's
remaining advantage or establish that worker parallelism caused it.

| c16 pure read | Compact main / full-image main | Borrowed-page B-link / full-image main | Borrowed-page B-link / compact main |
| --- | --- | --- | --- |
| Get | 1.020185 [0.979328, 1.033858] | 0.925849 [0.905112, 0.946629] | 0.915628 [0.887204, 0.945392] |
| Query(16) | 0.991332 [0.979683, 1.004566] | 0.960050 [0.942037, 0.971811] | 0.961573 [0.955687, 0.980308] |

Median Get p99 is 1.80 / 1.76 / 1.92 microseconds and Query(16) p99 is
3.12 / 3.16 / 3.32 microseconds for full-image main / compact main /
borrowed-page B-link. Pure reads use approximately 197% of one core for
all variants. The compact constructor leaves main's committed read path
unchanged; these pure-read differences are descriptive run variation,
not evidence that the WAL format accelerates reads.

## Architectural decision

The borrowed-page intervention improved Get by 10.45% and Query(16) by
30.51% against the same-source B-link control in the
[initial study](borrowed-page-view-study.md). The longer follow-up crosses
the 90%-of-main read gate for both operations by paired median. Every pair
passes against full-image main, but one Get pair is 0.887 against compact
main and the earlier short-window Get median was 0.899577. The Get margin
is small; three repetitions do not establish a robust universal read gate.

Compact WAL explains a substantial portion of main's former mixed deficit:
its own throughput rises 67% at c4 and 100% at c64 while WAL bytes/write
fall about 92%. It does not erase B-link's residual advantage in these
matched-WAL cells. The candidate retaining the read improvement remains
4.9% faster at c4 and 33.4% faster at c64, with lower write p99.

Keep the B-link experiment and the borrowed-page candidate. These results
strengthen the case for retaining B-link and do not support returning to
main on the assumption that compact WAL would catch up. Both changes remain
explicit experiments. Borrowed views provide no consistent mixed throughput
gain over default-read B-link; their reason to exist is the read improvement.

This is a 10,000-key, uniform, 512-byte, width-one comparison on one OCI
host. It does not establish broad superiority over RocksDB, repeat the
external-engine baseline, prove memory parity, or authorize a production
format migration. Before widening scope, confirm the narrow Get margin
against compact main with more paired observations and obtain RSS evidence;
do not treat high-concurrency mixed wins as proof that those gates pass.
