import importlib.util
import json
from pathlib import Path
import sys


SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
RATE = 50
ROWS = 2048
VALUE_BYTES = 512
QUEUE_BOUND = 8
MAX_IN_FLIGHT = 8
REQUEST_TIMEOUT_MS = 5000
DRAIN_TIMEOUT_MS = 15000
PRESSURE_INTERVAL_MS = 5000
MEMORY_MAX_BYTES = 256 * 1024**2
MEMORY_HIGH_BYTES = 192 * 1024**2
NATIVE_CACHE_BYTES = 64 * 1024**2
NATIVE_WBM_BYTES = 32 * 1024**2


def phase_offset_ns(tenant_number, rate, active_tenants):
    assert 1 <= tenant_number <= active_tenants
    assert rate > 0 and active_tenants > 0
    return (tenant_number - 1) * 1_000_000_000 // (rate * active_tenants)


def expected_cell(case):
    if case.startswith('tiny-'):
        mode = case.removeprefix('tiny-').removesuffix('-blink').removesuffix('-shared')
        return {'mode': mode, 'registered': 4, 'active': 3, 'idle': 1,
                'duration_ms': 10000, 'rows': ROWS}
    fields = case.split('-')
    if fields[-1] in {'blink', 'shared'}:
        fields = fields[:-1]
    assert len(fields) == 4 and fields[:2] == ['control', 'n28']
    assert fields[2] in {'r1', 'r2', 'r3'}
    return {'mode': fields[3], 'registered': 28, 'active': 28, 'idle': 0,
            'duration_ms': 60000, 'rows': ROWS}


def validate_report(report, case, backend, source_commit=SOURCE_COMMIT):
    expected = expected_cell(case)
    configuration = report['effective_configuration']
    workload = report['workload']
    assert report['compiled_source_revision'] == report['requested_source_revision'] == source_commit
    assert configuration['backend'] == backend
    assert configuration['registered_tenants'] == expected['registered']
    assert configuration['active_tenants'] == expected['active']
    assert configuration['opened_idle_tenants'] == expected['idle']
    assert configuration['initial_rows_per_tenant'] == expected['rows']
    assert configuration['duration_ms'] == expected['duration_ms']
    assert configuration['tenant_arrival_phase_selected'] == expected['mode']
    assert configuration['tenant_arrival_phase_applies'] is True
    assert configuration['tenant_arrival_phase'] == expected['mode']
    assert configuration['client_count'] == 1
    assert configuration['seed'] == 20261027
    assert configuration['per_tenant_rate'] == RATE
    assert configuration['queue_bound'] == QUEUE_BOUND
    assert configuration['max_in_flight'] == MAX_IN_FLIGHT
    assert configuration['request_timeout_ms'] == REQUEST_TIMEOUT_MS
    assert configuration['drain_timeout_ms'] == DRAIN_TIMEOUT_MS
    budget = configuration['backend_memory_budget']
    assert budget['max_bytes'] == MEMORY_MAX_BYTES
    assert budget['high_bytes'] == MEMORY_HIGH_BYTES
    assert budget['sample_interval_ms'] == PRESSURE_INTERVAL_MS
    assert workload['cell_valid'] and workload['valid_for_oracle']
    assert report['status'] == 'completed'
    assert report['post_measurement_full_population_oracle']['passed']
    assert report['reopened_full_population_oracle']['passed']
    assert not report['backend_memory_pressure']['samples_truncated']
    assert all(workload['accounting'].values())
    expected_per_tenant = expected['duration_ms'] * RATE // 1000
    assert workload['scheduled'] == expected_per_tenant * expected['active']
    assert workload['admitted'] == workload['completed'] == workload['successful']
    assert workload['rejected_conflicts'] == workload['other_error'] == 0
    assert workload['timed_out'] == workload['queued_at_end'] == workload['in_flight_at_end'] == 0
    expected_input = workload['deterministic_fingerprints']['input_fingerprint']
    expected_schedule = workload['deterministic_fingerprints']['schedule_fingerprint']
    assert expected_input and expected_schedule
    assert len(workload['per_tenant']) == expected['active']
    for tenant in workload['per_tenant']:
        tenant_number = tenant['tenant']
        expected_offset = None
        if tenant_number <= expected['active']:
            expected_offset = phase_offset_ns(tenant_number, RATE, expected['active']) if expected['mode'] == 'staggered' else 0
        assert tenant['arrival_phase_offset_nanos'] == expected_offset
        if tenant_number <= expected['active']:
            assert tenant['scheduled'] == expected_per_tenant
            assert tenant['admitted'] == tenant['completed'] == tenant['successful']
    for oracle_name in ['post_measurement_full_population_oracle', 'reopened_full_population_oracle']:
        oracle = report[oracle_name]
        assert oracle['verified_seed_rows'] == expected['registered'] * expected['rows']
    return {'case': case, 'backend': backend, 'mode': expected['mode'],
            'input_fingerprint': expected_input, 'schedule_fingerprint': expected_schedule,
            'request_count': workload['request_count'], 'request_fingerprint': workload['request_fingerprint'],
            'scheduled_fingerprint': workload['scheduled_fingerprint'],
            'scheduled': workload['scheduled'], 'per_tenant': workload['per_tenant']}


