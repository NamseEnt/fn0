import contextlib
import hashlib
import importlib.util
import io
import json
import re
import sys
import tarfile
import tempfile
from pathlib import Path


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            value.update(chunk)
    return value.hexdigest()


def main():
    archive_path = Path(sys.argv[1])
    sidecar = json.loads(archive_path.with_suffix(archive_path.suffix + ".json").read_text())
    assert digest(archive_path) == sidecar["sha256"]
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        with tarfile.open(archive_path, "r:gz") as archive:
            for member in archive:
                relative = Path(member.name)
                assert not relative.is_absolute() and ".." not in relative.parts
                assert member.isfile()
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                with archive.extractfile(member) as source, destination.open("wb") as output:
                    while chunk := source.read(1024 * 1024):
                        output.write(chunk)
        inventory = json.loads((root / "evidence-inventory.json").read_text())
        assert len(inventory["files"]) == sidecar["inventoried_files"]
        assert sum(entry["size_bytes"] for entry in inventory["files"]) == sidecar["inventoried_bytes"]
        actual = {path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file()}
        assert actual == {entry["path"] for entry in inventory["files"]} | {"evidence-inventory.json"}
        for entry in inventory["files"]:
            path = root / entry["path"]
            assert path.stat().st_size == entry["size_bytes"] and digest(path) == entry["sha256"], entry["path"]
        specification = importlib.util.spec_from_file_location("archived_summary", root / "controller/summarize.py")
        summary = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(summary)
        accepted_cells = 0
        full_prefixes = {}
        for stage, expected_count in [("preflight", 4), ("short", 24), ("sustained", 24)]:
            stage_root = root / stage
            previous = json.loads((stage_root / "matrix-summary.json").read_text())
            with contextlib.redirect_stdout(io.StringIO()):
                reproduced = summary.summarize(stage_root)
            assert reproduced == previous
            assert len(reproduced["cells"]) == expected_count
            accepted_cells += expected_count
            plan = json.loads((stage_root / "matrix-plan.json").read_text())
            assert digest(root / "controller/run_matrix.py") == plan["controller_sha256"]
            assert digest(root / "controller/checkpoint-costs-matrix.py") == plan["helper_sha256"]
            if stage != "preflight":
                for entry in json.loads((stage_root / "input-prefix-fingerprints.json").read_text()):
                    full_prefixes.setdefault(entry["workload"], entry["signature"])
                    assert full_prefixes[entry["workload"]] == entry["signature"]
        validation = json.loads((root / "validation/validation-complete.json").read_text())
        assert validation["completed"] and validation["failed"] == 0
        records = json.loads((root / "validation/commands.json").read_text())
        accepted = [record for record in records if record["exit_code"] == 0]
        assert len(accepted) == validation["binaries"]
        for record in accepted:
            assert record["source_sha"] == sidecar["identities"]["candidate"]["source_sha"]
            counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",
                                (root / "validation" / record["log"]).read_text())
            assert counts and list(map(int, counts[-1])) == [record["counts"][key] for key in ["passed", "failed", "ignored"]]
        result = {"verified": True, "inventoried_files": len(inventory["files"]), "accepted_cells": accepted_cells,
                  "functional_validation": validation, "database_archives_preserved_on_host": len(inventory["database_archives_preserved_on_host"])}
        print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
