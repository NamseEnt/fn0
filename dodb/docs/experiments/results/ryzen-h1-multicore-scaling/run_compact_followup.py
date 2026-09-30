import datetime
import json
import os
import pathlib
import random
import resource
import shutil
import subprocess
import time

source = pathlib.Path('/home/namse/dodb-scaling-source')
results = pathlib.Path('/home/namse/dodb-scaling-artifacts/restart-20260930-01')
raw = results / 'raw' / 'matrix'
data_root = pathlib.Path('/var/tmp/dodb-scaling-data')
dodb_binary = pathlib.Path('/home/namse/dodb-scaling-target/release/phase0-bench')
rocks_binary = pathlib.Path('/home/namse/rocksdb-bench-target/release/rocksdb-bench')
run_order = results / 'run-order.jsonl'

configurations = [
    ('1c', '0', 2),
    ('2c', '0,1', 2),
    ('4c', '0,1,2,3', 4),
    ('6c', '0,1,2,3,4,5', 6),
    ('6c12t', '0,1,2,3,4,5,6,7,8,9,10,11', 12),
]
workloads = [
    ('w1_uniform', 1, 'uniform'),
    ('w16_uniform', 16, 'uniform'),
    ('w16_compact', 16, 'same-leaf-heavy'),
    ('w16_spread', 16, 'different-leaf-heavy'),
]

raw.mkdir(parents=True, exist_ok=True)
data_root.mkdir(parents=True, exist_ok=True)
source_commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
source_status = subprocess.check_output(['git', 'status', '--porcelain=v2'], cwd=source, text=True)
if source_commit != '08e82ec484bd62aeb659471a39f317b86cfb488f' or source_status:
    raise SystemExit(f'unexpected source provenance: {source_commit} {source_status!r}')
if not dodb_binary.is_file() or not rocks_binary.is_file():
    raise SystemExit('benchmark binary missing')

runs = [(core_name, cpu_list, worker_count, workload_name, width, distribution, workload_index)
        for core_index, (core_name, cpu_list, worker_count) in enumerate(configurations)
        for workload_index, (workload_name, width, distribution) in enumerate(workloads)]
runs = [run for run in runs if (run[0], run[3]) in {("1c", "w16_compact")}]
random.Random(20260930).shuffle(runs)


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def block_stats():
    path = pathlib.Path('/sys/block/sda/sda2/stat')
    values = [int(value) for value in path.read_text().split()]
    return {'write_ios': values[4], 'sectors_written': values[6], 'io_ticks_ms': values[9], 'weighted_io_ticks_ms': values[10]}


def proc_rss_kib(process_id):
    try:
        for line in pathlib.Path(f'/proc/{process_id}/status').read_text().splitlines():
            if line.startswith('VmRSS:'):
                return int(line.split()[1])
    except FileNotFoundError:
        return None
    return None


