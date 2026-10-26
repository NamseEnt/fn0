# Reproduction

The diagnostic root is `/bench/zfs/db/backend-profiling-20261026` on the designated `opc@217.142.246.204` OCI host. Use `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes` and normal host-key validation. Keep the frozen `/bench/zfs/db/blink-checkpoint-costs-20261023/source` and its original `bin` directory unchanged. [The preceding experiment](../blink-checkpoint-costs-20261023/REPRODUCE.md) records the original source/build provenance.

## Existing evidence, without running workloads

From the fn0 checkout:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  dodb/docs/experiments/results/backend-profiling-20261026/verify_profiles.py \
  dodb/docs/experiments/results/backend-profiling-20261026/evidence/backend-profiling-20261026-evidence.tar.gz

PYTHONDONTWRITEBYTECODE=1 python3 \
  dodb/docs/experiments/results/backend-profiling-20261026/compare_profiles.py \
  dodb/docs/experiments/results/backend-profiling-20261026/evidence/backend-profiling-20261026-evidence.tar.gz \
  --output /tmp/backend-profiling-comparison.json
```

The verifier checks the archive digest, every inventoried file, raw source/binary identities, actual per-client input prefixes, latency counts, final/reopen validation, full-window acceptance and folded CPU sample totals. The compact comparison is derived from the archived raw/decoded reports and folded stacks. Reproducing it does not execute a workload.

Extract the archive to a separate directory for the original CPU `.data`, demangled `perf script` gzip files, every raw workload report, resource samples, BPF programs/maps, command records, complete per-phase SVGs and collector/reducer snapshots. Original binary/source payloads and task database payloads are excluded and remain at their recorded host locations. The archive includes the original source/license bundle for FlameGraph and its pinned Git identity.

## Fresh collection

Create a unique new root on the same ZFS dataset and copy the three collector scripts (`run_profiles.py`, `gate.py`, `collect_waits.py`) together into its `scripts` directory. The runner verifies the original source and executable hashes and refuses to overwrite a case directory. Keep at least 10 GiB free and 1.5 GiB host MemAvailable, the 3 GiB workload-process RSS guard, 4 GiB per-database guard and 2 GiB CPU trace ceiling.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 /path/to/scripts/run_profiles.py \
  --root /bench/zfs/db/a-new-unique-profile-root --stage preflight

PYTHONDONTWRITEBYTECODE=1 python3 /path/to/scripts/run_profiles.py \
  --root /bench/zfs/db/a-new-unique-profile-root --stage main
```

The host must provide passwordless task-scoped `sudo`, `perf` with DWARF unwinding, Python BCC, the kernel tracepoints and `rustfilt`. Collectors attach before the gate executes the workload. Preserve failed cases instead of overwriting them. Run main only after all four preflight cells pass; do not run unrelated build/reduction/compression work concurrently with measurements.

After all workloads finish, use the final `reduce_profiles.py` with the original host paths/binaries available for symbolization:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 /path/to/reduce_profiles.py \
  --root /bench/zfs/db/a-new-unique-profile-root \
  --flamegraph /path/to/FlameGraph
```

The FlameGraph checkout must be at `41fee1f99f9276008b7cd112fca19dc3ea84ac32`. CPU timestamps match the report's realtime window; checkpoint phases use the recorded checkpoint-event intervals. No request-level timestamp correlation or allocation/retained-heap profiling is performed. Off-CPU samples are deliberately pooled because the original collector's raw state-zero flag does not fully classify this kernel's preemption state. Full start-second buckets can contain waits ending beyond the window. Use the graphs and syscall timings with these declared limits.

`archive_profiles.py` freezes source/binary identities again, inventories evidence and excludes databases while retaining them on the host. It refuses to overwrite an archive. The archived initial reducer is retained for audit; the final reducer narrows allocator patterns, matches demangled Rust checkpoint methods and attributes allocation CPU to the nearest project caller. No workload was rerun for those reporting corrections.
