import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import sys


ROOT = Path('/home/opc/tenant-density-harness-20261027')
RUN_NAME = 'maintenance-cb0e04482-r1'
RUN_DIRECTORY = ROOT / 'arrival-phase' / RUN_NAME
DATABASE_ROOT = Path('/bench/zfs/db/tenant-density-harness-20261027') / RUN_NAME
CGROUP_ROOT = Path('/sys/fs/cgroup') / ('dodb-density-' + RUN_NAME)
SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
PLAN_COMMIT = 'a01f6e2d7ce3411366bf9b97a9a3addf4b12e614'
FRONTIER_REVIEW_SHA256 = '938d17f0f0334f8a1b7731dae5e155b59017c73bcae4dd1c79aa5cbcb4b7828e'
CONTROLS_REVIEW_SHA256 = 'b90ba441a9f66bf65809050db69cc802faa7516a3defc75978592de6dd2b9193'
HELPERS = {
    'resident-frontier/oci_resident_frontier.py': 'fbb07d16dc5614db8a596f199940144ea0b2ba19f2e0cfbf70c34b702481e8aa',
    'resident-frontier/resident_frontier_review.py': '6bd65facde7695c8e1357f32eae1a0252ac352449fa4c298a1e042b6439a0ee5',
    'capacity/capacity_review.py': 'd03cd663fa4dd4a1b2f87d716937ba8566d7c205032286853d876f416333ad57',
}
BACKENDS = ['blink', 'minimal-native-shared']
DURATION_MS = 300000
ACTIVE_TENANTS = 24


