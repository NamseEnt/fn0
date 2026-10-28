# Harness evidence

These artifacts belong to the tenant-density harness preparation increment. Read [the results and limitations](../../DENSITY-HARNESS-RESULTS.md) before interpreting them.

- `local-resource-*` contains the standalone local accounting-helper check. It is not live Linux backend-memory validation.
- `probe*-source.json` identifies the preserved dirty source snapshots used by the first compile probes. Archive hashes and individual member hashes are recorded separately from Git HEAD.
- `oci/host-pre-smoke-review.json` records the designated host, filesystem, memory/ARC, pinned native source, and toolchain observation before real smokes.
- `oci/builds/` preserves raw commands/provenance, output, and exit status. The clean A1 prewarm build is separate from new harness acceptance. Failed or interrupted probes are retained and are not passing runs.

The task-owned remote root is `/home/opc/tenant-density-harness-20261027`. Source snapshot archives, launcher logs, build output, and frozen smoke binaries remain there. Preserved databases live in uniquely named directories under `/bench/zfs/db/tenant-density-harness-20261027`. Historical host artifacts are not removed or reused as new results.

All smoke results are correctness/accounting checks. Their request rates, latency, and sampled memory must not be used as a density capacity, SLO pass, or throughput ranking. Unsupported metrics remain explicit, and filesystem allocated blocks are not physical write amplification.
