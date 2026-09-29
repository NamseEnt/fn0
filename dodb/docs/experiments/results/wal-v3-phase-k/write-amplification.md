# Phase K Write Amplification

No Phase K durable write-amplification measurements are retained yet. The OCI run did not start because the target was unreachable.

The local smoke recorded J/K logical WAL bytes of 85,240/10,916,541 and B-link materialization bytes of 6,447,104/10,285,056. It completed 2,702/4,008 transactions, so the byte totals and throughputs are not a controlled amplification comparison. No exact user-byte denominator or H1 comparison is claimed.

The benchmark JSON now reports logical WAL bytes, materialized B-link bytes, checkpoint bytes included in materialization writes, retained WAL bytes, and reclaimed WAL bytes. The measurement report still needs exact logical user bytes committed and total engine bytes written for H1, J, and K. Keep value payload bytes distinct from encoded key bytes and metadata bytes when deriving ratios.

Required ratios are WAL bytes per logical user byte, physical B-link plus checkpoint bytes per logical user byte, and total engine bytes per logical user byte. Use the same workload seed, value/key sizes, filesystem, and sync mode for all engines.
