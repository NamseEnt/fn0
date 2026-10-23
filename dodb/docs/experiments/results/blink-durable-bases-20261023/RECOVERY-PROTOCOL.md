# Durable page-base protocol

## Authority and identity

A version-3 segmented manifest is the sole checkpoint authority. It identifies the database UUID, tenant, shard, epoch, tree format, checkpoint LSN and tree metadata, ordered immutable page packs, and contiguous required WAL segments. The last pack target equals the checkpoint LSN. Each pack binds the same identity, a unique file number, target LSN, image count, length, and checksum. Data pages have their normal page identity and checksum. A missing or corrupt listed file fails recovery. Unknown unlisted artifacts cannot substitute for listed files.

## Checkpoint

1. Fix committed target C, validate invariants, and rotate WAL. Preserve page-chain metadata across the rotation only in the version-3 layout.
2. Snapshot dirty page images at C. The first checkpoint or consolidation snapshots all allocated pages. Later writes use a separate dirty set and newer WAL.
3. Exclusively create a uniquely named immutable pack; write its bound header, ordered page records, and checksum. Synchronize the file and its directory. Never overwrite an old pack or the materialized database cache during checkpoint.
4. Coordinator completion updates the latest manifest, preserving concurrent rotations and acknowledged post-C writes. Select old packs still owning a latest page and the new pack; consolidation replaces the entire prior pack set. Publish C, its tree metadata, pack set and post-C WAL through temporary-file write, file sync, atomic rename, and directory sync.
5. Only after publication remove obsolete covered WAL and fully superseded page packs, then synchronize the directory. Fail closed on checkpoint failure; do not roll live tree/revision state back.

Before step 4 is durable, old manifest bases plus all retained WAL reconstruct every acknowledged write even if a new pack is partial. After step 4 is durable, the new manifest's packs plus post-C WAL suffice regardless of interrupted cleanup. An unlisted partial pack remains a preserved orphan, never a recovery base.

## Recovery

Validate the authoritative manifest and all required immutable packs. Combine ordered packs into the latest checkpoint image per page; verify a complete allocated-page range for nonzero checkpoints. Validate every required WAL segment with the checkpoint images as the first seed and the preceding segment's committed output as each later seed. Apply only complete committed batches and require exact delta-base LSN/checksum, identity, range, and digest continuity. Only the active incomplete tail is truncatable.

Reconstruct the materialized database cache from authoritative checkpoint images plus committed WAL and manifest-selected tree metadata before loading the tree. Cache reconstruction is repeatable; an interrupted reconstruction cannot change checkpoint bases or manifest authority. Successful recovery must permit a new acknowledged mutation and another successful recovery, including after an incomplete transaction tail.

## Failure coverage

Inject failures and process termination before/after pack writes, pack sync, directory sync, checkpoint manifest publication, WAL unlink, base unlink, and reclamation directory sync. Cover background post-target acknowledgements and concurrent rotation. Separately corrupt/truncate listed packs and WAL, alter identity/LSN/base bytes, remove required files, and preserve unrecognized orphans. Unit/fault injection proves modeled write failures; SIGKILL proves process-crash recovery, not arbitrary hardware power-loss behavior.

## Implementation details

Pack headers and page images are 4096-byte aligned; page identity is carried by the existing page header. A pack ends with a whole-file CRC32C trailer; the manifest also binds that checksum and exact byte length. Coalescing groups at most 64 page images per write request. Request counters describe write-all API requests, not an assumed exact syscall count under partial writes.

Replayed post-checkpoint data pages populate the live dirty set after reopen. This is required before the next checkpoint can discard replayed WAL: reconstruction of the disposable database cache alone is insufficient. The regression fixture grows overflow pages, reopens, mutates an unrelated key, checkpoints, and reopens again.

After all required files and the tree validate, reopening can reclaim canonical obsolete WAL using identity/frame/digest/covered-range validation without requiring deleted delta bases. Unlisted base packs require complete identity/page/checksum validation and a target no newer than the authoritative checkpoint. Unknown/corrupt/partial or newer unlisted files remain untouched. Runtime retention admission bounds required recovery files and in-flight output; external disk samples include preserved orphans and the disposable cache as well.

Linux process I/O snapshots record write-request bytes (`wchar`), write syscalls (`syscw`), and kernel-reported block accounting before/after measurement and checkpoint drain. `wchar` includes metadata writes issued by the process and is not a physical-device write count. ZFS block accounting must not be inferred from a zero `/proc/self/io.write_bytes`. Reopen timing includes backend opening and the final-state oracle, so it is not a pure WAL replay timer.
