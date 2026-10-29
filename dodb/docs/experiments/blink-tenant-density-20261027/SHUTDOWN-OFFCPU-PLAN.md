# Observe kernel waits during the bounded shutdown stall

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

The `b22638bdc` diagnostic stopped at its first five-tenant B-link cell during setup. The trace completed final property capture, entered `async_server_shutdown`, and emitted no end/error marker before the parent watchdog killed it. No offered load or oracle ran. This interval contains both `server.shutdown().await` and joining the server task, so its marker does not distinguish connection drain, service/coordinator shutdown, scheduling, or kernel waits. The prior source-023 measurement failure has no corresponding trace and must not be assigned the same internal cause retroactively.

During the traced stall, setup-cgroup memory remained above its unchanged 192 MiB high limit, high events continued, and sampled CPU advanced very little. These are pressure and low-CPU observations, not proof of a particular kernel wait. Inspect thread waits before proposing an optimization or changing the policy.

## Approved next implementation and protocol

The primary owns design and interpretation. GPT-6 Luna at high reasoning effort implements only new external observer/controller/reviewer helpers under `evidence/shutdown-offcpu/`, returning concise summaries. Do not change Rust, engine/service code, the frozen b226 binaries, existing controllers/reducers, source checkout, deadlines, memory policy, ARC/swap policy, or original evidence.

First check read access for `/proc/<owned-pid>/task/<tid>/{stat,status,wchan,syscall,stack,schedstat}` using a disposable owned sleeping process. Record unsupported/denied reads explicitly. Do not change sysctls, install tools, attach a debugger, send signals to experiment workers, or read environments. If useful wait fields are unavailable, preserve the probe and stop for review rather than fabricating an attribution.

The observer runs outside backend cgroups. It watches the bounded structured stderr markers, verifies source/PID/start-time ownership, and samples only workers whose shutdown marker has remained outstanding for at least two seconds. Poll markers no faster than 100 ms; capture thread data at most once per second, at most twenty-four captures per worker. Stream bounded JSONL instead of accumulating a DOM. Bound thread count, file bytes, total output, and errors explicitly; report truncation and read duration. Preserve selected process-memory, CPU, scheduler, context-switch, kernel-wait, and cgroup counters. Reading task state adds perturbation; this is diagnostic evidence, not a throughput ranking.

Validate parser/limits/identity/error handling locally and with the owned sleeping-process probe. Reuse the accepted seventy-nine checks and archived default-feature b226 binaries only after hash verification; do not rebuild or repeat passed checks. Freeze controller/observer hashes, exact invocation, current host capacity, and native/binary identities before runtime.

Run up to three fresh B-link-only E3 five-tenant sixty-second cells at the original 1,000 ms pressure cadence, fifty arrivals/s per tenant, 512-byte values, 32,768 initial rows each, original key/transaction recipe, 256/192 MiB and zero backend swap, checkpoint settings, deadlines, and both oracles on completions. This is a new external-observation variant for wait attribution, not a report repair or an engine comparison. There is no invented paired RocksDB result. Stop at the first operational failure or invalid observation; do not repeat an unchanged failed attempt. If all three complete, say the stall was not reproduced in this bounded variant.

Keep the twelve-GiB pre-cell, ten-GiB runtime free-space, and 1.5 GiB MemAvailable guards. Keep the parent twenty-second and worker fifteen-second shutdown deadlines. Preserve raw statuses/review errors, traces, thread samples, databases, binaries, groups, and all unexecuted cells explicitly. New helpers must not turn partial workload counters into a validated cell or call unavailable throughput zero.

Report observed kernel symbols/wait channels and their timestamps relative to shutdown markers. Distinguish an observed wait from its suspected allocation/I/O origin, and acknowledge sampling gaps. A wait snapshot does not provide a complete causal profile. Do not promote defaults or close issue #126 from this diagnostic alone.
