# WAL format 3 (page-delta redo)

Code: `crates/dodb-storage/src/wal.rs` at `4ac3a2e` (format and recovery), `6bcf8bb` (planned Blink producer), `11639db` (counters survive a WAL reset).

## Versions

| Version | Written by | Records |
|---|---|---|
| 1 | nobody (read only) | Init (44-byte payload), PageImage, Commit |
| 2 | `BTreeStore` (`WalPageImageFormat::Baseline`) | Init, PageImage, Commit |
| 3 | `BlinkStore` (`WalPageImageFormat::ExperimentalBlink`) | Init, PageImage, Commit, PageDelta |

A WAL file uses one version for all of its frames. A Blink WAL that is still version 2 keeps writing version 2 page images until its next reset (checkpoint), which rewrites INIT as version 3. `BTreeStore` bytes are unchanged: the Phase A byte probe produces the same `btree.wal` and `btree.db` SHA256 before and after (`../byte-probe.txt`).

## Frame (unchanged)

48-byte header: magic `DWAL`, version u16, record type u8, reserved u8 = 0, frame length u32, payload length u32, record LSN u64, batch ID u64, record index u32, reserved u32 = 0, header CRC32C u32, payload CRC32C u32. Then the payload and a 4-byte trailing frame length.

## Record types

| Type | Value | Payload |
|---|---|---|
| Init | 1 | database UUID, tenant, shard, epoch, page size, history start LSN (52 bytes) |
| PageImage | 2 | page ID u64 + 4,096-byte page (4,104 bytes; frame 4,156 bytes) |
| Commit | 3 | first record LSN u64, record count u32, digest u32 (16 bytes; frame 68 bytes) |
| PageDelta | 4 | see below; version 3 only; Blink only; never a superblock page |

Each dirty page of a transaction is exactly one redo record, PageImage or PageDelta. Record LSNs are `first_record_lsn + record_index`, and `commit_lsn = first_record_lsn + record_count`, as before.

## PageDelta payload

```text
offset  size  field
0       8     page ID (u64 LE)
8       8     base page LSN (u64 LE): page LSN of the previous committed image of this page
16      2     span count (u16 LE), 1..=820
18      ...   spans, each: offset u16 LE, length u16 LE, then `length` replacement bytes
```

Header 18 bytes, 4 bytes per span. `PAGE_DELTA_MAX_SPANS = ceil(4096 / 5) = 820`. The payload must be shorter than a PageImage payload (4,104 bytes); the writer falls back to a PageImage otherwise.

### Canonical spans

1. Find every maximal run of bytes where the target page differs from the base page.
2. Walk the runs from low to high offset. Merge a run into the previous span when the unchanged gap between them is at most 4 bytes (`PAGE_DELTA_SPAN_MERGE_GAP`, the size of one span header). A gap of 4 costs the same bytes either way; merging keeps the span count lower.
3. A span's bytes are the target bytes over its whole range, including merged unchanged bytes.

Same base and target always give the same bytes (`encode(base, apply(base, delta)) == delta` is tested for 3,000 random Blink leaf mutations and 2,000 random raw edits).

### Validation

Decode rejects: payload shorter than 18 bytes or not shorter than a page-image payload; span count 0 or above 820; a truncated span header or span body; length 0; `offset >= 4096` or `offset + length > 4096`; a span that starts at or before `previous end + 4` (overlap, unsorted, or a gap that should have been merged); trailing bytes.

Apply rejects: base page LSN different from the delta's base LSN; a span whose first or last byte equals the base byte (not a maximal changed run); a span containing more than 4 unchanged bytes in a row.

After apply, recovery runs the same full Blink page validation used for page images (`validate_blink_page_image`: header, page ID, type, checksum over all 4,096 bytes, body layout), then checks that the page LSN equals the commit LSN. A rebuilt page that fails any check is corruption, not a torn tail.

## Commit digest

Version 3 digest is CRC32C over, for each redo record in order: record type (1 byte), record index (u32 LE), payload length (u32 LE), payload. Versions 1 and 2 keep the old digest (CRC32C of the concatenated payloads). Tests show that omission, duplication, reordering, PageImage↔PageDelta substitution, payload corruption with a fixed frame CRC, and a commit frame spliced from another WAL are all rejected.

## Full-image-first rule

For every data page, the first committed redo record after WAL initialization or reset is a PageImage. Only later changes of the same page in the same WAL history may be PageDelta records, each based on the page's previous committed image in that history.

The writer enforces this with a per-page chain (`page_chain`: page ID → page LSN and CRC32C of the latest committed image), O(unique pages since reset). The chain is rebuilt from committed records when the WAL opens, updated after each successful group sync, and cleared by `WalLog::reset`. A PageDelta is planned only when:

- the WAL is version 3 and Blink;
- the commit is marked eligible (planned Blink: the transaction did not emit a superblock image, so no split, allocation, reuse, free-list or overflow allocator change);
- the page is not a superblock slot;
- the page is in the chain (it has a full image in this history), either from an earlier group or from an earlier commit in the same group;
- a base image is available (same-group image, dirty-page copy, or committed page re-encoded after a flush);
- the base's page LSN and CRC match the chain entry — a mismatch is an invariant error and nothing is written;
- the encoded delta is smaller than a page image and rebuilds the target exactly.

Otherwise the page is written as a PageImage and the reason is counted (`WalRedoStats`).

## Recovery

The open-time scan reads frames in order. Redo records of a batch stay pending until its commit frame is complete and the digest matches; a torn tail or an uncommitted batch never reaches the recovered state. For Blink, each committed record updates `page ID → latest rebuilt image` (a PageDelta is applied to the image already in that map, so it never uses data-file bytes). Memory is O(unique pages in the WAL) plus one pending transaction. `recover_data_file` then writes the final image of every page whose last commit is newer than the checkpoint LSN.

Because every page's chain starts with a full image inside the WAL, the data-file copy of a page is never used as a delta base. A checkpoint that crashes after writing some dirty pages, or after a torn page write, leaves the WAL untouched until the checkpoint superblock is durable, and recovery overwrites those pages with the rebuilt images. `torn_checkpoint_data_page_is_rebuilt_from_wal_image_and_deltas` checks this with the data-file page set to the old checkpoint image, an intermediate committed image, the latest image, half old / half new, and random bytes.

`BTreeStore` recovery is unchanged: page images only, committed batches replayed in order with the same data-file LSN comparison.
