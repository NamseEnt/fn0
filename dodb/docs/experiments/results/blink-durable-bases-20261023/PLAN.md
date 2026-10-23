# Durable checkpoint bases and cross-segment page deltas

## Scope and baseline

Baseline: 55041aa19, preserving the original background-checkpoint branch and every earlier result. Worktree: /Users/namse/fn0-dodb-durable-bases. Branch: experiment/blink-durable-bases-20261023. Defaults remain legacy WAL, synchronous execution, and disabled automatic checkpoints. No automatic database conversion, transaction/condition/revision changes, admission changes, group-limit changes, sync removal, or invariant suppression.

## Architecture decision

Introduce an opt-in segmented-bases layout. Immutable, checksummed page packs become the authoritative checkpoint page storage. They replace checkpoint writes to mutable database pages; they are not a second full-page journal. The existing database file is initialization material and a materialized reopen cache. Runtime reads continue using the existing published in-memory page generations.

Manifest version 3 binds required page packs and required WAL segments to the checkpoint LSN. Version 2 remains unchanged for ordinary segmented databases. The first pack contains every allocated data page; later packs contain dirty pages only. Unchanged pages reuse older immutable packs. Packs that no longer own any latest page are reclaimed after manifest publication. Consolidation produces a complete replacement pack when pack count or stale storage crosses the declared bound; it is not performed at every checkpoint.

The active WAL retains page-chain metadata across segment rotation. Recovery seeds the first segment from the manifest's immutable page bases and carries recovered page state across subsequent segments. A base must match page identity, LSN, and checksum; page reuse, new pages, ineligible commits, and non-beneficial deltas retain full-image fallback. Recovery never relies on mutable database pages as delta bases.

The manifest's checkpoint base supplies exactly the committed target state, so pre-target WAL is reclaimable only after all new base files and the new manifest are durable. Until then, the old bases and all required WAL remain. This separates the lifetime of unchanged page bases from checkpoint and segment boundaries without retaining an unbounded redo history.

## Ordered work and gates

1. Preserve this plan and RECOVERY-PROTOCOL.md before implementation.
2. Implement opt-in page packs, manifest v3, seeded WAL scanning, and cross-segment chain transfer; wire synchronous storage and real service paths.
3. Prove repeated checkpoint/rotation/reopen, exact page-delta reuse, conditions/revisions/page reuse, corruption rejection, injected failures, SIGKILL, and recovery -> acknowledged write -> second recovery. Do not begin performance measurements if durability gates fail.
4. Implement separately selectable bounded write coalescing and integrate existing background execution. Keep one checkpoint per shard and latest-manifest publication.
5. Build clean, identity-bearing binaries and validate the unchanged baseline controls plus candidate on the authorized OCI host. Run RW80 (16 callers) and E (64 callers, width 16), 512-byte values, seed 20261013, real synchronous WAL. Compare ordinary segmented sync 128 MiB, bases sync 128 MiB, bases/coalesced sync 128 MiB, bases/coalesced background 128 MiB. Ten-second warmup, three rotating 60-second repetitions: 24 cells, approximately 30 minutes plus validation/builds.
6. Compare the selected candidate and baseline for at least 300 seconds per workload, three repetitions, requiring at least ten completed checkpoints per cell. If the count is insufficient, preserve that cell as incomplete steady-state evidence and use predeclared 900-second confirmation. Stop expansion on correctness/resource/provenance failures; preserve all attempts.
7. Report final/reopen validation, exact latency counts, input-prefix identity, throughput, p99.9/max, checkpoint phase timings/counts, WAL/page-pack/database writes, consolidation writes, retained/temporary/obsolete disk bytes, RSS, and reopen time. Never equate WAL reduction alone with total-write reduction or product adoption.

## Resource and adoption gates

WAL trigger remains 128 MiB and is a soft trigger. The live checkpoint page image is primary database storage. Required recovery storage plus in-flight pack bytes is bounded by that live image size plus the existing 512 MiB extra retention setting in the candidate. Admission conservatively reserves the larger of current checkpoint output and a complete consolidation image; reject/force cleanup before exceeding the bound. Snapshot images retain the existing separate memory bound. First-pack/consolidation peaks and materialized database-cache bytes are explicitly included in reports.

Pack chain count is at most eight after publication. Retaining another incremental pack also reserves one quarter of the extra budget as redo headroom; consolidate when projected retained packs would exceed the larger of the live image size and three quarters of the extra budget. Consolidate before exceeding eight packs or twice the complete live-page pack size. Peak disk includes old packs, the replacement, WAL, and unlisted files; obsolete/orphan files are never silently excluded from resource samples.

Correctness is mandatory. Performance adoption requires lower page-bootstrap amplification and process-issued write bytes, improved E throughput, no material RW80 regression, and acceptable p99.9/max/reopen cost at the declared limits. Report trade-offs rather than changing defaults if these conditions do not jointly hold.

## Capacity correction before accepted measurements

Historical E data show a primary database image of 372,842,496 bytes (about 356 MiB), before WAL. A combined 512 MiB cap for primary bases, redo and immutable replacement images cannot hold the required recovery working set. The correct comparison counts live bases as primary database storage, as with the existing database file, and limits additional recovery storage to 512 MiB. The 128 MiB trigger, 512 MiB snapshot-memory bound, 3 GiB RSS guard and 4 GiB total directory guard remain fixed. The earlier strict combined-cap attempt is preserved separately and cannot enter final implementation medians.
