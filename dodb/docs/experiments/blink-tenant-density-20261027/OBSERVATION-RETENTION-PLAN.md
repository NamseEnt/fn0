# Remove retained observation DOM overhead before engine-density attribution

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: follow-up plan only. Finish and preserve the frozen current-source main/supplemental cohorts before changing runtime source. Do not reinterpret those instrumented observations as an engine-only memory comparison.

## Measured trigger and source finding

The frozen one-tenant E2 cadence sensitivity completed identical sixty-second recipes. Shared RocksDB's measurement-worker lifetime kernel peak was 8,351,744 bytes at 1,000 ms and 35,336,192 bytes at 100 ms. Both met the objective. This is one cadence observation, not an exact isolated byte attribution or proof of the same effect at larger populations.

The worker keeps periodic observations as `Vec<serde_json::Value>` until shutdown. Nested per-database property maps therefore retain a much larger representation than their serialized byte count. The existing 8 MiB bound limits serialized native observation bytes, not exact heap. B-link's service-wide counters have a different representation. The earlier checkpoint-detail summary removed repeated event arrays, but retained property/counter DOM objects remain. Raw result and driver costs must stay visible.

## Implementation scope

Change only the opt-in worker pressure-observation representation and report serialization. Serialize each completed observation once and retain validated raw JSON frames instead of their DOM. Keep the 4,096-frame and 8 MiB serialized-byte bounds and explicit truncation. Stream raw frames into the final health report without rebuilding their entire DOM or holding another complete serialized report buffer in the worker. Preserve the report's JSON schema, timestamps, phases, property availability states, shared-resource counting, full final checkpoint events, and metadata.

Use the pinned serde_json implementation and its validated raw-value support if suitable. Confirm the local API before editing. Record frame/vector/allocator overhead honestly; a serialized-byte bound is still not an exact heap ceiling. Temporary sample construction and native property collection remain costs.

The driver currently parses worker health JSON outside the backend cgroup. Record its RSS separately and do not claim that moving parsing/retention out of the worker removes machine-wide cost. Keep all engine code, caches/WBM, checkpoint settings, source semantics, locks, durability, workload, admission, budget, deadlines, and host policy unchanged. Existing routes without pressure observations must retain their behavior. Do not add comments, lint suppressions, or Co-Authored-By trailers.

## Validation and bounded comparison

Preserve the implementation plan before editing and use a new clean runtime commit, unique run/database/cgroup namespaces, source/binary/native hashes, and unchanged resource guards. Test exact health-report JSON equivalence for empty and multiple frames, valid raw-frame handling, and unchanged limits/truncation. Run the existing library/binary/read/transaction checks, changed-target advisory Clippy comparison, formatting, and default/A1 compatibility links. Do not rerun earlier successful cells for reporting repairs.

Freeze an executable follow-up matrix after inspecting final current-source evidence and live capacity. Its first diagnostic should repeat both cadence settings at one tenant for each backend, with the same sixty-second recipe. Then compare both backends at the confirmed E2/E3 counts under the original main cadence. Include the previously failing six/eight-tenant B-link E3 conditions only as a new representation variant, not a retry of an unchanged failure. Preserve and stop for review on unexplained failures; do not relax the fifteen-second shutdown gate to obtain a pass. Use three fresh repetitions only at scientifically selected boundary-adjacent populations and rotate backend order.

If actual-boundary 180-second maintenance is repeated, preserve its 2,000 ms cadence as a separate cohort; reducing DOM retention does not remove the serialized 8 MiB bound. Do not silently increase retention limits or pool distinct cadence/duration cohorts.

Report the observed worker/driver memory and CPU/latency changes, operational failures, and instrumentation perturbation before making a stronger engine-density or adoption claim. This follow-up does not implement B-link eviction, global application byte admission, ARC enforcement, or new crash recovery. Issue #126 remains open.
