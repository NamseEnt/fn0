# Storage allocation sources and lifetime classification

The measured G0 OCI primary workload is 64 writers, width 16, uniform, sync-disabled. Harness allocation counters remain separate from storage counters.

| Source | G0 alloc calls / tx | G0 alloc bytes / tx | Lifetime | Arena decision |
|---|---:|---:|---|---|
| Planner | 103.46 | 9,766 | A: group-local metadata | G1 candidates rejected on total CPU |
| Leaf lane execution | 92.49 | 103,490 | Mixed: scratch plus owned page/redo outputs | Only scratch/metadata eligible; no lane arena retained |
| Admission | 70.17 | 4,866 | A: group-local descriptors and encoded checks | G1c cut calls but did not improve total work |
| PageDelta generation | 32.17 | 2,745 | A until WAL append | G3 halved calls but regressed throughput |
| Catalog/publication | 36.97 | 4,698 | B: may be reachable from pinned state | Never arena-own published objects |
| Job result collection | 7.04 | 6,053 | A until WAL append | G4 flat slice saved about one call but regressed throughput |
| WAL append | 0.39 | 3,889 | A until append completes | Keep bytes owned through append |
| Benchmark harness | 166.06 | 7,980 | Harness-owned | Excluded from storage headline |

The full stage output, frees and allocation-size buckets are in `oci-g0-stage-attribution.csv`. Persistent published `BlinkPage`, packed `LeafData`, final dirty images, catalog state, WAL/recovery state and returned results are class B and must remain normally owned. Temporary descriptors can be class A only while no reference escapes `apply_transaction_group`.
