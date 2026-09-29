# Public API byte comparison

The two captured outputs exercise seeds 11–13, value limits 120/40, and serial/two-worker execution through the public storage API. They record transaction results, LSNs, reads/queries/scans/reopen scans, and data/WAL SHA256 with sizes.

The final G0 runner uses `src/main.rs` and `Cargo.toml` in this directory. The Phase F runner was built from commit `74d65b6`, which is part of this repository history. To reproduce both runs:

1. Create the Phase F source worktree: `git worktree add --detach /tmp/dodb-phase-g-phase-f 74d65b6`.
2. From the repository root, run `cargo run --release --manifest-path docs/experiments/results/wal-v3-phase-g/crossver/Cargo.toml`.
3. Run `cargo run --release --manifest-path docs/experiments/results/wal-v3-phase-g/crossver/f-source/Cargo.toml`.
4. Compare stdout with `diff -u` against `output-g0.txt` and `output-phase-f.txt` respectively, then compare the two output files.

The F manifest points to `/tmp/dodb-phase-g-phase-f` and omits mimalloc because it reproduces the historical Phase F allocator selection. The G0 executable declares mimalloc at its binary boundary.
