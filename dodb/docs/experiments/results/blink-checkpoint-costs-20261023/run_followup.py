import importlib.util
import json
import os
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(os.environ["DODB_COST_CANDIDATE_ROOT"])
SOURCE = ROOT / "source"
DOCS = SOURCE / "dodb/docs/experiments/results/blink-checkpoint-costs-20261023"
os.environ["DODB_COST_ACCOUNTING_HELPER"] = str(SOURCE / "dodb/docs/experiments/results/blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py")
SPEC = importlib.util.spec_from_file_location("cost_summary", ROOT / "summarize.py")
REDUCER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REDUCER)


def main():
    while not (ROOT / "short-stage-complete.txt").exists():
        steps_path = ROOT / "pipeline-steps.json"
        if steps_path.exists():
            steps = json.loads(steps_path.read_text())
            if steps and steps[-1].get("exit_code", 0) != 0:
                raise SystemExit("prior build, validation or measurement gate failed")
        time.sleep(15)
    for stage in ["preflight", "short"]:
        summary = REDUCER.summarize(ROOT / stage)
        print(json.dumps({"stage": stage, "validated": summary["validated"], "gates": summary["followup_gates"]}), flush=True)
    decision = {"allowed": summary["followup_allowed"], "gates": summary["followup_gates"],
                "short_summary_sha256": REDUCER.digest(ROOT / "short/matrix-summary.json")}
    (ROOT / "followup-decision.json").write_text(json.dumps(decision, indent=2) + "\n")
    if not decision["allowed"]:
        (ROOT / "followup-skipped.txt").write_text("The predeclared short-matrix gates failed.\n")
        return
    environment = os.environ.copy()
    environment.update({"DODB_COST_RUN_ROOT": str(ROOT / "sustained"), "DODB_COST_DURATION_MS": "300000",
                        "DODB_COST_SMOKE": "0", "DODB_COST_PATHS": "candidate,minimal-native", "PYTHONDONTWRITEBYTECODE": "1"})
    with (ROOT / "sustained.log").open("xb") as stream:
        subprocess.run([sys.executable, str(DOCS / "run_matrix.py")], cwd=SOURCE, env=environment,
                       stdout=stream, stderr=subprocess.STDOUT, check=True)
    REDUCER.summarize(ROOT / "sustained")
    (ROOT / "all-stages-complete.txt").write_text("Functional validation, preflight, short and sustained matrices completed.\n")


if __name__ == "__main__":
    main()
