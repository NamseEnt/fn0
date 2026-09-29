import json
import os
import pathlib
import subprocess
import sys
import time

results = pathlib.Path(sys.argv[1])
raw = results / "raw"
raw.mkdir(parents=True, exist_ok=True)
variants = {
    "g0-mimalloc": pathlib.Path("/tmp/dodb-phase-g-g0-target/release/phase0-bench"),
    "g12-candidate": pathlib.Path("target/release/phase0-bench"),
}
scenarios = [
    (64, 16, "uniform", "disabled"),
    (64, 1, "uniform", "disabled"),
    (64, 16, "same-leaf-heavy", "disabled"),
    (64, 16, "different-leaf-heavy", "disabled"),
    (64, 16, "uniform", "real"),
    (64, 1, "uniform", "real"),
]
run_order = results / "local-run-order.jsonl"
with run_order.open("w", encoding="utf-8") as run_order_file:
    for scenario_index, (writer_count, transaction_width, distribution, sync_mode) in enumerate(scenarios):
        for repetition in range(1, 4):
            ordered_variants = list(variants.items())
            if (scenario_index + repetition) % 2:
                ordered_variants.reverse()
            for variant_name, binary_path in ordered_variants:
                seed = 979_000_000 + scenario_index * 1_009 + repetition
                stem = (
                    f"local-{sync_mode}-{scenario_index:02d}-rep{repetition}-{variant_name}-"
                    f"w{writer_count}-width{transaction_width}-{distribution}"
                )
                output_path = raw / f"{stem}.jsonl"
                log_path = raw / f"{stem}.log"
                temporary_directory = pathlib.Path("/tmp") / f"dodb-phase-g-{stem}"
                temporary_directory.mkdir(parents=True, exist_ok=True)
                environment = os.environ.copy()
                environment["TMPDIR"] = str(temporary_directory)
                command = [
                    str(binary_path),
                    "--engine", "planned-blink",
                    "--suite", "write",
                    "--writers", str(writer_count),
                    "--widths", str(transaction_width),
                    "--distributions", distribution,
                    "--duration", "2s",
                    "--warmup", "1s",
                    "--repetitions", "1",
                    "--cache-capacity", "256",
                    "--working-set", "100000",
                    "--key-size", "16",
                    "--value-size", "64",
                    "--group-limit", "64",
                    "--group-bytes", "4194304",
                    "--queue-capacity", "256",
                    "--collection-delay", "0us",
                    "--transaction-mode", "unconditional",
                    "--tokio-workers", "2",
                    "--sync-mode", sync_mode,
                    "--parallel-workers", "2",
                    "--parallel-min-mutations", "32",
                    "--seed", str(seed),
                    "--output", str(output_path),
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
                    "binary": str(binary_path.resolve()),
                    "command": command,
                    "started_unix": time.time(),
                }
                run_order_file.write(json.dumps(event, sort_keys=True) + "\n")
                run_order_file.flush()
                outcome = subprocess.run(command, env=environment, capture_output=True, text=True)
                log_path.write_text(outcome.stdout + outcome.stderr, encoding="utf-8")
                if outcome.returncode != 0:
                    raise SystemExit(f"{stem} failed; see {log_path}")
                records = output_path.read_text(encoding="utf-8").splitlines()
                if len(records) != 1:
                    raise SystemExit(f"{stem} emitted {len(records)} JSONL rows")
                record = json.loads(records[0])
                expected = (writer_count, transaction_width, distribution, sync_mode)
                actual = (
                    record["writers"], record["transaction_width"], record["distribution"], record["sync_mode"]
                )
                if expected != actual or record["errors"] != 0:
                    raise SystemExit(f"{stem} settings or errors do not match: {actual}, {record['errors']}")
                print(f"{stem} tx/s={record['logical_tx_per_second']:.0f}", flush=True)
