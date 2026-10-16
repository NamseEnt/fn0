# Header-only transaction state propagation

The transaction consumer needs presence and revision, not the previous payload.
The current adapter copies the entire prior record through the C++/Rust boundary
and then materializes a second payload Vec solely to inspect those fields.
Replace transactional state propagation with its ten-byte header and the full
native record length. Retain the same GetForUpdate calls, sorted protected-key
set, native value retrieval, lock lifetime, conditional semantics, revision
allocator, WAL/sync, errors, and validation of malformed headers and tombstone
trailing bytes. Ordinary Get and iterator reads still return complete values.
Metadata counter reads still return all bytes. This change does not eliminate
native old-value retrieval or claim it is unnecessary under the current
corruption-validation behavior.

## Evidence and controls

First extend the existing overwrite probe to accept 512 B, 64 KiB, and 1 MiB
values, preserving defaults of 512 B and 100 fingerprint workflows. Use an
explicit eight-workflow fingerprint for the 1 MiB condition to fit the short
measurement window; both versions use the same setting and validate the final
value, complete key set, per-key revision advancement, and reopened state.
Capture this probe-only before commit before changing the library/shim.

Primary matrix: existing service_workload, ReservedRanges/Shared 8, metrics
OFF, five scenarios, three repetitions, before/after binaries: 30 runs.
Same 16 clients, seed 20261013, 10,000 seed rows/tenant, 512-byte values, one
second warmup and five-second admission plus drain. Use the already-built
pre-header binary from the required-costs experiment as the before control.
Both allocators receive metrics-OFF and metrics-ON smoke checks.

Overwrite matrix: Put and Exists-conditioned overwrite of 16 existing keys,
512 B/64 KiB/1 MiB, metrics OFF, three repetitions, both probe versions: 36
runs. Add one metrics-ON diagnostic per size/mode/version: 12 diagnostic runs.
Diagnostics rank no throughput. Record complete latency samples, hashes,
commands, elapsed/CPU/RSS evidence, reachability, matching per-client input
prefixes, final-state/reopen results, and before/after source provenance.
Preserve rejected attempts. Archive validated task-owned databases using the
content-verified archive policy from the required-costs runner.

Run an isolated decoder projection probe against the original full-state
logical decoder for valid and malformed records, including every header byte,
short records, unknown tags, present-empty values, tombstone trailing bytes,
and revision boundaries. Report native bytes fetched separately from bytes
propagated to Rust; do not label a ten-byte return as a ten-byte native read.
Keep service and allocator defaults. Short runs remain preliminary; no
native-engine maximum or sustained/crash-recovery claim is authorized.
