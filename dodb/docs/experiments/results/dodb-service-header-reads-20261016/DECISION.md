# Reject header-only propagation for the service path

Measured source `3dce2f5f7d3ede40258e6eee6bcb4094f14fba30` correctly reduced
old-record propagation to ten bytes, and its decoder projection matched the
canonical state/errors in 101,592 cases. All service and overwrite validation
and reopen checks passed. Byte reduction did not establish an end-to-end
performance improvement. In the metrics-OFF 1 MiB Put overwrite case, median
throughput declined about 4.4% and p99 increased about 19.1%; all three paired
throughput ratios were below one and all three p99 ratios above one.

Do not promote or retain this candidate in the ordinary RocksDbService path.
Restore `src/lib.rs` and `shim/transaction_shim.cc` byte for byte to the
verified `f46340205` required-costs source. That preserves the accepted
metrics-OFF and counted-response-budget changes while restoring full old-state
materialization and the original FFI layout. Keep the synchronous control,
payload-size overwrite probe, standalone projection probe, source commit,
immutable OCI binaries, raw reports, archives, and derived evidence.

The projection decoder remains research-only and is not called by the service.
The measured candidate remains reproducible by checking out its recorded SHA.
No claim is made that eliminating old-value retrieval or weakening corruption
validation would be acceptable, or that the observed tail regression's storage
or scheduling cause has been isolated. This is a rejected performance candidate,
not a failed contract validation and not an accepted service optimization.