def compare_phases(synchronized, staggered):
    sync_workload = synchronized['workload']
    staggered_workload = staggered['workload']
    sync_configuration = synchronized['effective_configuration']
    staggered_configuration = staggered['effective_configuration']
    assert sync_configuration['backend'] == staggered_configuration['backend']
    assert sync_configuration['registered_tenants'] == staggered_configuration['registered_tenants']
    assert sync_configuration['active_tenants'] == staggered_configuration['active_tenants']
    assert sync_configuration['duration_ms'] == staggered_configuration['duration_ms']
    assert sync_workload['deterministic_fingerprints']['input_fingerprint'] == staggered_workload['deterministic_fingerprints']['input_fingerprint']
    assert sync_workload['deterministic_fingerprints']['schedule_fingerprint'] != staggered_workload['deterministic_fingerprints']['schedule_fingerprint']
    active_tenants = sync_configuration['active_tenants']
    request_matches = {
        'request_count': sync_workload['request_count'] == staggered_workload['request_count'],
        'request_fingerprint': sync_workload['request_fingerprint'] == staggered_workload['request_fingerprint'],
    }
    sync_rows = sync_workload['per_tenant']
    staggered_rows = staggered_workload['per_tenant']
    assert len(sync_rows) == len(staggered_rows) == active_tenants
    request_fields = ['tenant', 'request_count', 'request_fingerprint', 'operation_counts', 'transaction_counts', 'width_counts']
    sync_requests = [[row[field] for field in request_fields] for row in sync_rows]
    staggered_requests = [[row[field] for field in request_fields] for row in staggered_rows]
    request_matches['per_tenant_requests'] = sync_requests == staggered_requests
    rate = sync_configuration['per_tenant_rate']
    expected_offsets = [phase_offset_ns(tenant_number, rate, active_tenants)
                        for tenant_number in range(1, active_tenants + 1)]
    sync_mode = sync_configuration['tenant_arrival_phase']
    staggered_mode = staggered_configuration['tenant_arrival_phase']
    assert {sync_mode, staggered_mode} == {'synchronized', 'staggered'}
    sync_expected_offsets = [0] * active_tenants if sync_mode == 'synchronized' else expected_offsets
    staggered_expected_offsets = [0] * active_tenants if staggered_mode == 'synchronized' else expected_offsets
    assert [row['arrival_phase_offset_nanos'] for row in sync_rows] == sync_expected_offsets
    assert [row['arrival_phase_offset_nanos'] for row in staggered_rows] == staggered_expected_offsets
    dropped = sync_workload['dropped'] > 0 or staggered_workload['dropped'] > 0
    if not dropped:
        assert all(request_matches.values()), request_matches
    realized_fields = ['scheduled_fingerprint', 'scheduled', 'admitted', 'completed', 'successful']
    realized_matches = {field: sync_workload[field] == staggered_workload[field] for field in realized_fields}
    return {'recipe_matches': True, 'request_matches': request_matches,
            'schedule_identity_differs': True, 'expected_staggered_offsets_ns': expected_offsets,
            'realized_matches': realized_matches, 'realized_difference_observed': not all(realized_matches.values()),
            'either_mode_dropped_requests': bool(dropped)}


def slo_result(reduced):
    return bool(reduced['slo_pass'])


PHASE_PAIRS = [
    ('tiny-synchronized', 'tiny-staggered'),
    ('control-n28-r1-synchronized', 'control-n28-r1-staggered'),
    ('control-n28-r2-staggered', 'control-n28-r2-synchronized'),
    ('control-n28-r3-synchronized', 'control-n28-r3-staggered'),
]


def expected_report_cases():
    ordered_pairs = [
        'tiny-synchronized', 'tiny-staggered',
        'control-n28-r1-synchronized', 'control-n28-r1-staggered',
        'control-n28-r2-staggered', 'control-n28-r2-synchronized',
        'control-n28-r3-synchronized', 'control-n28-r3-staggered',
    ]
    report_cases = []
    for pair_number, pair_case in enumerate(ordered_pairs, start=1):
        backend_suffixes = ['-blink', '-shared']
        if pair_number % 2 == 0:
            backend_suffixes.reverse()
        report_cases.extend(pair_case + suffix for suffix in backend_suffixes)
    return report_cases


