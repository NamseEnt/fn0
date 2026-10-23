import hashlib
import importlib.util
import json
import os
import re
import tarfile
import tempfile
from pathlib import Path


DOCS = Path(__file__).parent
EVIDENCE = DOCS / "evidence"
ARCHIVE = EVIDENCE / "blink-checkpoint-costs-20261023-evidence.tar.gz"


def digest(contents):
    return hashlib.sha256(contents).hexdigest()


def main():
    sidecar = json.loads(ARCHIVE.with_suffix(ARCHIVE.suffix + ".json").read_text())
    assert digest(ARCHIVE.read_bytes()) == sidecar["sha256"]
    with tempfile.TemporaryDirectory(prefix="dodb-checkpoint-costs-verify-") as directory:
        root = Path(directory)
        with tarfile.open(ARCHIVE) as stream:
            inventory = json.load(stream.extractfile("evidence-file-sha256.json"))
            assert len(inventory) == sidecar["files"]
            for entry in inventory:
                relative = Path(entry["path"])
                assert not relative.is_absolute() and ".." not in relative.parts
                contents = stream.extractfile(entry["path"]).read()
                assert len(contents) == entry["size_bytes"] and digest(contents) == entry["sha256"], entry["path"]
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(contents)
        os.environ["DODB_COST_ACCOUNTING_HELPER"] = str(root / "controller/analyze_loss.py")
        assert digest((root / "summarize.py").read_bytes()) == digest((DOCS / "summarize.py").read_bytes())
        specification = importlib.util.spec_from_file_location("cost_reducer", DOCS / "summarize.py")
        reducer = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(reducer)
        references, identities, stages, summary_hashes = {}, {}, {}, {}
        for stage in ["preflight", "short", "sustained"]:
            stage_root = root / stage
            if not (stage_root / "matrix-complete.txt").exists():
                continue
            plan = json.loads((stage_root / "matrix-plan.json").read_text())
            assert digest((root / "controller/run_matrix.py").read_bytes()) == plan["controller_sha256"]
            assert digest((root / "controller/checkpoint_controller.py").read_bytes()) == plan["helper_sha256"]
            summary_bytes = (stage_root / "matrix-summary.json").read_bytes()
            summary_hashes[stage] = digest(summary_bytes)
            original = json.loads(summary_bytes)
            recomputed = reducer.summarize(stage_root)
            assert recomputed == original, stage
            for policy, identity in plan["identities"].items():
                identities.setdefault(policy, identity)
                assert identities[policy] == identity, policy
            for entry in json.loads((stage_root / "input-prefix-fingerprints.json").read_text()):
                references.setdefault(entry["workload"], entry["signature"])
                assert references[entry["workload"]] == entry["signature"], entry["case"]
            stages[stage] = {"cells": len(recomputed["cells"]), "summary_exactly_reproduced": True}
        assert {"preflight", "short"} <= set(stages)
        decision = json.loads((root / "followup-decision.json").read_text())
        assert decision["short_summary_sha256"] == summary_hashes["short"]
        assert decision["allowed"] == json.loads((root / "short/matrix-summary.json").read_text())["followup_allowed"]
        assert ("sustained" in stages) == decision["allowed"]
        validation = json.loads((root / "validation/validation-complete.json").read_text())
        commands = json.loads((root / "validation/commands.json").read_text())
        accepted = {}
        for command in commands:
            if command.get("exit_code") == 0:
                accepted[command["identity"]] = command
        counts = {"passed": 0, "failed": 0, "ignored": 0}
        for command in accepted.values():
            assert command["source_sha"] == validation["source_sha"] == identities["candidate"]["source_sha"]
            text = (root / "validation" / command["log"]).read_text()
            matches = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
            assert matches, command["identity"]
            observed = dict(zip(counts, map(int, matches[-1])))
            assert observed == command["counts"]
            for category, count in observed.items():
                counts[category] += count
        assert len(accepted) == validation["binaries"] == 20
        assert all(counts[category] == validation[category] for category in counts)
        final_identity = json.loads((root / "final-identity.json").read_text())
        for policy, identity in identities.items():
            assert final_identity[policy]["source_sha"] == identity["source_sha"]
            assert final_identity[policy]["binaries"] == identity["binaries"]
            assert final_identity[policy]["source_clean"]
        local_summary = json.loads((EVIDENCE / "local/summary.json").read_text())
        local_archive = EVIDENCE / "local" / local_summary["archive"]["name"]
        assert digest(local_archive.read_bytes()) == local_summary["archive"]["sha256"]
        with tarfile.open(local_archive) as stream:
            assert set(stream.getnames()) == set(local_summary["files"])
            for name, expected in local_summary["files"].items():
                assert digest(stream.extractfile(name).read()) == expected, name
        result = {"verified": True, "archive_sha256": sidecar["sha256"], "inventoried_files": len(inventory),
                  "stages": stages, "functional_binaries": len(accepted), "functional_counts": counts,
                  "cross_stage_input_prefixes_equal": True, "final_source_and_binary_identities_match": True,
                  "local_raw_logs_verified": len(local_summary["files"])}
        (EVIDENCE / "offline-verification.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
