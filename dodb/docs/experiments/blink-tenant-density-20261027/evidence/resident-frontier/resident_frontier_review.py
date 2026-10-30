import argparse
import hashlib
import json
from pathlib import Path
import re


SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
DESIGN_PLAN_COMMIT = '6f57299f3af21c19444ebb203e40d636abcd0a04'
NOFILE_SOFT_TARGET = 65536
RATE = 50
ROWS = 2048
SEED = 20261027
SCREEN_DURATION_MS = 30000
CONFIRMATION_DURATION_MS = 60000
BACKENDS = ['blink', 'minimal-native-shared']
RUN_NAME = 'resident-frontier-cb0e04482-r2'
DATABASE_ROOT = Path('/bench/zfs/db/tenant-density-harness-20261027') / RUN_NAME
CGROUP_ROOT = Path('/sys/fs/cgroup') / f'dodb-density-{RUN_NAME}'
EPOCHS = ['setup', 'measurement', 'oracle', 'reopen-oracle']
SCREEN_COUNTS_WITH_BASELINE = [32, 48, 64, 80, 96]
SCREEN_COUNTS_WITHOUT_BASELINE = [16, 24]
LIMITS = {
    'blink': {
        'blink_layout': {'layout': 'segmented-bases-coalesced', 'execution': 'background',
                         'segment_bytes': 256 * 1024},
        'checkpoint_policy': 'WalBytesThreshold { threshold_bytes: 1048576 }',
    },
    'minimal-native-shared': {
        'rocksdb_shared_resources': {'cache_bytes': 64 * 1024**2,
                                     'wbm_bytes': 32 * 1024**2,
                                     'write_buffer_bytes': 1024**2,
                                     'flush_threads': 1,
                                     'compaction_threads': 1,
                                     'max_open_files': 64},
    },
}


