import hashlib
import json
import math
import shutil
import statistics
from pathlib import Path

root = Path(__file__).resolve().parent
source_sha = "4bc4d42e4d816f4a11428f5423d89a4c35494f9b"
variant_order = ("main-btree", "blink-metrics-on", "blink-borrowed-pages")
variant_labels = {
    "main-btree": "A main-btree",
    "blink-metrics-on": "B planned-blink 기본",
    "blink-borrowed-pages": "C planned-blink borrowed",
}
seed_values = (0xD0DB2026, 0xD0DB2027, 0xD0DB2028)

for output_directory in (root / "runs", root / "logs"):
    output_directory.mkdir(exist_ok=True)
for source_path in sorted((root / "raw").glob("*.jsonl")):
    shutil.copy2(source_path, root / "runs" / source_path.name)
for source_path in sorted((root / "raw").glob("*.log")):
    shutil.copy2(source_path, root / "logs" / source_path.name)

(root / "logs" / "preflight-00.txt").write_text(
    "attempt=preflight-00\nstate=invalid-before-benchmark\nreason=remote result path typo caused the result-directory precondition to fail\nbenchmark_started=false\n",
    encoding="utf-8",
)

configuration = json.loads((root / "matrix-config.json").read_text())
run_events = [json.loads(line) for line in (root / "run-order.jsonl").read_text().splitlines()]
starts = [event for event in run_events if event.get("event") == "start"]
completes = [event for event in run_events if event.get("event") == "complete"]
failed_events = [event for event in run_events if event.get("event") == "run-failed"]
expected_schedule = {
    1: variant_order,
    2: ("blink-borrowed-pages", "main-btree", "blink-metrics-on"),
    3: ("blink-metrics-on", "blink-borrowed-pages", "main-btree"),
}
actual_schedule = {}
for read_kind in ("get", "query"):
    for repetition in range(1, 4):
        actual_schedule[(read_kind, repetition)] = tuple(
            event["variant"] for event in starts
            if event["read_kind"] == read_kind and event["repetition"] == repetition
        )
        if actual_schedule[(read_kind, repetition)] != expected_schedule[repetition]:
            raise ValueError(f"run order mismatch for {read_kind} repetition {repetition}")
if len(starts) != 18 or len(completes) != 18 or failed_events:
    raise ValueError("run event counts do not show 18 completed executions")

