# Blink Leaf Payload Sharing Results

## Scope and motivation

The experiment reduces the first-write copy-on-write (COW) cost of ordinary
Blink leaves in `planned-blink`. On a first overlay write,
`WorkingBlinkState::ensure_overlay_page()` clones the committed leaf. Before
this change, the clone copied the leaf entry vector and deep-copied every
entry's encoded key and inline value byte vector.

Base: `9c97fa7fb0c055d37cee9e3e8ddda628b3804259`  
Candidate: `d86f7f6c4fc10acec11ced6aa11d639a13db50ee`  
Implementation commit: `storage: share Blink leaf payloads across COW clones`

## Representation and preserved behavior

The old leaf representation used `LeafEntry.key: Vec<u8>` and
`BlinkValueRef::Inline(Vec<u8>)`. The candidate uses `Arc<[u8]>` for those two
leaf payloads. `LeafEntry` remains a value type and leaf entries remain in a
contiguous `Vec<LeafEntry>`. Overflow references, internal separator keys, and
leaf high keys are unchanged.

Cloning a leaf still allocates and copies its entry vector, and copies each
entry's `Revision` by value. It now clones the key and inline-value Arcs, so
the underlying immutable byte payloads are shared. A changed or inserted key
and value receives new shared storage. Restamping changes only the
overlay-owned entry's revision; it cannot mutate the committed base entry.

The transaction pipeline still builds each transaction's page image and WAL
commit boundary before installing the resulting state. The Arc values are
materialized as the same byte slices when encoding pages. No page or WAL
encoder layout, version, or recovery format changed. Direct-versus-reference
page codec tests and randomized codec equivalence continue to validate the
existing bytes.

## Correctness and local smoke

Added focused tests:

- `leaf_page_clone_shares_key_and_inline_value_payloads` checks key and inline
  value pointer sharing, revisions, and value equality.
- `working_overlay_shares_untouched_payloads_and_isolates_restamps` checks
  sharing for untouched entries and verifies an overlay replacement/restamp
  leaves committed bytes and revisions unchanged.
- `planned_same_leaf_chain_preserves_wal_boundaries_and_revisions` continues
  to verify distinct page images and revision/value state at each logical WAL
  boundary.

All required local gates passed:

```text
cargo fmt --all -- --check
cargo test -p dodb-core
cargo test -p dodb-storage
cargo test -p dodb-storage --bin phase0-bench
cargo test --workspace
git diff --check
```

The local Apple M1 release smoke used the prescribed width-1 workload for one
repetition. Throughput was 63,466 mutations/s on base and 68,978 mutations/s
on candidate (1.087x). Normalized leaf-load clone time fell from 885.6 to
239.8 ns/mutation (0.271x). Both runs had zero errors, overloads, full-state
clones, leaf-entry clones, leaf-install clones, and cached-refresh clones.
This is local smoke evidence; the adoption decision below uses the same-host
OCI A/B runs.

## OCI environment and method

The same host ran both revisions: OCI `VM.Standard.A1.Flex`, 2 OCPU, 12 GB
configured memory, ARM64 Neoverse-N1. The benchmark reported two logical CPUs.
The repository is on the 30 GiB XFS root filesystem (`/dev/mapper/ocivolume-root`),
with no storage layout changes. All runs used `sync-mode=disabled`; these are
CPU/engine diagnostics and are not durability-throughput measurements.

Base and candidate binaries were built independently from their exact source
commits into separate target directories, copied to separate paths, and
verified by SHA256:

| Binary | Source commit | SHA256 |
| --- | --- | --- |
| `target/leaf-payload-sharing-base-9c97fa7-fresh` | `9c97fa7fb0c055d37cee9e3e8ddda628b3804259` | `759b5fa624c6eec18cf1797afaca6890428dc25c137f89d778a35c5ba5ed1026` |
| `target/leaf-payload-sharing-shared-d86f7f6` | `d86f7f6c4fc10acec11ced6aa11d639a13db50ee` | `3899baaff31950565f21eed0d832b29fe2f1df268299ac7a740aba57619aa53f` |

Each base/candidate pair used the same CLI parameters and repetition seeds.
Each raw file has three records. Every row reports zero errors and overloads;
`full_state_clones`, `leaf_entries_clones_delta`,
`leaf_install_clones_delta`, and `cached_refresh_clones_delta` are all zero.
Page-image and WAL-byte counts are identical between revisions for each
workload.

## Width-1 primary A/B

All timing values are medians across the three runs, normalized by successful
mutations. Throughput is mutation operations per second. Parentheses show the
three-run min..max spread. Timing units are ns/mutation.

