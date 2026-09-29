# WAL v4 Logical Mutation Records

WAL v4 keeps the existing 48-byte frame header, CRC32C header and payload checksums, and trailing frame length. Format dispatch is explicit: `BlinkStore::open_with_wal` reads the retained physical page-image/PageDelta format; `BlinkStore::open_with_logical_wal` opens format 4. A format 3 WAL is rejected by the logical reader, and a format 4 WAL is rejected by the page-redo reader.

Record types 5 and 6 are `LogicalPut` and `LogicalDelete`. Each mutation payload is:

| Offset | Size | Meaning |
|---:|---:|---|
| 0 | 8 | logical revision, little-endian u64 |
| 8 | 4 | encoded-key length, little-endian u32 |
| 12 | 4 | value length, little-endian u32; `u32::MAX` marks a tombstone |
| 16 | variable | encoded key, followed by the Put value |

Each transaction retains its own mutation frames and Commit marker. The 24-byte Commit payload stores the first mutation frame LSN, mutation count, ordered transaction digest, and logical commit sequence. The physical frame LSN advances once per WAL frame. The logical sequence advances once per successful transaction. `TransactionResult` returns both values: `commit_lsn` is the physical Commit frame LSN and `revision` is the logical value used by `RevisionEquals`.

The transaction digest is CRC32C over the format version, record type, contiguous record index, and each exact mutation payload in order. Every frame independently validates header checksum, payload checksum, frame length, and trailing length. Recovery also validates batch identity, record order, commit count, digest, and agreement between the commit sequence and every mutation revision.

An append group encodes each successful logical transaction independently, writes the full ordered frame group, calls `sync_data` once, then publishes one immutable packed overlay segment and releases results. Conditions are evaluated before logging and are not logged or re-evaluated during recovery. Failed requests emit no mutation frames and consume no logical revision.

One transaction with `m` mutations uses `m + 1` WAL records and one 76-byte Commit frame. A Put mutation uses 68 bytes of fixed frame and payload overhead plus encoded-key and value bytes. A Delete mutation uses the same 68-byte overhead plus encoded-key bytes. The current INIT frame is 112 bytes and carries both checkpoint boundaries; earlier 104-byte v4 INIT frames are still accepted.

Superblock version 3 stores the physical WAL checkpoint LSN and logical checkpoint sequence in separate fields. WAL v4 INIT repeats both values after reclaim so revisions never depend on frame count. A v4 reader validates the INIT sequence boundary against a matching superblock checkpoint. When the WAL starts before the current superblock watermark, open validates it and returns only transactions beyond the physical checkpoint LSN for overlay recovery.

The WAL size limit is 256 MiB including INIT. A logical append group that would exceed the limit is rejected before any frame is written. At the four-segment bound the writer runs materialization and resets the covered prefix before admitting the next group.

OCI A1 byte-accounting control: 64 transactions, width 16, 64-byte values. It emitted 1,088 records (17.000 records/transaction), wrote 173,060 bytes (2,704.062 bytes/transaction), and used one sync for all 64 transactions. The exact source commit, binary hash, compiler, and raw output are in `raw/j2/j2-wal-bytes-oci-a1.log`. This used `MemoryFile` and is byte accounting, not a durable throughput result.

J4 real-sync measurements on ZFS recorded 2,717.1 WAL bytes and 17 logical frames per transaction for 64-writer width-16 uniform. The corresponding H1 values were 2,672.0 bytes and 16.60 frames per transaction. For the compact and spread controls J v4 used 2,748.3/2,702.1 bytes and 17 frames per transaction; H1 PageDelta used 536.0/663.3 bytes and 3 frames. The full six-scenario comparison, including syncs and transactions per sync, is in `tables.md` and `raw/j4/durable/`. These are appended WAL bytes, including bytes subsequently reclaimed by a materialization.

WAL v3 source and recovery remain in the same crate as the H1 physical reference. WAL v4 does not change the v3 writer or its on-disk framing.