binary_provenance = json.loads((root / "binary-provenance.json").read_text())
configuration_hashes = {
    "main-btree": binary_provenance["main-btree"]["sha256"],
    "blink-metrics-on": binary_provenance["blink-metrics-on"]["sha256"],
    "blink-borrowed-pages": binary_provenance["blink-borrowed-pages"]["sha256"],
}
records = {}
validation_checks = []
for read_kind in ("get", "query"):
    for repetition, seed in enumerate(seed_values, 1):
        for variant in variant_order:
            raw_path = root / "runs" / f"readers-1-{read_kind}-{variant}-rep{repetition}.jsonl"
            raw_lines = raw_path.read_text().splitlines()
            if len(raw_lines) != 1:
                raise ValueError(f"expected one raw record in {raw_path.name}")
            record = json.loads(raw_lines[0])
            event = next(
                event for event in completes
                if event["cell_id"] == f"readers-1-{read_kind}-{variant}-rep{repetition}"
            )
            if record.get("git_commit") != source_sha or event.get("checkout_sha") != source_sha:
                raise ValueError(f"source SHA mismatch in {raw_path.name}")
            if record.get("binary_sha256") != configuration_hashes[variant] or event.get("binary_sha256") != configuration_hashes[variant]:
                raise ValueError(f"binary hash mismatch in {raw_path.name}")
            expected_borrowed = variant == "blink-borrowed-pages"
            if record.get("blink_borrowed_page_views_enabled") is not expected_borrowed:
                raise ValueError(f"feature state mismatch in {raw_path.name}")
            if record.get("seed") != seed or record.get("working_set") != 4096 or record.get("cache_capacity") != 256:
                raise ValueError(f"workload seed or size mismatch in {raw_path.name}")
            if record.get("distribution") != "uniform" or record.get("read_limit") != 16:
                raise ValueError(f"Query/read configuration mismatch in {raw_path.name}")
            if record.get("readers") != 1 or record.get("tokio_workers") != 2 or record.get("sync_mode") != "real":
                raise ValueError(f"execution topology or durability mismatch in {raw_path.name}")
            if record.get("blink_read_observational_metrics_enabled") is not True:
                raise ValueError(f"read observation metrics inactive in {raw_path.name}")
            if record.get("errors") != 0 or record.get("overloads") != 0 or record.get("conflicts") != 0:
                raise ValueError(f"error gate failed in {raw_path.name}")
            if record.get("successful_reads", 0) <= 0:
                raise ValueError(f"no successful reads in {raw_path.name}")
            elapsed_ns = record.get("process_rss_window_end_offset_ns", 0)
            if elapsed_ns <= 0:
                raise ValueError(f"actual measurement elapsed time missing in {raw_path.name}")
            if variant != "main-btree":
                if record.get("active_generation_pins") != 0:
                    raise ValueError(f"active pins remain in {raw_path.name}")
                if record.get("generation_pins") != record.get("successful_reads"):
                    raise ValueError(f"pin count mismatch in {raw_path.name}")
                if record.get("read_operations_metric") != record.get("successful_reads"):
                    raise ValueError(f"read observation count mismatch in {raw_path.name}")
            if read_kind == "get":
                if record.get("sampled_read_verification_status") != "passed" or record.get("sampled_reads_indeterminate") != 0:
                    raise ValueError(f"Get value/revision validation failed in {raw_path.name}")
            else:
                if record.get("query_checked_requests") != record.get("successful_reads"):
                    raise ValueError(f"Query checked request count mismatch in {raw_path.name}")
                if record.get("returned_rows") != 16 * record.get("successful_reads"):
                    raise ValueError(f"Query returned row count mismatch in {raw_path.name}")
                if record.get("query_validation_failures") != 0:
                    raise ValueError(f"Query full-response validation failed in {raw_path.name}")
                if record.get("query_client_aggregation_passed") is not True or record.get("client_aggregation_passed") is not True:
                    raise ValueError(f"Query client aggregation failed in {raw_path.name}")
            rss = record.get("rss_kib")
            if record.get("process_rss_status") != "complete" or record.get("process_rss_sample_count", 0) <= 0:
                raise ValueError(f"measurement RSS incomplete in {raw_path.name}")
            if record.get("process_rss_collection_failures") != 0 or record.get("process_rss_max_sample_interval_ns", 0) <= 0:
                raise ValueError(f"measurement RSS failure metadata invalid in {raw_path.name}")
            if not isinstance(rss, dict) or rss.get("sample_count", 0) <= 0:
                raise ValueError(f"process RSS sampling missing in {raw_path.name}")
            records[(read_kind, repetition, variant)] = record
            validation_checks.append({
                "read_kind": read_kind,
                "repetition": repetition,
                "variant": variant,
                "seed": seed,
                "source_sha": record["git_commit"],
                "binary_sha256": record["binary_sha256"],
                "feature_enabled": record["blink_borrowed_page_views_enabled"],
                "successful_reads": record["successful_reads"],
                "elapsed_ns": elapsed_ns,
                "errors": record["errors"],
                "overloads": record["overloads"],
                "conflicts": record["conflicts"],
                "active_generation_pins": record["active_generation_pins"],
                "read_metrics_enabled": record["blink_read_observational_metrics_enabled"],
                "measurement_rss_status": record["process_rss_status"],
                "measurement_rss_samples": record["process_rss_sample_count"],
                "measurement_rss_collection_failures": record["process_rss_collection_failures"],
                "measurement_rss_max_gap_ns": record["process_rss_max_sample_interval_ns"],
                "process_rss_samples": rss["sample_count"],
                "process_rss_collection_failures": rss["collection_failures"],
                "process_rss_max_gap_ns": rss["maximum_sample_gap_ns"],
            })

