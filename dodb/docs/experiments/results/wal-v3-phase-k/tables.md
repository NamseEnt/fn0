# Phase K Performance Tables

No Phase K OCI performance rows are retained yet. The registered SSH target timed out during a read-only host check, so no run was started. Do not interpret laptop or in-memory results as durable engine throughput.

The initial OCI gate must report synchronous J versus background K for 64 writers, width 16, uniform distribution, with real sync. Report throughput, materialization total, worker CPU, data write/sync, checkpoint write/sync, publication pause, writer-blocked time, overlay lag, retained WAL, and backpressure. Require at least 90% less materialization-related writer blocking before running the six-scenario matrix.

The final matrix must include H1, J, K, K/J, and K/H1 for 16w width1 uniform; 16w width16 uniform; 64w width1 uniform; 64w width16 uniform; 64w width16 compact; and 64w width16 spread, with three same-session real-sync repetitions. H1 is the decision baseline.

120-second sustained and RocksDB runs remain gated on K/H1 >= 1.20x for 64w width16, K/H1 >= 0.95x for 64w width1, and a stable sustained run.
