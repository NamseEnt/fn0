import argparse
import hashlib
import importlib.util
import json
import os
import platform
import shlex
import shutil
import subprocess
import tarfile
import time
from pathlib import Path


module_path = Path(__file__).with_name('run-rocksdb-read-write-admission.py')
module_spec = importlib.util.spec_from_file_location('admission_runner', module_path)
admission_runner = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(admission_runner)
checked_output = admission_runner.attribution.checked_output
CONDITIONS = [
    ('service-before-async', 'before', 'rocksdb', 'async'),
    ('service-after-async', 'after', 'rocksdb', 'async'),
    ('service-after-dedicated', 'after', 'rocksdb', 'dedicated'),
    ('core-after-dedicated', 'after', 'rocksdb-direct', 'dedicated'),
]
ALLOCATORS = ['serialized', 'reserved-ranges']
SCENARIOS = ['rw95', 'rw80', 'rw50', 'rw20', 'mt-mix']


def digest(path):
    checksum = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            checksum.update(chunk)
    return checksum.hexdigest()


def archive_database(run_path):
    started = time.monotonic()
    database = run_path / 'db'
    assert database.is_dir() and not database.is_symlink()
    entries = {}
    for path in sorted(database.rglob('*')):
        assert not path.is_symlink()
        if path.is_file():
            entries[path.relative_to(run_path).as_posix()] = digest(path)
    assert entries
    archive_path = run_path / 'db.tar.gz'
    with tarfile.open(archive_path, 'w:gz', compresslevel=1) as archive:
        archive.add(database, arcname='db')
    observed = {}
    with tarfile.open(archive_path, 'r:gz') as archive:
        for member in archive.getmembers():
            if member.isdir():
                continue
            assert member.isfile() and member.name in entries
            checksum = hashlib.sha256()
            with archive.extractfile(member) as content:
                for chunk in iter(lambda: content.read(1024 * 1024), b''):
                    checksum.update(chunk)
            observed[member.name] = checksum.hexdigest()
    assert observed == entries
    with archive_path.open('rb') as content:
        os.fsync(content.fileno())
    directory_descriptor = os.open(run_path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory_descriptor)
    finally:
        os.close(directory_descriptor)
    archive_record = {
        'passed': True, 'archive_sha256': digest(archive_path),
        'archive_bytes': archive_path.stat().st_size,
        'files': entries, 'elapsed_seconds': time.monotonic() - started,
        'policy': 'validated task-owned DB archived and content-verified before removing its recoverable working directory',
    }
    (run_path / 'database-archive.json').write_text(json.dumps(archive_record, indent=2) + '\n')
    shutil.rmtree(database)


