import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
DATABASE_ROOT = Path('/bench/zfs/db/tenant-density-harness-20261027')
CASES = [
    ('blink-all-active', 'blink', 'fixed-total'),
    ('native-all-active', 'minimal-native', 'fixed-total'),
    ('native-sparse', 'minimal-native', 'fixed-per-tenant'),
    ('blink-sparse', 'blink', 'fixed-per-tenant'),
    ('shared-all-active', 'minimal-native-shared', 'fixed-total'),
    ('shared-sparse', 'minimal-native-shared', 'fixed-per-tenant'),
]


SHARED_ARGUMENTS = [
    '--rocksdb-shared-cache-bytes', str(8 * 1024**2),
    '--rocksdb-shared-wbm-bytes', str(4 * 1024**2),
    '--rocksdb-write-buffer-bytes', str(64 * 1024),
    '--rocksdb-flush-threads', '1', '--rocksdb-compaction-threads', '1',
    '--rocksdb-max-open-files', '64',
]


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def capacity():
    filesystem = os.statvfs('/bench/zfs/db')
    memory = {}
    for line in Path('/proc/meminfo').read_text().splitlines():
        name, value = line.split(':', 1)
        if name in ['MemAvailable', 'SwapFree']:
            memory[name] = int(value.split()[0]) * 1024
    return {'zfs_free_bytes': filesystem.f_bavail * filesystem.f_frsize, 'memory_bytes': memory}


def command(arguments):
    return subprocess.check_output(arguments, cwd=SOURCE, text=True).strip()


