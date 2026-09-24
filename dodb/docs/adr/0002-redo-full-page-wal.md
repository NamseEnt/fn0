# ADR 0002: Redo-only full-page after-image WAL

Status: accepted

The durability layer uses redo-only WAL containing full-page after-images, with
NO-STEAL and NO-FORCE. ARIES is not the target design. Phase 2 implements this
decision with explicit framing, checksums, commit records, and startup redo.

The WAL carries the complete encoded page image and an explicit commit marker;
the WAL sync is the durability point. The Phase 0 page LSN/checksum fields and
durable-file seam remain the lower-level contracts used by the implementation.
