# Planned Blink Leaf Ownership Results

## Motivation

The direct page encoding run left 176.729 ms of measured width-1 clone work,
40.97% of physical execution. Most of that work came from copying a leaf page
into a private cache, copying its entry vector for each mutation, copying the
result back into the working state, and refreshing the cached copy at each
transaction boundary.

## Previous Duplicate Ownership

The planned serial path held one `BlinkPage` in `WorkingBlinkState.pages` and a
second `BlinkPage` in `CachedLeaf`. Mutations copied the entry vector again,
then installed another page copy and refreshed the cache after restamping.

## Design

### Working overlay ownership

`WorkingBlinkState.pages` owns the only mutable page copy for the batch. Its
`ensure_overlay_page()` helper copies a page from immutable committed state
only when that page first enters the overlay.

### PageId-only CachedLeaf

`CachedLeaf` stores only the leaf `PageId`. Cache range checks read the current
overlay-first page through `WorkingBlinkState`, preserving high-key and first
entry lower-bound checks.

### First-write COW

Committed `BlinkState` remains immutable until WAL success. The first planned
mutation of a committed leaf copies it once into `WorkingBlinkState`; later
mutations of that page reuse the overlay-owned copy. The `leaf_load_clone`
metric now measures only this base-to-overlay COW. Its historical field names
remain for artifact compatibility.

### In-place leaf mutation

Existing-key replacement and insertion mutate the overlay leaf's entries
directly. The code preserves provisional page LSN updates, exact leaf fit
checks, dirty tracking, value allocation, and freeing replaced overflow
chains. Normal leaf mutations no longer increment entry-vector or install
clone counters.

### Split ownership transfer

On insertion overflow, the planned path verifies the route before taking the
old high key and moving the entry vector out with `mem::take`. Existing split
helpers then install the left and right pages and parent separator. This avoids
a pre-split entry-vector clone while retaining split behavior.

## Correctness

The focused same-leaf transaction test verifies one base-to-overlay clone for
four mutations, zero entry/install/refresh clones, separate revisions and
values in each transaction's WAL page image, and the existing final read and
invariant checks. A focused oversized existing-key update test verifies that
an error leaves committed pages, root/free-list/high-water state, superblock,
slot, revision, LSN, batch ID, publication count, WAL commit count, and visible
contents unchanged.