def run_case(args, allocator, condition, scenario, repetition, sequence, smoke=False, metrics_enabled=False):
    label, version, backend, executor = condition
    binary = getattr(args, version + '_binary')
    run_name = f'{sequence:03d}-{scenario}-{allocator}-{label}-rep{repetition}'
    run_path = args.result_root / ('smoke' if smoke else 'runs') / run_name
    run_path.mkdir(parents=True, exist_ok=False)
    assert shutil.disk_usage(run_path).free >= 1024 * 1024 * 1024
    command = [str(binary), '--backend', backend, '--client-executor', executor,
        '--scenario', scenario, '--seed', '20261013', '--clients', '4' if smoke else '16',
        '--rows', '10000', '--value-size', '512', '--call-cap', '1000000',
        '--warmup-ms', '50' if smoke else '1000', '--measurement-ms', '100' if smoke else '5000',
        '--revision-allocator', allocator, '--blocking-admission', 'shared', '--shared-blocking-slots', '8',
        '--research-metrics', str(int(metrics_enabled)), '--read-stage-metrics', str(int(metrics_enabled)), '--detailed-write-metrics', str(int(metrics_enabled)),
        '--data-dir', str(run_path / 'db'), '--report', str(run_path / 'report.json')]
    (run_path / 'command.txt').write_text(shlex.join(command) + '\n')
    before = digest(binary)
    started = time.monotonic()
    with (run_path / 'stdout.txt').open('w') as output:
        completed = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT)
    (run_path / 'exit-status.txt').write_text(str(completed.returncode) + '\n')
    provenance = {
        'condition': label, 'allocator': allocator, 'version': version,
        'source_sha': getattr(args, version + '_sha'), 'backend': backend, 'executor': executor,
        'scenario': scenario, 'repetition': repetition, 'sequence': sequence, 'metrics_enabled': metrics_enabled,
        'elapsed_seconds': time.monotonic() - started,
        'binary_sha256_before': before, 'binary_sha256_after': digest(binary),
    }
    (run_path / 'provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    assert completed.returncode == 0, run_name
    assert provenance['binary_sha256_after'] == before, run_name
    report = json.loads((run_path / 'report.json').read_text())
    admission_runner.verify_report(report, metrics_enabled)
    assert report['backend'] == backend
    assert report['configuration']['client_executor'] == executor
    assert report['configuration']['rocksdb']['revision_allocator'] == allocator
    assert report['configuration']['rocksdb']['blocking_operation_limit'] == 8
    assert report['configuration']['rocksdb']['blocking_admission'] == 'shared-8'
    if not smoke:
        assert report['process_metrics']['sampling_completeness'] == 'complete'
    archive_database(run_path)
    (run_path / 'checked.json').write_text(json.dumps({'passed': True}) + '\n')
    print(json.dumps({'run': run_name, 'smoke': smoke, 'passed': True}), flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--before-binary', required=True, type=Path)
    parser.add_argument('--after-binary', required=True, type=Path)
    parser.add_argument('--before-sha', required=True)
    parser.add_argument('--after-sha', required=True)
    parser.add_argument('--result-root', required=True, type=Path)
    args = parser.parse_args()
    assert platform.machine() == 'aarch64'
    assert checked_output(['hostname']) == 'instance-20260923-1013'
    assert checked_output(['git', 'rev-parse', 'HEAD']) == args.after_sha
    assert not checked_output(['git', 'status', '--porcelain'])
    assert checked_output(['findmnt', '-n', '-o', 'FSTYPE', '-T', str(args.result_root.parent)]) == 'zfs'
    args.result_root.mkdir(exist_ok=False)
    plan = {
        'before_sha': args.before_sha, 'after_sha': args.after_sha,
        'before_binary_sha256': digest(args.before_binary), 'after_binary_sha256': digest(args.after_binary),
        'conditions': CONDITIONS, 'allocators': ALLOCATORS, 'scenarios': SCENARIOS,
        'repetitions': 3, 'performance_runs': 120, 'smoke_runs': 12,
        'comparisons': {'optimization': ['service-before-async', 'service-after-async'],
            'engine_core': ['service-after-dedicated', 'core-after-dedicated']},
        'latency_recording': 'all recorded completions', 'default_promotion': False,
        'database_archive_policy': 'archive, verify every file content, fsync, then remove only the archived task-owned working DB',
    }
    (args.result_root / 'plan.json').write_text(json.dumps(plan, indent=2) + '\n')
    commands = [['date', '-u'], ['hostname'], ['uname', '-a'], ['lscpu'], ['free', '-h'],
        ['df', '-h', '/bench/zfs/db'], ['findmnt', '-T', '/bench/zfs/db'],
        ['rustup', 'run', '1.97.1', 'rustc', '-vV'], ['git', 'status', '--short', '--branch'],
        ['git', '-C', '/bench/zfs/db/product-build/rocksdb-source', 'rev-parse', 'HEAD'],
        ['sha256sum', '/bench/zfs/db/product-build/rocksdb-source/librocksdb.a']]
    with (args.result_root / 'host-and-source.txt').open('w') as output:
        for command in commands:
            output.write(shlex.join(command) + '\n')
            output.flush()
            subprocess.run(command, stdout=output, stderr=subprocess.STDOUT, check=True)
    smoke_sequence = 0
    for allocator in ALLOCATORS:
        for condition in CONDITIONS:
            smoke_sequence += 1
            run_case(args, allocator, condition, 'smoke', 0, smoke_sequence, True)
    for allocator in ALLOCATORS:
        for condition in CONDITIONS[1:3]:
            smoke_sequence += 1
            run_case(args, allocator, condition, 'smoke', 0, smoke_sequence, True, True)
    assert smoke_sequence == plan['smoke_runs']
    sequence = 0
    for repetition in range(1, 4):
        for scenario_index, scenario in enumerate(SCENARIOS):
            allocator_order = ALLOCATORS if (scenario_index + repetition) % 2 else list(reversed(ALLOCATORS))
            for allocator_index, allocator in enumerate(allocator_order):
                offset = (scenario_index + repetition + allocator_index - 1) % len(CONDITIONS)
                order = CONDITIONS[offset:] + CONDITIONS[:offset]
                if repetition == 2:
                    order = list(reversed(order))
                for condition in order:
                    sequence += 1
                    run_case(args, allocator, condition, scenario, repetition, sequence)
    assert sequence == plan['performance_runs']
    assert not checked_output(['git', 'status', '--porcelain'])
    (args.result_root / 'complete.json').write_text(json.dumps({'passed': True, 'runs': sequence}) + '\n')


if __name__ == '__main__':
    main()
