import argparse
import importlib.util
import json
import platform
import shlex
import subprocess
import time
from pathlib import Path


module_path = Path(__file__).with_name('run-rocksdb-required-costs.py')
module_spec = importlib.util.spec_from_file_location('required_costs', module_path)
required = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(required)
CONDITIONS = [('service-before', 'before', 'rocksdb', 'async'), ('service-after', 'after', 'rocksdb', 'async')]
SIZES = [512, 65_536, 1_048_576]
MODES = ['put', 'exists']


def verify_overwrite(report, enabled, size, mode, version):
    assert report['validation'] and report['reopen_validation']
    assert report['mode'] == mode and report['value_bytes'] == size
    assert report['clients'] == report['initial_keys'] == 16
    assert report['shared_slots'] == 8 and report['revision_allocator'] == 'reserved-ranges'
    assert report['warmup_seconds'] == 1 and report['measurement_seconds'] == 5
    assert report['calls'] == report['latency']['samples']
    prefix = 8 if size == 1_048_576 else 100
    assert report['input_prefix_workflows'] == prefix
    assert len(report['client_prefixes']) == 16
    assert all(client['workflows'] == prefix for client in report['client_prefixes'])
    assert report['research_metrics']['enabled'] is enabled
    if not enabled:
        return
    measured = report['research_metrics']['tenants']['tenant_1']
    roles = measured['write_reads']['measurement']
    for role_name, role in roles.items():
        expected_calls = report['calls'] if role_name == ('condition_key' if mode == 'exists' else 'mutation_only_key') else 0
        assert role['keys'] == role['found_keys'] == expected_calls
        assert role['encoded_bytes_fetched'] == expected_calls * (size + 10)
        assert role['payload_bytes_fetched'] == expected_calls * size
        assert all(histogram['count'] == expected_calls for histogram in role['timings'].values())
        if version == 'after':
            assert role['encoded_bytes_returned_by_shim'] == expected_calls * 10
    outcomes = measured['outcomes']['measurement']
    assert outcomes['user_commits'] == outcomes['successful_writes'] == report['calls']
    assert outcomes['condition_rejections'] == outcomes['other_errors'] == outcomes['user_transaction_counter_accesses'] == 0


