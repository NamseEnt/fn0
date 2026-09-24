# ADR 0003: OCC with a shard commit coordinator

Status: accepted

Transactions execute optimistically, track the full point read set including
missing keys, validate at commit, and serialize physical commit preparation
through a shard-level coordinator. Future WAL group commit is coordinated at
that boundary.

The Phase 0 reference model captures validation and atomic apply semantics but
does not implement concurrency, locking, or physical coordination.

