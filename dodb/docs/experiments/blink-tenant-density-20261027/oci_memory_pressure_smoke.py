import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
DATABASE_ROOT = Path('/bench/zfs/db/tenant-density-harness-20261027')
MAX_BYTES = 256 * 1024**2
HIGH_BYTES = 192 * 1024**2
SHARED_ARGUMENTS = [
    '--rocksdb-shared-cache-bytes', str(8 * 1024**2),
    '--rocksdb-shared-wbm-bytes', str(4 * 1024**2),
    '--rocksdb-write-buffer-bytes', str(64 * 1024),
    '--rocksdb-flush-threads', '1', '--rocksdb-compaction-threads', '1',
    '--rocksdb-max-open-files', '64',
]
BLINK_ARGUMENTS = [
    '--blink-wal-layout', 'segmented-bases-coalesced',
    '--blink-checkpoint-execution', 'background',
    '--blink-wal-segment-bytes', '8192', '--blink-checkpoint-wal-bytes', '16384',
]


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024**2), b''):
            result.update(chunk)
    return result.hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def capacity():
    filesystem = os.statvfs(DATABASE_ROOT)
    memory = {}
    for line in Path('/proc/meminfo').read_text().splitlines():
        name, value = line.split(':', 1)
        if name in ['MemTotal', 'MemAvailable', 'SwapFree']:
            memory[name] = int(value.split()[0]) * 1024
    return {'zfs_free_bytes': filesystem.f_bavail * filesystem.f_frsize, 'memory_bytes': memory}


def guard():
    state = capacity()
    assert state['zfs_free_bytes'] >= 12 * 1024**3, state
    assert state['memory_bytes']['MemAvailable'] >= 1500 * 1024**2, state
    return state


def prepare_group(path):
    path.mkdir(exist_ok=False)
    (path / 'cgroup.subtree_control').write_text('+memory')
    assert not (path / 'cgroup.procs').read_text().strip()


def group_snapshot(path):
    fields = ['memory.max', 'memory.high', 'memory.current', 'memory.peak',
              'memory.events', 'memory.events.local', 'memory.stat',
              'memory.pressure', 'memory.swap.max', 'memory.swap.current',
              'cgroup.events', 'cgroup.procs']
    return {name: (path / name).read_text() if (path / name).exists() else None for name in fields}


def run_bounded(invocation, environment, log_path, timeout):
    with log_path.open('x') as output:
        child = subprocess.Popen(invocation, cwd=SOURCE, env=environment,
                                 stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            return child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGTERM)
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait(timeout=5)
            return 'controller_timeout'


def probe_kernel(run_directory, group_root):
    path = group_root / 'allocation-probe'
    prepare_group(path)
    for name, value in [('memory.max', 64 * 1024**2), ('memory.high', 48 * 1024**2),
                        ('memory.swap.max', 0), ('memory.oom.group', 1)]:
        (path / name).write_text(str(value))
    probe = '\n'.join([
        'import os, pathlib, time',
        f'pathlib.Path({str(path / "cgroup.procs")!r}).write_text(str(os.getpid()))',
        'values = []',
        'for chunk_number in range(256):',
        '    values.append(bytearray(1024 * 1024))',
        '    time.sleep(0.001)',
    ])
    before = group_snapshot(path)
    exit_code = run_bounded(['/usr/bin/python3', '-c', probe], os.environ.copy(),
                            run_directory / 'allocation-probe.log', 30)
    after = group_snapshot(path)
    events = dict(line.split() for line in after['memory.events.local'].splitlines())
    result = {'exit_code': exit_code, 'before': before, 'after': after,
              'passed': exit_code == -signal.SIGKILL and int(events.get('oom_kill', 0)) > 0
                        and int(events.get('high', 0)) > 0 and not after['cgroup.procs'].strip(),
              'scope': 'Isolated anonymous-allocation probe; not a database workload or density result'}
    write_json(run_directory / 'allocation-probe.json', result)
    assert result['passed'], result


def invocation(binary, backend, active_count, data_path, report_path, group_path, budget=MAX_BYTES, high=HIGH_BYTES):
    arguments = [str(binary), '--backend', backend, '--data-dir', str(data_path),
                 '--report', str(report_path), '--seed', '20261027',
                 '--registered-tenants', '17', '--active-tenants', str(active_count),
                 '--opened-idle-tenants', '0' if active_count == 17 else '2',
                 '--client-count', '1', '--initial-rows', '256', '--value-bytes', '4096',
                 '--topology', 'fixed-per-tenant', '--duration-ms', '3000', '--per-tenant-rate', '10',
                 '--queue-bound', '8', '--max-in-flight', '8', '--request-timeout-ms', '5000',
                 '--drain-timeout-ms', '15000', '--profile', 'write-heavy', '--width', 'mixed',
                 '--backend-cgroup-root', str(group_path), '--backend-memory-max-bytes', str(budget),
                 '--backend-memory-high-bytes', str(high), '--pressure-sample-interval-ms', '100']
    arguments.extend(BLINK_ARGUMENTS if backend == 'blink' else SHARED_ARGUMENTS)
    return arguments


