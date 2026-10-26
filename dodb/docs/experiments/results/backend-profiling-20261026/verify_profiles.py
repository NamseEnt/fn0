import argparse
import hashlib
import json
import tarfile
from pathlib import Path


def validation_passed(value):
    if isinstance(value, dict):
        if "passed" in value:
            return value["passed"] is True
        return bool(value) and all(validation_passed(nested) for nested in value.values())
    return value is True


def validate_latency_counts(value):
    if isinstance(value, dict):
        if "count" in value and "sample_count" in value:
            assert value["count"] == value["sample_count"]
        for nested in value.values():
            validate_latency_counts(nested)
    elif isinstance(value, list):
        for nested in value:
            validate_latency_counts(nested)


parser = argparse.ArgumentParser()
parser.add_argument("archive", type=Path)
options = parser.parse_args()
sidecar = json.loads(options.archive.with_suffix(options.archive.suffix + ".json").read_text())
checksum = hashlib.sha256()
with options.archive.open("rb") as source_file:
    for chunk in iter(lambda: source_file.read(1024 * 1024), b""):
        checksum.update(chunk)
assert checksum.hexdigest() == sidecar["sha256"]
with tarfile.open(options.archive, "r:gz") as archive:
    inventory = json.load(archive.extractfile("archive-inventory.json"))
    assert len(inventory["files"]) == sidecar["inventoried_files"]
    expected_names = {"archive-inventory.json", *(entry["path"] for entry in inventory["files"])}
    assert set(archive.getnames()) == expected_names
    for entry in inventory["files"]:
        content = archive.extractfile(entry["path"])
        checksum = hashlib.sha256()
        byte_count = 0
        for chunk in iter(lambda: content.read(1024 * 1024), b""):
            checksum.update(chunk)
            byte_count += len(chunk)
        assert checksum.hexdigest() == entry["sha256"], entry["path"]
        assert byte_count == entry["bytes"], entry["path"]
    summary = json.load(archive.extractfile("main/profile-summary.json"))
    assert len(summary["cases"]) == 8
    preflight_reports = [entry["path"] for entry in inventory["files"]
                         if entry["path"].startswith("preflight/") and entry["path"].endswith("/workload-report.json")]
    assert len(preflight_reports) == 4
    for report_name in preflight_reports:
        report = json.load(archive.extractfile(report_name))
        assert report["run_status"] == "completed"
        assert report["git_commit"] == "cc80d1fdc7a98792be1a739056a3013debf39268"
        assert validation_passed(report.get("final_validation", report.get("validation")))
        assert validation_passed(report["reopen_validation"])
        validate_latency_counts(report)
    prefixes = {}
    profiles = 0
    for case, value in summary["cases"].items():
        assert value["run"]["source_sha"] == "cc80d1fdc7a98792be1a739056a3013debf39268"
        assert value["timing"]["measurement_complete"]
        assert value["timing"]["accepted_as_full_window"]
        report = json.load(archive.extractfile(f"main/{case}/workload-report.json"))
        assert report["run_status"] == "completed"
        assert report["git_commit"] == value["run"]["source_sha"]
        assert report["binary_sha256"] == value["run"]["binary_sha256"]
        validate_latency_counts(report)
        assert validation_passed(report.get("final_validation", report.get("validation")))
        assert validation_passed(report["reopen_validation"])
        if "blink_checkpoint_metrics" in report and report["blink_checkpoint_metrics"].get("checkpoint_events"):
            metrics = report["blink_checkpoint_metrics"]
            assert not metrics["checkpoints_failed"]
            assert not metrics["tasks_running"]
            assert not metrics["retained_image_bytes"]
            assert not metrics["omitted_checkpoint_events"]
            window = report["measurement_window_unix_nanos"]
            events = [event for event in metrics["checkpoint_events"]
                      if event["succeeded"] and int(window["start"]) <= int(event["started_unix_nanos"])
                      and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
            assert len(events) >= 3
            assert any(event["writes_during_checkpoint"] > 0 for event in events)
        for counters in [report.get("counters", {})] + [tenant.get("counters", {})
                                                        for tenant in report.get("per_tenant", {}).values()]:
            assert not counters.get("overloads", 0)
            assert not counters.get("other_errors", 0)
        signature_file = f"main/{case.split('-')[0]}-input-prefix.json"
        if case.startswith("RW80"):
            signature = sorted([[prefix["client_index"], prefix["profile"], prefix["workflow_count"], prefix["fingerprint"]]
                                for tenant in report["per_tenant"].values()
                                for prefix in tenant["counters"]["client_input_prefixes"]])
        else:
            signature = sorted([[client["client_index"], client["measurement_input_prefix_batches"],
                                 client["measurement_input_prefix_fingerprint_fnv1a64"]]
                                for client in report["per_client"]])
        assert json.load(archive.extractfile(signature_file)) == signature
        prefixes.setdefault(case.split("-")[0], signature)
        assert prefixes[case.split("-")[0]] == signature
        if case.endswith("profile"):
            profiles += 1
            cpu = value["cpu"]["measurement"]
            assert cpu["samples"] > 100
            fold = archive.extractfile(f"main/{case}/cpu/measurement.folded").read().decode()
            assert sum(int(line.rsplit(" ", 1)[1]) for line in fold.splitlines()) == cpu["samples"]
            assert value["waits"]["included_start_bucket_seconds"] >= 58
            waits = json.load(archive.extractfile(f"main/{case}/waits/waits.json"))
            assert waits["map_insert_failures"] == [0, 0]
    assert profiles == 4
result = {"passed": True, "files_verified": len(inventory["files"]), "main_cells": 8,
          "preflight_cells": len(preflight_reports), "profiled_cells": profiles, "archive_sha256": sidecar["sha256"]}
print(json.dumps(result, indent=2))
