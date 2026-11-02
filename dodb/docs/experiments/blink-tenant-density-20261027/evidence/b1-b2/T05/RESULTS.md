# T05 root reachability, source leases, and safe GC

Status: accepted for the T05 storage foundation. Implementation commit: `5d7d267ab` (`Add bounded paged source leases and collection`), pushed to the dedicated branch.

`VersionIndex::visit_sources` walks the immutable radix tree with a bounded traversal stack and emits checkpoint, WAL, and delta-base dependencies. For a WAL-to-WAL delta chain, it emits a conservative retain-all-WAL marker because the descriptor does not encode the full ancestry chain. GC writes reachability markers to a per-run scratch directory, then checks candidate source paths one at a time. It only accepts explicitly supplied recognized candidates, verifies their source header digest before unlink, preserves unknown files, and syncs the parent directory after unlink.

`SourceManager` holds bounded root registrations, charges each registration and open source handle through the shared resource manager, and caps concurrent handles. Root registration checks the index store identity. `pin_manifest_root` reads the selected durable root from `PagedManifestStore`; other roots can be pinned for runtime readers and operations. A GC gate stops new root registrations and source opens from beginning after the root snapshot and until sweep completes. Existing leases remain active and keep their files from unlinking.

## Acceptance evidence

- A durable manifest is published, closed, reopened, and used to restore its root pin while an older reader root remains pinned.
- The old reader reopens its checkpoint source and reads the exact page bytes after checkpoint publication. GC preserves both roots' sources; dropping only the old root allows its checkpoint source to be reclaimed while the current root's source remains.
- Delta-to-WAL dependencies retain the WAL candidate run conservatively. The broader retention is released after the root is dropped.
- A barrier-synchronized load/retirement race either admits a verified active lease and prevents unlink, or starts GC first and rejects the late load before unlink.
- Injected failures before unlink preserve the file. An injected failure after unlink reports the interruption with the unreachable file removed.
- Missing reachable files fail source acquisition and fail collection. A wrong header identity fails closed. An unrecognized foreign file remains untouched.
- Open-handle and root-pin caps reject excess work. GC reports candidate, protected, leased, and unlinked bytes separately; `unlinked_open_bytes` remains zero.

## Limits

This is local storage correctness evidence. No SIGKILL process test, whole-process RSS cap, service-route integration, OCI run, performance matrix, or production suitability was evaluated. Source candidates are supplied explicitly; the new source manager is not yet connected to segmented-WAL startup cleanup or checkpoint scheduling. The current WAL delta fallback can retain all WAL segments until a checkpoint removes the dependency.

The full test log, focused attempts including failures, compile checks, strict Clippy output, and formatting logs are retained under `logs/`. See `receipt.json` for hashes and exact commands.
