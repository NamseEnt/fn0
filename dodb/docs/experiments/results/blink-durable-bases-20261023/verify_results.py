import contextlib
import hashlib
import io
import json
import re
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

import summarize


DIRECTORY = Path(__file__).resolve().parent
EVIDENCE = DIRECTORY / "evidence"


def read_json(path):
    return json.loads(path.read_text())


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def original_values_match(original, recomputed):
    if isinstance(original, dict):
        return all(key in recomputed and original_values_match(value, recomputed[key]) for key, value in original.items())
    if isinstance(original, list):
        return len(original) == len(recomputed) and all(original_values_match(earlier, later) for earlier, later in zip(original, recomputed))
    return original == recomputed


def extract(archive_path, destination):
    subprocess.run([sys.executable, str(DIRECTORY / "verify_archive.py"), str(archive_path)], check=True)
    with tarfile.open(archive_path, "r:gz") as archive:
        for member in archive.getmembers():
            target = (destination / member.name).resolve()
            assert member.isfile() and target.is_relative_to(destination.resolve()), member.name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(archive.extractfile(member).read())


def verify_tests(root):
    summary = read_json(root / "validation/validation-complete.json")
    records = read_json(root / "validation/commands.json")
    assert len(records) == summary["binaries"] == 20
    totals = {"passed": 0, "failed": 0, "ignored": 0}
    for record in records:
        assert record["exit_code"] == 0 and record["source_sha"] == summary["source_sha"]
        log = (root / "validation" / record["log"]).read_text()
        matches = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log)
        assert len(matches) == 1, record["identity"]
        counts = dict(zip(totals, map(int, matches[0])))
        assert counts == record["counts"]
        for key in totals:
            totals[key] += counts[key]
    assert all(totals[key] == summary[key] for key in totals)
    return summary


