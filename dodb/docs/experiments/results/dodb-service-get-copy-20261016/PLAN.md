# Eliminate the ordinary Get encoded-buffer copy

The restored service still allocates an encoded Rust Vec from the native
malloc buffer and then allocates the returned payload Vec in decode_state.
Remove only the intermediate encoded Vec for ordinary Get: borrow the native
slice during the existing decoder, return its owned RevisionState, and free
the native allocation with a guard. Keep the same canonical decoder/errors,
full-value result, C++ native Get/copy, key/value encoding, budgets, metrics,
executor, admission/lifecycle, transaction state path, WAL and sync.

First commit the unchanged-path read probe, then commit the Get change.
Both versions use the same read-only probe: sixteen fixed seeded keys, one per
client, 512 B/64 KiB/1 MiB values, shared eight slots, sixteen async clients,
one-second warmup, five-second measurement plus drain. Seed and compute the
fixed per-client key/value fingerprint before measurement. Every returned value
and revision is checked, and final/reopened complete ordered state is verified.
Latency covers every API call; CPU/RSS is sampled during measurement plus drain
and includes driver full-value comparisons. No writes occur in that window.

Primary read matrix: both versions x three sizes x three repetitions, metrics
OFF: 18 runs. Add one metrics-ON diagnostic per size/version: six runs. Rotate
version order. Capture binary/source hashes, commands, exit status, exact
fingerprints, full samples, completion, CPU/RSS and replay/reopen evidence.
Reuse the content-verified recoverable DB archive policy. Keep rejected attempts.

Run one additional before-original / final-source confirmatory mixed pass on
both allocators and all five existing scenarios: 20 metrics-OFF runs, plus eight
metrics-OFF/ON smoke cases. This checks final source after the write candidate's
rejection and the Get change; one pass is not a new replicated ranking.
The main workload, conditions, widths, budgets, durability and first-100 input
prefixes remain unchanged. Preserve Serialized and Shared 8 defaults.
Use the authorized OCI host/ZFS, pinned native archive, Rust 1.97.1 +crc,
clean source and immutable binaries, with no concurrent build or benchmark.
