import json
import gzip
import pathlib
import shlex
import subprocess
import sys
import time


artifact_root = pathlib.Path(__file__).resolve().parents[1]
sync_modes = [sys.argv[1]] if len(sys.argv) > 1 else ["disabled", "real"]
remote_host = "opc@217.142.246.204"
identity = "/Users/namse/Downloads/ssh-key-2026-09-23.key"
binary = "/home/opc/phase0-phase-i-locality"
source_cwd = "/home/opc/dodb-wal-v3"
source_commit = "94cfbbe9f8403c0ff2ee0ef063c5d9daa347de90"
remote_raw = "/home/opc/phase-i-build/locality"
remote_temp = "/bench/zfs/db/phase-i-tmp"
scenarios = [
    (64, 16, "uniform"),
    (64, 1, "uniform"),
    (64, 16, "same-leaf-heavy"),
    (64, 16, "different-leaf-heavy"),
    (16, 16, "uniform"),
]

for sync_mode in sync_modes:
    local_raw = artifact_root / "raw" / "locality" / sync_mode
    local_raw.mkdir(parents=True, exist_ok=True)
    run_order = local_raw / "run-order.jsonl"
    subprocess.run(
        [
            "ssh",
            "-o",
            "IdentitiesOnly=yes",
            "-i",
            identity,
            remote_host,
            f"mkdir -p {shlex.quote(remote_raw)} {shlex.quote(remote_temp)}",
        ],
        check=True,
    )
    with run_order.open("w", encoding="utf-8") as run_order_file:
        for scenario_index, (writer_count, width, distribution) in enumerate(scenarios):
            for repetition in range(1, 4):
                seed = 982_100_000 + scenario_index * 1_009 + repetition
                stem = f"locality-{sync_mode}-{scenario_index:02d}-rep{repetition}-w{writer_count}-width{width}-{distribution}"
                remote_output = f"{remote_raw}/{stem}.jsonl"
                remote_locality = f"{remote_raw}/{stem}.groups.jsonl"
                command = [
                    binary,
                    "--engine",
                    "planned-blink",
                    "--suite",
                    "write",
                    "--writers",
                    str(writer_count),
                    "--widths",
                    str(width),
                    "--distributions",
                    distribution,
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
                    f"rm -f {shlex.quote(remote_locality)}; "
                    f"DODB_PHASE_I_LOCALITY_OUTPUT={shlex.quote(remote_locality)} "
                    f"TMPDIR={shlex.quote(remote_temp)} {shlex.join(command)}"
                )
                event = {
                    "binary": binary,
                    "distribution": distribution,
                    "git_commit": source_commit,
                    "group_locality_output": remote_locality,
                    "repetition": repetition,
                    "seed": seed,
                    "source_cwd": source_cwd,
                    "sync_mode": sync_mode,
                    "width": width,
                    "writers": writer_count,
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
                        f"{remote_host}:{remote_locality}",
                        str(local_raw / f"{stem}.groups.jsonl"),
                    ],
                    check=True,
                )
                run_rows = [
                    json.loads(line)
                    for line in (local_raw / f"{stem}.jsonl").read_text().splitlines()
                    if line.strip()
                ]
                group_rows = [
                    json.loads(line)
                    for line in (local_raw / f"{stem}.groups.jsonl").read_text().splitlines()
                    if line.strip()
                ]
                if not any(row.get("git_commit") == source_commit for row in run_rows):
                    raise SystemExit(f"{stem} reported an unexpected git_commit")
                if not group_rows or any(
                    row.get("git_commit") != source_commit for row in group_rows
                ):
                    raise SystemExit(f"{stem} has missing or unexpected group samples")
                group_path = local_raw / f"{stem}.groups.jsonl"
                compressed_path = local_raw / f"{stem}.groups.jsonl.gz"
                with group_path.open("rb") as input_file:
                    with compressed_path.open("wb") as raw_output:
                        with gzip.GzipFile(
                            filename="", mode="wb", fileobj=raw_output, mtime=0
                        ) as compressed_output:
                            while chunk := input_file.read(1024 * 1024):
                                compressed_output.write(chunk)
                group_path.unlink()
                print(f"completed {stem}: {len(group_rows)} physical WAL groups")