fingerprint_validation = json.loads((root / "query-fingerprint-validation.json").read_text())
if fingerprint_validation.get("passed") is not True or len(fingerprint_validation.get("checks", [])) != 15:
    raise ValueError("Query input fingerprint validation did not pass all expected checkpoints")

with (root / "execution-manifest.tsv").open("w", encoding="utf-8") as output_file:
    output_file.write("execution_index\tread_kind\trepetition\tseed\tvariant\tengine\tbinary_sha256\tfeature_enabled\tcommand\toutput\n")
    for execution_index, event in enumerate(starts, 1):
        cell_id = event["cell_id"]
        read_kind = event["read_kind"]
        repetition = event["repetition"]
        variant = event["variant"]
        record = records[(read_kind, repetition, variant)]
        command = json.dumps(event["command"], ensure_ascii=False, separators=(",", ":"))
        output_file.write("\t".join(map(str, (
            execution_index, read_kind, repetition, record["seed"], variant,
            record["engine"], record["binary_sha256"],
            str(record["blink_borrowed_page_views_enabled"]).lower(), command,
            event["output"],
        ))) + "\n")

with (root / "per-run-comparison.tsv").open("w", encoding="utf-8") as output_file:
    output_file.write("workload\trepetition\tseed\tvariant\tthroughput_ops_s\tp50_us\tp95_us\tp99_us\tcpu_one_core_pct\tcpu_machine_pct\tmeasurement_rss_peak_bytes\tmeasurement_rss_end_bytes\tmeasurement_rss_samples\tmeasurement_rss_failures\tmeasurement_rss_max_gap_ms\tprocess_hwm_kib\tprocess_samples\tprocess_failures\tprocess_max_gap_ms\terrors\toverloads\tconflicts\tactive_pins\n")
    for read_kind in ("get", "query"):
        for repetition, seed in enumerate(seed_values, 1):
            for variant in variant_order:
                record = records[(read_kind, repetition, variant)]
                elapsed_seconds = record["process_rss_window_end_offset_ns"] / 1e9
                throughput = record["successful_reads"] / elapsed_seconds
                rss = record["rss_kib"]
                values = (
                    read_kind, repetition, seed, variant, f"{throughput:.3f}",
                    record["read_p50_us"], record["read_p95_us"], record["read_p99_us"],
                    record["cpu_utilization_percent_one_core"], record["cpu_utilization_percent_machine"],
                    record["process_rss_peak_observed_bytes"], record["process_rss_end_bytes"],
                    record["process_rss_sample_count"], record["process_rss_collection_failures"],
                    f"{record['process_rss_max_sample_interval_ns'] / 1e6:.3f}",
                    rss["hwm"], rss["sample_count"], rss["collection_failures"],
                    f"{rss['maximum_sample_gap_ns'] / 1e6:.3f}", record["errors"],
                    record["overloads"], record["conflicts"], record["active_generation_pins"],
                )
                output_file.write("\t".join(map(str, values)) + "\n")

