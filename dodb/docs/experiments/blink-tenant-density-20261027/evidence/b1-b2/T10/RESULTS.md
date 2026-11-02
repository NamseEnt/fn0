# T10 streaming checkpoint progress

## Completed substep: bounded paged base-pack output

- Added an ordered `VersionIndex::visit_descriptors` traversal with fixed-depth stack usage and full page-key/path validation. It does not collect a whole-store page-ID set.
- Added a paged durable base-pack writer that accepts exact page images one at a time, verifies page identity and target LSN, preserves sorted order, coalesces at most four images, syncs bounded chunks, writes the final checksum, and syncs the containing directory.
- Added `PagedReadHandle::write_checkpoint_pack`, which counts descriptors in one bounded pass, reserves checkpoint progress memory before buffer allocation, then reloads each exact page through the version-index root and page loader. It verifies image LSN and CRC against the descriptor before writing.
- The focused fixture streams eight pages through a two-page cache in both coalesced and uncoalesced modes. It scans each completed pack and compares every image byte-for-byte. A pinned old reader is streamed after a newer writer transaction and still produces the exact captured pages. A page-write fault leaves an incomplete pack and does not publish a manifest.

## Verification

- Focused fixture: 1 passed.
- Feature library suite: 294 passed, 0 failed, 4 ignored in 124.83 seconds.
- Default and feature library checks, targeted formatting, and `git diff --check` passed.
- Strict feature Clippy exits 101 with 32 existing crate findings. No finding points to the new descriptor traversal or streamed pack writer; no lint suppressions were added.
- Raw logs: `logs/paged-streamed-pack-focused.log`, `logs/paged-streamed-pack-feature-lib-final.log`, default/feature check logs, and `logs/paged-streamed-pack-strict-clippy-final2.log`.
- Initial source commit: `971bb8143b122c5f44169c72ce8ac897da359011`.

T10 remains partial. Streamed pack writing is not yet connected to durable checkpoint-index promotion, paged manifest publication, or WAL segment boundary rotation. Source/index disk reservations, incremental checkpoints, checkpoint debt scheduling, and the complete crash/reopen matrix remain outstanding. T10 remains the next task; no service route, default, or performance experiment changed.

## Completed substep: incremental paged pack output and index application

- Source commit: `e741dd19d0e6940af6290a85413c81d3292f01be`.

- Added an incremental pack request that emits only descriptors whose image LSN is newer than the previous checkpoint. It returns no pack when the target contains no newer pages and marks emitted packs as incremental.
- Added incremental index application that validates the exact target page set, image LSNs, CRCs, and pack checksum before replacing dirty locators. Unchanged descriptors and version tokens remain intact; the resulting index root is promoted to durable storage.
- The focused writer fixture commits a newer two-key transaction, verifies that only a strict subset of the eight-page target is emitted, and compares every packed image to the target index descriptor.
- The focused index test verifies changed-locator replacement, unchanged-source retention, token preservation, and durable-root validation.
- Full feature library suite at this implementation stage: 295 passed, 0 failed, 4 ignored. Default and feature checks and formatting passed; one default test run emitted an unused feature-only import warning that was fixed and verified clean by the final compile check. Strict feature Clippy remains at 32 existing findings, with no diagnostic in the new incremental code.
- A full-fixture attempt to apply the incremental root while old snapshot handles remained pinned exceeded the fixture's version-index resource capacity. The failed log is preserved. The smaller index test passes; realistic checkpoint/index overlap and progress-pool admission remain open acceptance work.

T10 remains partial. The incremental helper now composes durable index promotion, a synced next WAL segment, PagedManifest publication, reopen, and exact reads from old and new pack sources. The live PagedTransactionWriter still does not atomically capture and rotate at the target LSN, and the complete streamed-pack publication path is not integrated. Disk capacity reservations, crash-boundary coverage, scheduler defer/wake, and checkpoint debt accounting remain outstanding. No service route, default, or performance matrix changed.


## Completed substep: progress reservation and incremental manifest reopen

- Complete and incremental index builders now reserve one 128 KiB non-borrowable progress unit before descriptor updates. Durable root promotion reserves another 128 KiB while that lease remains live, so the tested index-build peak is 256 KiB.
- A deterministic 64 KiB fixture rejects checkpoint index work before changing the root or progress accounting. The 256 KiB fixture builds both complete and incremental roots.
- The incremental integration fixture writes a valid prior pack, applies one newer page through an incremental pack, syncs a new WAL segment boundary, publishes manifest generation 2 at LSN 12, reopens the manifest, and reads byte-exact unchanged and changed pages from their referenced pack sources.
- Default library tests: 221 passed, 0 failed, 4 ignored. Feature library tests: 296 passed, 0 failed, 4 ignored. Default/feature checks and formatting passed. Strict feature Clippy remains at 32 existing crate findings.
- Final source commit: `0bdce39d1174eb71c87dc1fa12b4521969228ab8`.

T10 remains partial. The publication test composes storage primitives, but the actual paged transaction writer does not yet serialize target capture with WAL rotation and publication. Source/index disk reservations, combined loader/output/index overlap, complete-pack publication, crash fault matrix, resumable worker slices, and checkpoint debt scheduling remain outstanding.

## Completed substep: serialized writer checkpoint publication

- Source commit: `d99c3e465` (`Integrate paged checkpoints with writer WAL rotation`).
- Added `PagedTransactionWriter::checkpoint_to_manifest`. The mutable writer captures its current committed LSN while excluding later writes, writes and syncs a complete or incremental pack, applies and durably promotes its index root, initializes and syncs the next WAL segment at the captured LSN and current next batch ID, syncs the WAL directory, then publishes the next manifest pointer. Only after pointer publication succeeds does it install the new reader generation and WAL in the writer. Uncertain manifest publication marks the writer unavailable.
- WAL source header identity is exposed from validated scans and newly initialized segments. The writer records the new segment identity for subsequent page locators. The checkpoint API verifies the configured pack directory and that the manifest names the writer's active WAL segment.
- A disk-backed integration fixture performs a complete checkpoint, commits to the newly active WAL, applies an incremental checkpoint, publishes manifest generation 3, reopens the manifest, and reads the exact latest value from the new checkpoint source.
- Feature library tests: 297 passed, 0 failed, 4 ignored. Default library tests: 221 passed, 0 failed, 4 ignored. The focused writer/manifest integration test passed. Default and feature checks plus formatting passed. Strict Clippy reports the same 32 existing crate findings; it reports no diagnostic in the new writer integration, WAL source identity, or source-directory helper.
- Raw logs: `logs/paged-writer-manifest-integration-focused.log`, `logs/paged-writer-manifest-feature-lib.log`, `logs/paged-writer-manifest-default-lib.log`, `logs/paged-writer-manifest-default-check.log`, `logs/paged-writer-manifest-feature-check.log`, `logs/paged-writer-manifest-format.log`, and `logs/paged-writer-manifest-strict-clippy-all-targets.log`.

T10 remains partial. The successful test proves serialized in-process target capture, complete and incremental publication, next-segment rotation, and normal manifest reopen. Process-crash and publication-fault coverage, source-catalog registration and GC retention across the writer path, source/index disk reservations, loader/index/output overlap accounting, the pinned-root capacity counterexample, resumable slices, and checkpoint debt scheduling remain open. No service route, default, or performance matrix changed. T17 was not started because the frozen matrix gate is separate.
