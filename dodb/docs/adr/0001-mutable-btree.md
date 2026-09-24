# ADR 0001: Use a mutable B+Tree

Status: accepted

The future storage engine will be a mutable B+Tree with fixed 4KiB slotted
pages. Persistent copy-on-write B+Trees are not part of v1. Small values are
inline and large values use overflow pages.

This keeps page identity stable for the planned redo full-page WAL and avoids
introducing full-MVCC storage into the first version.

