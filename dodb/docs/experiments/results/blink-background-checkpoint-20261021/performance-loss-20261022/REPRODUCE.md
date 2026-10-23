# Reproducing the performance-loss investigation

## Frozen implementation and identities

This investigation changes measurement controllers and checkpoint policy only. Engine source remains `d240416f0e0efafa7c08bb1e868387ebd0d894bc`, with the already validated frozen executables:

| Executable | SHA256 |
| --- | --- |
| service_workload | 09bd505d77ce0bcbc61ca46f7389373e60bc1ab1bf6b80a90077bed328418e40 |
| unconditional_write_workload | a697a8c4479835e7fa54aaef4acd16025770193289dbaf63e5ada4fbec48e9cc |

The final main controller was committed before measurement as `b6dc7544bce98e2f0cdc2c516b74aec5dbf42256`; its two script digests are recorded independently in each matrix plan. Diagnostic profiling adds a separately hashed controller. The original controller is copied unchanged from the parent `run_matrix.py` into `controller/run_matrix_base.py`.

Use only `opc@217.142.246.204`, `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation. The measured dataset is ZFS `/bench/zfs/db`, `sync=standard`, `compression=off`, `recordsize=4K`, `logbias=latency`, on two Neoverse-N1 cores. Before/after environment records and the prior build provenance identify the host, clean implementation checkout, Rust 1.97.1, ARM64 `+crc`, and pinned RocksDB 11.8.1.

## Main and preflight

Allocate fresh roots when reproducing; never overwrite the saved accepted cases. The saved roots are `/bench/zfs/db/blink-performance-loss-20261022`, `-smoke`, `-profiles`, and `-profiles-launch`. Supply `source` and `bin` aliases to the verified checkout and frozen binaries. The wrappers reject source drift or binary changes before execution. Copy this directory's controllers and analyzers into `controller/`.

```sh
PYTHONDONTWRITEBYTECODE=1 \
DODB_LOSS_RUN_ROOT=/bench/zfs/db/blink-performance-loss-20261022 \
DODB_LOSS_SMOKE=0 \
DODB_LOSS_CONTROLLER_SHA=b6dc7544bce98e2f0cdc2c516b74aec5dbf42256 \
DODB_LOSS_BASE_RUNNER=/bench/zfs/db/blink-performance-loss-20261022/controller/run_matrix_base.py \
python3 -u /bench/zfs/db/blink-performance-loss-20261022/controller/run_loss.py
```

The main matrix is four paths, two workloads, three rotating repetitions, each 60 seconds after ten seconds of warmup. Preflight sets `DODB_LOSS_SMOKE=1` and a separate root, measuring off/512 MiB for eight seconds after one second of warmup. Main threshold cases must complete at least one checkpoint inside the measurement window. Every case retains its full argv, raw/enriched reports, hashes, resource samples, database-file inventory, and final/reopen validation. Successful database directories are removed after their evidence is saved. Never rerun accepted cells to repair analysis.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 controller/analyze_loss.py \
  /bench/zfs/db/blink-performance-loss-20261022
```

The analyzer independently verifies controller/compile-time source/binary identities, raw/enriched equality and hashes, complete case sets, fixed input prefixes, all latency sample counts and histogram sums, outcomes, final/reopen checks, resource limits, checkpoint presence and drain. p99.9 is computed from the complete histogram, using the executable's rank and bucket conventions, after checking that the independently recomputed p99 matches its reported value. E reports exact maximum latency; RW80 reports the upper bound of the last occupied histogram bucket.

## Separate diagnostic profiles

Run `run_profiles.py` with the same environment pointing at the separate `-profiles` root, only after main measurement has finished. It measures E off/128 MiB for 60 seconds each and attaches user CPU sampling plus privileged entry/exit tracing for `pwrite64`, `fdatasync`, and `fsync`. It captures file-descriptor mappings and both monotonic and wall clocks. These runs retain successful databases. Their timings are excluded from the uninstrumented performance medians.

The supervisor saved in the controller directory waits for all 24 main cases before launching the initial attached profiles. The initial 128 MiB trace failed full byte conservation and is retained as rejected time attribution. A separate `-profiles-launch` root runs `run_launch_profile.py` with the same environment and E/128 MiB only; it launches the workload under inherited syscall tracing, executes the benchmark as opc, and records the exact wrapper command. Its syscall attribution is accepted only when actual WAL write bytes equal reconstructed physical append bytes. This additional diagnostic does not rerun or replace a headline cell. Profiling commands, exit status, trace files, stderr, and descriptor snapshots are evidence. Any profiling limitation must remain visible in the report. Syscall durations include time blocked in the kernel; CPU samples represent sampled CPU execution, not elapsed checkpoint time.

## Historical accounting and offline evidence

```sh
PYTHONDONTWRITEBYTECODE=1 python3 analyze_historical.py
```

This reads the preserved `blink-wal-reclamation-20261020/oci/reports` originals. It verifies saved hashes and controller/build identity, but explicitly records that historical raw reports lack a compile-time `git_commit`. Their identity claim is narrower than the new frozen-binary matrix. Paths in `historical-loss.json` are relative to the `dodb/` subtree.

WAL append conservation is `ending retained bytes - starting retained bytes + reclaimed bytes + reset reinitialization bytes`. Divide by successful mutation transactions, not API calls, to obtain bytes per write transaction. Retained WAL is a resource measurement; it is not the amount written over the whole window. Excluding checkpoint wall time from an observed rate is descriptive accounting, not an experimentally measured counterfactual.

```sh
python3 archive_loss.py \
  /bench/zfs/db/blink-performance-loss-20261022-evidence.tar.gz \
  /bench/zfs/db/blink-performance-loss-20261022 \
  /bench/zfs/db/blink-performance-loss-20261022-smoke \
  /bench/zfs/db/blink-performance-loss-20261022-profiles \
  /bench/zfs/db/blink-performance-loss-20261022-profiles-launch

python3 verify_loss.py evidence/blink-performance-loss-20261022-evidence.tar.gz
```

Archive verification checks the archive digest, every regular-file inventory, safe paths, and independently recomputes every matrix summary without SSH or benchmark binaries. Summaries must match byte for byte. Source trees, executables, and databases are excluded; database filename/size/hash manifests and RocksDB logs remain included.

Legacy checkpoint reclamation is the net reduction in file size. Exact physical append accounting adds the replacement INIT bytes (`wal_bytes_after`, normally 104) for each reset. The accepted launch trace confirms this equality byte for byte.

Run `analyze_profiles.py <profile-root>` on the host to decode and reduce traces. The archive verifier also recomputes every saved profile summary from its preserved decoded trace with `analyze_profiles.py <profile-root> --offline`; it retains `attribution_accepted=false` for incomplete capture rather than presenting partial syscall time as complete attribution. No benchmark executable or perf installation is needed for that offline reduction.