def digest(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def write_json(path, value):
    path = Path(path)
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')
    temporary.replace(path)


def load_module(name, path):
    sys.path.insert(0, str(Path(path).parent))
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[name] = module
    specification.loader.exec_module(module)
    return module


def dependencies():
    evidence = Path(__file__).resolve().parent.parent
    paths = {}
    for relative_path, expected_hash in HELPERS.items():
        path = evidence / relative_path
        if not path.is_file() or digest(path) != expected_hash:
            raise AssertionError(f'frozen helper differs: {relative_path}')
        paths[relative_path] = path
    frontier = load_module('maintenance_frontier', paths['resident-frontier/oci_resident_frontier.py'])
    reviewer = load_module('maintenance_frontier_reviewer', paths['resident-frontier/resident_frontier_review.py'])
    capacity_review = load_module('maintenance_capacity_review', paths['capacity/capacity_review.py'])
    return frontier, reviewer, capacity_review, paths


def sequence():
    cells = []
    for repetition in range(1, 4):
        backend_order = BACKENDS if repetition % 2 else list(reversed(BACKENDS))
        for backend in backend_order:
            suffix = 'blink' if backend == 'blink' else 'shared'
            cells.append({'case': f'maintenance-n24-r{repetition}-{suffix}',
                          'backend': backend, 'repetition': repetition})
    return cells


def expected_cell(case):
    match = re.fullmatch(r'maintenance-n24-r([1-3])-(blink|shared)', case)
    if not match:
        raise AssertionError(f'cell is outside the frozen maintenance matrix: {case}')
    return {'kind': 'maintenance', 'active_tenants': ACTIVE_TENANTS,
            'duration_ms': DURATION_MS,
            'backend': 'blink' if match.group(2) == 'blink' else 'minimal-native-shared'}


def configure_reviewer(reviewer):
    reviewer.expected_cell = expected_cell
    reviewer.RUN_NAME = RUN_NAME
    reviewer.RUN_DIRECTORY = RUN_DIRECTORY
    reviewer.DATABASE_ROOT = DATABASE_ROOT
    reviewer.CGROUP_ROOT = CGROUP_ROOT
    return reviewer


def maintenance_metrics(report, capacity_review):
    reduced = capacity_review.reduce_report(report, SOURCE_COMMIT)
    health = report['memory_diagnostic']['engine_health_reports'][1]
    checkpoints = health['blink_checkpoint_metrics']
    pressure_samples = [sample['snapshot'] for sample in report['backend_memory_pressure']['samples']
                        if sample['epoch'] == 'measurement' and sample['event'] == 'periodic']
    checkpoint_counts = [sample['blink_checkpoints']['checkpoints_completed']
                         for sample in health['periodic_pressure']['samples']
                         if isinstance(sample.get('blink_checkpoints'), dict)
                         and 'checkpoints_completed' in sample['blink_checkpoints']]
    result = {
        'worst_tenant_p99_upper_exclusive_ms': reduced['worst_tenant_p99_upper_exclusive_ms'],
        'cell_peak_bytes': reduced['cell_peak_bytes'],
        'epoch_peaks_bytes': reduced['epoch_peaks_bytes'],
        'cell_events': reduced['cell_events'],
        'measurement_sample_count': reduced['measurement_sample_count'],
        'maximum_measurement_sample_gap_ms': reduced['maximum_measurement_sample_gap_ms'],
        'native_serialized_bytes': reduced['native_serialized_bytes'],
        'completed_at_offer_end': reduced['completed_at_offer_end'],
        'offer_end_pending_requests': reduced['scheduled'] - reduced['completed_at_offer_end'],
        'offer_end_queued': sum(tenant['queued'] for tenant in report['workload']['offer_boundary_accounting']),
        'offer_end_in_flight': sum(tenant['in_flight'] for tenant in report['workload']['offer_boundary_accounting']),
        'filesystem_snapshots': reduced['filesystem_snapshots'],
        'backend_offer_io_delta': reduced['backend_offer_io_delta'],
        'whole_device_offer_write_bytes': reduced['whole_device_offer_write_bytes'],
        'first_last_periodic_measurement_memory_current':
            [pressure_samples[0]['memory.current'], pressure_samples[-1]['memory.current']]
            if pressure_samples else [],
        'checkpoint_counter_scope': health['blink_checkpoint_metrics_scope'],
        'checkpoints_completed': checkpoints.get('checkpoints_completed'),
        'checkpoints_failed': checkpoints.get('checkpoints_failed'),
        'writes_during_checkpoint': checkpoints.get('writes_during_checkpoint'),
        'periodic_checkpoint_count_increase':
            checkpoint_counts[-1] - checkpoint_counts[0] if checkpoint_counts else None,
        'shared_resource_snapshots': health['shared_resource_snapshots'],
        'reference_cycle_count': {'status': 'unavailable',
                                  'reason': 'no cumulative flush/compaction count in this frozen report'},
    }
    result['blink_multiple_cycles_observed'] = (
        result['periodic_checkpoint_count_increase'] >= 2 if checkpoint_counts else None)
    return result


def validate_entry(frontier, reviewer, capacity_review):
    prior_directory = ROOT / 'arrival-phase' / 'resident-frontier-cb0e04482-r2'
    prior_path = prior_directory / 'review-r2.json'
    if digest(prior_path) != FRONTIER_REVIEW_SHA256:
        raise AssertionError('canonical frontier review differs')
    prior = read_json(prior_path)
    if prior.get('completed_run_evidence_valid') is not True or prior.get('review_valid') is not True:
        raise AssertionError('prior frontier is not operationally valid')
    if {backend: value['accepted_count'] for backend, value in prior['decisions'].items()} != {
            'blink': 24, 'minimal-native-shared': 28}:
        raise AssertionError('accepted populations differ from maintenance entry design')
    recomputed = reviewer.review_directory(prior_directory, prior_path, capacity_review)
    if recomputed != prior:
        raise AssertionError('prior frontier no longer matches its preserved receipt')
    controls_path = ROOT / 'resident-frontier' / '714320187' / 'final-review.json'
    controls = frontier.validate_controls_review(controls_path, CONTROLS_REVIEW_SHA256)
    prior_provenance = read_json(prior_directory / 'provenance.json')
    for process in read_json(prior_directory / 'completion.json')['processes']:
        process_path = Path('/proc') / str(process['pid']) / 'stat'
        try:
            raw = process_path.read_text()
            if int(raw[raw.rfind(')') + 2:].split()[19]) == process['start_time_ticks']:
                raise AssertionError('prior workload identity is still live')
        except FileNotFoundError:
            pass
    controls_directory = ROOT / 'arrival-phase' / frontier.CONTROLS_RUN
    for group_name in [frontier.CONTROLS_RUN, frontier.RUN_NAME]:
        group = Path('/sys/fs/cgroup') / ('dodb-density-' + group_name)
        for process_list in group.rglob('cgroup.procs'):
            if process_list.read_text().strip():
                raise AssertionError(f'prior group is populated: {process_list.parent}')
    for process in read_json(controls_directory / 'completion.json')['processes']:
        process_path = Path('/proc') / str(process['pid']) / 'stat'
        try:
            raw = process_path.read_text()
            if int(raw[raw.rfind(')') + 2:].split()[19]) == process['start_time_ticks']:
                raise AssertionError('prior controls identity is still live')
        except FileNotFoundError:
            pass
    return controls, prior_provenance


def review_run(run_directory, reviewer, capacity_review, capacity_module):
    run_directory = Path(run_directory)
    provenance = read_json(run_directory / 'provenance.json')
    if provenance['source_commit'] != SOURCE_COMMIT or provenance['plan_commit'] != PLAN_COMMIT:
        raise AssertionError('maintenance provenance differs')
    summary = read_json(run_directory / 'summary.json') if (run_directory / 'summary.json').is_file() else []
    planned = sequence()
    if [item['case'] for item in summary] != [item['case'] for item in planned[:len(summary)]]:
        raise AssertionError('maintenance execution order differs')
    cells = {}
    failures = []
    reports = {}
    for outcome in summary:
        case = outcome['case']
        try:
            reviewed = reviewer.review_cell(run_directory, case, capacity_review)
            report = read_json(run_directory / (case + '.json'))
            reviewed['maintenance'] = maintenance_metrics(report, capacity_review)
            cells[case] = reviewed
            reports[case] = report
        except Exception as error:
            failures.append({'case': case, 'error': f'{type(error).__name__}: {error}'})
    terminal_path = run_directory / 'completion.json'
    completed = terminal_path.is_file()
    if not completed:
        terminal_path = run_directory / 'failure-receipt.json'
    terminal = read_json(terminal_path)
    process_records = terminal['processes']
    live = [record for record in process_records if capacity_module.process_identity_live(record)]
    pairs = {}
    for repetition in range(1, 4):
        left = f'maintenance-n24-r{repetition}-blink'
        right = f'maintenance-n24-r{repetition}-shared'
        if left in reports and right in reports:
            pairs[str(repetition)] = capacity_review.compare_pair(reports[left], reports[right])
    all_complete = completed and len(cells) == 6 and not failures and not live and len(pairs) == 3
    route_passes = {backend: sum(cell['slo_pass'] for cell in cells.values()
                                 if cell['backend'] == backend) for backend in BACKENDS}
    coverage = [cell['maintenance']['blink_multiple_cycles_observed']
                for cell in cells.values() if cell['backend'] == 'blink']
    return {'source_commit': SOURCE_COMMIT, 'plan_commit': PLAN_COMMIT,
            'provenance_sha256': digest(run_directory / 'provenance.json'),
            'completed_run_evidence_valid': all_complete,
            'accepted_run': all_complete and all(count == 3 for count in route_passes.values()),
            'slo_passes_by_backend': route_passes,
            'blink_multiple_cycle_coverage': len(coverage) == 3 and all(coverage),
            'operationally_valid_cell_count': len(cells), 'failures': failures,
            'successful_requests': sum(cell['successful'] for cell in cells.values()),
            'oracle_count': 2 * len(cells), 'cells': cells, 'pairs': pairs,
            'terminal_sha256': digest(terminal_path), 'process_records': process_records,
            'all_recorded_processes_exited': not live,
            'omitted_cases': planned[len(summary):]}


def run(preflight_only=False):
    if os.geteuid() != 0:
        raise AssertionError('maintenance requires the designated root OCI harness')
    frontier, reviewer, capacity_review, helper_paths = dependencies()
    controls, prior_provenance = validate_entry(frontier, reviewer, capacity_review)
    frozen_paths = frontier.controls_helper_paths(controls, Path(__file__).resolve().parent)
    controls_module = load_module('maintenance_controls', frozen_paths['oci_arrival_controls.py'])
    capacity_module = load_module('maintenance_capacity', frozen_paths['oci_capacity.py'])
    validation_module = load_module('maintenance_validation', frozen_paths['oci_arrival_validate.py'])
    before_source = frontier.validate_current_source_native(validation_module)
    if (before_source['source_commit'] != SOURCE_COMMIT or before_source['source_status'] != ''
            or before_source['native_archive_sha256_before'] != frontier.EXPECTED_NATIVE_SHA256):
        raise AssertionError('frozen source/native provenance differs')
    nofile = controls_module.set_controller_nofile_limit()
    nofile['preflight_child'] = controls_module.verify_preflight_child_limits(nofile['proc_hard'])
    validation_path = ROOT / 'arrival-phase' / frontier.VALIDATION_RUN / 'arrival-validation-summary.json'
    frontier.prepare_validated_binaries(controls['review']['validation'], validation_path, None)
    capacity_runtime = frontier.CapacityRuntime(capacity_module)
    capacity_before = capacity_runtime.capacity()
    capacity_runtime.guard(capacity_before, before=True)
    if DATABASE_ROOT.exists() or CGROUP_ROOT.exists() or RUN_DIRECTORY.exists():
        raise AssertionError('maintenance namespace already exists; refusing replay')
    if preflight_only:
        print(json.dumps({'preflight_passed': True, 'source_native': before_source,
                          'capacity': capacity_before, 'runtime_limits': nofile,
                          'prior_source_commit': prior_provenance['source_commit']}, sort_keys=True), flush=True)
        return 0
    RUN_DIRECTORY.mkdir()
    DATABASE_ROOT.mkdir()
    capacity_runtime.prepare_group(CGROUP_ROOT)
    binaries, validation = frontier.prepare_validated_binaries(
        controls['review']['validation'], validation_path, RUN_DIRECTORY)
    configure_reviewer(reviewer)
    runner = frontier.ResidentRunner(controls_module, capacity_runtime, reviewer,
                                    {'runtime_limits': nofile, 'source_commit': SOURCE_COMMIT},
                                    RUN_DIRECTORY, DATABASE_ROOT, CGROUP_ROOT, binaries)
    runner.plan_commit = PLAN_COMMIT
    provenance = {'source_commit': SOURCE_COMMIT, 'plan_commit': PLAN_COMMIT,
                  'run_name': RUN_NAME, 'controller_sha256': digest(__file__),
                  'helpers': {str(path): digest(path) for path in [*helper_paths.values(), *frozen_paths.values()]},
                  'validation': validation, 'validated_binaries': binaries,
                  'source_native_before': before_source, 'runtime_limits': nofile,
                  'capacity_before': capacity_before, 'sequence': sequence(),
                  'duration_ms': DURATION_MS, 'frontier_review_sha256': FRONTIER_REVIEW_SHA256,
                  'controls_review_sha256': CONTROLS_REVIEW_SHA256,
                  'filesystem_properties': capacity_module.command([
                      'zfs', 'get', '-Hp', '-o', 'property,value',
                      'used,available,recordsize,compression,atime,sync', 'dodbbench/db']),
                  'processes_before': capacity_module.command([
                      'ps', '-eo', 'pid,ppid,uid,pcpu,rss,args', '--sort=-pcpu'])}
    write_json(RUN_DIRECTORY / 'provenance.json', provenance)

    def request_stop(signum, frame):
        runner.stop_reason = f'parent_signal_{signum}'

    signal.signal(signal.SIGTERM, request_stop)
    signal.signal(signal.SIGINT, request_stop)
    failure = None
    paired_reports = {}
    try:
        for cell in sequence():
            report, outcome = controls_module.ArrivalExperiment.run_cell(
                runner, cell['case'], 'staggered', cell['backend'],
                ACTIVE_TENANTS, ACTIVE_TENANTS, 0, DURATION_MS)
            reviewer.parse_command(read_json(RUN_DIRECTORY / (cell['case'] + '-command.json')), cell['case'])
            paired_reports.setdefault(cell['repetition'], {})[cell['backend']] = report
            pair = paired_reports[cell['repetition']]
            if all(backend in pair for backend in BACKENDS):
                capacity_review.compare_pair(pair[BACKENDS[0]], pair[BACKENDS[1]])
            print(json.dumps({'case': cell['case'], 'operational_pass': True,
                              'slo_pass': outcome['reduced']['slo_pass']}, sort_keys=True), flush=True)
        after_source = frontier.validate_current_source_native(validation_module)
        if (after_source['source_commit'] != SOURCE_COMMIT or after_source['source_status'] != ''
                or after_source['native_archive_sha256_before'] != frontier.EXPECTED_NATIVE_SHA256):
            raise AssertionError('source/native changed during maintenance')
    except Exception as error:
        failure = {'type': type(error).__name__, 'message': str(error)}
    unique_processes = {(record['pid'], record['start_time_ticks']): record for record in runner.all_processes}
    processes = [{**record, 'same_live_process': capacity_module.process_identity_live(record)}
                 for record in unique_processes.values()]
    if any(record['same_live_process'] for record in processes):
        failure = {'type': 'LiveProcess', 'message': 'recorded workload identity remains live'}
    terminal = {'completed': failure is None, 'failure': failure, 'processes': processes,
                'capacity_after': capacity_runtime.capacity(), 'cases_attempted': len(runner.outcomes)}
    write_json(RUN_DIRECTORY / ('failure-receipt.json' if failure else 'completion.json'), terminal)
    review = review_run(RUN_DIRECTORY, reviewer, capacity_review, capacity_module)
    write_json(RUN_DIRECTORY / 'final-review.json', review)
    print(json.dumps({key: value for key, value in review.items()
                      if key not in ['cells', 'process_records', 'pairs']}, sort_keys=True), flush=True)
    return 1 if failure or not review['completed_run_evidence_valid'] else 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Run the frozen N24 maintenance comparison')
    parser.add_argument('--preflight-only', action='store_true')
    arguments = parser.parse_args()
    raise SystemExit(run(arguments.preflight_only))
