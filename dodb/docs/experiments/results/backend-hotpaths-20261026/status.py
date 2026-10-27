import json
import os
from pathlib import Path


root = Path(os.environ.get("DODB_HOTPATH_ROOT", "/bench/zfs/db/backend-hotpaths-20261026"))
result = {"root": str(root)}
steps = root / "pipeline-steps.json"
if steps.exists():
    result["steps"] = [{key: record[key] for key in ["name", "source_sha", "exit_code"] if key in record}
                       for record in json.loads(steps.read_text())]
validation = root / "validation/validation-complete.json"
if validation.exists():
    result["validation"] = json.loads(validation.read_text())
elif (root / "validation/commands.json").exists():
    records = json.loads((root / "validation/commands.json").read_text())
    result["validation"] = {"completed_binaries": sum(record.get("exit_code") == 0 for record in records),
                            "current": records[-1]["name"] if records else None}
for stage in ["preflight", "short", "sustained"]:
    completed = root / stage / "completed-cases.txt"
    if completed.exists():
        cases = completed.read_text().splitlines()
        result[stage] = {"completed": len(cases), "last_completed": cases[-1] if cases else None,
                         "stage_complete": (root / stage / "matrix-complete.txt").exists()}
    log = root / f"{stage}.log"
    if log.exists():
        with log.open("rb") as stream:
            stream.seek(max(0, log.stat().st_size - 2500))
            result[f"{stage}_tail"] = stream.read().decode(errors="replace").splitlines()[-3:]
print(json.dumps(result, indent=2))
