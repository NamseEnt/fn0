# T00 Contract Specification

Status: specification drafted; implementation and experimental execution have not started.

The contract freezes the compatibility boundary, version and source identity, fixed-width manifest and index formats, commit-gated WAL location visibility, bounded reconstruction, source reachability and retirement, shared resource categories, lock/wait/error behavior, checkpoint slice fairness, and acknowledged-state transition ordering. Existing integration symbols are listed from the current source tree.

No runtime code, database, or benchmark was changed or exercised by T00. T01 is the next task. The comparison experiment remains T17 and requires completion of T16 plus its separately frozen matrix.