ratio_rows = []
summary_rows = []
for read_kind in ("get", "query"):
    throughput_by_variant = {}
    for variant in variant_order:
        throughput_by_variant[variant] = []
        for repetition, seed in enumerate(seed_values, 1):
            record = records[(read_kind, repetition, variant)]
            throughput_by_variant[variant].append(record["successful_reads"] / (record["process_rss_window_end_offset_ns"] / 1e9))
    for repetition, seed in enumerate(seed_values, 1):
        rate_a = throughput_by_variant["main-btree"][repetition - 1]
        rate_b = throughput_by_variant["blink-metrics-on"][repetition - 1]
        rate_c = throughput_by_variant["blink-borrowed-pages"][repetition - 1]
        ratio_data = {
            "C/B": rate_c / rate_b,
            "B/A": rate_b / rate_a,
            "C/A": rate_c / rate_a,
        }
        ratio_rows.append({"workload": read_kind, "repetition": repetition, "seed": seed, **ratio_data})
    for ratio_name in ("C/B", "B/A", "C/A"):
        ratio_values = [row[ratio_name] for row in ratio_rows if row["workload"] == read_kind]
        summary_rows.append({"workload": read_kind, "metric": ratio_name, "median": statistics.median(ratio_values), "min": min(ratio_values), "max": max(ratio_values)})
    for variant in variant_order:
        runs = [records[(read_kind, repetition, variant)] for repetition in range(1, 4)]
        rates = throughput_by_variant[variant]
        rate_median = statistics.median(rates)
        rate_spread_percent = (max(rates) - min(rates)) / rate_median * 100
        summary_rows.append({
            "workload": read_kind,
            "metric": variant,
            "throughput_median": rate_median,
            "throughput_min": min(rates),
            "throughput_max": max(rates),
            "throughput_spread_percent": rate_spread_percent,
            "p50_median": statistics.median(record["read_p50_us"] for record in runs),
            "p50_min": min(record["read_p50_us"] for record in runs),
            "p50_max": max(record["read_p50_us"] for record in runs),
            "p95_median": statistics.median(record["read_p95_us"] for record in runs),
            "p95_min": min(record["read_p95_us"] for record in runs),
            "p95_max": max(record["read_p95_us"] for record in runs),
            "p99_median": statistics.median(record["read_p99_us"] for record in runs),
            "p99_min": min(record["read_p99_us"] for record in runs),
            "p99_max": max(record["read_p99_us"] for record in runs),
            "cpu_one_core_median": statistics.median(record["cpu_utilization_percent_one_core"] for record in runs),
            "measurement_rss_peak_median_mib": statistics.median(record["process_rss_peak_observed_bytes"] for record in runs) / 1048576,
            "measurement_rss_peak_min_mib": min(record["process_rss_peak_observed_bytes"] for record in runs) / 1048576,
            "measurement_rss_peak_max_mib": max(record["process_rss_peak_observed_bytes"] for record in runs) / 1048576,
        })

with (root / "paired-ratios.tsv").open("w", encoding="utf-8") as output_file:
    output_file.write("workload\trepetition\tseed\tC_over_B\tB_over_A\tC_over_A\n")
    for row in ratio_rows:
        output_file.write(f"{row['workload']}\t{row['repetition']}\t0x{row['seed']:x}\t{row['C/B']:.6f}\t{row['B/A']:.6f}\t{row['C/A']:.6f}\n")


def find_summary(read_kind, name):
    return next(row for row in summary_rows if row["workload"] == read_kind and row["metric"] == name)

report_lines = [
    "# borrowed-page 옵션의 Query16 및 Get 비교",
    "",
    f"- 공통 소스 SHA: `{source_sha}`",
    "- 구성 A: `main-btree`, 기본 바이너리",
    "- 구성 B: `planned-blink`, 기본 feature 바이너리",
    "- 구성 C: `planned-blink`, `blink-borrowed-page-views` feature 바이너리",
    "- 18회 측정: 구성 3 × Get/Query16 × 반복 3, 전부 5초 측정과 1초 warmup, real sync",
    "- 입력: ARM64 OCI A1, reader 1, Tokio worker 2, working set 4096, cache 256, key/value 16/64 bytes, uniform, Query limit 16",
    "- 처리량은 `successful_reads / process_rss_window_end_offset_ns`로 다시 계산했다. 정수 millisecond 시간은 분모로 사용하지 않았다.",
    "- p50/p95/p99 요약은 실행별 percentile의 반복 중앙값과 범위다. 요청 표본을 합쳐 percentile을 다시 계산하지 않았다.",
    "",
    "## 짝 비교 비율",
    "",
    "| workload | 반복 | seed | C/B borrowed 효과 | B/A main 대비 | C/A main 대비 |",
    "|---|---:|---:|---:|---:|---:|",
]
for row in ratio_rows:
    report_lines.append(f"| {row['workload']} | {row['repetition']} | `0x{row['seed']:x}` | {row['C/B']:.4f}x | {row['B/A']:.4f}x | {row['C/A']:.4f}x |")
