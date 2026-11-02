# T07 bounded page loading results

## Changes

- Added a bounded decoded-page cache keyed by store identity, page ID, exact version token, source identity, and delta-chain identity.
- Added coalesced foreground loads with explicit limits for concurrent loads and followers. Cancellation, source errors, shutdown, and cache pressure produce terminal errors and release reservations.
- Added immutable, checksummed delta-chain sidecars. Index publication validates the final sidecar entry, and source collection marks every exact source named by a reachable chain.
- Added direct checkpoint and WAL frame reads through source leases. WAL frame headers, payload checksums, page IDs, delta bases, and reconstructed page LSNs and checksums are validated without scanning WAL.
- Added coverage for current and old versions after cache eviction, and for one, two, and eight delta updates across separate WAL sources.

## Verification

The post-commit serialized feature library suite completed with 292 passed, 0 failed, and 4 ignored. The targeted current/old-source and one/two/eight-delta cases are included in that run. Default and feature library checks, formatting, and `git diff --check` passed.

Strict feature Clippy still reports 12 findings in the existing B-link and WAL modules. No lint suppression was added. The raw logs and their SHA-256 values are recorded in `receipt.json`.

## Limits

The demand-loaded reader is exercised through storage fixtures. It is not connected to the production service route; T08-T12 own that integration. No OCI run, performance comparison, capacity claim, or physical-device miss claim was made. Reservation limits were enforced, but peak category counters were not collected by this correctness suite. T17 remains deferred pending its separately authorized frozen matrix.