def sha256_file(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def backend_from_suffix(suffix):
    if suffix == 'blink':
        return 'blink'
    if suffix == 'shared':
        return 'minimal-native-shared'
    raise AssertionError(f'unknown backend suffix: {suffix}')


def expected_cell(case):
    matched = re.fullmatch(r'(screen|confirm|fallback)-n([0-9]+)(?:-r([1-3]))?-(blink|shared)', case)
    if not matched:
        raise AssertionError(f'cell name is outside the resident frontier matrix: {case}')
    kind = matched.group(1)
    active_tenants = int(matched.group(2))
    if active_tenants <= 0:
        raise AssertionError(f'cell active population is invalid: {case}')
    if kind == 'screen':
        if matched.group(3) is not None:
            raise AssertionError(f'screen cell has a repetition suffix: {case}')
        duration_ms = SCREEN_DURATION_MS
    else:
        if matched.group(3) is None:
            raise AssertionError(f'confirmation cell is missing its repetition number: {case}')
        duration_ms = CONFIRMATION_DURATION_MS
    return {'kind': kind, 'active_tenants': active_tenants,
            'duration_ms': duration_ms, 'backend': backend_from_suffix(matched.group(4))}


def validate_report(report, case, backend, source_commit=SOURCE_COMMIT):
    expected = expected_cell(case)
    configuration = report['effective_configuration']
    workload = report['workload']
    if report['compiled_source_revision'] != source_commit or report['requested_source_revision'] != source_commit:
        raise AssertionError(f'cell source revision differs: {case}')
    expected_configuration = {
        'backend': backend,
        'registered_tenants': expected['active_tenants'],
        'active_tenants': expected['active_tenants'],
        'opened_idle_tenants': 0,
        'initial_rows_per_tenant': ROWS,
        'duration_ms': expected['duration_ms'],
        'tenant_arrival_phase_selected': 'staggered',
        'tenant_arrival_phase_applies': True,
        'tenant_arrival_phase': 'staggered',
        'client_count': 1,
        'seed': SEED,
        'per_tenant_rate': RATE,
        'queue_bound': 8,
        'max_in_flight': 8,
        'request_timeout_ms': 5000,
        'drain_timeout_ms': 15000,
        'profile': 'mixed',
        'width': 'mixed 80/15/5 for 1/4/16',
        'value_bytes': 512,
        'topology': 'fixed-per-tenant',
    }
    for field, expected_value in expected_configuration.items():
        if configuration.get(field) != expected_value:
            raise AssertionError(f'cell configuration {field} differs: {case}')
    memory_budget = configuration.get('backend_memory_budget', {})
    if (memory_budget.get('max_bytes') != 256 * 1024**2
            or memory_budget.get('high_bytes') != 192 * 1024**2
            or memory_budget.get('sample_interval_ms') != 5000):
        raise AssertionError(f'backend memory budget differs: {case}')
    if expected['backend'] != backend:
        raise AssertionError(f'cell backend differs from its name: {case}')
    if backend == 'minimal-native-shared':
        configured = configuration.get('rocksdb_shared_resources', {}).get('configured', {})
        if configured != LIMITS[backend]['rocksdb_shared_resources']:
            raise AssertionError(f'shared RocksDB resource configuration differs: {case}')
    else:
        layout = configuration.get('blink_layout', {})
        if any(layout.get(field) != value for field, value in LIMITS[backend]['blink_layout'].items()):
            raise AssertionError(f'B-link layout differs: {case}')
        if configuration.get('checkpoint_policy') != LIMITS[backend]['checkpoint_policy']:
            raise AssertionError(f'B-link checkpoint policy differs: {case}')
    if report.get('status') != 'completed' or not workload.get('cell_valid') or not workload.get('valid_for_oracle'):
        raise AssertionError(f'workload is not complete and oracle-valid: {case}')
    if not all(workload.get('accounting', {}).values()):
        raise AssertionError(f'workload accounting is invalid: {case}')
    if not report.get('post_measurement_full_population_oracle', {}).get('passed'):
        raise AssertionError(f'post-measurement oracle failed: {case}')
    if not report.get('reopened_full_population_oracle', {}).get('passed'):
        raise AssertionError(f'reopen oracle failed: {case}')
    if report.get('backend_memory_pressure', {}).get('samples_truncated'):
        raise AssertionError(f'backend pressure observations are truncated: {case}')
    backend_samples = report['backend_memory_pressure'].get('samples', [])
    if (len(backend_samples) > 4096
            or report['backend_memory_pressure'].get('sample_limit') != 4096):
        raise AssertionError(f'backend pressure sample count exceeds its observation bound: {case}')
    for health_report in report['memory_diagnostic']['engine_health_reports']:
        periodic_pressure = health_report['periodic_pressure']
        if periodic_pressure.get('samples_truncated'):
            raise AssertionError(f'native pressure observations are truncated: {case}')
        if (len(periodic_pressure.get('samples', [])) > 4096
                or periodic_pressure.get('sample_limit') != 4096
                or periodic_pressure.get('serialized_sample_bytes', 0) > 8 * 1024 * 1024
                or periodic_pressure.get('serialized_sample_byte_limit') != 8 * 1024 * 1024):
            raise AssertionError(f'native pressure observations exceed their frozen bounds: {case}')
    expected_requests = expected['duration_ms'] * RATE // 1000 * expected['active_tenants']
    if workload.get('scheduled') != expected_requests:
        raise AssertionError(f'scheduled request count differs: {case}')
    if workload.get('admitted') != workload.get('completed') or workload.get('completed') != workload.get('successful'):
        raise AssertionError(f'workload completion accounting differs: {case}')
    if workload.get('dropped') != 0 or workload.get('timed_out') != 0:
        raise AssertionError(f'workload has drops or timeouts: {case}')
    if len(workload.get('per_tenant', [])) != expected['active_tenants']:
        raise AssertionError(f'per-tenant rows do not cover the active population: {case}')
    return expected


def parse_command(command_record, case):
    expected = expected_cell(case)
    arguments = command_record.get('command', [])
    option_values = {}
    for option in ['--backend', '--data-dir', '--report', '--registered-tenants',
                   '--active-tenants', '--opened-idle-tenants', '--initial-rows',
                   '--duration-ms', '--pressure-sample-interval-ms', '--tenant-arrival-phase',
                   '--per-tenant-rate', '--seed', '--value-bytes', '--client-count',
                   '--backend-memory-max-bytes', '--backend-memory-high-bytes',
                   '--backend-cgroup-root']:
        try:
            option_values[option] = arguments[arguments.index(option) + 1]
        except (ValueError, IndexError):
            raise AssertionError(f'command is missing {option}: {case}')
    expected_values = {
        '--backend': expected['backend'],
        '--registered-tenants': str(expected['active_tenants']),
        '--active-tenants': str(expected['active_tenants']),
        '--opened-idle-tenants': '0',
        '--initial-rows': str(ROWS),
        '--duration-ms': str(expected['duration_ms']),
        '--pressure-sample-interval-ms': '5000',
        '--tenant-arrival-phase': 'staggered',
        '--per-tenant-rate': str(RATE),
        '--seed': str(SEED),
        '--value-bytes': '512',
        '--client-count': '1',
        '--backend-memory-max-bytes': str(256 * 1024**2),
        '--backend-memory-high-bytes': str(192 * 1024**2),
        '--data-dir': str(DATABASE_ROOT / case),
        '--backend-cgroup-root': str(CGROUP_ROOT / case),
    }
    if option_values != expected_values:
        raise AssertionError(f'cell command differs from the frozen recipe: {case}')
    if command_record.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError(f'cell command source differs: {case}')
    return expected


def validate_cell_cgroups(case):
    if not CGROUP_ROOT.is_dir():
        raise AssertionError('resident frontier cgroup root is missing')
    root_group = CGROUP_ROOT
    if (root_group / 'memory.max').read_text().strip() != 'max':
        raise AssertionError('resident frontier cgroup root memory.max is not unbounded')
    if (root_group / 'cgroup.procs').read_text().strip():
        raise AssertionError('resident frontier grouping root still contains processes')
    cell_group = root_group / case
    observed_groups = [cell_group, *[cell_group / epoch for epoch in EPOCHS]]
    for group_path in observed_groups:
        if not group_path.is_dir():
            raise AssertionError(f'required backend cgroup is missing: {group_path}')
        limits = {
            'memory.max': str(256 * 1024**2),
            'memory.high': str(192 * 1024**2),
            'memory.swap.max': '0',
        }
        if any((group_path / name).read_text().strip() != expected for name, expected in limits.items()):
            raise AssertionError(f'backend cgroup limits differ: {group_path}')
        if (group_path / 'cgroup.procs').read_text().strip():
            raise AssertionError(f'backend cgroup still contains a process: {group_path}')
    return {'root': str(root_group), 'cell': str(cell_group), 'epoch_count': len(EPOCHS),
            'empty': True, 'limits_valid': True}


def validate_owned_process_limits(records):
    if not records:
        raise AssertionError('owned workload and descendant limits are missing')
    expected_limits = {'soft': NOFILE_SOFT_TARGET, 'hard': 524288, 'units': 'files'}
    if any(record.get('limits') != expected_limits for record in records):
        raise AssertionError('owned workload or descendant did not inherit the frozen open-file limits')
    if not any(record.get('role') == 'owned-workload-process' for record in records):
        raise AssertionError('owned workload process limits are missing')
    if not any(record.get('role') == 'owned-descendant' for record in records):
        raise AssertionError('owned workload descendant limits are missing')
    return True


def review_cell(run_directory, case, capacity_reviewer):
    run_directory = Path(run_directory)
    command_path = run_directory / f'{case}-command.json'
    status_path = run_directory / f'{case}-status.json'
    report_path = run_directory / f'{case}.json'
    log_path = run_directory / f'{case}.log'
    host_samples_path = run_directory / f'{case}-host-samples.json'
    for path in [command_path, status_path, report_path, log_path, host_samples_path]:
        if not path.is_file() or path.is_symlink():
            raise AssertionError(f'cell evidence is missing or not regular: {path.name}')
    command_record = read_json(command_path)
    status = read_json(status_path)
    report = read_json(report_path)
    expected = parse_command(command_record, case)
    validate_report(report, case, expected['backend'])
    cgroup_review = validate_cell_cgroups(case)
    if status.get('report_sha256') != sha256_file(report_path):
        raise AssertionError(f'cell report hash differs from its status: {case}')
    if status.get('case') != case or status.get('backend') != expected['backend']:
        raise AssertionError(f'cell status identity differs: {case}')
    if status.get('exit_code') != 0 or status.get('stop_reason') is not None or status.get('review_error'):
        raise AssertionError(f'cell did not exit operationally clean: {case}')
    validate_owned_process_limits(status.get('owned_process_limits', []))
    host_samples = read_json(host_samples_path)
    if not isinstance(host_samples, list) or len(host_samples) > 4096:
        raise AssertionError(f'host observation file exceeds its record bound: {case}')
    data_directory = Path(command_record['command'][command_record['command'].index('--data-dir') + 1])
    if data_directory != DATABASE_ROOT / case or not data_directory.is_dir():
        raise AssertionError(f'cell database is missing or outside its preserved namespace: {case}')
    database_files = [path for path in data_directory.rglob('*') if path.is_file()]
    database_bytes = sum(path.stat().st_size for path in database_files)
    if not database_files or database_bytes <= 0:
        raise AssertionError(f'cell database has no preserved data: {case}')
    reduced = capacity_reviewer.reduce_report(report, SOURCE_COMMIT)
    stored_reduced = status.get('reduced', {})
    if stored_reduced.get('operational_pass') is not True or reduced.get('operational_pass') is not True:
        raise AssertionError(f'cell is not operationally valid: {case}')
    if stored_reduced.get('slo_pass') != reduced.get('slo_pass'):
        raise AssertionError(f'cell SLO decision differs from the immutable reducer: {case}')
    return {
        'case': case,
        'backend': expected['backend'],
        'active_tenants': expected['active_tenants'],
        'kind': expected['kind'],
        'duration_ms': expected['duration_ms'],
        'valid': True,
        'operational_pass': True,
        'slo_pass': reduced['slo_pass'],
        'scheduled': reduced['scheduled'],
        'successful': reduced['successful'],
        'aggregate_p99': reduced['aggregate_p99'],
        'cell_peak_bytes': reduced['cell_peak_bytes'],
        'report_sha256': sha256_file(report_path),
        'cgroups': cgroup_review,
        'status_sha256': sha256_file(status_path),
        'command_sha256': sha256_file(command_path),
        'log_sha256': sha256_file(log_path),
        'host_samples_sha256': sha256_file(host_samples_path),
        'host_sample_count': len(host_samples),
        'database': {'path': str(data_directory), 'file_count': len(database_files),
                     'total_file_bytes': database_bytes, 'preserved': True},
    }


def expected_screen_schedule(baseline_by_backend):
    counts = sorted({tenant_count for backend in BACKENDS for tenant_count in (
        SCREEN_COUNTS_WITH_BASELINE if baseline_by_backend[backend]
        else SCREEN_COUNTS_WITHOUT_BASELINE
    )})
    schedule = []
    for group_number, tenant_count in enumerate(counts, start=1):
        first_backend = BACKENDS[(group_number - 1) % len(BACKENDS)]
        eligible = [backend for backend in BACKENDS if tenant_count in (
            SCREEN_COUNTS_WITH_BASELINE if baseline_by_backend[backend]
            else SCREEN_COUNTS_WITHOUT_BASELINE)]
        backend_order = [backend for backend in [first_backend, *[item for item in BACKENDS if item != first_backend]]
                         if backend in eligible]
        schedule.append({'count_group_number': group_number, 'active_tenants': tenant_count,
                         'first_backend': first_backend, 'backend_order': backend_order})
    return schedule


def route_decision(backend, cells, baseline_exists):
    screens = [cell for cell in cells.values() if cell['backend'] == backend and cell['kind'] == 'screen']
    passing_counts = sorted(cell['active_tenants'] for cell in screens if cell['valid'] and cell['slo_pass'])
    candidate = max(passing_counts, default=None)
    candidate_cells = sorted(
        [cell for cell in cells.values() if cell['backend'] == backend and cell['kind'] == 'confirm'
         and cell['active_tenants'] == candidate],
        key=lambda cell: int(re.fullmatch(r'confirm-n[0-9]+-r([1-3])-(?:blink|shared)', cell['case']).group(1)),
    ) if candidate is not None else []
    candidate_accepts = (len(candidate_cells) == 3
                         and all(cell['valid'] and cell['operational_pass'] and cell['slo_pass']
                                 for cell in candidate_cells))
    if candidate is None:
        return {'backend': backend, 'candidate_count': None,
                'accepted_count': 28 if baseline_exists else None,
                'accepted_from': 'preserved_control_baseline' if baseline_exists else None,
                'fallback_count': None, 'omission_reason': 'no_passing_screen'}
    if candidate_accepts:
        return {'backend': backend, 'candidate_count': candidate,
                'accepted_count': candidate, 'accepted_from': 'candidate_confirmed',
                'fallback_count': None}
    fallback = max((count for count in passing_counts if count < candidate), default=None)
    if fallback is None:
        return {'backend': backend, 'candidate_count': candidate,
                'accepted_count': 28 if baseline_exists else None,
                'accepted_from': 'preserved_control_baseline' if baseline_exists else None,
                'fallback_count': None, 'omission_reason': 'no_lower_passing_screen'}
    fallback_cells = sorted(
        [cell for cell in cells.values() if cell['backend'] == backend and cell['kind'] == 'fallback'
         and cell['active_tenants'] == fallback],
        key=lambda cell: int(re.fullmatch(r'fallback-n[0-9]+-r([1-3])-(?:blink|shared)', cell['case']).group(1)),
    )
    fallback_accepts = (len(fallback_cells) == 3
                        and all(cell['valid'] and cell['operational_pass'] and cell['slo_pass']
                                for cell in fallback_cells))
    if fallback_accepts:
        return {'backend': backend, 'candidate_count': candidate,
                'accepted_count': fallback, 'accepted_from': 'single_fallback_confirmed',
                'fallback_count': fallback}
    return {'backend': backend, 'candidate_count': candidate,
            'accepted_count': 28 if baseline_exists else None,
            'accepted_from': 'preserved_control_baseline' if baseline_exists else None,
            'fallback_count': fallback,
            'omission_reason': 'candidate_and_single_fallback_not_confirmed'}


def validate_frontier_decisions(state, reviewed_cells):
    baseline_by_backend = {backend: state['controls_baseline'][backend]['accepted'] for backend in BACKENDS}
    expected_schedule = expected_screen_schedule(baseline_by_backend)
    if state.get('screen_schedule') != expected_schedule:
        raise AssertionError('screen count union or alternating backend order differs from the frozen rule')
    screens_by_backend = {backend: {} for backend in BACKENDS}
    for cell in reviewed_cells.values():
        if cell['kind'] == 'screen':
            if cell['active_tenants'] not in (
                    SCREEN_COUNTS_WITH_BASELINE if baseline_by_backend[cell['backend']]
                    else SCREEN_COUNTS_WITHOUT_BASELINE):
                raise AssertionError(f'screen count is not allowed for its baseline route: {cell["case"]}')
            screens_by_backend[cell['backend']][str(cell['active_tenants'])] = {
                'operational_pass': cell['operational_pass'], 'slo_pass': cell['slo_pass'],
                'case': cell['case'],
            }
    for backend in BACKENDS:
        ordered_results = sorted((int(count), result) for count, result in screens_by_backend[backend].items())
        missed = False
        for tenant_count, result in ordered_results:
            if missed:
                raise AssertionError(f'higher screen ran after a route SLO miss: {backend} N={tenant_count}')
            if not result['slo_pass']:
                missed = True
        expected_candidate = max((tenant_count for tenant_count, result in ordered_results
                                  if result['operational_pass'] and result['slo_pass']), default=None)
        if state.get('candidate_counts', {}).get(backend) != expected_candidate:
            raise AssertionError(f'candidate selection differs from the highest passing screen: {backend}')
    expected_screen_omissions = set()
    for backend in BACKENDS:
        allowed_counts = (SCREEN_COUNTS_WITH_BASELINE if baseline_by_backend[backend]
                          else SCREEN_COUNTS_WITHOUT_BASELINE)
        missed = False
        for tenant_count in allowed_counts:
            result = screens_by_backend[backend].get(str(tenant_count))
            if missed:
                suffix = 'blink' if backend == 'blink' else 'shared'
                expected_screen_omissions.add((f'screen-n{tenant_count}-{suffix}',
                                               'higher_screen_omitted_after_first_route_slo_miss', 'screen'))
            elif result is not None and not result['slo_pass']:
                missed = True
    actual_screen_omissions = {
        (record.get('case'), record.get('reason'), record.get('stage'))
        for record in state.get('omitted_cases', []) if record.get('stage') == 'screen'
    }
    if actual_screen_omissions != expected_screen_omissions:
        raise AssertionError('screen omissions do not match the first route SLO miss rule')
    omission_records = state.get('omitted_cases', [])
    omission_cases = [record.get('case') for record in omission_records]
    if len(omission_cases) != len(set(omission_cases)):
        raise AssertionError('frontier records duplicate omitted cases')
    computed_decisions = {
        backend: route_decision(backend, reviewed_cells, baseline_by_backend[backend])
        for backend in BACKENDS
    }
    for backend in BACKENDS:
        candidate = state.get('candidate_counts', {}).get(backend)
        candidate_cells = [cell for cell in reviewed_cells.values()
                           if cell['backend'] == backend and cell['kind'] == 'confirm']
        if candidate is None and candidate_cells:
            raise AssertionError(f'candidate confirmations ran without a passing screen: {backend}')
        if any(cell['active_tenants'] != candidate for cell in candidate_cells):
            raise AssertionError(f'candidate confirmation population differs from the selected candidate: {backend}')
        candidate_rounds = sorted(int(re.fullmatch(
            r'confirm-n[0-9]+-r([1-3])-(?:blink|shared)', cell['case']).group(1))
                                   for cell in candidate_cells)
        if candidate_rounds != list(range(1, len(candidate_rounds) + 1)):
            raise AssertionError(f'candidate repetitions are not a fresh sequential prefix: {backend}')
        if any(not cell['slo_pass'] for cell in candidate_cells[:-1]):
            raise AssertionError(f'candidate repetitions continued after the first SLO miss: {backend}')
        decision = computed_decisions[backend]
        fallback_cells = [cell for cell in reviewed_cells.values()
                          if cell['backend'] == backend and cell['kind'] == 'fallback']
        if decision['fallback_count'] is None and fallback_cells:
            raise AssertionError(f'fallback ran when the frozen rule did not permit one: {backend}')
        if any(cell['active_tenants'] != decision['fallback_count'] for cell in fallback_cells):
            raise AssertionError(f'more than one fallback population was searched: {backend}')
        fallback_rounds = sorted(int(re.fullmatch(
            r'fallback-n[0-9]+-r([1-3])-(?:blink|shared)', cell['case']).group(1))
                                  for cell in fallback_cells)
        if fallback_rounds != list(range(1, len(fallback_rounds) + 1)):
            raise AssertionError(f'fallback repetitions are not a fresh sequential prefix: {backend}')
        if any(not cell['slo_pass'] for cell in fallback_cells[:-1]):
            raise AssertionError(f'fallback repetitions continued after the first SLO miss: {backend}')
    if state.get('decisions') != computed_decisions:
        raise AssertionError('accepted counts differ from the candidate and single-fallback rule')
    return computed_decisions


def review_directory(run_directory, output_path=None, capacity_reviewer=None):
    run_directory = Path(run_directory)
    provenance = read_json(run_directory / 'provenance.json')
    if provenance.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('resident frontier source provenance differs')
    if provenance.get('design_plan_commit') != DESIGN_PLAN_COMMIT:
        raise AssertionError('resident frontier plan provenance differs')
    if capacity_reviewer is None:
        raise AssertionError('immutable capacity reviewer must be supplied')
    statuses = sorted(run_directory.glob('*-status.json'))
    if len(statuses) > 22:
        raise AssertionError('resident frontier exceeds the frozen twenty-two-cell cap')
    reviewed_cells = {}
    operational_failure_cases = []
    for status_path in statuses:
        case = status_path.name.removesuffix('-status.json')
        try:
            reviewed_cells[case] = review_cell(run_directory, case, capacity_reviewer)
        except Exception as error:
            status_record = read_json(status_path)
            operational_failure_cases.append({
                'case': case, 'error_type': type(error).__name__, 'error': str(error),
                'available_unvalidated_counters': status_record.get('available_unvalidated_counters'),
            })
    decisions = read_json(run_directory / 'frontier.json')
    terminal_path = run_directory / 'completion.json'
    if not terminal_path.is_file():
        terminal_path = run_directory / 'failure-receipt.json'
    terminal = read_json(terminal_path) if terminal_path.is_file() else {}
    completed_terminal = terminal_path.name == 'completion.json' and terminal.get('completed') is True
    summary_path = run_directory / 'summary.json'
    summary = read_json(summary_path) if summary_path.is_file() else []
    recomputed_decisions = (validate_frontier_decisions(decisions, reviewed_cells)
                            if completed_terminal and not operational_failure_cases else {})
    process_records = terminal.get('processes', [])
    processes_exited = (bool(process_records) and all(
        record.get('same_live_process') is False for record in process_records)) if completed_terminal else all(
            record.get('same_live_process') is False for record in process_records)
    database_cases = []
    for command_path in sorted(run_directory.glob('*-command.json')):
        case = command_path.name.removesuffix('-command.json')
        command = read_json(command_path).get('command', [])
        try:
            database_path = Path(command[command.index('--data-dir') + 1])
        except (ValueError, IndexError):
            database_cases.append({'case': case, 'preserved': False, 'reason': 'data-dir missing'})
            continue
        if database_path != DATABASE_ROOT / case:
            database_cases.append({'case': case, 'path': str(database_path), 'preserved': False,
                                   'reason': 'path outside frozen namespace'})
            continue
        files = [path for path in database_path.rglob('*') if path.is_file()] if database_path.is_dir() else []
        database_cases.append({'case': case, 'path': str(database_path), 'preserved': bool(files),
                               'file_count': len(files),
                               'total_file_bytes': sum(path.stat().st_size for path in files)})
    output = {
        'source_commit': SOURCE_COMMIT,
        'design_plan_commit': DESIGN_PLAN_COMMIT,
        'provenance_sha256': sha256_file(run_directory / 'provenance.json'),
        'frontier_sha256': sha256_file(run_directory / 'frontier.json'),
        'actual_cell_count': len(statuses),
        'operationally_valid_cell_count': sum(cell['operational_pass'] for cell in reviewed_cells.values()),
        'operational_failure_cases': operational_failure_cases,
        'terminal_receipt': {'path': str(terminal_path) if terminal_path.is_file() else None,
                             'sha256': sha256_file(terminal_path) if terminal_path.is_file() else None,
                             'process_count': len(process_records),
                             'all_recorded_processes_exited': processes_exited},
        'databases': {'attempted_cell_count': len(database_cases), 'cases': database_cases,
                      'all_attempted_databases_preserved': all(item['preserved'] for item in database_cases)
                      if database_cases else None},
        'cells': reviewed_cells,
        'decisions': recomputed_decisions,
        'omissions': decisions.get('omitted_cases', []),
        'execution_order': [{'case': outcome.get('case'), 'stage': outcome.get('stage'),
                             'repetition': outcome.get('repetition')}
                            for outcome in summary],
        'cross_population_throughput_comparisons_performed': False,
        'cells_remaining_after_global_stop': decisions.get('omitted_after_stop', []),
        'review_valid': not operational_failure_cases and processes_exited and terminal_path.is_file()
        and decisions.get('terminal_status') == ('completed' if completed_terminal else 'failed'),
        'completed_run_evidence_valid': (completed_terminal and not operational_failure_cases
                                         and processes_exited),
        'accepted_run': False,
    }
    if output_path is None:
        output_path = run_directory / 'final-review.json'
    output_path = Path(output_path)
    serialized = json.dumps(output, indent=2, sort_keys=True) + '\n'
    if output_path.exists() or output_path.is_symlink():
        if output_path.is_symlink() or not output_path.is_file():
            raise AssertionError('existing frontier review output is not a regular file')
        if output_path.read_text() != serialized:
            raise AssertionError('existing frontier review differs; refusing to overwrite')
        return output
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open('x') as output_file:
        output_file.write(serialized)
    return output


def main():
    parser = argparse.ArgumentParser(description='Review resident frontier runtime evidence')
    parser.add_argument('--run-directory', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--capacity-reviewer', type=Path, required=True)
    arguments = parser.parse_args()
    from importlib.util import spec_from_file_location, module_from_spec
    specification = spec_from_file_location('resident_capacity_review', arguments.capacity_reviewer)
    capacity_reviewer = module_from_spec(specification)
    sys_modules = __import__('sys').modules
    sys_modules[specification.name] = capacity_reviewer
    specification.loader.exec_module(capacity_reviewer)
    result = review_directory(arguments.run_directory, arguments.output, capacity_reviewer)
    print(json.dumps({'review_valid': result['review_valid'],
                      'actual_cell_count': result['actual_cell_count'],
                      'operationally_valid_cell_count': result['operationally_valid_cell_count']}, sort_keys=True))
    return 0 if result['review_valid'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
