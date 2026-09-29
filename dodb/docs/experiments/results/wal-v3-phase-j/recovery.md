# WAL v4 Recovery

WAL v4 replay begins from the durable B-link checkpoint plus complete logical transactions after its physical WAL checkpoint LSN. The superblock separately stores the logical checkpoint sequence. A 60-byte v4 INIT payload stores both boundaries; the reader accepts the earlier 52-byte v4 INIT shape for pre-J3 files.

On open, the logical reader validates the complete WAL stream, including transactions already covered by a newer superblock. It returns only committed transactions with physical Commit LSN greater than the superblock watermark for overlay recovery. The highest logical sequence and physical Commit LSN are retained even when there are no transactions left to replay. If the WAL INIT boundary equals the superblock LSN, its sequence must equal the superblock sequence. If a crash occurred after checkpoint sync but before reset, the older INIT is allowed and the covered prefix is filtered.

Recovery never reruns request conditions. It applies only admitted mutations from committed transactions. Mutations are replayed in commit order into a sorted key map, so the latest committed Put or Delete for a key wins. The recovered map becomes one packed immutable overlay segment over the checkpointed B-link base. The next logical revision is one greater than the latest sequence, and the next physical frame LSN is one greater than the latest Commit frame LSN.

An incomplete final mutation or Commit frame truncates the WAL to the beginning of that logical transaction. A complete committed prefix before an incomplete group tail remains recoverable. A checksum failure in a complete frame, malformed length, wrong identity, mixed version, bad record order, or transaction digest mismatch is corruption and is not silently skipped.

| Failure point | Durable view behavior | Reopen behavior |
|---|---|---|
| request validation or logical WAL encoding | no new view | no new commit |
| incomplete mutation or Commit tail | no new view | discard the incomplete transaction tail |
| append error before a complete Commit frame | no new view; shard is degraded after uncertain I/O | recover only the complete committed prefix |
| during WAL sync | no new view and caller receives an uncertain durability error | recover according to bytes retained by the file implementation |
| after WAL sync, before view publication | no new view in the old process | recover the durable transaction from logical WAL |
| materializer page write or data sync, before watermark sync | old base remains the durable checkpoint | replay the retained logical WAL |
| checkpoint watermark sync, before view publication | replacement base is durable; WAL is retained | open replacement base and filter covered transactions |
| retired-page rewrite after watermark sync | replacement base is durable | rebuild the inactive-page free list and replay only transactions beyond the watermark |
| WAL truncate/reset | replacement base is durable before any reclaim | use the retained old WAL, empty-WAL initialization, or the synced new INIT boundary |

`dodb-testkit/tests/phase_j_materialization.rs` runs these boundaries with volatile/durable file images and simulates process loss. It also injects a StorageFull page write and a data-sync I/O failure. The value and revision remain recoverable in every case. Storage unit tests additionally cover reopen, later writes, second reopen, page reuse, overflow values, B-link splits, and randomized differential behavior.