def validate_report(path, source_commit, backend, topology):
    report = json.loads(path.read_text())
    failures = []

    def require(condition, description):
        if not condition:
            failures.append(description)

    require(report.get('status') == 'completed', 'cell status is not completed')
    require(report.get('compiled_source_revision') == source_commit, 'driver source revision mismatch')
    configuration = report.get('effective_configuration', {})
    require(configuration.get('backend') == backend, 'backend mismatch')
    require(configuration.get('topology') == topology, 'topology mismatch')
    require(configuration.get('registered_tenants') == 17, 'registered population mismatch')
    active_count = 17 if topology == 'fixed-total' else 3
    require(configuration.get('active_tenants') == active_count, 'active population mismatch')
    require(configuration.get('measurement_never_opened_tenants') == (0 if active_count == 17 else 12), 'measurement epoch population mismatch')
    workload = report.get('workload', {})
    require(workload.get('cell_valid') is True, 'workload is invalid')
    require(workload.get('valid_for_oracle') is True, 'workload has uncertain outcomes')
    require(workload.get('completed', 0) > 0, 'no completed requests')
    require(workload.get('scheduled') == (1700 if topology == 'fixed-total' else 90), 'offered request count mismatch')
    require(workload.get('request_count') == workload.get('completed'), 'executed fingerprint count mismatch')
    for field in ['request_fingerprint', 'scheduled_fingerprint']:
        fingerprint = workload.get(field)
        require(isinstance(fingerprint, str) and fingerprint.startswith('fnv1a64:') and len(fingerprint) == 24, f'missing {field}')
    observed_in_flight = workload.get('observed_max_in_flight', 0)
    require(1 <= observed_in_flight <= 8, 'observed concurrency outside configured bounds')
    if topology == 'fixed-total':
        require(observed_in_flight > 1, 'fixed-total clients did not overlap')
    else:
        require(report.get('offered_load_elapsed_ms', 0) >= 3000, 'offered window ended before configured duration')
    accounting = workload.get('accounting', {})
    require(bool(accounting) and all(value is True for value in accounting.values()), 'workload accounting failed')
    for field in ['rejected_conflicts', 'other_error', 'timed_out', 'dropped', 'queued_at_end', 'in_flight_at_end']:
        require(workload.get(field) == 0, f'nonzero or absent {field}')
    require(workload.get('scheduled') == workload.get('admitted', 0) + workload.get('dropped', 0), 'arrival accounting mismatch')
    tenant_reports = workload.get('per_tenant', [])
    require(len(tenant_reports) == active_count, 'per-tenant coverage mismatch')
    require({tenant.get('tenant') for tenant in tenant_reports} == set(range(1, active_count + 1)), 'active tenant identities mismatch')
    require(all(tenant.get('completed', 0) > 0 for tenant in tenant_reports), 'an active tenant has no completed requests')
    for field in ['scheduled', 'admitted', 'completed', 'successful', 'rejected_conflicts', 'other_error', 'timed_out']:
        require(sum(tenant.get(field, 0) for tenant in tenant_reports) == workload.get(field), f'per-tenant {field} total mismatch')
    histogram = workload.get('latency_histogram', {})
    require(histogram.get('observation_count') == workload.get('completed'), 'latency observation count mismatch')
    require(sum(histogram.get('bucket_counts', [])) == workload.get('completed'), 'latency histogram count mismatch')
    for container in [workload, *tenant_reports]:
        operation_histograms = container.get('operation_latency_histograms', {})
        require(set(operation_histograms) == {'Get', 'Query', 'Write'}, 'operation latency groups absent')
        require(sum(histogram.get('observation_count', 0) for histogram in operation_histograms.values()) == container.get('completed'), 'operation latency total mismatch')
        for histogram in operation_histograms.values():
            require(sum(histogram.get('bucket_counts', [])) == histogram.get('observation_count'), 'operation latency bucket count mismatch')
    for oracle_name in ['post_measurement_full_population_oracle', 'reopened_full_population_oracle']:
        require(report.get(oracle_name, {}).get('passed') is True, f'{oracle_name} failed')
        require(report.get(oracle_name, {}).get('verified_seed_rows') == 17 * 256, f'{oracle_name} population mismatch')
    process_summary = report.get('measurement_worker_process_sampling', {})
    require(process_summary.get('same_backend_identity_at_offer_boundaries') is True, 'measurement backend identity mismatch')
    diagnostic = report.get('memory_diagnostic', {})
    health_reports = diagnostic.get('engine_health_reports', [])
    require(len(health_reports) == 4, 'expected setup, measurement, oracle, and reopen health reports')
    for health in health_reports:
        require(health.get('shutdown_complete') is True, 'worker shutdown incomplete')
        require(health.get('compiled_source_revision') == source_commit, 'worker source revision mismatch')
        require(health.get('backend') == backend, 'worker backend mismatch')
    if backend == 'minimal-native-shared' and len(health_reports) == 4:
        expected_config = {'cache_bytes': 8 * 1024**2, 'wbm_bytes': 4 * 1024**2,
            'write_buffer_bytes': 64 * 1024, 'flush_threads': 1,
            'compaction_threads': 1, 'max_open_files': 64}
        require(configuration.get('rocksdb_shared_resources', {}).get('configured') == expected_config, 'driver resource configuration mismatch')
        for epoch_index, health in enumerate(health_reports):
            resource_config = health.get('effective_backend_configuration', {}).get('shared_resources', {})
            require(resource_config.get('configured') == expected_config, 'worker resource configuration mismatch')
            require(resource_config.get('process_memory_limit_enforced') is False, 'misleading process memory enforcement')
            require(resource_config.get('memtables_charged_to_cache') is True, 'memtables not charged to cache')
            require(resource_config.get('wbm_allow_stall') is False, 'rejected WBM stall policy enabled')
            require(resource_config.get('max_write_buffer_size_to_maintain_per_tenant') == 1, 'history target mismatch')
            snapshots = health.get('shared_resource_snapshots', {})
            snapshot_names = ['before_shutdown']
            if epoch_index == 1:
                snapshot_names += ['measurement_start', 'measurement_end']
            for snapshot_name in snapshot_names:
                snapshot = snapshots.get(snapshot_name, {})
                expected_opened = active_count + (0 if active_count == 17 else 2) if epoch_index == 1 else 17
                require(snapshot.get('opened_tenants') == expected_opened, 'native opened tenant count mismatch')
                require(snapshot.get('tenants_verified_using_shared_resources') == expected_opened, 'shared native pointer verification mismatch')
                require(snapshot.get('all_opened_tenants_use_shared_resources') is True, 'tenant resource sharing failed')
                usage = snapshot.get('usage', {})
                require(usage.get('cache_capacity_bytes') == expected_config['cache_bytes'], 'native cache capacity mismatch')
                require(usage.get('wbm_limit_bytes') == expected_config['wbm_bytes'], 'native WBM limit mismatch')
                require(usage.get('flush_threads') == 1 and usage.get('compaction_threads') == 1, 'native background pool mismatch')
                for field in ['cache_usage_bytes', 'cache_pinned_usage_bytes', 'wbm_usage_bytes', 'wbm_mutable_bytes', 'wbm_cache_reservation_bytes']:
                    require(isinstance(usage.get(field), int) and usage[field] >= 0, f'missing native usage {field}')
    if backend == 'blink' and len(health_reports) == 4:
        measured = health_reports[1]
        effective = measured.get('effective_backend_configuration', {})
        require(effective.get('cache_capacity_is_blink_memory_limit') is False, 'misleading B-link memory-limit configuration')
        layout = effective.get('wal_layout', {})
        require(layout.get('layout') == 'segmented-bases-coalesced', 'WAL layout mismatch')
        require(layout.get('execution') == 'background', 'configured checkpoint execution mismatch')
        require(layout.get('segment_bytes') == 8192, 'WAL segment size mismatch')
        checkpoint = measured.get('blink_checkpoint_metrics', {})
        require(checkpoint.get('execution') == 'background', 'checkpoint execution mismatch')
        require(checkpoint.get('checkpoints_failed') == 0, 'checkpoint failure')
    return {'passed': not failures, 'failures': failures}