def run_one(engine, core_name, cpu_list, worker_count, workload_name, width, distribution, workload_index, repetition):
    seed = 977000000 + workload_index * 10000 + repetition
    run_name = f'{engine}-{core_name}-{workload_name}-rep{repetition + 1}'
    output_path = raw / f'{run_name}.jsonl'
    log_path = raw / f'{run_name}.log'
    data_dir = data_root / 'dodb' if engine == 'dodb' else pathlib.Path('/bench/zfs/db/rocksdb')
    shutil.rmtree(data_dir, ignore_errors=True)
    data_dir.mkdir(parents=True, exist_ok=True)
    if engine == 'dodb':
        command = [
            'taskset', '--cpu-list', cpu_list, str(dodb_binary),
            '--engine', 'parallel-blink', '--suite', 'write', '--writers', '64',
            '--widths', str(width), '--distributions', distribution,
            '--duration', '5s', '--warmup', '2s', '--repetitions', '1',
            '--cache-capacity', '256', '--working-set', '100000', '--key-size', '16',
            '--value-size', '64', '--group-limit', '64', '--group-bytes', '4194304',
            '--queue-capacity', '256', '--collection-delay', '0us',
            '--transaction-mode', 'unconditional', '--tokio-workers', str(worker_count),
            '--blink-workers', str(worker_count), '--seed', str(seed),
            '--output', str(output_path),
        ]
        environment = os.environ.copy()
        environment['DODB_BENCH_DIR'] = str(data_dir)
        cwd = source
    else:
        command = [
            'taskset', '--cpu-list', cpu_list, str(rocks_binary), '--pipelined', 'off',
            '--mode', 'bench', '--writers', '64', '--width', str(width),
            '--distribution', distribution, '--working-set', '100000', '--key-size', '16',
            '--value-size', '64', '--warmup-ms', '2000', '--duration-ms', '5000',
            '--window-ms', '0', '--monitor-ms', '1000', '--seed', str(seed),
            '--scenario-index', str(workload_index), '--repetition', str(repetition + 1),
            '--data-dir', str(data_dir), '--output', str(output_path),
        ]
        environment = os.environ.copy()
        cwd = source
    started = time.monotonic()
    usage_before = resource.getrusage(resource.RUSAGE_CHILDREN)
    block_before = block_stats()
    peak_rss = 0
    start_record = {
        'event': 'start', 'timestamp': utc_now(), 'run': run_name, 'engine': engine,
        'source_commit': source_commit, 'cpu_list': cpu_list, 'worker_count': worker_count,
        'writers': 64, 'width': width, 'distribution': distribution, 'seed': seed,
        'warmup_ms': 2000, 'duration_ms': 5000, 'command': command,
        'data_dir': str(data_dir), 'output': str(output_path),
    }
    with run_order.open('a', encoding='utf-8') as order_file:
        order_file.write(json.dumps(start_record, sort_keys=True) + '\n')
    with log_path.open('w', encoding='utf-8') as log_file:
        process = subprocess.Popen(command, cwd=cwd, env=environment, stdout=log_file, stderr=subprocess.STDOUT)
        while process.poll() is None:
            peak_rss = max(peak_rss, proc_rss_kib(process.pid) or 0)
            time.sleep(0.5)
        exit_code = process.wait()
    elapsed = time.monotonic() - started
    usage_after = resource.getrusage(resource.RUSAGE_CHILDREN)
    block_after = block_stats()
    result_lines = [json.loads(line) for line in output_path.read_text().splitlines() if line.strip()] if output_path.exists() else []
    expected_engine = 'parallel-blink' if engine == 'dodb' else 'rocksdb'
    if exit_code != 0 or len(result_lines) != 1 or result_lines[0].get('engine') != expected_engine:
        raise SystemExit(f'invalid result for {run_name}: exit={exit_code} rows={len(result_lines)} actual_engine={result_lines[0].get("engine") if result_lines else None}')
    if engine == 'dodb' and result_lines[0].get('git_commit') != source_commit:
        raise SystemExit(f'dodb row commit mismatch for {run_name}')
    complete_record = dict(start_record)
    complete_record.update({
        'event': 'complete', 'timestamp': utc_now(), 'exit_code': exit_code,
        'elapsed_seconds': elapsed, 'peak_rss_sampled_kib': peak_rss,
        'user_cpu_seconds': usage_after.ru_utime - usage_before.ru_utime,
        'system_cpu_seconds': usage_after.ru_stime - usage_before.ru_stime,
        'voluntary_context_switches': usage_after.ru_nvcsw - usage_before.ru_nvcsw,
        'involuntary_context_switches': usage_after.ru_nivcsw - usage_before.ru_nivcsw,
        'cpu_utilization_one_core_percent': 100.0 * (usage_after.ru_utime - usage_before.ru_utime + usage_after.ru_stime - usage_before.ru_stime) / elapsed,
        'block_device_before': block_before, 'block_device_after': block_after,
        'result_row': result_lines[0],
    })
    complete_record.pop('command')
    with run_order.open('a', encoding='utf-8') as order_file:
        order_file.write(json.dumps(complete_record, sort_keys=True) + '\n')
    print(f'{run_name} complete exit={exit_code} elapsed={elapsed:.1f}s', flush=True)
    shutil.rmtree(data_dir, ignore_errors=True)


for core_name, cpu_list, worker_count, workload_name, width, distribution, workload_index in runs:
    for repetition in range(6, 12):
        engine_order = ['dodb', 'rocksdb'] if (repetition + workload_index + len(cpu_list)) % 2 == 0 else ['rocksdb', 'dodb']
        for engine in engine_order:
            run_one(engine, core_name, cpu_list, worker_count, workload_name, width, distribution, workload_index, repetition)