| Metric | Base median (min..max) | Candidate median (min..max) | Candidate/base |
| --- | ---: | ---: | ---: |
| Mutation ops/s | 35,851.7 (34,553.2..36,279.3) | 42,948.3 (42,573.4..43,012.2) | **1.198x** |
| Leaf-load clone | 2,039.8 (2,025.7..2,045.5) | 652.6 (631.6..726.1) | **0.320x** |
| Physical execution | 5,823.6 (5,822.1..5,862.4) | 4,783.3 (4,712.9..4,883.3) | 0.821x |
| Physical mutation | 3,240.8 (3,227.2..3,242.5) | 1,927.1 (1,881.2..2,012.5) | 0.595x |
| Physical page encode | 2,153.0 (2,148.6..2,162.7) | 2,474.5 (2,456.6..2,479.8) | 1.149x |
| WAL append | 5,140.1 (4,764.2..5,806.5) | 5,387.8 (5,020.1..5,568.0) | 1.048x |
| WAL group encode | 2,910.9 (2,450.1..3,718.0) | 3,450.4 (3,009.9..3,673.5) | 1.185x |
| Planning | 2,729.0 (2,728.2..2,830.0) | 2,614.0 (2,517.4..2,664.0) | 0.958x |
| Catalog construction | 2,721.9 (2,695.7..2,725.7) | 1,602.6 (1,585.1..1,622.7) | 0.589x |
| Generation publication | 1,757.2 (1,754.8..1,799.4) | 1,372.3 (1,345.1..1,372.8) | 0.781x |
| State install | 1,373.1 (1,347.6..1,424.9) | 588.1 (559.3..593.3) | 0.428x |
| Leaf-load clones/mutation | 1.00 | 1.00 | 1.000x |
| Page images/mutation | 1.00 | 1.00 | 1.000x |
| WAL bytes/mutation | 4,224 | 4,224 | 1.000x |

The first-write clone time fell by 68.0% on the OCI host. Throughput rose
19.8%.

## Width-16 regression A/B

| Metric | Base | Candidate | Candidate/base |
| --- | ---: | ---: | ---: |
| Throughput median (min..max), mutation ops/s | 29,018.6 (27,521.4..29,379.6) | 39,291.0 (37,855.0..40,007.5) | **1.354x** |
| Leaf-load clone median (min..max), ns/mutation | 58,400.3 (58,264.5..63,868.2) | 21,441.4 (21,373.9..21,786.7) | 0.367x |
| Errors, all repetitions | 0 | 0 | — |
| Overloads, all repetitions | 0 | 0 | — |

## Unrelated component movement

WAL group encoding increased 18.5% by median. Its base spread was
2,450.1..3,718.0 ns/mutation and candidate spread was 3,009.9..3,673.5, so
the ranges overlap. The `wal.rs` encoder was unchanged by this experiment,
page images and WAL bytes per mutation are unchanged, and total WAL append
rose only 4.8%. The timing spread and unchanged encoding path make run-level
variance the more likely explanation; the evidence does not establish an Arc
effect in WAL encoding.

Physical page encoding rose 14.9%, just below the 15% reporting threshold.
Unlike WAL group encoding, all three candidate repetitions are above the base
range. The timed region covers encoding dirty pages through
`encode_blink_page`; leaf key/value access now uses the shared slice
representation. An Arc-backed slice access/cache-footprint cost is a possible
source-based explanation, but the encoder does not clone Arcs in this timed
region, so atomic refcount work is not an explanation for this measurement.
The A/B data shows the movement; a causal claim would need a separate profile.

Other tracked normalized costs changed by: physical mutation -40.5%, WAL
append +4.8%, planning -4.2%, catalog construction -41.1%, generation
publication -21.9%, and state install -57.2%.

## Classification and remaining costs

**Strong success.** The width-1 leaf-load clone ratio is 0.320 (at most 0.50),
throughput is 1.198x (at least 1.03x), and the correctness/invariant counters
are green. Width-16 throughput is 1.354x base (above 0.95x).

The leaf-load clone is no longer the largest observed physical subcost in
this diagnostic. Candidate physical page encoding is 2.47 us/mutation and
planning is 2.61 us/mutation; total WAL append is 5.39 us/mutation. Sync was
disabled, so none of these values describe durable-write throughput.

## Artifacts

Raw OCI artifacts:

| Repository artifact | SHA256 |
| --- | --- |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/leaf-payload-sharing/planned-base-width1.jsonl` | `13d0ded839fefbae1892c9e24d5c762ca4b5cf07b0925a258b0f52ac4eb1b6b0` |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/leaf-payload-sharing/planned-shared-width1.jsonl` | `d71c9a9caa66c8f96a84eff007d8f207f354df79990ef1bdeb2f986a2482cc04` |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/leaf-payload-sharing/planned-base-width16.jsonl` | `3f689195a70c4669d8ae76a8c968f204f80a3bd93049d3daa0fe28bdb3aa50a3` |
| `docs/experiments/results/oci-a1-2ocpu-12g-200g/leaf-payload-sharing/planned-shared-width16.jsonl` | `9e9108b349c7ed78944eeb916e1d559bf1b7f72ff0110b75798bf2c2e2ab3469` |

Each repository artifact SHA256 matched its remote backup copy at
`/home/opc/dodb-oci-artifacts-leaf-payload-sharing-d86f7f6/`.
