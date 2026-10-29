# Preserve the active build cache while recovering boot capacity

The first clean observation-retention test build passed but reduced boot free space to 1,123,028,992 bytes, only slightly above the existing one-GiB build guard. The complete active target occupies about 4.3 GiB; three `dodb-rocksdb` rlibs each embed approximately 1.2 GB of native archive data. Do not begin additional feature links with that boot headroom.

Relocate the complete cache from `/home/opc/tenant-density-harness-20261027/target` to `/bench/zfs/db/tenant-density-build-cache-20261029/target`. Keep the existing logical Cargo target path as a symlink. Before activation, copy all files/metadata/hardlinks and require an empty checksum-based dry-run comparison. Verify runtime/native file hashes before and after. Only then remove the checksum-verified duplicate on boot. This preserves the active cache rather than cleaning it.

Require at least twelve GiB projected benchmark free space after copying. The observed benchmark availability was 37,298,245,632 bytes before relocation, so the 4.3 GiB cache and subsequent builds fit while preserving the ten-GiB experiment reserve. Recheck actual capacity after activation and before every later build/cell.

Keep all source checkouts, the native archive, frozen runtime binaries, unit/build logs, accepted/failed reports, databases, and cgroups. No workload runs concurrently with the move. Preserve the relocation plan, checksum comparison, critical file identities, and before/after capacity under a unique controller namespace. Build-cache occupancy is a host provenance change and must remain explicit in later timing comparisons; workload databases remain on their original ZFS filesystem.