def main():
    with tempfile.TemporaryDirectory(prefix="dodb-durable-base-evidence-") as temporary:
        root = Path(temporary)
        primary = root / "primary"
        rejected = root / "rejected"
        archives = {}
        for name, destination in [("blink-durable-bases-20261023-primary-budget-evidence.tar.gz", primary),
                                  ("blink-durable-bases-20261023-evidence.tar.gz", rejected)]:
            path = EVIDENCE / name
            extract(path, destination)
            archives[name] = read_json(path.with_suffix(path.suffix + ".json"))
        selection = read_json(primary / "selection.json")
        assert hashlib.sha256((primary / "short/matrix-summary.json").read_bytes()).hexdigest() == selection["short_summary_sha256"]
        tests = verify_tests(primary)
        references = {}
        command_references = {}
        coverage = []
        stages = {}
        identities = read_json(primary / "binary-identities.json")
        final_identity = read_json(EVIDENCE / "final-oci-identity.json")
        assert final_identity["source_clean"] and final_identity["source_sha"] == tests["source_sha"]
        assert all(final_identity["frozen_binaries_verified_after_measurements"][name]["sha256"] == identity["sha256"]
                   for name, identity in identities.items())
        write_json(EVIDENCE / "binary-identities.json", identities)
        write_json(EVIDENCE / "selection.json", selection)
        write_json(EVIDENCE / "validation-summary.json", tests)
        write_json(EVIDENCE / "preserved-oci-database-inventory.json", read_json(rejected / "preserved-database-inventory.json"))
        for stage in ["preflight", "short", "sustained"]:
            stage_root = primary / stage
            original = read_json(stage_root / "matrix-summary.json") if (stage_root / "matrix-summary.json").exists() else None
            if original:
                write_json(EVIDENCE / f"{stage}-summary.original.json", original)
            with contextlib.redirect_stdout(io.StringIO()):
                summarize.summarize(stage_root)
            recomputed = read_json(stage_root / "matrix-summary.json")
            assert original is None or original_values_match(original, recomputed), stage
            plan = read_json(stage_root / "matrix-plan.json")
            assert plan["controller_sha256"] == hashlib.sha256((DIRECTORY / "run_matrix.py").read_bytes()).hexdigest()
            assert plan["base_controller_sha256"] == hashlib.sha256((DIRECTORY.parent / "blink-background-checkpoint-20261021/run_matrix.py").read_bytes()).hexdigest()
            assert all(identity["source_sha"] == plan["source_sha"] and identity["sha256"] == plan["binary_hashes"][name]
                       for name, identity in identities.items())
            write_json(EVIDENCE / f"{stage}-summary.json", recomputed)
            stages[stage] = {"cells": len(recomputed["cells"]), "validated": recomputed["validated"],
                             "original_summary_present": original is not None,
                             "original_summary_values_match": original_values_match(original, recomputed) if original is not None else None}
            for cell in recomputed["cells"]:
                evidence = stage_root / "reports" / cell["case"]
                report = read_json(evidence / "workload-report.raw.json")
                command = read_json(evidence / "command.json")
                arguments = command["argv"][1:]
                assert len(arguments) % 2 == 0
                options = dict(zip(arguments[::2], arguments[1::2]))
                assert len(options) * 2 == len(arguments)
                layout, execution = plan["policies"][cell["policy"]]
                configuration = report["blink_checkpoint_configuration"]
                assert configuration["layout"] == options["--blink-wal-layout"] == layout
                assert configuration["execution"] == options["--blink-checkpoint-execution"] == execution
                assert configuration["segment_bytes"] == int(options["--blink-wal-segment-bytes"]) == 33554432
                assert int(options["--measurement-ms"]) == plan["duration_ms"]
                assert int(options["--warmup-ms"]) == plan["warmup_ms"]
                assert options["--blink-checkpoint-wal-bytes"] == "134217728"
                assert report["backend"] == ("blink" if cell["workload"] == "RW80" else "Blink")
                assert options["--backend"] == "blink"
                variable_options = {"--data-dir", "--report", "--warmup-ms", "--measurement-ms", "--blink-wal-layout", "--blink-checkpoint-execution"}
                contract = {key: value for key, value in options.items() if key not in variable_options}
                command_references.setdefault(cell["workload"], contract)
                assert command_references[cell["workload"]] == contract
                signature = summarize.HELPERS.input_prefix(report, cell["workload"])
                references.setdefault(cell["workload"], signature)
                assert references[cell["workload"]] == signature
                samples = read_json(evidence / "resource-samples.json")
                controller = read_json(evidence / "controller-result.json")
                assert len(samples) == controller["sample_count"]
                maximum_gap = cell["external_resource_maximum_gap_seconds"]
                trailing = controller["elapsed_seconds"] - samples[-1]["elapsed_seconds"]
                assert samples[0]["elapsed_seconds"] <= maximum_gap + 0.1
                assert 0 <= trailing <= maximum_gap + 0.1
                coverage.append({"stage": stage, "case": cell["case"], "samples": len(samples),
                                 "first_sample_seconds": samples[0]["elapsed_seconds"],
                                 "last_sample_seconds": samples[-1]["elapsed_seconds"],
                                 "child_seconds": controller["elapsed_seconds"], "trailing_seconds": trailing,
                                 "maximum_gap_seconds": maximum_gap,
                                 "internal_omitted_samples": cell["internal_omitted_resource_samples"]})
        assert read_json(EVIDENCE / "short-summary.json")["selected_candidate"] == selection["candidate"]
        result = {"verified": True, "archives": archives, "stages": stages, "tests": tests,
                  "cross_stage_input_prefixes_match": True, "selection_summary_hash_matches": True,
                  "controller_source_hashes_match": True, "command_contracts_match": True,
                  "final_source_and_binary_identities_match": True,
                  "original_summaries_preserved": True, "all_original_summary_values_match": True,
                  "resource_coverage": coverage,
                  "adoption_gate_passed": read_json(EVIDENCE / "sustained-summary.json")["adoption_gate_passed"]}
        write_json(EVIDENCE / "independent-verification.json", result)
        print(json.dumps({key: value for key, value in result.items() if key not in {"resource_coverage", "archives"}}, indent=2))


if __name__ == "__main__":
    main()
