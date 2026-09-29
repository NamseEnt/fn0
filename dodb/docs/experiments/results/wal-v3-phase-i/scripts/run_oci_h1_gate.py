import json
import pathlib
import shlex
import subprocess
import time


artifact_root = pathlib.Path(__file__).resolve().parents[1]
remote_host = "opc@217.142.246.204"
identity = "/Users/namse/Downloads/ssh-key-2026-09-23.key"
binary = "/home/opc/phase0-phase-i-h1"
source_cwd = "/home/opc/dodb-wal-v3"
source_commit = "94cfbbe9f8403c0ff2ee0ef063c5d9daa347de90"
remote_root = "/home/opc/phase-i-build/h1-gate"
remote_temp = "/bench/zfs/db/phase-i-tmp"
local_raw = artifact_root / "raw" / "h1-gate"
local_raw.mkdir(parents=True, exist_ok=True)
subprocess.run(
    [
        "ssh",
        "-o",
        "IdentitiesOnly=yes",
        "-i",
        identity,
        remote_host,
        f"mkdir -p {shlex.quote(remote_root)} {shlex.quote(remote_temp)}",
    ],
    check=True,
)
run_order = local_raw / "run-order.jsonl"
with run_order.open("w", encoding="utf-8") as run_order_file:
    for repetition in range(1, 4):
        sync_order = ["disabled", "real"] if repetition % 2 else ["real", "disabled"]
        for sync_mode in sync_order:
            seed = 983_100_000 + repetition
            stem = f"h1-{sync_mode}-rep{repetition}-w64-width16-uniform"
            remote_output = f"{remote_root}/{stem}.jsonl"
            remote_log = f"{remote_root}/{stem}.log"
            command = [
                binary,
                "--engine",
                "planned-blink",
                "--suite",
                "write",
                "--writers",
                "64",
                "--widths",
                "16",
                "--distributions",
                "uniform",
                "--duration",
                "5s",
                "--warmup",
                "2s",
                "--repetitions",
                "1",
                "--cache-capacity",
                "256",
                "--working-set",
                "100000",
                "--key-size",
                "16",
                "--value-size",
                "64",
                "--group-limit",
                "64",
                "--group-bytes",
                "4194304",
                "--queue-capacity",
                "256",
                "--collection-delay",
                "0us",
                "--transaction-mode",
                "unconditional",
                "--tokio-workers",
                "2",
                "--sync-mode",
                sync_mode,
                "--parallel-workers",
                "2",
                "--parallel-min-mutations",
                "32",
                "--seed",
                str(seed),
                "--output",
                remote_output,
            ]
            remote_script = (
                f"set -eu; cd {shlex.quote(source_cwd)}; "
                f"test \"$(git rev-parse HEAD)\" = {shlex.quote(source_commit)}; "
                f"TMPDIR={shlex.quote(remote_temp)} perf stat -x, -e cycles,instructions "
                f"-- {shlex.join(command)} 2> {shlex.quote(remote_log)}"
            )
            event = {
                "binary": binary,
                "binary_sha256": "6a1a9e3911e8623e27e52a686024ebd766e63a9f697b709a50c30fe698bd5d09",
                "git_commit": source_commit,
                "repetition": repetition,
                "seed": seed,
                "source_cwd": source_cwd,
                "sync_mode": sync_mode,
                "writers": 64,
                "width": 16,
                "distribution": "uniform",
                "command": command,
                "started_unix": time.time(),
            }
            run_order_file.write(json.dumps(event, sort_keys=True) + "\n")
            run_order_file.flush()
            outcome = subprocess.run(
                [
                    "ssh",
                    "-o",
                    "IdentitiesOnly=yes",
                    "-i",
                    identity,
                    remote_host,
                    remote_script,
                ],
                capture_output=True,
                text=True,
            )
            (local_raw / f"{stem}.log").write_text(
                outcome.stdout + outcome.stderr, encoding="utf-8"
            )
            if outcome.returncode != 0:
                raise SystemExit(f"{stem} failed; see local log")
            subprocess.run(
                [
                    "scp",
                    "-q",
                    "-o",
                    "IdentitiesOnly=yes",
                    "-i",
                    identity,
                    f"{remote_host}:{remote_output}",
                    str(local_raw / f"{stem}.jsonl"),
                ],
                check=True,
            )
            subprocess.run(
                [
                    "scp",
                    "-q",
                    "-o",
                    "IdentitiesOnly=yes",
                    "-i",
                    identity,
                    f"{remote_host}:{remote_log}",
                    str(local_raw / f"{stem}.perf.txt"),
                ],
                check=True,
            )
            rows = [
                json.loads(line)
                for line in (local_raw / f"{stem}.jsonl").read_text().splitlines()
                if line.strip()
            ]
            run_rows = [row for row in rows if row.get("record_type") == "run"]
            if len(run_rows) != 1 or run_rows[0].get("git_commit") != source_commit:
                raise SystemExit(f"{stem} reported an unexpected source commit or row count")
            perf_text = (local_raw / f"{stem}.perf.txt").read_text(encoding="utf-8")
            if "cycles:u" not in perf_text or "instructions:u" not in perf_text:
                raise SystemExit(f"{stem} has missing perf counters")
            print(f"completed {stem}: {run_rows[0]['logical_tx_per_second']:.0f} tx/s")