def run_smoke(run_name, source_commit, selected_cases):
    assert run_name and all(character.isalnum() or character in '-_' for character in run_name)
    assert command(['git', 'rev-parse', 'HEAD']) == source_commit
    assert not command(['git', 'status', '--porcelain'])
    assert capacity()['zfs_free_bytes'] >= 10 * 1024**3
    run_directory = ROOT / 'smokes' / run_name
    run_directory.mkdir(parents=True, exist_ok=False)
    database_directory = DATABASE_ROOT / run_name
    database_directory.mkdir(parents=True, exist_ok=False)
    assert os.stat(database_directory).st_dev == os.stat('/bench/zfs/db').st_dev
    binaries = run_directory / 'bin'
    binaries.mkdir()
    identities = {}
    for name in ['tenant_density_workload', 'memory_diagnostic_worker']:
        original = ROOT / 'target/release' / name
        destination = binaries / name
        shutil.copy2(original, destination)
        identities[name] = {'sha256': digest(destination), 'bytes': destination.stat().st_size}
    (run_directory / 'binary-identities.json').write_text(json.dumps(identities, indent=2) + '\n')
    environment = os.environ.copy()
    environment['DODB_SOURCE_REVISION'] = source_commit
    results = []
    for case_name, backend, topology in CASES:
        if case_name not in selected_cases:
            continue
        preflight = capacity()
        if preflight['zfs_free_bytes'] < 10 * 1024**3 or preflight['memory_bytes']['MemAvailable'] < 1500 * 1024**2:
            raise RuntimeError(f'capacity guard prevents {case_name}: {preflight}')
        report = run_directory / f'{case_name}.json'
        invocation = [str(binaries / 'tenant_density_workload'),
            '--backend', backend, '--data-dir', str(database_directory / case_name),
            '--report', str(report), '--seed', '20261027',
            '--registered-tenants', '17', '--active-tenants', '17' if topology == 'fixed-total' else '3',
            '--opened-idle-tenants', '0' if topology == 'fixed-total' else '2',
            '--client-count', '4' if topology == 'fixed-total' else '1',
            '--initial-rows', '256', '--value-bytes', '512', '--topology', topology,
            '--queue-bound', '8', '--max-in-flight', '8', '--request-timeout-ms', '5000',
            '--drain-timeout-ms', '15000', '--profile', 'mixed' if topology == 'fixed-total' else 'write-heavy',
            '--width', 'mixed']
        if topology == 'fixed-total':
            invocation += ['--operation-count', '1700']
        else:
            invocation += ['--duration-ms', '3000', '--per-tenant-rate', '10']
        if backend == 'minimal-native-shared':
            invocation += SHARED_ARGUMENTS
        if backend == 'blink':
            invocation += ['--blink-wal-layout', 'segmented-bases-coalesced',
                '--blink-checkpoint-execution', 'background', '--blink-wal-segment-bytes', '8192',
                '--blink-checkpoint-wal-bytes', '16384']
        metadata = {'source_commit': source_commit, 'controller_sha256': digest(Path(__file__).resolve()), 'command': invocation, 'preflight': preflight,
            'started_unix_seconds': time.time(), 'scope': 'Correctness/accounting smoke only; no throughput, SLO, or density ranking.'}
        (run_directory / f'{case_name}-command.json').write_text(json.dumps(metadata, indent=2) + '\n')
        with (run_directory / f'{case_name}.log').open('w') as output:
            try:
                result = subprocess.run(invocation, cwd=SOURCE, env=environment, stdout=output, stderr=subprocess.STDOUT, timeout=180)
                exit_code = result.returncode
            except subprocess.TimeoutExpired:
                exit_code = 'controller_timeout'
        try:
            validation = validate_report(report, source_commit, backend, topology) if report.exists() else {'passed': False, 'failures': ['report absent']}
        except (ValueError, TypeError, KeyError) as error:
            validation = {'passed': False, 'failures': [f'invalid report: {error}']}
        status = {'case': case_name, 'exit_code': exit_code, 'validation': validation, 'ended_unix_seconds': time.time(), 'capacity_after': capacity(),
            'report_exists': report.exists(), 'report_sha256': digest(report) if report.exists() else None}
        (run_directory / f'{case_name}-status.json').write_text(json.dumps(status, indent=2) + '\n')
        results.append(status)
        (run_directory / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
        print(json.dumps(status), flush=True)
        if exit_code != 0 or not validation['passed']:
            return 1
    return 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--cases', nargs='+', choices=[case[0] for case in CASES], default=[case[0] for case in CASES if case[1] != 'minimal-native-shared'])
    options = parser.parse_args()
    return run_smoke(options.run_name, options.source_commit, options.cases)


if __name__ == '__main__':
    raise SystemExit(main())
