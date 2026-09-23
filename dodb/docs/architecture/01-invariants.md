# Architecture invariants

## Durability

After a future transaction returns `COMMIT SUCCESS`, restart must expose all
of its committed effects.

After a checkpoint at LSN `N` completes, the database file is authoritative for
committed state through `N`; recovery needs only complete committed WAL units
with commit LSN greater than `N`.

## Atomicity and visibility

- A multi-key transaction is applied wholly or not at all; partial commit is
  forbidden.
- Uncommitted transaction state is not visible to another transaction.

## Revision

Revision identifies the key's last committed state transition and is based on
the commit LSN. A missing key still has a revision:

```text
never existed             Missing(0)
inserted at commit LSN 100 Present(100)
deleted at commit LSN 120  Missing(120)
```

This distinguishes deletion from never-existing state and detects missing ->
insert -> delete ABA.

## Identity and routing

- `TenantId != ShardId` semantically; tenant/shard equality is not an
  invariant.
- The physical B+Tree key contains `(pk, sk)` only. Tenant-to-shard routing
  is outside the B+Tree key.

## File format

- Disk formats have explicit magic values, versions, endian rules, and
  checksums.
- Unknown versions are rejected explicitly; they are never read with a
  best-effort decoder.
- Page checksum validation covers the whole 4096-byte page with the checksum
  field zeroed during calculation.
- `checkpoint_lsn` never decreases and advances only after the database file
  and checkpoint superblock metadata have both been synced.
- A reset WAL retains database identity and starts record LSNs strictly after
  the durable checkpoint LSN.

## Transaction validation

Optimistic validation checks the full point read set, including reads that
observed `Missing(revision)`, as well as all write keys. Range reads are not a
v1 transactional primitive.
