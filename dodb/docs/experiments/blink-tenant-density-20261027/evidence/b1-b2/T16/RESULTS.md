# T16 Results

Status: partial. Source commit: `cb427a3b83ae97ed4017cedfba84714fc0da3a45`. The runtime route remains opt-in.

## Implementation and checks

The feature-enabled worker reports actual paged route verification, version-index and delta-chain format identities, loader/cache counters, source opens, resource limits and category peaks, maintenance state, and bounded process/cgroup samples. The service now initializes a fresh store on the paged route and returns the committed logical revision consistently. A new service test covers fresh write, read, conditional update, shutdown, and reopen.

Local checks passed: storage feature tests (324 passed, 4 ignored), server feature tests (54 passed), formatting/diff checks, server feature check, and strict server Clippy. Storage feature Clippy reported 34 findings in unrelated existing code (14 library, 20 test); no suppressions were introduced.

## Paired OCI correctness smoke

The resident and candidate cells used source commit `cb427a3b83ae97ed4017cedfba84714fc0da3a45`, the same pinned RocksDB native build, seed `20261027`, 4 registered and active tenants, 256 initial rows per tenant, 400 fixed-total mixed operations, one client, and the same 256 MiB cgroup memory cap / 192 MiB high limit with swap disabled.

Both completed 400/400 operations. Request fingerprint was `fnv1a64:1bcef6c52fb9114e` and scheduled fingerprint was `fnv1a64:c862f3ff84e00270` in both cells. Both full-population post-measurement and reopened oracles passed, verifying 1024 seeded rows and 55 mutated keys. The candidate explicitly requested and verified the paged route and reported formats `DDBVIDX1/v2` and `DDBVCHN1/v1`. These are correctness smoke results, not SLO or performance evidence.

The candidate’s reported resource ledger peaked at 159141633 bytes of memory and 80971052 bytes of disk. Its cache recorded 2459 misses, 2427 evictions, and 2459 source opens. Kernel cgroup memory peak was 12,218,368 bytes for resident and 34,410,496 bytes for paging.

## Readiness gaps and capacity gate

The paging snapshot records reachable source bytes as unavailable because the service does not expose a bounded reachability walk. Allocator retained bytes are also unavailable. The report preserves these gaps explicitly; T16 experiment readiness is therefore partial.

At the capacity check, `/bench/zfs/db` had 10,982,387,712 bytes free, only 244,969,472 bytes above the 10 GiB reserve. No workload processes remained. T17 was not launched. Failed smoke reports, logs, and databases, the final successful databases, build logs, frozen binaries and pinned native provenance remain preserved on the OCI host. Copies of reports/logs and the binary hash manifests are in this evidence directory.

## Failed attempts

- Initial feature build at `18679ccff` hit the JSON macro recursion limit.
- Build at `f2b5eccee` required serialization support for paged resource metrics.
- The first candidate smoke exhausted its response budget at the lower per-tenant memory cap; the cap was raised.
- The next candidate smoke found a missing fresh-store paged marker; bootstrap initialization and committed revision reporting were fixed.
- Storage feature Clippy still reports 34 pre-existing findings in unrelated locations.

The raw reports, logs, and failed databases are preserved under `remote/smoke-failed/98df8edc9` and on OCI. See `receipt.json` for exact paths, hashes, configuration, checks, and remaining gates.

Capacity provenance: see logs/oci-capacity-gate.log for the raw OCI resource snapshot and terminal-state checks.

## Follow-up source-boundary measurement

Commit `85c0ffac08dce0328be8193d6a8a4a657034dec4` adds reachable-source measurement at explicit service observation boundaries. The preserved paired reports under `remote/smoke-instrumented-85c0ffac0/` show the candidate's reachable logical source bytes at the measurement start and end as 2,282,528 and 3,195,260 bytes across four files. The post-measurement health snapshots also report 3,195,260 bytes. These values cover logical file lengths; they are not allocated filesystem blocks or physical disk-return measurements.

The measurement-worker snapshot includes 3,195,260 reachable bytes. Periodic resource samples still report `reachable_source_bytes` as unavailable because the full reachability walk is deliberately not performed at every sample. Allocator-retained bytes remain unavailable in every phase. The pair completed 400/400 operations per route with matching request/schedule fingerprints and passing full-population and reopen oracles; it remains correctness and instrumentation evidence, not a performance result.

T16 remains partial. Before accepting readiness, either add and validate bounded periodic reachability and allocator-retention observations, or amend the T16 measurement contract with a reviewed explicit boundary for each unavailable quantity and show how the existing cgroup/process observations bound the experiment. Then freeze the numeric T17 matrix and its input recipes, hashes, overlap sizing, acceptance rules, reviewer, and capacity. Do not run T17 under this receipt.
