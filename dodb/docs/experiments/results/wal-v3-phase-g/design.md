# Phase G allocation design and measured outcomes

## Global allocator

The workspace pins `mimalloc = 0.1.52`. Every executable target in this workspace selects `mimalloc::MiMalloc` at its binary boundary. The phase0 `churn-counters` wrapper delegates alloc, zeroed alloc, free and realloc to mimalloc and records size buckets. The storage library declares no global allocator. `dodb-server` is a library-only package in this checkout; its production executable is outside this repository and must select mimalloc at its own binary boundary.

## Attribution

Harness allocations are reported separately from storage. Storage stages include admission, planner, leaf-job construction, leaf-lane execution, job result collection, packed-leaf mutation/COW, PageDelta generation, WAL preparation, WAL append, catalog/publication, state install, dirty tracking and other storage. Global allocator calls/frees/bytes and buckets from 16 bytes through >4 KiB are recorded per stage. Arena logical allocation counters are separate from allocator calls.

## Lifetime classes

| Allocation source | Required lifetime | Rule |
|---|---|---|
| Admission request descriptors and encoded-key scratch | One transaction group | G1c tested group `Bump`; rejected because fewer calls did not lower total work |
| Planner dependencies, last-writer metadata and temporary indexes | One transaction group | G1a/G1b tested group `BumpVec`; rejected on OCI CPU gate |
| Leaf job descriptors and operation indexes | One transaction group through worker join | G2 tested shared contiguous vector/ranges; rejected because total throughput fell |
| Worker result and transaction-to-leaf descriptors | One transaction group through WAL append | G4 tested one flat result-metadata slice; rejected because a small call reduction regressed throughput |
| PageDelta span metadata | One encoder call | G3 removed the span `Vec`; the second page scan cost more CPU than it saved |
| PageDelta bytes | Until WAL append completes | Keep owned until append; never reset an arena before append completes |
| Packed leaf COW output, published `LeafData`, dirty page images | Beyond commit/publication | Normal owned memory only; never arena-owned |
| Catalog entries and published generations | Beyond group while pinned/readable | Normal owned memory only; never arena-owned |
| Returned `TransactionResult` | Outside `apply_transaction_group` | Normal owned memory only |

## Arena reset discipline

G1c candidates put group scratch behind the synchronous `apply_transaction_group` call and reset it before the next group. Their arena-reuse correctness test ran group sizes 1, 64, 3, 128, 2, 32, 1 and 256, verified the values, and reopened the database. It passed. The candidate did not store arena pointers in published state, catalog entries, dirty pages, WAL state, recovery state or returned results. Panic/error cleanup and worker-lane arenas were not retained because the entire arena candidate failed the CPU gate.

No arena capacity survives in the final code, so there is no high-water memory policy to tune. RSS sampling and the 120-second run were gated behind full-matrix success and were not triggered.

## Rejected group-buffer alternatives

G2's operation slice was kept alive through worker collection with owned Arc storage, not a borrowed coordinator arena. G4's contiguous transaction-result records stayed group-local and were consumed before publication. Reusing either buffer across groups would need a separate return-to-owner protocol on every success and error path. Neither measured candidate justified adding that lifetime protocol.