At implementation commit `fd0c1029ad053a00ea99eef45f524f6ca795b289`, these
local correctness gates passed:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core`
- `cargo test -p dodb-storage`
- `cargo test -p dodb-storage --bin phase0-bench`
- `cargo test --workspace`
- `git diff --check`
- Existing planned randomized differential, leaf/internal/root split,
  planned WAL failure, WAL sync failure, parallel WAL failure, and pinned
  generation tests all passed.
- The recovery child suite passed 8/8 in both storage and workspace runs; no
  recovery child failed during this task.

Local width-1 smoke used 16 writers, `different-leaf-heavy`, working set and
cache 4096, disabled sync, zero delay, and 1 s warmup/duration. It reported
zero errors, overloads, full-state clones, entry clones, install clones, and
cached refresh clones. This is local smoke evidence only.

## OCI Method

SSH works with `/Users/namse/Downloads/ssh-key-2026-09-23.key`, fingerprint
`SHA256:i9NJ3mGGroLtw6CjNS8XJCfT4OjDlfp384E0cOk71ro`, and
`IdentitiesOnly=yes`. The previous failure used the wrong
`/Users/namse/.ssh/id_late_sh_ed25519` identity. The remote precheck found a
clean worktree and no running benchmark process. Both release runs used
implementation commit `fd0c1029ad053a00ea99eef45f524f6ca795b289`; every raw row
records that exact `git_commit`.

The instance has 2 logical CPUs and 12 GiB RAM. Its 200 GB block device is not
mounted as the benchmark filesystem: `/home/opc/dodb` resides on the 30 GB XFS
root filesystem, with 18 GB free before the run. These sync-disabled runs are
OCI CPU and engine diagnostics, not durability results or measurements on the
200 GB volume.

Both workloads used 16 writers, working set 100,000, cache 4,096, 16-byte keys,
64-byte values, group limit 64, group byte limit 4,194,304, queue 256, zero
collection delay, disabled sync, two Tokio workers, 1 s warmup, 2 s duration,
and three repetitions. Width-1 used `different-leaf-heavy` and seed
`0x3a042026`; width-16 used `uniform` and seed `0x3a032026`.

## Width-1 Result

The table compares the previous OCI median with the median of the three new
repetitions. All three runs completed with zero errors and zero overloads.

| metric | before | after | change |
|---|---:|---:|---:|
| throughput (mut/s) | 17,964.27 | 20,123.54 | +12.02% |
| processing (ms) | 1,891.100 | 1,877.333 | -0.73% |
| physical execution (ms) | 431.334 | 271.109 | -37.14% |
| physical mutation (ms) | 217.881 | 111.993 | -48.60% |
| mutation residual (ms) | 89.449 | 47.512 | -46.88% |
| physical restamp (ms) | 3.857 | 4.127 | +7.00% |
| leaf load clone (ms) | 28.236 | 64.481 | +128.36% |
| leaf entries clone (ms) | 40.149 | 0.000 | -100% |
| leaf install clone (ms) | 60.047 | 0.000 | -100% |
| cached refresh (ms) | 48.297 | 0.000 | -100% |
| physical residual (ms) | 45.128 | 26.124 | -42.12% |
| clone bundle (ms) | 176.729 | 64.481 | -63.50% |
| physical page encode (ms) | 79.592 | 88.604 | +11.32% |
| superblock encode (ms) | 36.580 | 40.261 | +10.06% |
| planning (ms) | 397.507 | 443.824 | +11.65% |
| WAL append (ms) | 606.015 | 659.585 | +8.84% |
| WAL group encode (ms) | 475.750 | 514.048 | +8.05% |
| catalog construction (ms) | 103.312 | 120.480 | +16.62% |
| generation publication (ms) | 81.850 | 81.073 | -0.95% |

## Clone Elimination

In all three OCI width-1 repetitions,
`leaf_entries_clones_delta == 0`, `leaf_install_clones_delta == 0`, and
`cached_refresh_clones_delta == 0`. Their cumulative timings are also zero.
The first-write base-to-overlay COW remains expected: median time 64.481 ms,
median count 40,283, and 1.0 COW per planned mutation. Each repetition had one
load clone per planned mutation.

## Width-16 Check

The width-16 control completed with zero errors and zero overloads. Its median
was 18,746.32 mut/s versus the prior 18,016.82 mut/s, a 4.05% increase.

## Interpretation

**Structural success.** The three targeted clone counts are zero, the
64.481 ms clone bundle is below the 88.365 ms structural threshold, and
throughput is above the 17,964.27 mut/s baseline. The 64.481 ms bundle is above
the 61.855 ms strong-success threshold, so the result does not meet the strong
classification.

## Next Bottleneck

The largest measured physical subcomponent is page encoding at 88.604 ms,
followed by first-write leaf COW at 64.481 ms. The largest top-level component
is WAL append at 659.585 ms, including 514.048 ms of WAL group encoding. These
timings identify measurement priorities; they do not by themselves show that
allocation is the limiting cost.

## Allocator / Arena Assessment

**Not recommend.** WAL group encoding is the largest part of the top-level
WAL append time, and page-encoding time increased from the prior result. The
timings do not isolate allocation cost or show that an arena would improve
either path. No `bumpalo`, arena, or allocator dependency was added.

## Experiment Status

Implementation commit: `fd0c1029ad053a00ea99eef45f524f6ca795b289`.

Implementation and local correctness work are complete. The OCI diagnostic is
classified as structural success; the 30 GB root-filesystem and disabled-sync
limits above apply. Committed Blink state remains immutable until WAL success.
Only batch-local `WorkingBlinkState` ownership changed. Page format, split
semantics, WAL semantics, transaction ordering, and publication semantics are
unchanged. Phase 4 status is unchanged and Phase 5 has not started.

## Artifacts

- Local smoke: `/tmp/dodb-leaf-ownership-smoke.jsonl` (local-only, not an OCI
  performance artifact).
- OCI width-1:
  `docs/experiments/results/oci-a1-2ocpu-12g-200g/leaf-ownership/planned-leaf-ownership-width1.jsonl`
- OCI width-16:
  `docs/experiments/results/oci-a1-2ocpu-12g-200g/leaf-ownership/planned-leaf-ownership-width16.jsonl`
- OCI backup: `/home/opc/dodb-oci-artifacts-leaf-ownership-fd0c102/`
- Raw artifacts were copied locally and SHA256 matched the OCI copies before
  moving them outside the OCI worktree.
- Width-1 SHA256:
  `315cb6b1a9559b0c7821c0da522656361c5856d856e9ab2c14728c69b34f2e47`
- Width-16 SHA256:
  `f00f12947b4c9c9e14a90268ff6eb60a94b8fdbed3861ff8e3b841bfa098c632`
