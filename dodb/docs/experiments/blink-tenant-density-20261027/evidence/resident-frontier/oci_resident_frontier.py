import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
RUN_NAME = 'resident-frontier-cb0e04482-r2'
SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
DESIGN_PLAN_COMMIT = '6f57299f3af21c19444ebb203e40d636abcd0a04'
CONTROLS_RUN = 'arrival-controls-cb0e04482-r1'
VALIDATION_RUN = 'arrival-cb0e04482-r1'
CONTROLS_PLAN_COMMIT = 'e02a1fbdde81fd6fc363a26e5341c8c61000023e'
CONTROLS_CONTROLLER_COMMIT = '95c3b8c5323415dfc493bdfcb32c7dc5e057e8b8'
EXPECTED_NATIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
DATABASE_ROOT = Path('/bench/zfs/db/tenant-density-harness-20261027')
NOFILE_SOFT_TARGET = 65536
BACKENDS = ['blink', 'minimal-native-shared']
ROWS = 2048
VALUE_BYTES = 512
SEED = 20261027
RATE = 50
SCREEN_COUNTS_WITH_BASELINE = [32, 48, 64, 80, 96]
SCREEN_COUNTS_WITHOUT_BASELINE = [16, 24]
SCREEN_DURATION_MS = 30000
CONFIRMATION_DURATION_MS = 60000
MAX_NEW_CELLS = 22