def validate(report, backend, active_count, source_commit):
    assert report['status'] == 'completed'
    assert report['compiled_source_revision'] == source_commit
    workload = report['workload']
    assert workload['cell_valid'] and workload['valid_for_oracle']
    assert workload['scheduled'] == workload['admitted'] == workload['completed'] == workload['successful'] == active_count * 30
    assert all(workload[field] == 0 for field in ['rejected_conflicts', 'other_error', 'timed_out', 'dropped', 'queued_at_end', 'in_flight_at_end'])
    assert all(value is True for value in workload['accounting'].values())
    assert len(workload['per_tenant']) == active_count
    assert all(tenant['successful'] == 30 for tenant in workload['per_tenant'])
    for name in ['post_measurement_full_population_oracle', 'reopened_full_population_oracle']:
        assert report[name]['passed'] and report[name]['verified_seed_rows'] == 17 * 256
    pressure = report['backend_memory_pressure']
    assert pressure['enabled'] and not pressure['samples_truncated']
    assert pressure['samples']
    for final in [pressure['cell']['final'], *[epoch['final'] for epoch in pressure['epochs'].values()]]:
        assert final['memory.max'] == MAX_BYTES and final['memory.high'] == HIGH_BYTES
        assert final['memory.swap.max'] == final['memory.swap.current'] == 0
        assert final['cgroup.procs'] == []
        assert final['memory.events']['oom_kill'] == 0
        assert isinstance(final['memory.peak'], int)
    boundary = workload['offer_boundary']['backend_memory_pressure_after']
    assert boundary['measurement']['cgroup.procs']
    health = report['memory_diagnostic']['engine_health_reports']
    assert len(health) == 4
    for epoch in health:
        assert epoch['shutdown_complete'] and epoch['backend'] == backend
        assert epoch['compiled_source_revision'] == source_commit
        membership = epoch['backend_memory_cgroup']
        assert membership['verified'] and membership['before_runtime_and_backend']
        assert membership['pid'] == epoch['worker_pid']
        assert membership['snapshot']['memory.max'] == MAX_BYTES
        assert epoch['periodic_pressure']['enabled'] and not epoch['periodic_pressure']['samples_truncated']
    samples = health[1]['periodic_pressure']['samples']
    assert any(sample['phase'] == 'measurement-including-drain' for sample in samples)
    if backend == 'minimal-native-shared':
        assert all(sample['shared_resources']['all_opened_tenants_use_shared_resources'] for sample in samples)
        assert any(sample['rocksdb_properties']['databases'] for sample in samples)
    else:
        assert health[1]['blink_checkpoint_metrics']['checkpoints_failed'] == 0
    return {'passed': True, 'successful_requests': workload['successful'],
            'cell_kernel_peak_bytes': pressure['cell']['final']['memory.peak'],
            'measurement_kernel_peak_bytes': pressure['epochs']['measurement']['final']['memory.peak'],
            'cgroup_samples': len(pressure['samples']), 'worker_measurement_samples': len(samples)}