def load_capacity_review():
    remote_helper = Path('/home/opc/tenant-density-harness-20261027/capacity_review.py')
    local_helper = Path(__file__).resolve().parents[1] / 'capacity' / 'capacity_review.py'
    helper_path = remote_helper if remote_helper.is_file() else local_helper
    sys.path.insert(0, str(helper_path.parent))
    specification = importlib.util.spec_from_file_location('arrival_capacity_review', helper_path)
    module = importlib.util.module_from_spec(specification)
    sys.modules['arrival_capacity_review'] = module
    specification.loader.exec_module(module)
    return module


def review_directory(run_directory, capacity_reviewer=None):
    directory = Path(run_directory)
    capacity_reviewer = capacity_reviewer or load_capacity_review()
    provenance = json.loads((directory / 'provenance.json').read_text())
    source_commit = provenance['source_commit']
    assert source_commit == SOURCE_COMMIT
    cases = {}
    for status_path in sorted(directory.glob('*-status.json')):
        status = json.loads(status_path.read_text())
        report_path = directory / (status['case'] + '.json')
        if not report_path.is_file():
            cases[status['case']] = {'status': 'missing_report',
                                     'available_unvalidated_counters': status.get('available_unvalidated_counters')}
            continue
        assert digest_path(report_path) == status['report_sha256']
        report = json.loads(report_path.read_text())
        try:
            reduced = capacity_reviewer.reduce_report(report, source_commit)
            arrival = validate_report(report, status['case'], status['backend'], source_commit)
            cases[status['case']] = {'reduced': reduced, 'arrival': arrival,
                                     'status': status.get('status', 'completed')}
        except Exception as error:
            cases[status['case']] = {
                'status': 'review_error',
                'review_error': {'type': type(error).__name__, 'message': str(error)},
                'available_unvalidated_counters': report.get('workload'),
            }
    same_mode_pairs = {}
    for pair_path in sorted(directory.glob('*-pair.json')):
        pair = json.loads(pair_path.read_text())
        report_paths = [directory / (case + '.json') for case in pair['cases']]
        if not all(path.is_file() for path in report_paths):
            same_mode_pairs[pair_path.stem] = {'realized_comparison_available': False,
                                               'reason': 'at least one report is unavailable'}
            continue
        reports = [json.loads(path.read_text()) for path in report_paths]
        try:
            same_mode_pairs[pair_path.stem] = capacity_reviewer.compare_pair(*reports)
        except Exception as error:
            same_mode_pairs[pair_path.stem] = {'review_error': {
                'type': type(error).__name__, 'message': str(error)}}
    phase_pairs = {}
    for synchronized_case, staggered_case in PHASE_PAIRS:
        for backend, suffix in [('blink', '-blink'), ('minimal-native-shared', '-shared')]:
            synchronized_path = directory / (synchronized_case + suffix + '.json')
            staggered_path = directory / (staggered_case + suffix + '.json')
            key = synchronized_case + '--' + staggered_case + '-' + backend
            if not synchronized_path.is_file() or not staggered_path.is_file():
                continue
            try:
                phase_pairs[key] = compare_phases(json.loads(synchronized_path.read_text()),
                                                  json.loads(staggered_path.read_text()))
            except Exception as error:
                phase_pairs[key] = {'review_error': {'type': type(error).__name__,
                                                     'message': str(error)}}
    summary_path = directory / 'summary.json'
    summary_cases = []
    if summary_path.is_file():
        summary_cases = [row['case'] for row in json.loads(summary_path.read_text())]
    tiny_slo_cases = [
        'tiny-synchronized-blink', 'tiny-synchronized-shared',
        'tiny-staggered-blink', 'tiny-staggered-shared',
    ]
    tiny_slo_gate = all(cases.get(case, {}).get('reduced', {}).get('slo_pass') is True
                        for case in tiny_slo_cases)
    output = {'source_commit': source_commit, 'cases': cases,
              'same_mode_pairs': same_mode_pairs, 'phase_pairs': phase_pairs,
              'summary_cases': summary_cases, 'tiny_slo_gate': tiny_slo_gate,
              'complete': len(cases) == 16 and set(cases) == set(expected_report_cases())
              and summary_cases == expected_report_cases()
              and len(same_mode_pairs) == 8 and len(phase_pairs) == 8
              and tiny_slo_gate
              and all('reduced' in case for case in cases.values())
              and all('review_error' not in pair for pair in same_mode_pairs.values())
              and all('review_error' not in pair for pair in phase_pairs.values())}
    (directory / 'local-review.json').write_text(json.dumps(output, indent=2) + '\n')
    return output


def digest_path(file_path):
    import hashlib
    return hashlib.sha256(Path(file_path).read_bytes()).hexdigest()


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='Review immutable E2 arrival control reports')
    parser.add_argument('run_directory', type=Path)
    arguments = parser.parse_args()
    result = review_directory(arguments.run_directory)
    print(json.dumps({'source_commit': result['source_commit'], 'cases': len(result['cases']),
                      'same_mode_pairs': len(result['same_mode_pairs']),
                      'phase_pairs': len(result['phase_pairs']), 'complete': result['complete']},
                     sort_keys=True))