def sha256_file(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def write_json(path, value):
    temporary_path = Path(path).with_suffix(Path(path).suffix + '.tmp')
    temporary_path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')
    temporary_path.replace(path)


def load_json(path):
    return json.loads(Path(path).read_text())


def load_module(name, path):
    module_path = Path(path)
    if not module_path.is_file():
        raise AssertionError(f'required helper is missing: {module_path}')
    sys.path.insert(0, str(module_path.parent))
    specification = importlib.util.spec_from_file_location(name, module_path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[name] = module
    specification.loader.exec_module(module)
    return module


def validate_current_source_native(validation_module):
    original_directory = Path.cwd()
    os.chdir(SOURCE)
    try:
        return validation_module.validate_source_and_native()
    finally:
        os.chdir(original_directory)


def validate_controls_review(path, expected_sha256):
    review_path = Path(path)
    if not review_path.is_file() or review_path.is_symlink():
        raise AssertionError('canonical controls review is missing or is not a regular file')
    actual_sha256 = sha256_file(review_path)
    if actual_sha256 != expected_sha256:
        raise AssertionError('canonical controls review hash does not match the supplied hash')
    review = load_json(review_path)
    if review.get('validation', {}).get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('canonical controls review source commit differs')
    controls_provenance = review.get('provenance', {})
    if (controls_provenance.get('plan_commit') != CONTROLS_PLAN_COMMIT
            or controls_provenance.get('actual_controller_commit') != CONTROLS_CONTROLLER_COMMIT):
        raise AssertionError('canonical controls review plan or controller commit differs')
    collection_validity = review.get('collection_validity', {})
    if collection_validity.get('completed_run_evidence_valid') is not True:
        raise AssertionError('canonical controls review does not validate a completed operational cohort')
    cells = review.get('cells', {})
    if cells.get('completed') is not True or cells.get('actual_unique_count') != 16:
        raise AssertionError('canonical controls review is incomplete')
    if cells.get('operationally_valid_count') != 16 or cells.get('tiny_slo_gate') is not True:
        raise AssertionError('canonical controls review failed operational or tiny gates')
    requests = review.get('requests', {})
    expected_request_count = requests.get('expected_complete_cohort')
    if (expected_request_count != 1_014_000
            or requests.get('validated_successful_requests') != 1_014_000
            or review.get('oracles', {}).get('validated_oracle_count') != 32
            or review.get('oracles', {}).get('expected_complete_cohort') != 32
            or review.get('databases', {}).get('preserved_database_count') != 16
            or review.get('processes', {}).get('all_recorded_processes_exited') is not True
            or review.get('processes', {}).get('owned_workload_child_count', 0) < 16
            or review.get('cgroups', {}).get('cgroup_count') != 81
            or review.get('cgroups', {}).get('all_cgroup_process_lists_empty') is not True):
        raise AssertionError('canonical controls review lacks complete oracle, database, process, or cgroup evidence')
    results = cells.get('case_results', {})
    baseline_slo = {}
    for backend in BACKENDS:
        route_passes = []
        for round_number in range(1, 4):
            case = f'control-n28-r{round_number}-staggered-{backend_suffix(backend)}'
            result = results.get(case, {})
            route_passes.append(
                result.get('valid') is True
                and result.get('operational_pass') is True
                and result.get('slo_pass') is True
            )
        baseline_slo[backend] = all(route_passes)
    validation = review.get('validation', {})
    if (validation.get('source_commit') != SOURCE_COMMIT
            or validation.get('passing_tests') != 85
            or validation.get('same_17_existing_clippy_diagnostics') is not True
            or validation.get('default_link_was_last') is not True
            or validation.get('native_archive_sha256') != EXPECTED_NATIVE_SHA256
            or validation.get('validation_binaries_valid') is not True
            or validation.get('runtime_binaries_match_validation') is not True):
        raise AssertionError('canonical controls review does not preserve frozen validation provenance')
    runtime_limits = review.get('runtime_limits', {})
    if (runtime_limits.get('new_soft') != NOFILE_SOFT_TARGET
            or runtime_limits.get('hard') != 524288
            or runtime_limits.get('preflight_child_exited') is not True):
        raise AssertionError('canonical controls review does not verify the controller open-file limit')
    return {
        'path': str(review_path),
        'sha256': actual_sha256,
        'review': review,
        'accepted_staggered_n28_baseline_by_backend': baseline_slo,
    }


def backend_suffix(backend):
    return 'shared' if backend == 'minimal-native-shared' else 'blink'


def screen_schedule(baseline_by_backend):
    schedule = []
    counts = sorted({count for backend in BACKENDS for count in (
        SCREEN_COUNTS_WITH_BASELINE if baseline_by_backend[backend]
        else SCREEN_COUNTS_WITHOUT_BASELINE
    )})
    for count_group_number, tenant_count in enumerate(counts, start=1):
        eligible_backends = [
            backend for backend in BACKENDS
            if tenant_count in (SCREEN_COUNTS_WITH_BASELINE if baseline_by_backend[backend]
                                else SCREEN_COUNTS_WITHOUT_BASELINE)
        ]
        first_backend = BACKENDS[(count_group_number - 1) % len(BACKENDS)]
        backend_order = [first_backend, *[backend for backend in BACKENDS
                                           if backend != first_backend]]
        ordered_eligible = [backend for backend in backend_order if backend in eligible_backends]
        schedule.append({'count_group_number': count_group_number,
                         'active_tenants': tenant_count,
                         'first_backend': first_backend,
                         'backend_order': ordered_eligible})
    return schedule


def eligible_screen_count(tenant_count, baseline_exists, higher_screen_missed):
    allowed_counts = SCREEN_COUNTS_WITH_BASELINE if baseline_exists else SCREEN_COUNTS_WITHOUT_BASELINE
    return tenant_count in allowed_counts and not higher_screen_missed


def decide_route_acceptance(backend, screen_results, confirmation_results, baseline_exists):
    passing_screen_counts = sorted(
        tenant_count for tenant_count, result in screen_results.items()
        if result.get('operational_pass') is True and result.get('slo_pass') is True
    )
    candidate = max(passing_screen_counts, default=None)
    if candidate is None:
        return {'backend': backend, 'candidate_count': None, 'accepted_count': 28 if baseline_exists else None,
                'accepted_from': 'preserved_control_baseline' if baseline_exists else None,
                'fallback_count': None, 'omission_reason': 'no_passing_screen'}
    candidate_runs = confirmation_results.get(('candidate', candidate), [])
    if len(candidate_runs) == 3 and all(result.get('operational_pass') is True
                                        and result.get('slo_pass') is True for result in candidate_runs):
        return {'backend': backend, 'candidate_count': candidate, 'accepted_count': candidate,
                'accepted_from': 'candidate_confirmed', 'fallback_count': None}
    lower_counts = [tenant_count for tenant_count in passing_screen_counts if tenant_count < candidate]
    fallback = max(lower_counts, default=None)
    if fallback is None:
        return {'backend': backend, 'candidate_count': candidate,
                'accepted_count': 28 if baseline_exists else None,
                'accepted_from': 'preserved_control_baseline' if baseline_exists else None,
                'fallback_count': None, 'omission_reason': 'no_lower_passing_screen'}
    fallback_runs = confirmation_results.get(('fallback', fallback), [])
    if len(fallback_runs) == 3 and all(result.get('operational_pass') is True
                                       and result.get('slo_pass') is True for result in fallback_runs):
        return {'backend': backend, 'candidate_count': candidate, 'accepted_count': fallback,
                'accepted_from': 'single_fallback_confirmed', 'fallback_count': fallback}
    return {'backend': backend, 'candidate_count': candidate,
            'accepted_count': 28 if baseline_exists else None,
            'accepted_from': 'preserved_control_baseline' if baseline_exists else None,
            'fallback_count': fallback, 'omission_reason': 'candidate_and_single_fallback_not_confirmed'}


class CapacityRuntime:
    def __init__(self, module):
        self.module = module
        self.BASE_ARGUMENTS = module.BASE_ARGUMENTS
        self.BACKEND_ARGUMENTS = module.BACKEND_ARGUMENTS
        self.first_cell_guard_used = False

    def capacity(self):
        return self.module.capacity()

    def guard(self, state, before=False):
        if before and not self.first_cell_guard_used:
            self.module.guard(state, before=True)
            self.first_cell_guard_used = True
        else:
            self.module.guard(state, before=False)

    def prepare_group(self, group_path):
        return self.module.prepare_group(group_path)

    def kill_group(self, group_path):
        return self.module.kill_group(group_path)

    def process_identity_live(self, identity):
        return self.module.process_identity_live(identity)

    def reduce_report(self, report, source_commit):
        return self.module.reduce_report(report, source_commit)

    def write_json(self, path, value):
        return self.module.write_json(path, value)


class ResidentRunner:
    def __init__(self, controls_module, capacity_runtime, reviewer_module, validation,
                 run_directory, database_directory, group_root, frozen_binaries):
        self.controls_module = controls_module
        self.capacity = capacity_runtime
        self.reviewer = reviewer_module
        self.validation = validation
        self.plan_commit = DESIGN_PLAN_COMMIT
        self.nofile_identity = validation['runtime_limits']
        self.frozen_binaries = frozen_binaries
        self.run_directory = run_directory
        self.database_directory = database_directory
        self.group_root = group_root
        self.binaries = run_directory / 'bin'
        self.all_processes = []
        self.outcomes = []
        self.pair_reports = {}
        self.stop_reason = None

    def _read_owned_limits(self, pid):
        return self.controls_module.ArrivalExperiment._read_owned_limits(self, pid)

    def run_cell(self, case, backend, active_tenants, duration_ms, stage, repetition=None):
        report, outcome = self.controls_module.ArrivalExperiment.run_cell(
            self, case, 'staggered', backend, active_tenants, active_tenants, 0, duration_ms)
        command_path = self.run_directory / f'{case}-command.json'
        command_record = load_json(command_path)
        command_record['stage'] = stage
        command_record['population_role'] = 'all_registered_tenants_active'
        command_record['design_plan_commit'] = DESIGN_PLAN_COMMIT
        command_record['repetition'] = repetition
        write_json(command_path, command_record)
        outcome['stage'] = stage
        outcome['repetition'] = repetition
        status_path = self.run_directory / f'{case}-status.json'
        write_json(status_path, outcome)
        self.capacity.write_json(self.run_directory / 'summary.json', self.outcomes)
        return report, outcome


def runtime_helper(primary_path, fallback_path):
    primary = Path(primary_path)
    if primary.is_file():
        return primary
    fallback = Path(fallback_path)
    if fallback.is_file():
        return fallback
    raise AssertionError(f'required frozen helper is missing: {primary}')


def controls_helper_paths(controls_review, evidence_directory):
    review = controls_review['review']
    helper_records = review.get('helpers', {})
    controls_hash = helper_records.get('arrival-phase/oci_arrival_controls.py', {}).get('sha256')
    capacity_hash = helper_records.get('capacity/oci_capacity.py', {}).get('sha256')
    reviewer_hash = helper_records.get('capacity/capacity_review.py', {}).get('sha256')
    arrival_review_hash = helper_records.get('arrival-phase/arrival_review.py', {}).get('sha256')
    validation_hash = helper_records.get('arrival-phase/oci_arrival_validate.py', {}).get('sha256')
    expected = {
        'oci_arrival_controls.py': controls_hash,
        'oci_capacity.py': capacity_hash,
        'capacity_review.py': reviewer_hash,
        'arrival_review.py': arrival_review_hash,
        'oci_arrival_validate.py': validation_hash,
    }
    runtime_directory = ROOT
    capacity_directory = Path(evidence_directory).parent / 'capacity'
    paths = {
        'oci_arrival_controls.py': runtime_helper(runtime_directory / 'oci_arrival_controls.py',
                                                 Path(evidence_directory).parent / 'arrival-phase' / 'oci_arrival_controls.py'),
        'oci_capacity.py': runtime_helper(ROOT / 'oci_capacity.py', capacity_directory / 'oci_capacity.py'),
        'capacity_review.py': runtime_helper(ROOT / 'capacity_review.py', capacity_directory / 'capacity_review.py'),
        'arrival_review.py': runtime_helper(runtime_directory / 'arrival_review.py',
                                            Path(evidence_directory).parent / 'arrival-phase' / 'arrival_review.py'),
        'oci_arrival_validate.py': runtime_helper(runtime_directory / 'oci_arrival_validate.py',
                                                  Path(evidence_directory).parent / 'arrival-phase' / 'oci_arrival_validate.py'),
    }
    for helper_name, helper_path in paths.items():
        if not expected[helper_name] or sha256_file(helper_path) != expected[helper_name]:
            raise AssertionError(f'frozen controls helper hash differs: {helper_name}')
    return paths


def prepare_validated_binaries(validation_review, validation_summary_path, run_directory):
    summary = load_json(validation_summary_path)
    if summary.get('status') != 'completed' or summary.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('preserved default binary validation summary is invalid')
    if len(summary.get('outcomes', [])) != 4 or any(outcome.get('exit_code') != 0 for outcome in summary['outcomes']):
        raise AssertionError('preserved default binary validation checks are incomplete')
    if summary['outcomes'][0].get('passing_tests') != 85 or summary['outcomes'][1].get('same_17_existing_clippy_diagnostics') is not True:
        raise AssertionError('preserved native validation does not match the accepted 85-test and 17-warning baseline')
    if summary['outcomes'][-1].get('name') != 'default-link' or summary.get('default_link_was_last') is not True:
        raise AssertionError('default binary validation link was not last')
    if summary.get('native_archive_sha256_before') != EXPECTED_NATIVE_SHA256 or summary.get('native_archive_sha256_after') != EXPECTED_NATIVE_SHA256:
        raise AssertionError('preserved native validation archive hash differs')
    recorded = validation_review.get('default_binaries', {})
    if recorded != summary.get('default_binaries'):
        raise AssertionError('canonical controls review and validation summary binary records differ')
    source_directory = ROOT / 'arrival-phase' / VALIDATION_RUN / 'bin'
    copied = {}
    for binary_name in ['tenant_density_workload', 'memory_diagnostic_worker']:
        source_binary = source_directory / binary_name
        expected_identity = summary['default_binaries'][binary_name]
        if (not source_binary.is_file() or source_binary.stat().st_size != expected_identity['bytes']
                or sha256_file(source_binary) != expected_identity['sha256']):
            raise AssertionError(f'validated default binary differs: {binary_name}')
        copied[binary_name] = expected_identity
    if run_directory is not None:
        binary_directory = run_directory / 'bin'
        binary_directory.mkdir()
        for binary_name, expected_identity in copied.items():
            source_binary = source_directory / binary_name
            destination = binary_directory / binary_name
            with source_binary.open('rb') as source_file, destination.open('xb') as destination_file:
                shutil.copyfileobj(source_file, destination_file, 1024 * 1024)
            shutil.copystat(source_binary, destination)
            if sha256_file(destination) != expected_identity['sha256']:
                raise AssertionError(f'copied default binary differs: {binary_name}')
    return copied, summary


def baseline_review_cells(controls_review, backend):
    results = controls_review['review']['cells']['case_results']
    return [
        {'case': f'control-n28-r{round_number}-staggered-{backend_suffix(backend)}',
         **results[f'control-n28-r{round_number}-staggered-{backend_suffix(backend)}']}
        for round_number in range(1, 4)
    ]


def make_frontier_state(baseline_by_backend):
    return {
        'source_commit': SOURCE_COMMIT,
        'design_plan_commit': DESIGN_PLAN_COMMIT,
        'screen_schedule': screen_schedule(baseline_by_backend),
        'screen_results': {backend: {} for backend in BACKENDS},
        'confirmation_results': {backend: {'candidate': [], 'fallback': []} for backend in BACKENDS},
        'candidate_counts': {},
        'decisions': {},
        'omitted_cases': [],
        'omitted_after_stop': [],
        'global_stop': None,
    }


def execute_screen_schedule(runner, state, baseline_by_backend, checkpoint):
    higher_screen_missed = {backend: False for backend in BACKENDS}
    for group in state['screen_schedule']:
        tenant_count = group['active_tenants']
        for backend in BACKENDS:
            allowed = tenant_count in (SCREEN_COUNTS_WITH_BASELINE if baseline_by_backend[backend]
                                       else SCREEN_COUNTS_WITHOUT_BASELINE)
            if allowed and higher_screen_missed[backend]:
                record_omission(state, case_name('screen', tenant_count, backend),
                                'higher_screen_omitted_after_first_route_slo_miss', 'screen')
        for backend in group['backend_order']:
            if higher_screen_missed[backend]:
                continue
            cell_case = case_name('screen', tenant_count, backend)
            _, outcome = runner.run_cell(cell_case, backend, tenant_count, SCREEN_DURATION_MS, 'screen')
            state['screen_results'][backend][str(tenant_count)] = {
                'operational_pass': outcome['reduced']['operational_pass'],
                'slo_pass': outcome['reduced']['slo_pass'],
                'case': cell_case,
            }
            if not outcome['reduced']['slo_pass']:
                higher_screen_missed[backend] = True
            checkpoint(state)
    return higher_screen_missed


def record_omission(state, case, reason, stage):
    state['omitted_cases'].append({'case': case, 'stage': stage, 'reason': reason})


def case_name(stage, tenant_count, backend, repetition=None):
    suffix = backend_suffix(backend)
    if stage == 'screen':
        return f'screen-n{tenant_count}-{suffix}'
    prefix = 'confirm' if stage == 'candidate' else 'fallback'
    return f'{prefix}-n{tenant_count}-r{repetition}-{suffix}'


def run_frontier(plan_commit, controls):
    evidence_directory = Path(__file__).resolve().parent
    helper_paths = controls_helper_paths(controls, evidence_directory)
    controls_module = load_module('resident_frozen_arrival_controls', helper_paths['oci_arrival_controls.py'])
    capacity_module = load_module('resident_capacity_runtime', helper_paths['oci_capacity.py'])
    validation_module = load_module('resident_arrival_validation', helper_paths['oci_arrival_validate.py'])
    review_module = load_module('resident_frontier_reviewer', evidence_directory / 'resident_frontier_review.py')
    source_native = validate_current_source_native(validation_module)
    if source_native['source_commit'] != SOURCE_COMMIT or source_native['source_status'] != '':
        raise AssertionError('source must be clean at the frozen commit')
    if source_native['native_archive_sha256_before'] != EXPECTED_NATIVE_SHA256:
        raise AssertionError('current native archive differs from the frozen binary provenance')
    nofile_identity = controls_module.set_controller_nofile_limit()
    nofile_identity['preflight_child'] = controls_module.verify_preflight_child_limits(nofile_identity['proc_hard'])
    validation_summary_path = ROOT / 'arrival-phase' / VALIDATION_RUN / 'arrival-validation-summary.json'
    frozen_binaries, validation_summary = prepare_validated_binaries(
        controls['review']['validation'], validation_summary_path, None)
    capacity_runtime = CapacityRuntime(capacity_module)
    before = capacity_runtime.capacity()
    capacity_runtime.guard(before, before=True)
    run_directory = ROOT / 'arrival-phase' / RUN_NAME
    run_directory.mkdir(parents=False, exist_ok=False)
    frozen_binaries, validation_summary = prepare_validated_binaries(
        controls['review']['validation'], validation_summary_path, run_directory)
    database_directory = DATABASE_ROOT / RUN_NAME
    database_directory.mkdir(exist_ok=False)
    group_root = Path('/sys/fs/cgroup') / f'dodb-density-{RUN_NAME}'
    capacity_runtime.prepare_group(group_root)
    helper_records = controls['review']['helpers']
    baseline_by_backend = controls['accepted_staggered_n28_baseline_by_backend']
    provenance = {
        'run_name': RUN_NAME,
        'source_commit': SOURCE_COMMIT,
        'design_plan_commit': plan_commit,
        'controller_sha256': sha256_file(Path(__file__).resolve()),
        'reviewer_sha256': sha256_file(evidence_directory / 'resident_frontier_review.py'),
        'controls_review_path': controls['path'],
        'controls_review_sha256': controls['sha256'],
        'accepted_staggered_n28_baseline_by_backend': baseline_by_backend,
        'controls_controller_sha256': helper_records['arrival-phase/oci_arrival_controls.py']['sha256'],
        'controls_capacity_controller_sha256': helper_records['capacity/oci_capacity.py']['sha256'],
        'controls_capacity_reviewer_sha256': helper_records['capacity/capacity_review.py']['sha256'],
        'controls_arrival_reviewer_sha256': helper_records['arrival-phase/arrival_review.py']['sha256'],
        'validation_controller_sha256': helper_records['arrival-phase/oci_arrival_validate.py']['sha256'],
        'validation_summary_sha256': sha256_file(ROOT / 'arrival-phase' / VALIDATION_RUN / 'arrival-validation-summary.json'),
        'validation': validation_summary,
        'source_native_before': source_native,
        'runtime_limits': nofile_identity,
        'capacity_before': before,
        'validated_binaries': frozen_binaries,
        'policy': {
            'population': 'all_registered_tenants_active',
            'rows_per_tenant': ROWS,
            'value_bytes': VALUE_BYTES,
            'seed': SEED,
            'rate_per_active_tenant': RATE,
            'screen_duration_ms': SCREEN_DURATION_MS,
            'confirmation_duration_ms': CONFIRMATION_DURATION_MS,
            'pressure_sample_interval_ms': 5000,
            'arrival_phase': 'staggered',
            'max_new_cells': MAX_NEW_CELLS,
            'screen_counts_with_accepted_n28_baseline': SCREEN_COUNTS_WITH_BASELINE,
            'screen_counts_without_accepted_n28_baseline': SCREEN_COUNTS_WITHOUT_BASELINE,
            'max_candidate_fallbacks_per_backend': 1,
            'backend_memory_max_bytes': 256 * 1024**2,
            'backend_memory_high_bytes': 192 * 1024**2,
            'backend_memory_swap_max_bytes': 0,
            'observation_bytes_per_report': 8 * 1024 * 1024,
            'observation_samples_per_report': 4096,
        },
        'scope': 'bounded staggered resident-population frontier; no engine maximum or adoption claim',
    }
    write_json(run_directory / 'provenance.json', provenance)
    runner = ResidentRunner(controls_module, capacity_runtime, review_module,
                            {'runtime_limits': nofile_identity, 'source_commit': SOURCE_COMMIT}, run_directory,
                            database_directory, group_root, frozen_binaries)
    def request_stop(signum, frame):
        runner.stop_reason = f'parent_signal_{signum}'

    signal.signal(signal.SIGTERM, request_stop)
    signal.signal(signal.SIGINT, request_stop)
    state = make_frontier_state(baseline_by_backend)
    state['controls_baseline'] = {
        backend: {'accepted': baseline_by_backend[backend],
                  'cases': [record['case'] for record in baseline_review_cells(controls, backend)]}
        for backend in BACKENDS
    }
    write_json(run_directory / 'frontier.json', state)
    try:
        higher_screen_missed = execute_screen_schedule(
            runner, state, baseline_by_backend,
            lambda updated_state: write_json(run_directory / 'frontier.json', updated_state))
        for backend in BACKENDS:
            passing_counts = [int(tenant_count) for tenant_count, result in state['screen_results'][backend].items()
                              if result['operational_pass'] and result['slo_pass']]
            candidate = max(passing_counts, default=None)
            state['candidate_counts'][backend] = candidate
            if candidate is None:
                record_omission(state, f'confirm-no-candidate-{backend_suffix(backend)}',
                                'route_has_no_passing_screen', 'candidate_confirmation')
        write_json(run_directory / 'frontier.json', state)
        candidate_active = {backend: state['candidate_counts'][backend] is not None for backend in BACKENDS}
        candidate_missed = {backend: False for backend in BACKENDS}
        for round_number in range(1, 4):
            first_backend = BACKENDS[(round_number - 1) % len(BACKENDS)]
            backend_order = [first_backend, *[backend for backend in BACKENDS if backend != first_backend]]
            for backend in backend_order:
                if not candidate_active[backend] or candidate_missed[backend]:
                    continue
                candidate = state['candidate_counts'][backend]
                cell_case = case_name('candidate', candidate, backend, round_number)
                _, outcome = runner.run_cell(cell_case, backend, candidate,
                                             CONFIRMATION_DURATION_MS, 'candidate_confirmation', round_number)
                state['confirmation_results'][backend]['candidate'].append({
                    'case': cell_case,
                    'active_tenants': candidate,
                    'operational_pass': outcome['reduced']['operational_pass'],
                    'slo_pass': outcome['reduced']['slo_pass'],
                })
                if not outcome['reduced']['slo_pass']:
                    candidate_missed[backend] = True
                    for skipped_round in range(round_number + 1, 4):
                        record_omission(state, case_name('candidate', candidate, backend, skipped_round),
                                        'remaining_candidate_repetitions_omitted_after_first_slo_miss',
                                        'candidate_confirmation')
                write_json(run_directory / 'frontier.json', state)
        fallback_counts = {}
        for backend in BACKENDS:
            candidate = state['candidate_counts'][backend]
            candidate_runs = state['confirmation_results'][backend]['candidate']
            candidate_accepted = (candidate is not None and len(candidate_runs) == 3
                                  and all(result['operational_pass'] and result['slo_pass']
                                          for result in candidate_runs))
            if candidate_accepted:
                fallback_counts[backend] = None
                continue
            lower_passing = [int(tenant_count) for tenant_count, result in state['screen_results'][backend].items()
                             if candidate is not None
                             if int(tenant_count) < candidate and result['operational_pass'] and result['slo_pass']]
            fallback_counts[backend] = max(lower_passing, default=None)
            if fallback_counts[backend] is None:
                record_omission(state, f'fallback-unavailable-{backend_suffix(backend)}',
                                'no_lower_passing_screen_for_single_fallback', 'fallback_confirmation')
        fallback_active = {backend: fallback_counts[backend] is not None for backend in BACKENDS}
        fallback_missed = {backend: False for backend in BACKENDS}
        for round_number in range(1, 4):
            first_backend = BACKENDS[(round_number - 1) % len(BACKENDS)]
            backend_order = [first_backend, *[backend for backend in BACKENDS if backend != first_backend]]
            for backend in backend_order:
                if not fallback_active[backend] or fallback_missed[backend]:
                    continue
                fallback = fallback_counts[backend]
                cell_case = case_name('fallback', fallback, backend, round_number)
                _, outcome = runner.run_cell(cell_case, backend, fallback,
                                             CONFIRMATION_DURATION_MS, 'fallback_confirmation', round_number)
                state['confirmation_results'][backend]['fallback'].append({
                    'case': cell_case,
                    'active_tenants': fallback,
                    'operational_pass': outcome['reduced']['operational_pass'],
                    'slo_pass': outcome['reduced']['slo_pass'],
                })
                if not outcome['reduced']['slo_pass']:
                    fallback_missed[backend] = True
                    for skipped_round in range(round_number + 1, 4):
                        record_omission(state, case_name('fallback', fallback, backend, skipped_round),
                                        'remaining_fallback_repetitions_omitted_after_first_slo_miss',
                                        'fallback_confirmation')
                write_json(run_directory / 'frontier.json', state)
        for backend in BACKENDS:
            state['decisions'][backend] = decide_route_acceptance(
                backend, {int(tenant_count): result for tenant_count, result in state['screen_results'][backend].items()},
                {
                    ('candidate', state['candidate_counts'][backend]): state['confirmation_results'][backend]['candidate'],
                    ('fallback', fallback_counts[backend]): state['confirmation_results'][backend]['fallback'],
                }, baseline_by_backend[backend])
        final_source_native = validate_current_source_native(validation_module)
        if final_source_native['source_commit'] != SOURCE_COMMIT or final_source_native['source_status'] != '':
            raise AssertionError('source changed during the resident frontier')
        if final_source_native['native_archive_sha256_before'] != EXPECTED_NATIVE_SHA256:
            raise AssertionError('native archive changed during the resident frontier')
        state['source_native_after'] = final_source_native
        state['terminal_status'] = 'completed'
        write_json(run_directory / 'frontier.json', state)
        final = {
            'completed': True,
            'source_commit': SOURCE_COMMIT,
            'design_plan_commit': plan_commit,
            'actual_controller_sha256': sha256_file(Path(__file__).resolve()),
            'cases': len(runner.outcomes),
            'decisions': state['decisions'],
            'processes': [
                {'pid': identity['pid'], 'role': identity['role'],
                 'start_time_ticks': identity['start_time_ticks'],
                 'same_live_process': capacity_runtime.process_identity_live(identity)}
                for identity in runner.all_processes
            ],
            'capacity_after': capacity_runtime.capacity(),
        }
        if len(runner.outcomes) > MAX_NEW_CELLS or any(process['same_live_process'] for process in final['processes']):
            raise AssertionError('resident frontier exceeded cell cap or left a child process running')
        state['terminal_status'] = 'completed'
        write_json(run_directory / 'frontier.json', state)
        write_json(run_directory / 'completion.json', final)
        return final
    except Exception as error:
        state['global_stop'] = {'type': type(error).__name__, 'message': str(error)}
        state['terminal_status'] = 'failed'
        all_scheduled = []
        attempted_cases = {outcome['case'] for outcome in runner.outcomes}
        for screen_group in state['screen_schedule']:
            for backend in screen_group['backend_order']:
                all_scheduled.append(case_name('screen', screen_group['active_tenants'], backend))
        for backend in BACKENDS:
            candidate = state['candidate_counts'].get(backend)
            if candidate is not None:
                for round_number in range(1, 4):
                    all_scheduled.append(case_name('candidate', candidate, backend, round_number))
        for case in all_scheduled:
            if case not in attempted_cases and not any(row['case'] == case for row in state['omitted_cases']):
                state['omitted_after_stop'].append({'case': case, 'reason': 'global_operational_stop'})
        for backend in BACKENDS:
            candidate = state['candidate_counts'].get(backend)
            if candidate is None:
                continue
            candidate_runs = state['confirmation_results'][backend]['candidate']
            candidate_accepted = (len(candidate_runs) == 3
                                  and all(result['operational_pass'] and result['slo_pass']
                                          for result in candidate_runs))
            if candidate_accepted:
                continue
            screen_counts = [int(tenant_count) for tenant_count, result in state['screen_results'][backend].items()
                             if int(tenant_count) < candidate and result['operational_pass'] and result['slo_pass']]
            fallback = max(screen_counts, default=None)
            if fallback is not None:
                for round_number in range(1, 4):
                    case = case_name('fallback', fallback, backend, round_number)
                    if case not in attempted_cases and not any(row['case'] == case for row in state['omitted_cases']):
                        state['omitted_after_stop'].append({'case': case, 'reason': 'global_operational_stop'})
        state['decisions'] = {}
        write_json(run_directory / 'frontier.json', state)
        failure = {
            'completed': False,
            'source_commit': SOURCE_COMMIT,
            'design_plan_commit': plan_commit,
            'actual_controller_sha256': sha256_file(Path(__file__).resolve()),
            'cases_attempted': len(runner.outcomes),
            'failure': state['global_stop'],
            'outcomes': runner.outcomes,
            'processes': [
                {'pid': identity['pid'], 'role': identity['role'],
                 'start_time_ticks': identity['start_time_ticks'],
                 'same_live_process': capacity_runtime.process_identity_live(identity)}
                for identity in runner.all_processes
            ],
            'capacity_after': capacity_runtime.capacity(),
        }
        try:
            failure['source_native_after'] = validate_current_source_native(validation_module)
        except Exception as source_error:
            failure['source_native_after_error'] = f'{type(source_error).__name__}:{source_error}'
        write_json(run_directory / 'failure-receipt.json', failure)
        raise


def main():
    parser = argparse.ArgumentParser(description='Run the bounded staggered resident-population frontier')
    parser.add_argument('--plan-commit', required=True)
    parser.add_argument('--controls-review', type=Path, required=True)
    parser.add_argument('--controls-review-sha256', required=True)
    arguments = parser.parse_args()
    if arguments.plan_commit != DESIGN_PLAN_COMMIT:
        raise AssertionError('design plan commit differs from the frozen plan')
    if os.geteuid() != 0:
        raise AssertionError('resident frontier requires the designated root OCI harness')
    controls = validate_controls_review(arguments.controls_review, arguments.controls_review_sha256)
    try:
        result = run_frontier(arguments.plan_commit, controls)
    except Exception:
        raise
    print(json.dumps({'completed': result['completed'], 'cases': result['cases'],
                      'decisions': result['decisions']}, sort_keys=True), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
