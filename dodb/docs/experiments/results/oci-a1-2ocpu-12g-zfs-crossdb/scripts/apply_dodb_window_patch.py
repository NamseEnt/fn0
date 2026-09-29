import sys

path = sys.argv[1]
source = open(path, encoding="utf-8").read()


def replace_once(old, new):
    global source
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"expected one match, found {count}: {old[:80]!r}")
    source = source.replace(old, new)


replace_once(
    "    read_latency: LatencySamples,\n}\n\nimpl WorkerStats {",
    "    read_latency: LatencySamples,\n    window_timeline: Vec<(u64, u64)>,\n}\n\nimpl WorkerStats {",
)
replace_once(
    "            read_latency: LatencySamples::with_seed(seed ^ 0x2222),\n        }",
    "            read_latency: LatencySamples::with_seed(seed ^ 0x2222),\n            window_timeline: Vec::new(),\n        }",
)
replace_once(
    "        for value in other.read_latency.values {\n            self.read_latency.push(value);\n        }\n",
    "        for value in other.read_latency.values {\n            self.read_latency.push(value);\n        }\n        self.window_timeline.extend(other.window_timeline);\n",
)
replace_once(
    "async fn writer_loop(\n    adapter: Arc<dyn EngineAdapter>,\n    workload: WorkloadConfig,\n    seed: u64,\n    worker_id: usize,\n    deadline: Instant,\n    quota: Option<MixQuota>,\n    warmup: bool,\n)",
    "async fn writer_loop(\n    adapter: Arc<dyn EngineAdapter>,\n    workload: WorkloadConfig,\n    seed: u64,\n    worker_id: usize,\n    deadline: Instant,\n    quota: Option<MixQuota>,\n    warmup: bool,\n    interval_start: Instant,\n)",
)
replace_once(
    "                Ok(_) => {\n                    stats.successful_transactions += 1;\n                    stats.mutation_ops += width;\n",
    "                Ok(_) => {\n                    stats.successful_transactions += 1;\n                    stats.mutation_ops += width;\n                    stats.window_timeline.push((\n                        (started + elapsed - interval_start).as_nanos() as u64,\n                        elapsed.as_nanos() as u64,\n                    ));\n",
)
replace_once(
    "    let deadline = Instant::now() + duration;\n    let workload = WorkloadConfig {",
    "    let interval_start = Instant::now();\n    let deadline = interval_start + duration;\n    let workload = WorkloadConfig {",
)
replace_once(
    "            quota.clone(),\n            warmup,\n        )));\n    }\n    for worker_id in 0..scenario.readers {",
    "            quota.clone(),\n            warmup,\n            interval_start,\n        )));\n    }\n    for worker_id in 0..scenario.readers {",
)
window_block = """    if wall >= Duration::from_secs(20) {
        let mut timeline = measured.window_timeline.clone();
        timeline.sort_unstable();
        let window_nanos = 10_000_000_000u64;
        let wall_nanos = wall.as_nanos() as u64;
        let window_count = wall_nanos.div_ceil(window_nanos);
        json.u64("window_seconds", 10);
        json.u64("window_count", window_count);
        for window_index in 0..window_count {
            let start = window_index * window_nanos;
            let end = start + window_nanos;
            let mut latencies: Vec<u64> = timeline
                .iter()
                .filter(|(finish, _)| *finish >= start && *finish < end)
                .map(|(_, latency)| *latency)
                .collect();
            latencies.sort_unstable();
            let span = (end.min(wall_nanos) - start) as f64 / 1e9;
            let percentile = |fraction: f64| {
                if latencies.is_empty() {
                    0.0
                } else {
                    latencies[((latencies.len() - 1) as f64 * fraction).round() as usize] as f64
                        / 1_000.0
                }
            };
            json.u64(
                &format!("window_{window_index:02}_successful_transactions"),
                latencies.len() as u64,
            );
            json.f64(&format!("window_{window_index:02}_span_seconds"), span);
            json.f64(
                &format!("window_{window_index:02}_logical_tx_per_second"),
                latencies.len() as f64 / span,
            );
            json.f64(&format!("window_{window_index:02}_p50_us"), percentile(0.50));
            json.f64(&format!("window_{window_index:02}_p95_us"), percentile(0.95));
            json.f64(&format!("window_{window_index:02}_p99_us"), percentile(0.99));
        }
    }
    json.finish()
}
"""
build_record_start = source.index("fn build_record(")
finish_index = source.index("    json.finish()\n}\n", build_record_start)
source = source[:finish_index] + window_block + source[finish_index + len("    json.finish()\n}\n"):]
open(path, "w", encoding="utf-8").write(source)
print("patched", path)
