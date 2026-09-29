import pathlib
import shlex
import subprocess


artifact_root = pathlib.Path(__file__).resolve().parents[1]
remote_host = "opc@217.142.246.204"
identity = "/Users/namse/Downloads/ssh-key-2026-09-23.key"
source_cwd = "/home/opc/dodb-wal-v3"
source_commit = "94cfbbe9f8403c0ff2ee0ef063c5d9daa347de90"
binary = "/home/opc/phase0-phase-i-h1"
remote_root = "/home/opc/phase-i-build/perf"
remote_temp = "/bench/zfs/db/phase-i-tmp"
remote_data = f"{remote_root}/h1-current.perf.data"
remote_profile = f"{remote_root}/h1-current.jsonl"
remote_script = (
    f"set -eu; cd {shlex.quote(source_cwd)}; "
    f"test \"$(git rev-parse HEAD)\" = {shlex.quote(source_commit)}; "
    f"mkdir -p {shlex.quote(remote_root)} {shlex.quote(remote_temp)}; "
    f"TMPDIR={shlex.quote(remote_temp)} perf record -F 99 -g --call-graph fp "
    f"-o {shlex.quote(remote_data)} -- {shlex.quote(binary)} "
    f"--engine planned-blink --suite write --writers 64 --widths 16 "
    f"--distributions uniform --duration 10s --warmup 0s --repetitions 1 "
    f"--cache-capacity 256 --working-set 100000 --key-size 16 --value-size 64 "
    f"--group-limit 64 --group-bytes 4194304 --queue-capacity 256 "
    f"--collection-delay 0us --transaction-mode unconditional --tokio-workers 2 "
    f"--sync-mode disabled --parallel-workers 2 --parallel-min-mutations 32 "
    f"--seed 983200001 --output {shlex.quote(remote_profile)}"
)
subprocess.run(
    [
        "ssh",
        "-o",
        "IdentitiesOnly=yes",
        "-i",
        identity,
        remote_host,
        remote_script,
    ],
    check=True,
)
raw_root = artifact_root / "raw" / "profiles"
perf_root = artifact_root / "perf" / "verified"
raw_root.mkdir(parents=True, exist_ok=True)
perf_root.mkdir(parents=True, exist_ok=True)
for remote_path, local_path in (
    (remote_profile, raw_root / "h1-current.jsonl"),
    (remote_data, perf_root / "h1-current.perf.data"),
):
    subprocess.run(
        [
            "scp",
            "-q",
            "-o",
            "IdentitiesOnly=yes",
            "-i",
            identity,
            f"{remote_host}:{remote_path}",
            str(local_path),
        ],
        check=True,
    )
for report_name, arguments in (
    (
        "h1-current-flat-report.txt",
        ["perf", "report", "--stdio", "--no-children", "--percent-limit", "0.05"],
    ),
    (
        "h1-current-callgraph.txt",
        ["perf", "report", "--stdio", "--call-graph", "fractal,0.5,caller", "--percent-limit", "0.2"],
    ),
):
    remote_report = f"{remote_root}/{report_name}"
    remote_command = (
        f"cd {shlex.quote(source_cwd)}; "
        f"perf report --stdio -i {shlex.quote(remote_data)} "
        f"{' '.join(shlex.quote(argument) for argument in arguments[3:])} "
        f"> {shlex.quote(remote_report)}"
    )
    subprocess.run(
        [
            "ssh",
            "-o",
            "IdentitiesOnly=yes",
            "-i",
            identity,
            remote_host,
            remote_command,
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
            f"{remote_host}:{remote_report}",
            str(perf_root / report_name),
        ],
        check=True,
    )
