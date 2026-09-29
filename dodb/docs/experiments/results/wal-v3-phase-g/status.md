# Phase G execution status

Phase G implementation and gated measurements are complete. The final candidate is G0 mimalloc-only because all arena and reusable-buffer candidates were rejected on total CPU or throughput.

The OCI benchmark host is 217.142.246.204. Strict host-key verification used the existing known_hosts entry. OCI work used detached temporary worktrees; `/home/opc/dodb` was left untouched on its original branch.

mimalloc 0.1.52 is selected at binary boundaries for all executable targets in this workspace. `dodb-server` is library-only here, so its production executable target is absent; that executable must select mimalloc at its own crate boundary. No global allocator is hidden in `dodb-storage`.

G0 has the requested sync-disabled controls, six durable scenarios, three repetitions, stage/size-bucket allocation attribution, and perf stat/perf sampling. The Phase F-to-G public-API deterministic byte comparison passed across 12 configurations. G1a/G1b/G1c, G2/G2b, G3 and G4 candidate changes were measured and rejected. Their source snapshots, release test logs, raw results and decision table are preserved.

Final/G0 is 1.000x because the final candidate is G0. The arena improvement threshold was not met, so the 14-scenario matrix, Phase G RocksDB rerun and 120-second RSS run were not triggered. Final source is committed and pushed; checksums were verified and the worktree is expected clean after the final push verification.
