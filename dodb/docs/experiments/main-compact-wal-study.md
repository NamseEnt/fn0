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

## Planned OCI matrix

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
code generation. The source checkout and raw commit must match the exact
pushed candidate revision. Throughput, latency, resources, WAL bytes,
group sizes, seeds, binary hashes, and command evidence remain in raw results.

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