report_lines.extend(["", "## workload별 요약", "", "| workload | 구성 | 처리량 중앙값 ops/s (범위) | 반복 범위/중앙값 | C/B | B/A | C/A | p50 us 중앙값 (범위) | p95 us 중앙값 (범위) | p99 us 중앙값 (범위) | CPU 한 코어 중앙값 | 관측 RSS peak MiB 중앙값 (범위) |", "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"])
for read_kind in ("get", "query"):
    ratios = {row["metric"]: row for row in summary_rows if row["workload"] == read_kind and row["metric"] in ("C/B", "B/A", "C/A")}
    for variant in variant_order:
        row = find_summary(read_kind, variant)
        option_ratio = f"{ratios['C/B']['median']:.4f}x" if variant == "blink-borrowed-pages" else "—"
        base_ratio = f"{ratios['B/A']['median']:.4f}x" if variant == "blink-metrics-on" else "—"
        borrowed_ratio = f"{ratios['C/A']['median']:.4f}x" if variant == "blink-borrowed-pages" else "—"
        report_lines.append(
            f"| {read_kind} | {variant_labels[variant]} | {row['throughput_median']:.0f} ({row['throughput_min']:.0f}–{row['throughput_max']:.0f}) | {row['throughput_spread_percent']:.2f}% | {option_ratio} | {base_ratio} | {borrowed_ratio} | {row['p50_median']:.3f} ({row['p50_min']:.3f}–{row['p50_max']:.3f}) | {row['p95_median']:.3f} ({row['p95_min']:.3f}–{row['p95_max']:.3f}) | {row['p99_median']:.3f} ({row['p99_min']:.3f}–{row['p99_max']:.3f}) | {row['cpu_one_core_median']:.2f}% | {row['measurement_rss_peak_median_mib']:.2f} ({row['measurement_rss_peak_min_mib']:.2f}–{row['measurement_rss_peak_max_mib']:.2f}) |"
        )