def run(run_name, source_commit):
    assert os.geteuid() == 0, 'Run the bounded controller with sudo; keep the driver outside its backend groups'
    assert run_name and all(character.isalnum() or character in '-_' for character in run_name)
    assert subprocess.check_output(['git', '-c', f'safe.directory={SOURCE}', 'rev-parse', 'HEAD'], cwd=SOURCE, text=True).strip() == source_commit
    assert not subprocess.check_output(['git', '-c', f'safe.directory={SOURCE}', 'status', '--porcelain'], cwd=SOURCE, text=True).strip()
    run_directory = ROOT / 'pressure' / run_name
    run_directory.mkdir(parents=True, exist_ok=False)
    database_directory = DATABASE_ROOT / run_name
    database_directory.mkdir(exist_ok=False)
    group_root = Path('/sys/fs/cgroup') / f'dodb-density-{run_name}'
    prepare_group(group_root)
    binaries = run_directory / 'bin'
    binaries.mkdir()
    identities = {}
    for name in ['tenant_density_workload', 'memory_diagnostic_worker']:
        destination = binaries / name
        shutil.copy2(ROOT / 'target/release' / name, destination)
        identities[name] = {'sha256': digest(destination), 'bytes': destination.stat().st_size}
    write_json(run_directory / 'provenance.json', {'source_commit': source_commit, 'binaries': identities,
        'controller_sha256': digest(Path(__file__)), 'capacity_before': guard(), 'runtime_uid': os.geteuid(),
        'scope': 'Memory enforcement and correctness only; no density, SLO, or performance ranking',
        'cgroup_root': str(group_root), 'kernel': os.uname().release})
    probe_kernel(run_directory, group_root)
    environment = os.environ.copy()
    environment['DODB_SOURCE_REVISION'] = source_commit
    outcomes = []
    reports = {}
    for case_name, backend, active_count in [
        ('blink-all-active', 'blink', 17), ('shared-all-active', 'minimal-native-shared', 17),
        ('blink-sparse', 'blink', 3), ('shared-sparse', 'minimal-native-shared', 3),
        ('expected-worker-oom', 'blink', 17),
    ]:
        before = guard()
        group_path = group_root / case_name
        prepare_group(group_path)
        report_path = run_directory / f'{case_name}.json'
        arguments = invocation(binaries / 'tenant_density_workload', backend, active_count,
                               database_directory / case_name, report_path, group_path,
                               4 * 1024**2 if case_name == 'expected-worker-oom' else MAX_BYTES,
                               4 * 1024**2 if case_name == 'expected-worker-oom' else HIGH_BYTES)
        write_json(run_directory / f'{case_name}-command.json', {'command': arguments, 'capacity_before': before,
                    'started_unix_seconds': time.time(), 'source_commit': source_commit})
        exit_code = run_bounded(arguments, environment, run_directory / f'{case_name}.log', 180)
        outcome = {'case': case_name, 'exit_code': exit_code, 'capacity_after': capacity(),
                   'ended_unix_seconds': time.time(), 'report_exists': report_path.exists()}
        try:
            report = json.loads(report_path.read_text())
            if case_name == 'expected-worker-oom':
                assert exit_code != 0 and exit_code != 'controller_timeout'
                assert report['status'] == 'failed_or_inconclusive'
                pressure = report['backend_memory_pressure']
                assert pressure['enabled'] and pressure['cell']['final']['memory.events']['oom_kill'] > 0
                assert all(epoch['final']['cgroup.procs'] == [] for epoch in pressure['epochs'].values())
                outcome['validation'] = {'passed': True, 'scope': 'Expected worker OOM; driver survived and preserved partial diagnostics',
                    'oom_kills': pressure['cell']['final']['memory.events']['oom_kill']}
            else:
                assert exit_code == 0
                outcome['validation'] = validate(report, backend, active_count, source_commit)
                reports[case_name] = report
            outcome['report_sha256'] = digest(report_path)
        except (AssertionError, OSError, ValueError, KeyError, TypeError) as error:
            outcome['validation'] = {'passed': False, 'reason': repr(error)}
        outcomes.append(outcome)
        write_json(run_directory / f'{case_name}-status.json', outcome)
        write_json(run_directory / 'summary.json', outcomes)
        print(json.dumps(outcome), flush=True)
        if not outcome['validation']['passed']:
            return 1
    pairs = {}
    for topology in ['all-active', 'sparse']:
        left = reports[f'blink-{topology}']['workload']
        right = reports[f'shared-{topology}']['workload']
        fields = ['request_count', 'request_fingerprint', 'scheduled_fingerprint', 'scheduled',
                  'admitted', 'completed', 'successful']
        matches = {field: left[field] == right[field] for field in fields}
        for field in ['tenant', *fields, 'operation_counts', 'transaction_counts', 'width_counts']:
            matches[f'per_tenant_{field}'] = [tenant[field] for tenant in left['per_tenant']] == [tenant[field] for tenant in right['per_tenant']]
        pairs[topology] = matches
    write_json(run_directory / 'matched-inputs.json', pairs)
    assert all(all(pair.values()) for pair in pairs.values())
    write_json(run_directory / 'acceptance.json', {'passed': True, 'source_commit': source_commit,
        'correctness_cells': 4, 'successful_requests': sum(outcome['validation'].get('successful_requests', 0) for outcome in outcomes),
        'expected_worker_oom_verified': True, 'anonymous_probe_verified': True,
        'capacity_after': capacity(), 'scope': 'Enforced-budget correctness and instrumentation; no density or performance result'})
    return 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--source-commit', required=True)
    options = parser.parse_args()
    raise SystemExit(run(options.run_name, options.source_commit))
