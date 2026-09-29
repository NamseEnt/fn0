import json
import pathlib
import shlex
import subprocess
import sys
import time

results = pathlib.Path(sys.argv[1])
raw = results / "raw"
raw.mkdir(parents=True, exist_ok=True)
identity = "/Users/namse/Downloads/ssh-key-2026-09-23.key"
remote_host = "opc@217.142.246.204"
remote_results = "/tmp/dodb-phase-g-g3-micro-results"
remote_temporary_root = "/bench/zfs/db/phase-g-tmp"
variants = {
    "g0-mimalloc": "/tmp/dodb-phase-g-g0/target/g0-plain/release/phase0-bench",
    "g3-two-pass-delta-encoder": "/tmp/dodb-phase-g-g3/target/g3-plain/release/phase0-bench",
}
scenarios = [
    (64, 16, "uniform", "disabled"),
    (64, 16, "uniform", "real"),
    (64, 1, "uniform", "real"),
]
run_order = results / "oci-g3-run-order.jsonl"

subprocess.run(
    [
        "ssh",
        "-o",
        "IdentitiesOnly=yes",
        "-i",
        identity,
        remote_host,
        f"mkdir -p {shlex.quote(remote_results)} {shlex.quote(remote_temporary_root)}",
    ],
    check=True,
)

with run_order.open("w", encoding="utf-8") as run_order_file:
    for scenario_index, (writer_count, transaction_width, distribution, sync_mode) in enumerate(scenarios):
        for repetition in range(1, 4):
            ordered_variants = list(variants.items())
            if (scenario_index + repetition) % 2:
                ordered_variants.reverse()
            for variant_name, remote_binary in ordered_variants:
                seed = 979_100_000 + scenario_index * 1_009 + repetition
                stem = (
                    f"g3-micro-{sync_mode}-{scenario_index:02d}-rep{repetition}-"
                    f"{variant_name}-w{writer_count}-width{transaction_width}-{distribution}"
                )
                remote_output = f"{remote_results}/{stem}.jsonl"
                remote_temporary = f"{remote_temporary_root}/{stem}"
                local_output = raw / f"{stem}.jsonl"
                local_log = raw / f"{stem}.log"
                remote_benchmark_command = [
                    remote_binary,
                    "--engine",
                    "planned-blink",
                    "--suite",
                    "write",
                    "--writers",
                    str(writer_count),
                    "--widths",
                    str(transaction_width),
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
                    f"mkdir -p {shlex.quote(remote_temporary)} && "
                    f"TMPDIR={shlex.quote(remote_temporary)} "
                    f"{shlex.join(remote_benchmark_command)} && "
                    f"rm -rf {shlex.quote(remote_temporary)}"
                )
                ssh_command = [
                    "ssh",
                    "-o",
                    "IdentitiesOnly=yes",
                    "-i",
                    identity,
                    remote_host,
                    remote_script,
                ]
                event = {
                    "scenario_index": scenario_index,
                    "repetition": repetition,
                    "variant": variant_name,
                    "writers": writer_count,
                    "width": transaction_width,
                    "distribution": distribution,
                    "sync_mode": sync_mode,
                    "seed": seed,
                    "binary": remote_binary,
                    "command": remote_benchmark_command,
                    "started_unix": time.time(),
                }
                run_order_file.write(json.dumps(event, sort_keys=True) + "\n")
                run_order_file.flush()
                outcome = subprocess.run(ssh_command, capture_output=True, text=True)
                local_log.write_text(outcome.stdout + outcome.stderr, encoding="utf-8")
                if outcome.returncode != 0:
                    raise SystemExit(f"{stem} failed; see {local_log}")
                subprocess.run(
                    [
                        "scp",
                        "-o",
                        "IdentitiesOnly=yes",
                        "-i",
                        identity,
                        f"{remote_host}:{remote_output}",
                        str(local_output),
                    ],
                    check=True,
                )
                record = json.loads(local_output.read_text(encoding="utf-8").splitlines()[0])
                expected = (writer_count, transaction_width, distribution, sync_mode)
                actual = (
                    record["writers"],
                    record["transaction_width"],
                    record["distribution"],
                    record["sync_mode"],
                )
                if expected != actual or record["errors"] != 0:
                    raise SystemExit(f"{stem} settings or errors do not match: {actual}, {record['errors']}")
                print(f"{stem} tx/s={record['logical_tx_per_second']:.0f}", flush=True)