report_lines.extend(["", "## 유효성 및 자원 측정", "", f"- 18/18 run이 source SHA, binary SHA, feature 기록, 정확성 검사, 양수 성공 수, 오류/overload/conflict 0을 통과했다. 유효 run 실패는 0건이다.", f"- B와 C의 종료 시 active generation pin은 모두 0이다. 두 Blink 구성에서 generation pin 수와 읽기 관측 횟수는 성공 read 수와 일치한다.", f"- Get은 9/9 실행의 seed value/revision 검사가 통과했고 indeterminate가 0이다. Query는 9/9 실행에서 성공 요청 모두 응답 검사를 거쳤고 각 요청당 16행, validation failure 0, client aggregation 통과다.", f"- Query 입력 fingerprint는 같은 반복의 3개 구성에서 동일 길이 체크포인트 15개(반복당 5개)가 일치했다.", "- 측정 구간 내부 RSS 계측은 18/18 complete, 수집 실패 0이다. 표본 수는 496–497, 최대 표본 간격은 10.25–15.18 ms다. 측정 중 최대 RSS는 `process_rss_peak_observed_bytes`를 사용했다.", "- 보조 프로세스 RSS 표본은 실행당 320–325개, 최대 간격 20.71–31.54 ms다. 실행 종료 시점 `/proc` 소멸 경합으로 보조 sampler는 실행당 1회 실패를 기록했다. 이는 측정 프로세스 내부 RSS 계측의 0회 실패와 구분해 보존했다.", "- CPU 사용률은 한 코어 환산 약 99%였다. 결과는 OCI 2 OCPU 환경에서 단일 reader가 CPU를 대부분 사용한 짧은 측정이다.", "- 두 무효 사전 시도는 결과 경로 오기 및 결과/DB namespace 중복으로 벤치마크 시작 전에 종료됐다. 원인은 `logs/preflight-00.txt`, `logs/preflight-01.txt`에 남겼다. 무효 성능 실행은 없다.", "", "## 해석", "", "C/B는 Get에서 반복 중앙값 기준 약 +1.6%, Query16에서 약 +0.6%다. 옵션 효과는 작고, 세 반복 방향은 일관됐다. B/A Query16은 약 0.693x였고 C/A도 약 0.697x라서 borrowed-page 옵션은 재현된 Query16 격차를 사실상 해소하지 못했다. Get은 C/A 약 1.001x로 main과 거의 같았다.", "", "옵션 자체는 소폭의 순수 읽기 개선을 보였지만 Query16 p50/p95/p99는 B와 C가 거의 같고 main보다 높았다. Blink 두 구성은 main보다 측정 RSS가 약 6–7 MiB 높았다. 따라서 이번 실험은 기존 옵션의 제한적인 효과만 보여준다. 단일 OCI host, 3회, 5초 측정이므로 엔진 전체 채택 판단이나 장시간 안정성 판단으로 확장하지 않는다. 120초 지속 측정은 범위 밖이다.", "", "## 빌드와 안전성 테스트", "", "기본 바이너리는 기준 결과의 exact source SHA, phase0-bench source hash, Cargo.lock, Cargo 설정, ARM64 release 출처와 해시가 일치해 재사용했다. feature-on 빌드는 같은 rustc/cargo/toolchain과 `.cargo/config.toml`의 `-C target-feature=+crc` 설정에서 별도 target 디렉터리로 만들었다. 변경한 Cargo feature만 `blink-borrowed-page-views`다.", "", "feature-on OCI storage 테스트는 `dodb-storage` 전체 release 테스트에서 181 passed, 0 failed, 4 ignored, `phase0-bench`에서 42 passed, 0 failed였다. 검사나 안전장치를 비활성화하지 않았다.", "", "## 코드 경로 확인", "", "feature는 `GenerationPin`의 `ReadPageSource::Page`와 `page()`에서 borrowed page view 사용을 선택한다. `GenerationPin` 수명·pin 집계와 Drop 해제는 공통이며, `can_reuse_pages()`는 active pin이 0일 때만 재사용을 허용한다. phase0-bench의 `blink_borrowed_page_views_enabled`, read metric, active pin 필드는 두 바이너리에서 확인했다.", ""])
(root / "REPORT.ko.md").write_text("\n".join(report_lines), encoding="utf-8")

validation = {
    "accepted": True,
    "source_sha": source_sha,
    "planned_runs": 18,
    "completed_runs": len(validation_checks),
    "failed_run_events": len(failed_events),
    "valid_run_failures": 0,
    "invalid_preflight_attempts": [
        {"id": "preflight-00", "benchmark_started": False, "reason": "result path precondition failed after a path typo"},
        {"id": "preflight-01", "benchmark_started": False, "reason": "result directory collided with default data namespace"},
    ],
    "query_common_fingerprint_checkpoints": len(fingerprint_validation["checks"]),
    "all_checks": validation_checks,
    "paired_ratio_summary": summary_rows,
}
(root / "validation.json").write_text(json.dumps(validation, indent=2, sort_keys=True) + "\n", encoding="utf-8")

with (root / "attempts.tsv").open("w", encoding="utf-8") as output_file:
    output_file.write("attempt_id\tstate\tbenchmark_started\treason\n")
    output_file.write("preflight-00\tinvalid\tfalse\tresult path typo failed a precondition before benchmark launch\n")
    output_file.write("preflight-01\tinvalid\tfalse\tresults and default data namespace overlapped before benchmark launch\n")
    output_file.write("matrix-18\taccepted\ttrue\t18 of 18 executions completed and passed validation\n")

checksums = []
for artifact in sorted(root.rglob("*")):
    if artifact.is_file() and artifact.name != "SHA256SUMS":
        digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
        checksums.append(f"{digest}  {artifact.relative_to(root).as_posix()}")
(root / "SHA256SUMS").write_text("\n".join(checksums) + "\n", encoding="utf-8")
print(f"validated={len(validation_checks)}/18 query_fingerprint_checkpoints={len(fingerprint_validation['checks'])} checksums={len(checksums)}")
