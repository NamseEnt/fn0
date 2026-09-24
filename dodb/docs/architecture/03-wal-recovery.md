# Write-ahead logging and crash recovery

Phase 2 makes the Phase 1 mutable B+Tree crash-safe with a separate redo-only
WAL. The data file remains a fixed-page file with the Phase 0 double
superblock. The WAL is the durability authority; the data file is a replay
target and may lag the committed view.

## Commit unit and ordering

`prepare_batch` executes all requests in one private overlay. A preparation
failure publishes nothing, so the existing Phase 1 all-or-nothing batch
contract is preserved. A batch containing mutations becomes one storage WAL
commit unit. All document transitions in that unit receive the same committed
LSN, including tombstones. Reads in the same batch observe the private overlay
but are not independently durable units.

The WAL-backed path is:

```text
prepare overlay
  -> stamp page images and revisions with the commit LSN
  -> append PAGE_IMAGE frames and a COMMIT frame
  -> sync the WAL
  -> durability point
  -> publish committed cache/dirty-page state
  -> return success
```

No changed data page or superblock is written to the data file before the WAL
sync. This is NO-STEAL and NO-FORCE: only committed images enter the dirty
page set, and data-file flushing is not required for success.

If WAL sync fails, the shard is marked degraded and rejects further writes
until it is reopened. A successful data-file flush failure is handled the same
way because the in-process persistence state is then uncertain, although the
WAL remains sufficient for recovery.

## WAL format

The WAL begins with one `INIT` frame containing the database UUID, tenant,
shard, shard epoch, and page size. Every frame has the explicit little-endian
layout below; Rust memory layouts are never written directly:

| Offset | Size | Field |
| ---: | ---: | --- |
| 0 | 4 | magic `DWAL` |
| 4 | 2 | format version |
| 6 | 1 | record type |
| 7 | 1 | reserved flags, currently zero |
| 8 | 4 | complete frame length |
| 12 | 4 | payload length |
| 16 | 8 | record LSN |
| 24 | 8 | batch ID |
| 32 | 4 | record index |
| 36 | 4 | reserved, currently zero |
| 40 | 4 | header CRC32C |
| 44 | 4 | payload CRC32C |
| 48 | variable | payload |
| end | 4 | repeated frame length |

The current record types are `INIT`, `PAGE_IMAGE`, and `COMMIT`. A page-image
payload is a page ID followed by one complete 4096-byte encoded image. Page IDs
0 and 1 contain encoded superblocks; other IDs contain checked data pages. A
commit payload records the first page-image LSN, page count, and a CRC32C over
the ordered page-image payloads.

The frame length, header checksum, payload checksum, trailing length, record
type, version, batch sequence, and strictly increasing LSN sequence are all
validated. The WAL identity must match the requested database/shard identity.

## LSN and revisions

Record LSNs are one strictly increasing sequence in one WAL. The `INIT` frame
uses LSN 0. Page-image frames consume record LSNs and the following `COMMIT`
frame's LSN is the commit LSN. A page header and every document revision
changed by the commit use that commit LSN. The next record LSN is persisted by
WAL history rather than guessed from the data-file superblock. After Phase 5
WAL reset, the new `INIT` records the checkpoint LSN immediately before its
history and the next record LSN is the checkpoint LSN plus one.

The superblock `checkpoint_lsn` remains the last checkpoint boundary. Ordinary
WAL commits do not advance it; only a formal Phase 5 checkpoint does.

## Recovery and redo

Open validates the data-file superblocks, scans the WAL, and truncates only an
incomplete final frame. Complete page images without a matching complete
commit are ignored. A checksum, framing, identity, version, impossible-length,
or middle-record error fails recovery; it is never skipped.

Complete commits are replayed in commit order. For a data page, redo is needed
when the on-disk page is missing, invalid, or has a lower page LSN than the WAL
image. An invalid page is therefore repairable when a committed full-page image
exists. Superblock images are replayed in commit order as part of the same
commit unit. Recovery syncs the data file before serving requests, then runs the
existing structural invariant checker. Replaying the same WAL is idempotent.

The WAL is retained from database creation in Phase 2. No reset or truncation
is attempted after data pages are flushed. This intentionally avoids the
legacy shadow-file reset window and leaves checkpoint/retention machinery to a
later phase.

## Data-page flushing

After WAL durability, changed encoded data pages and the selected superblock
image are held as committed dirty images. Reads consult these images before
the data file, so cache capacity zero does not expose stale data. `flush` writes
only committed images and then syncs the data file. A crash before that flush
is repaired from WAL. A crash during a flush is also repaired from WAL.

## Legacy shadow-file finding

The inspected `NamseEnt/namseent` implementation contributed useful ideas:
full-page images, WAL sync before logical success, delayed stale-page writes,
page overlays, cache publication, and request batching. Its WAL body format,
assertion-heavy codecs, and shadow lifecycle were not reused.

The shadow reset window is a real loss scenario, not only a theoretical concern:

```text
WAL is synced
shadow receives and syncs the image
main data file is still stale
WAL is truncated/reset
process crashes before main is synced
```

After restart, the main file contains the old image and the WAL no longer
contains the redo record. The new design has no shadow file and never retires a
WAL record merely because another file received the image.

## Scope and limitations

Phase 2 established the single-commit append path. Phase 3 and Phase 4 reuse
the same framing and recovery rules for a group of logical point-key
transactions:
each transaction still has its own page-image sequence and `COMMIT` frame, but
several sequences may share one WAL sync. Recovery replays those committed
units in order.

OCC validation, MVCC, replication, networking, and background flush workers
remain outside this WAL document. Phase 5 adds local checkpoint and WAL
reclamation plus the contract for an external crash-consistent storage backup;
remote retention, replication, and public PITR remain outside the phase. A
caller may invoke `flush` for a clean data-file image; successful writes do not
depend on doing so.
