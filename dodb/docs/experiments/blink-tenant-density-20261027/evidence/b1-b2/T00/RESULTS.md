# T00 Contract Review

Status: accepted and frozen by independent review. This task changed documentation only; it did not run the implementation, modify a database, or produce experimental measurements.

## Review result

The contract was checked against section 3, the deterministic fixture specification, the mandatory acceptance matrix, and the T00 task card in `B1-B2-IMPLEMENTATION-PLAN.md`. The review found that the contract already specified the compatibility boundary, identity/version rules, persistent layouts, commit-gated WAL visibility, bounded reconstruction, root/source retention, resource ownership, lock/wait/error behavior, checkpoint fairness, transition ordering, integration points, and planned proofs. The amendment also records the version-index v2 and bounded delta-chain sidecar required to reconstruct without scanning WAL.

The review found that deterministic fixture values were stated only in the plan. The contract now freezes seeds `12601` and `12602`, value sizes `512/513/4096` bytes, widths `1/4/16`, conditions and delete/reinsert behavior, tenant populations `1/4/8`, above-cache ratios, reservation boundaries, long-reader checkpoint count, crash/write/crash coverage, fair-turn bound, `1 MiB` slice limit, and the eight-delta limit. These are correctness fixtures, not production defaults.

## Acceptance mapping

| Decision | Contract location and planned proof |
| --- | --- |
| Explicit opt-in, fresh-store boundary, fail-closed compatibility | Compatibility boundary; T04/T12/T15 no-mutation and route-matrix tests |
| Store/source/page-version identity and reuse | Identity and version keys; T02/T03/T07/T08/T11/T15 oracle and reuse tests |
| Fixed persistent formats and publication ordering | Persistent formats; T03/T04/T07/T10/T11 format and crash-boundary tests |
| Commit-gated WAL locations and bounded reconstruction | WAL locators and reconstruction; T02/T07/T11 image/delta/commit tests |
| Durable/runtime roots, leases, and GC | Roots, leases, and reclamation; T05/T10/T11/T15 pinned-reader and unlink tests |
| Shared accounting and progress capacity | Memory and disk reservations; T01/T09/T10/T12-T15 category-boundary and ownership tests |
| Lock ordering, cancellation, shutdown, and error classes | Lock order, waits, cancellation, and errors; T01/T06/T07/T09/T12/T15 barrier and error-propagation tests |
| Fair, bounded checkpoint work | Fair maintenance slices; T13-T15 round-robin, slice, pressure, and service tests |
| Acknowledged-state visibility and recovery ordering | State transitions and acknowledged-state protection; T02/T04/T05/T09/T11/T15 fault and crash/write/crash tests |
| Existing code migration points | Existing integration points and required migrations; task ownership lists T01-T16 |
| Exact deterministic correctness inputs | Deterministic correctness fixtures; corresponding named cases in T01-T16 and T15 service acceptance |
| T17 experiment and adoption | Acceptance mapping and T17 card; separate matrix freeze and execution authorization |

The Rust API names and visibility may be refined as stated in the contract, but ownership, durability, and failure guarantees are fixed. The frozen layouts and fixture values contain no unresolved format or policy alternatives. The contract is a target specification and makes no claim that its acceptance tests have passed.

## Evidence

- Implementation-freeze commit: this task's contract commit, recorded by the T00 receipt update.
- Source tree reviewed: continuation branch at `8c4e39ec4f6e2a50685bcabd617defc1760f4d45`.
- Verification: document-to-plan acceptance review; no code tests were run because this task changed only the specification.
- Remaining work: execute the first unmet implementation card, T14, after preserving the existing T01-T13 receipts; then complete T15 and T16. T17 remains separately authorized.