def overwrite_case(args, version, size, mode, enabled, repetition, sequence):
    label = 'on' if enabled else 'off'
    run_name = f'{sequence:03d}-{size}-{mode}-{version}-{label}-rep{repetition}'
    run_path = args.result_root / 'overwrite' / run_name
    run_path.mkdir(parents=True, exist_ok=False)
    binary = getattr(args, version + '_probe')
    prefix = 8 if size == 1_048_576 else 100
    command = ['/usr/bin/time', '-v', '-o', str(run_path / 'process-time.txt'), str(binary),
        '--mode', mode, '--metrics', str(int(enabled)), '--value-size', str(size),
        '--prefix-workflows', str(prefix), '--data-dir', str(run_path / 'db'), '--report', str(run_path / 'report.json')]
    (run_path / 'command.txt').write_text(shlex.join(command) + '\n')
    before = required.digest(binary)
    started = time.monotonic()
    with (run_path / 'stdout.txt').open('w') as output:
        completed = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT)
    (run_path / 'exit-status.txt').write_text(str(completed.returncode) + '\n')
    provenance = {'version': version, 'value_size': size, 'mode': mode, 'metrics_enabled': enabled,
        'source_sha': args.before_probe_sha if version == 'before' else args.after_sha,
        'sequence': sequence, 'repetition': repetition, 'elapsed_seconds': time.monotonic() - started,
        'binary_sha256_before': before, 'binary_sha256_after': required.digest(binary)}
    (run_path / 'provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    assert completed.returncode == 0, run_name
    assert provenance['binary_sha256_after'] == before
    verify_overwrite(json.loads((run_path / 'report.json').read_text()), enabled, size, mode, version)
    required.archive_database(run_path)
    (run_path / 'checked.json').write_text(json.dumps({'passed': True}) + '\n')
    print(json.dumps({'run': run_name, 'overwrite': True, 'passed': True}), flush=True)


def main():
    parser = argparse.ArgumentParser()
    for version in ['before', 'after']:
        parser.add_argument('--' + version + '-binary', required=True, type=Path)
        parser.add_argument('--' + version + '-probe', required=True, type=Path)
        parser.add_argument('--' + version + '-sha', required=True)
    parser.add_argument('--before-probe-sha', required=True)
    parser.add_argument('--result-root', required=True, type=Path)
    args = parser.parse_args()
    checked_output = required.checked_output
    assert platform.machine() == 'aarch64' and checked_output(['hostname']) == 'instance-20260923-1013'
    assert checked_output(['git', 'rev-parse', 'HEAD']) == args.after_sha
    assert not checked_output(['git', 'status', '--porcelain'])
    assert checked_output(['findmnt', '-n', '-o', 'FSTYPE', '-T', str(args.result_root.parent)]) == 'zfs'
    args.result_root.mkdir(exist_ok=False)
    plan = {'before_sha': args.before_sha, 'after_sha': args.after_sha, 'before_probe_sha': args.before_probe_sha,
        'conditions': CONDITIONS, 'scenarios': required.SCENARIOS, 'sizes': SIZES, 'modes': MODES,
        'service_runs': 30, 'smoke_runs': 8, 'overwrite_performance_runs': 36, 'overwrite_diagnostic_runs': 12,
        'repetitions': 3, 'service_allocator': 'reserved-ranges',
        'scope': 'header propagation only; native reads, protected locks, validation and durability retained',
        'process_time_scope': 'overwrite process-time.txt covers the entire executable including setup/validation, not only the measurement window'}
    for version in ['before', 'after']:
        plan[version + '_binary_sha256'] = required.digest(getattr(args, version + '_binary'))
        plan[version + '_probe_sha256'] = required.digest(getattr(args, version + '_probe'))
    (args.result_root / 'plan.json').write_text(json.dumps(plan, indent=2) + '\n')
    commands = [['date', '-u'], ['hostname'], ['uname', '-a'], ['lscpu'], ['free', '-h'],
        ['df', '-h', '/bench/zfs/db'], ['findmnt', '-T', '/bench/zfs/db'],
        ['git', 'status', '--short', '--branch'], ['rustup', 'run', '1.97.1', 'rustc', '-vV'],
        ['git', '-C', '/bench/zfs/db/product-build/rocksdb-source', 'rev-parse', 'HEAD'],
        ['sha256sum', '/bench/zfs/db/product-build/rocksdb-source/librocksdb.a']]
    with (args.result_root / 'host-and-source.txt').open('w') as output:
        for command in commands:
            output.write(shlex.join(command) + '\n')
            output.flush()
            subprocess.run(command, stdout=output, stderr=subprocess.STDOUT, check=True)
    sequence = 0
    for allocator in required.ALLOCATORS:
        for condition in CONDITIONS:
            for enabled in [False, True]:
                sequence += 1
                required.run_case(args, allocator, condition, 'smoke', 0, sequence, True, enabled)
    assert sequence == 8
    sequence = 0
    for repetition in range(1, 4):
        for scenario_index, scenario in enumerate(required.SCENARIOS):
            order = CONDITIONS if (scenario_index + repetition) % 2 else list(reversed(CONDITIONS))
            for condition in order:
                sequence += 1
                required.run_case(args, 'reserved-ranges', condition, scenario, repetition, sequence)
    assert sequence == 30
    sequence = 0
    for repetition in range(1, 4):
        for size_index, size in enumerate(SIZES):
            for mode_index, mode in enumerate(MODES):
                order = ['before', 'after'] if (size_index + mode_index + repetition) % 2 else ['after', 'before']
                for version in order:
                    sequence += 1
                    overwrite_case(args, version, size, mode, False, repetition, sequence)
    assert sequence == 36
    for size in SIZES:
        for mode in MODES:
            for version in ['before', 'after']:
                sequence += 1
                overwrite_case(args, version, size, mode, True, 1, sequence)
    assert sequence == 48
    assert not checked_output(['git', 'status', '--porcelain'])
    (args.result_root / 'complete.json').write_text(json.dumps({'passed': True, 'service_runs': 30, 'overwrite_runs': 48, 'smoke_runs': 8}) + '\n')


if __name__ == '__main__':
    main()
