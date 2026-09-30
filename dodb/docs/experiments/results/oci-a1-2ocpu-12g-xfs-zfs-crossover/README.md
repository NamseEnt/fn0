# OCI A1 XFS and ZFS dodb Crossover Experiment

## Scope

This experiment isolates the observed reversal between XFS and ZFS by sweeping the payload size written before each durable sync and dodb transaction width. It uses only XFS and ZFS on the same `/dev/sda4` partition, instance, and dodb binary. It does not measure ext4 or Btrfs, define a product workload, evaluate snapshots or backups, optimize the storage engine, or select a filesystem.

The stage order is XFS-pre, ZFS, XFS-post. Each stage runs five fio repetitions at each payload and five dodb repetitions at each transaction width. XFS-pre and XFS-post remain separate in reported results. The binary and this experiment's measurements are not pooled with the earlier four-filesystem benchmark because the exact earlier binary was unavailable and a new release binary was built.

## Inputs and provenance

- OCI instance: `VM.Standard.A1.Flex`, 2 OCPUs, 12 GB RAM, Osaka region, same instance OCID and attached boot volume as the four-filesystem benchmark.
- Device: the same 153.4 GiB `/dev/sda4` partition on the same boot-volume device `/dev/sda`.
- Source commit: `08e82ec484bd62aeb659471a39f317b86cfb488f`.
- Engine: `parallel-blink`.
- Release binary SHA-256: `3756eac0557bbe0d87ba3b2ea46ed088df46aa7e8d79fae56f570d8eb52469b6`.
- Rust and Cargo: `1.98.1`.
- The requested earlier binary SHA-256 was not present in the checked host and worktree paths. A detached checkout of the pinned source was built once and that executable is used unchanged for XFS-pre, ZFS, and XFS-post.

## Measurement outline

The fio sweep performs sequential buffered writes with `psync` and one `fdatasync` after each operation. It tests 4 KiB, 16 KiB, 64 KiB, 256 KiB, and 1 MiB payloads, using a fresh preallocated 256 MiB file for each point and five repetitions. If adjacent initial payload points reverse the operations-per-second ordering, only that interval is refined with its geometric midpoint, with five repetitions per stage.

The dodb sweep holds the established diagnostic settings fixed and tests transaction widths 1, 2, 4, 8, and 16. Every point has five single-repetition runs with matching seeds across stages, real sync, unconditional transactions, 64 writers, a 100,000-key working set, 16-byte keys, and 64-byte values.

## Results

Results and the crossover analysis are in [`summary-tables.md`](summary-tables.md). The payload and transaction-width charts are SVG files in [`fio/`](fio/) and [`dodb/`](dodb/). All raw fio JSON and dodb JSONL, per-run metadata, logs, environment captures, setup output, and OCI API responses are preserved in the result tree. Exact setup and command lines are in [`filesystem-setup.md`](filesystem-setup.md) and [`benchmark-commands.md`](benchmark-commands.md).

The experiment reports XFS-pre and XFS-post separately to expose time-related storage drift. Any relation between fio payload crossover and dodb transaction width or WAL bytes per sync is treated as descriptive evidence; the analysis does not infer causation from correlation alone.

## Findings

- The fio median operations/s ordering did not cross: XFS-pre and XFS-post were faster than ZFS at all five measured payloads from 4 KiB through 1 MiB. No refinement point was needed. At 4 KiB the XFS and ZFS median gap did not exceed the combined within-stage sample standard deviation; at 16 KiB through 1 MiB it did. XFS fio CV was high at 4 KiB and 16 KiB (about 23%), driven by a low first repetition, so those medians have visibly greater run spread.
- dodb throughput crossed between width 4 and width 8 in both comparisons: ZFS led at widths 1, 2, and 4; XFS led at widths 8 and 16. The median throughput gaps exceeded the combined within-stage sample standard deviation at all five widths.
- At width 4, WAL bytes/sync were about 24.4 KiB on XFS and 25.9 KiB on ZFS; at width 8 they were about 50.5 KiB on XFS and 49.6 KiB on ZFS. The payload sweep showed no corresponding fio rank reversal, so these results do not support the simple hypothesis that the dodb reversal is explained by a standalone sync-payload-size crossover. WAL sync mean and groups/s follow the dodb ordering more closely: ZFS has lower sync mean and more groups/s through width 4, while XFS has lower sync mean and more groups/s at widths 8 and 16. This is an observed association, not proof of cause.
- XFS-pre to XFS-post median throughput drift was small: fio ranged from -2.2% to +2.0%; dodb ranged from -0.7% to +3.0% (width 16 approximately 0%). dodb WAL sync mean drift ranged from -0.7% to +2.0%. XFS stages are kept separate and are not pooled.
- All accepted dodb runs reported zero errors, conflicts, and overloads. One ZFS 4 KiB fio repetition overlapped a host-state probe; that raw attempt is retained under `raw/rejected-interference/` and excluded. Its isolated same-command replacement is the accepted fifth repetition.

These results apply only to the tested OCI instance, device, binary, and diagnostic settings. They do not select a filesystem or establish a causal filesystem mechanism.
