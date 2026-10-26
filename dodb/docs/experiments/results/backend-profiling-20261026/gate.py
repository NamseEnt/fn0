import json
import os
import sys
import time
from pathlib import Path


root = Path(sys.argv[1])
(root / "workload-pid.json").write_text(json.dumps({"pid": os.getpid(), "uid": os.getuid()}))
deadline = time.monotonic() + 90
while not (root / "release-workload").exists():
    if time.monotonic() >= deadline:
        raise RuntimeError("collector readiness timed out")
    time.sleep(0.02)
os.execv(sys.argv[2], sys.argv[2:])
