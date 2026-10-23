import importlib.util
import os
from pathlib import Path


ROOT = Path(os.environ["DODB_BASE_RUN_ROOT"])
SOURCE = ROOT / "source"
HELPER = SOURCE / "dodb/docs/experiments/results/blink-background-checkpoint-20261021/run_oci_build_monitored.py"
SPEC = importlib.util.spec_from_file_location("monitored_build", HELPER)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
MODULE.ROOT = ROOT
MODULE.main()
