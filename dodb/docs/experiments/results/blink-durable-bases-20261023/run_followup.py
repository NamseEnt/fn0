import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(os.environ["DODB_BASE_RUN_ROOT"])
SOURCE = ROOT / "source"
DOCS = SOURCE / "dodb/docs/experiments/results/blink-durable-bases-20261023"
ENVIRONMENT = os.environ.copy()
ENVIRONMENT["PYTHONDONTWRITEBYTECODE"] = "1"
ENVIRONMENT["DODB_BASE_SOURCE_SHA"] = subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip()


def run(name, command, environment):
    print(json.dumps({"starting": name}), flush=True)
    with (ROOT / f"{name}.log").open("wb") as stream:
        result = subprocess.run(command, cwd=SOURCE, env=environment, stdout=stream, stderr=subprocess.STDOUT)
    if result.returncode:
        raise SystemExit(result.returncode)
    print(json.dumps({"completed": name}), flush=True)


def main():
    while not (ROOT / "short-stage-complete.txt").exists():
        steps_path = ROOT / "pipeline-steps.json"
        if steps_path.exists():
            steps = json.loads(steps_path.read_text())
            if steps and steps[-1].get("exit_code", 0) != 0:
                raise SystemExit("prior validation or short-matrix gate failed")
        time.sleep(15)
    run("short-summary", [sys.executable, str(DOCS / "summarize.py"), str(ROOT / "short")], ENVIRONMENT)
    summary_path = ROOT / "short/matrix-summary.json"
    summary = json.loads(summary_path.read_text())
    assert summary["validated"] and not summary["smoke"]
    candidate = summary["selected_candidate"]
    selection = {"source_sha": summary["source_sha"], "candidate": candidate, "scores": summary["selection_scores"],
                 "short_summary_sha256": hashlib.sha256(summary_path.read_bytes()).hexdigest(),
                 "selection": "maximum throughput-ratio geometric mean; ties within one percentage point prefer lower E p99.9"}
    (ROOT / "selection.json").write_text(json.dumps(selection, indent=2) + "\n")
    sustained = ROOT / "sustained"
    sustained.mkdir(exist_ok=False)
    (sustained / "source").symlink_to(SOURCE)
    (sustained / "bin").symlink_to(ROOT / "bin")
    environment = ENVIRONMENT.copy()
    environment.update({"DODB_BASE_RUN_ROOT": str(sustained), "DODB_BASE_SMOKE": "0", "DODB_BASE_MEASUREMENT_MS": "300000",
                        "DODB_BASE_PATHS": f"segmented-sync,{candidate}"})
    run("sustained", [sys.executable, str(DOCS / "run_matrix.py")], environment)
    run("sustained-summary", [sys.executable, str(DOCS / "summarize.py"), str(sustained)], ENVIRONMENT)
    (ROOT / "all-stages-complete.txt").write_text("validation, preflight, short and sustained matrices completed\n")


if __name__ == "__main__":
    main()
