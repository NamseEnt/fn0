# Phase K Read-Path Regression Check

The existing ignored `measure_bounded_overlay_read_path` microbenchmark was run from monorepo commit `08e82ec484bd62aeb659471a39f317b86cfb488f` on the OCI A1 host. It uses a MemoryFile fixture, not the ZFS durability workload. The release test binary SHA256, host, branch, and source path are in `run-order.jsonl`; complete rows are in `phase-k-read.log`.

At four overlays, the maximum J0/H1 ratios were GET 1.407x (oldest hit), Query limit 8 1.750x, and Scan limit 8 1.705x. Phase J recorded 1.459x, 1.745x, and 1.710x respectively. GET improved; Query increased by 0.005x and Scan improved by 0.005x. This is not a material read-path regression.
