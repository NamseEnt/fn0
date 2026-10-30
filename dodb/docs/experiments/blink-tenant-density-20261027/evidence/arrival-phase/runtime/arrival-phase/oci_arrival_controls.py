import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource
import shutil
import signal
import subprocess
import sys
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
RUN_NAME = 'arrival-controls-cb0e04482-r1'
SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
EXPECTED_PLAN_COMMIT = 'e02a1fbdde81fd6fc363a26e5341c8c61000023e'
VALIDATION_RUN = 'arrival-cb0e04482-r1'
EXPECTED_NATIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
NOFILE_SOFT_TARGET = 65536
PRESSURE_INTERVAL_MS = 5000
CONTROLLER_TIMEOUT_SECONDS = 1200
PARENT_STOP_WATCHDOG_MS = 20000
CONTROLLER_CHILD_STOP_WAIT_SECONDS = 5


def digest(file_path):
    value = hashlib.sha256()
    with Path(file_path).open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def write_json(file_path, value):
    temporary_path = file_path.with_suffix(file_path.suffix + '.tmp')
    temporary_path.write_text(json.dumps(value, indent=2) + '\n')
    temporary_path.replace(file_path)


def load_module(name, file_path):
    module_path = Path(file_path)
    sys.path.insert(0, str(module_path.parent))
    specification = importlib.util.spec_from_file_location(name, module_path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[name] = module
    specification.loader.exec_module(module)
    return module


def parse_proc_limits(pid):
    limits_path = Path('/proc') / str(pid) / 'limits'
    limits = {}
    for line in limits_path.read_text().splitlines()[1:]:
        fields = line.split()
        if fields[:2] == ['Max', 'open']:
            limits['soft'] = int(fields[3]) if fields[3] != 'unlimited' else None
            limits['hard'] = int(fields[4]) if fields[4] != 'unlimited' else None
            limits['units'] = fields[5] if len(fields) > 5 else 'files'
            break
    if 'soft' not in limits:
        raise AssertionError(f'RLIMIT_NOFILE missing from /proc/{pid}/limits')
    return limits


def proc_identity(pid):
    raw_stat = (Path('/proc') / str(pid) / 'stat').read_text()
    fields = raw_stat[raw_stat.rfind(')') + 2:].split()
    return {'pid': pid, 'parent_pid': int(fields[1]), 'start_time_ticks': int(fields[19])}


def process_tree(root_pid):
    identities = {}
    for process_path in Path('/proc').iterdir():
        if process_path.name.isdigit():
            try:
                identity = proc_identity(int(process_path.name))
                identities[identity['pid']] = identity
            except (FileNotFoundError, ProcessLookupError, PermissionError, ValueError):
                continue
    owned = {root_pid}
    selected = []
    while True:
        children = [identity for identity in identities.values()
                    if identity['parent_pid'] in owned and identity['pid'] not in owned]
        if not children:
            break
        selected.extend(children)
        owned.update(identity['pid'] for identity in children)
    return [identities[root_pid], *selected] if root_pid in identities else selected


def set_controller_nofile_limit():
    old_soft, hard_limit = resource.getrlimit(resource.RLIMIT_NOFILE)
    proc_limits_before = parse_proc_limits(os.getpid())
    if hard_limit != resource.RLIM_INFINITY and hard_limit < NOFILE_SOFT_TARGET:
        raise AssertionError(f'RLIMIT_NOFILE hard limit is below {NOFILE_SOFT_TARGET}: {hard_limit}')
    resource.setrlimit(resource.RLIMIT_NOFILE, (NOFILE_SOFT_TARGET, hard_limit))
    new_soft, new_hard = resource.getrlimit(resource.RLIMIT_NOFILE)
    assert new_soft == NOFILE_SOFT_TARGET and new_hard == hard_limit
    return {'controller_pid': os.getpid(), 'original_soft': old_soft,
            'original_hard': None if hard_limit == resource.RLIM_INFINITY else hard_limit,
            'new_soft': new_soft, 'new_hard': new_hard,
            'proc_hard': None if hard_limit == resource.RLIM_INFINITY else hard_limit,
            'proc_limits_before': proc_limits_before,
            'controller_proc_limits': parse_proc_limits(os.getpid())}


def validate_inherited_nofile_limits(limits, expected_hard):
    assert limits == {'soft': NOFILE_SOFT_TARGET, 'hard': expected_hard, 'units': 'files'}
    return True


def verify_preflight_child_limits(expected_hard):
    child = subprocess.Popen(
        [sys.executable, '-c', 'import time; time.sleep(0.5)'],
        stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        start_new_session=True)
    identity = None
    try:
        identity = proc_identity(child.pid)
        assert identity['parent_pid'] == os.getpid()
        limits = parse_proc_limits(child.pid)
        validate_inherited_nofile_limits(limits, expected_hard)
        exit_code = child.wait(timeout=2)
        assert exit_code == 0
        try:
            current_identity = proc_identity(child.pid)
            same_live_process = current_identity['start_time_ticks'] == identity['start_time_ticks']
        except (FileNotFoundError, ProcessLookupError):
            same_live_process = False
        assert not same_live_process
        return {'role': 'preflight-limit-child', **identity,
                'limits': limits, 'exit_code': exit_code,
                'same_live_process_after_exit': same_live_process}
    except Exception:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=5)
        raise


def validation_contract(validation, validator_module):
    assert validation['status'] == 'completed'
    assert validation['source_commit'] == SOURCE_COMMIT
    assert validation['source_status'] == ''
    assert validation['outcomes'][0]['passing_tests'] == 85
    assert validation['outcomes'][1]['same_17_existing_clippy_diagnostics'] is True
    assert validation['outcomes'][0]['exit_code'] == validation['outcomes'][1]['exit_code'] == 0
    assert validation['outcomes'][2]['name'] == 'a1-link' and validation['outcomes'][2]['exit_code'] == 0
    assert validation['outcomes'][3]['name'] == 'default-link' and validation['outcomes'][3]['exit_code'] == 0
    assert validation['outcomes'][-1]['name'] == 'default-link'
    assert validation['default_link_was_last'] is True
    assert validation['native_archive_sha256_before'] == EXPECTED_NATIVE_SHA256
    assert validation['native_archive_sha256_after'] == EXPECTED_NATIVE_SHA256
    assert validation['native_archive_bytes_before'] == validation['native_archive_bytes_after']
    assert validation['target_symlink_before'] == validation['target_symlink_after']
    assert validation['resolved_target_directory_before'] == validation['resolved_target_directory_after']
    assert set(validation['default_binaries']) == {'tenant_density_workload', 'memory_diagnostic_worker'}
    for binary_name, identity in validation['default_binaries'].items():
        source_binary = ROOT / 'arrival-phase' / VALIDATION_RUN / 'bin' / binary_name
        assert source_binary.is_file()
        assert source_binary.stat().st_size == identity['bytes']
        assert digest(source_binary) == identity['sha256']
    return validation


def read_completed_validation(validator_module):
    summary_path = ROOT / 'arrival-phase' / VALIDATION_RUN / 'arrival-validation-summary.json'
    if not summary_path.is_file():
        raise AssertionError(f'completed arrival validation summary is missing: {summary_path}')
    validation = json.loads(summary_path.read_text())
    validation['summary_sha256'] = digest(summary_path)
    validation['current_source_native'] = validator_module.validate_source_and_native()
    return validation_contract(validation, validator_module)


def capacity_controller_path():
    remote_helper = ROOT / 'oci_capacity.py'
    if remote_helper.is_file():
        return remote_helper
    local_helper = Path(__file__).resolve().parents[1] / 'capacity' / 'oci_capacity.py'
    if local_helper.is_file():
        return local_helper
    raise AssertionError('existing capacity controller helper is unavailable')


def copy_validated_binaries(validation, runtime_directory):
    source_directory = ROOT / 'arrival-phase' / VALIDATION_RUN / 'bin'
    destination_directory = runtime_directory / 'bin'
    destination_directory.mkdir()
    copied = {}
    for binary_name, identity in validation['default_binaries'].items():
        source_binary = source_directory / binary_name
        destination_binary = destination_directory / binary_name
        with source_binary.open('rb') as source_file, destination_binary.open('xb') as destination_file:
            shutil.copyfileobj(source_file, destination_file, 1024 * 1024)
        shutil.copystat(source_binary, destination_binary)
        assert destination_binary.stat().st_size == identity['bytes']
        assert digest(destination_binary) == identity['sha256']
        copied[binary_name] = {'bytes': identity['bytes'], 'sha256': identity['sha256']}
    return copied


def cell_sequence():
    pair_specs = [
        ('tiny-synchronized', 'synchronized', 4, 3, 1, 10000),
        ('tiny-staggered', 'staggered', 4, 3, 1, 10000),
        ('control-n28-r1-synchronized', 'synchronized', 28, 28, 0, 60000),
        ('control-n28-r1-staggered', 'staggered', 28, 28, 0, 60000),
        ('control-n28-r2-staggered', 'staggered', 28, 28, 0, 60000),
        ('control-n28-r2-synchronized', 'synchronized', 28, 28, 0, 60000),
        ('control-n28-r3-synchronized', 'synchronized', 28, 28, 0, 60000),
        ('control-n28-r3-staggered', 'staggered', 28, 28, 0, 60000),
    ]
    for pair_number, (case, mode, registered, active, idle, duration_ms) in enumerate(pair_specs, start=1):
        backends = ['blink', 'minimal-native-shared']
        if pair_number % 2 == 0:
            backends.reverse()
        yield {'case': case, 'mode': mode, 'registered': registered, 'active': active,
               'idle': idle, 'duration_ms': duration_ms, 'pair_number': pair_number,
               'backends': backends}


class ArrivalExperiment:
    def __init__(self, plan_commit, capacity_module, reviewer_module, validation,
                 nofile_identity, frozen_binaries):
        assert plan_commit == EXPECTED_PLAN_COMMIT
        self.capacity = capacity_module
        self.reviewer = reviewer_module
        self.validation = validation
        self.plan_commit = plan_commit
        self.nofile_identity = nofile_identity
        self.frozen_binaries = frozen_binaries
        self.run_directory = ROOT / 'arrival-phase' / RUN_NAME
        self.database_directory = Path('/bench/zfs/db/tenant-density-harness-20261027') / RUN_NAME
        self.group_root = Path('/sys/fs/cgroup') / ('dodb-density-' + RUN_NAME)
        self.source_commit = SOURCE_COMMIT
        self.all_processes = []
        self.outcomes = []
        self.pair_reports = {}
        self.phase_comparisons = {}
        self.stop_reason = None
        assert not self.run_directory.exists(), f'preserved runtime namespace already exists: {self.run_directory}'
        self.run_directory.mkdir(parents=True)
        self.binaries = self.run_directory / 'bin'
        try:
            self.database_directory.mkdir(parents=True, exist_ok=False)
            self.capacity.prepare_group(self.group_root)
            copied = copy_validated_binaries(validation, self.run_directory)
            assert copied == frozen_binaries
            self.capacity_before = self.capacity.capacity()
            self.capacity.guard(self.capacity_before, before=True)
            self._write_provenance(copied)
        except Exception as error:
            receipt = {'completed': False, 'source_commit': SOURCE_COMMIT,
                       'plan_commit': self.plan_commit, 'runtime_uid': os.geteuid(),
                       'initialization_failure': {'type': type(error).__name__,
                                                  'message': str(error)},
                       'validation_sha256': validation.get('summary_sha256'),
                       'source_validation': validation.get('current_source_native'),
                       'runtime_limits': nofile_identity}
            try:
                receipt['capacity_at_failure'] = self.capacity.capacity()
            except Exception as capacity_error:
                receipt['capacity_error'] = f'{type(capacity_error).__name__}:{capacity_error}'
            self.capacity.write_json(self.run_directory / 'initialization-failure-receipt.json', receipt)
            raise

    def _write_provenance(self, copied):
        provenance = {
            'run_name': RUN_NAME,
            'source_commit': SOURCE_COMMIT,
            'plan_commit': self.plan_commit,
            'runtime_uid': os.geteuid(),
            'source_validation': self.validation['current_source_native'],
            'validation_sha256': digest(ROOT / 'arrival-phase' / VALIDATION_RUN / 'arrival-validation-summary.json'),
            'arrival_validator_sha256': self.validation['controller_sha256'],
            'build_helper_sha256': self.validation['build_helper_sha256'],
            'validation': {
                'passing_tests': self.validation['outcomes'][0]['passing_tests'],
                'same_17_existing_clippy_diagnostics': self.validation['outcomes'][1]['same_17_existing_clippy_diagnostics'],
                'default_link_was_last': self.validation['default_link_was_last'],
                'native_commit': self.validation['native_commit'],
                'native_archive_bytes': self.validation['native_archive_bytes_before'],
                'native_archive_sha256_before': self.validation['native_archive_sha256_before'],
                'native_archive_sha256_after': self.validation['native_archive_sha256_after'],
                'default_binaries': copied,
            },
            'controller_sha256': digest(Path(__file__)),
            'reviewer_sha256': digest(Path(__file__).with_name('arrival_review.py')),
            'capacity_controller_sha256': self.capacity.digest(Path(self.capacity.__file__)),
            'capacity_reviewer_sha256': self.capacity.digest(Path(self.capacity.__file__).with_name('capacity_review.py')),
            'runtime_limits': self.nofile_identity,
            'capacity_before': self.capacity_before,
            'kernel': os.uname().release,
            'hostname': os.uname().nodename,
            'online_cpus': os.cpu_count(),
            'filesystem_properties': self.capacity.command([
                'zfs', 'get', '-Hp', '-o', 'property,value',
                'used,available,recordsize,compression,atime,sync', 'dodbbench/db']),
            'processes_before': self.capacity.command([
                'ps', '-eo', 'pid,ppid,uid,pcpu,rss,args', '--sort=-pcpu']),
            'policy': {
                'pressure_sample_interval_ms': PRESSURE_INTERVAL_MS,
                'backend_memory_max_bytes': 256 * 1024**2,
                'backend_memory_high_bytes': 192 * 1024**2,
                'backend_memory_swap_max_bytes': 0,
                'request_timeout_ms': 5000,
                'drain_timeout_ms': 15000,
                'internal_shutdown_timeout_ms': 15000,
                'parent_stop_watchdog_ms': PARENT_STOP_WATCHDOG_MS,
                'controller_child_stop_wait_seconds': CONTROLLER_CHILD_STOP_WAIT_SECONDS,
                'controller_timeout_seconds': CONTROLLER_TIMEOUT_SECONDS,
                'arrival_modes': ['synchronized', 'staggered'],
                'cells': 16,
                'planned_requests': 1014000,
                'full_population_oracles': 32,
            },
            'scope': 'E2 tiny correctness gates and matched N28 arrival-phase controls; no frontier or adoption claim',
        }
        self.capacity.write_json(self.run_directory / 'provenance.json', provenance)

    def _read_owned_limits(self, pid):
        try:
            raw_stat = (Path('/proc') / str(pid) / 'stat').read_text()
            start_time_ticks = int(raw_stat[raw_stat.rfind(')') + 2:].split()[19])
            limits = parse_proc_limits(pid)
            validate_inherited_nofile_limits(limits, self.nofile_identity['proc_hard'])
            return {'pid': pid, 'start_time_ticks': start_time_ticks, 'limits': limits}
        except (FileNotFoundError, ProcessLookupError):
            return None

    def run_cell(self, case, mode, backend, registered, active, idle, duration_ms):
        before = self.capacity.capacity()
        self.capacity.guard(before, before=True)
        assert not any(self.capacity.process_identity_live(identity) for identity in self.all_processes)
        report_path = self.run_directory / (case + '.json')
        group_path = self.group_root / case
        self.capacity.prepare_group(group_path)
        initial_rows = 2048
        arguments = [str(self.binaries / 'tenant_density_workload'), '--backend', backend,
                     '--data-dir', str(self.database_directory / case), '--report', str(report_path),
                     '--registered-tenants', str(registered), '--active-tenants', str(active),
                     '--opened-idle-tenants', str(idle), '--initial-rows', str(initial_rows),
                     '--duration-ms', str(duration_ms), '--pressure-sample-interval-ms', str(PRESSURE_INTERVAL_MS),
                     '--backend-cgroup-root', str(group_path), *self.capacity.BASE_ARGUMENTS,
                     '--tenant-arrival-phase', mode, *self.capacity.BACKEND_ARGUMENTS[backend]]
        command_record = {'command': arguments, 'capacity_before': before,
                          'source_commit': SOURCE_COMMIT, 'mode': mode,
                          'started_unix_seconds': time.time(), 'stage': 'E2'}
        self.capacity.write_json(self.run_directory / (case + '-command.json'), command_record)
        environment = os.environ.copy()
        environment['DODB_SOURCE_REVISION'] = SOURCE_COMMIT
        samples = []
        process_limits = []
        stop_reason = None
        execution_error = None
        child = None
        started = time.monotonic()
        try:
            with (self.run_directory / (case + '.log')).open('x') as output:
                child = subprocess.Popen(arguments, cwd=SOURCE, env=environment, stdout=output,
                                         stderr=subprocess.STDOUT, start_new_session=True)
                inherited = self._read_owned_limits(child.pid)
                if inherited is None:
                    raise AssertionError(f'owned workload process exited before its inherited limits were verified: {case}')
                self.all_processes.append({'pid': child.pid, 'role': 'owned-workload-process',
                                           'start_time_ticks': inherited['start_time_ticks']})
                process_limits.append({'role': 'owned-workload-process', **inherited})
                while child.poll() is None:
                    state = self.capacity.capacity()
                    samples.append(state)
                    for identity in process_tree(child.pid):
                        observed = self._read_owned_limits(identity['pid'])
                        if not observed or observed['start_time_ticks'] != identity['start_time_ticks']:
                            continue
                        process_key = (identity['pid'], identity['start_time_ticks'])
                        if not any((known['pid'], known['start_time_ticks']) == process_key
                                   for known in self.all_processes):
                            self.all_processes.append({'pid': identity['pid'],
                                                       'role': 'owned-descendant',
                                                       'start_time_ticks': identity['start_time_ticks']})
                        process_limits.append({'role': 'owned-descendant', **observed})
                    for identity in self.all_processes:
                        observed = self._read_owned_limits(identity['pid'])
                        if observed and observed['start_time_ticks'] == identity['start_time_ticks']:
                            process_limits.append({'role': identity['role'], **observed})
                    self.capacity.guard(state)
                    if time.monotonic() - started >= CONTROLLER_TIMEOUT_SECONDS:
                        stop_reason = 'controller_timeout'
                        break
                    if self.stop_reason is not None:
                        stop_reason = self.stop_reason
                        break
                    time.sleep(2)
                if child.poll() is None and stop_reason is not None:
                    self.capacity.kill_group(group_path)
                    os.killpg(child.pid, signal.SIGTERM)
                    try:
                        child.wait(timeout=CONTROLLER_CHILD_STOP_WAIT_SECONDS)
                    except subprocess.TimeoutExpired:
                        os.killpg(child.pid, signal.SIGKILL)
                        child.wait(timeout=5)
                exit_code = child.wait()
        except Exception as error:
            execution_error = error
            if child is not None and child.poll() is None:
                stop_reason = f'controller_exception:{type(error).__name__}'
                try:
                    self.capacity.kill_group(group_path)
                    os.killpg(child.pid, signal.SIGTERM)
                    child.wait(timeout=CONTROLLER_CHILD_STOP_WAIT_SECONDS)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait(timeout=5)
                except ProcessLookupError:
                    child.wait(timeout=5)
            try:
                self.capacity.kill_group(group_path)
            except Exception:
                pass
            exit_code = child.returncode if child is not None else None
        finally:
            if child is not None and child.poll() is None:
                try:
                    self.capacity.kill_group(group_path)
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait(timeout=5)
                except (ProcessLookupError, subprocess.TimeoutExpired):
                    pass
        self.capacity.write_json(self.run_directory / (case + '-host-samples.json'), samples)
        outcome = {'case': case, 'backend': backend, 'mode': mode, 'stage': 'E2',
                   'registered': registered, 'active': active, 'idle': idle,
                   'duration_ms': duration_ms, 'sample_interval_ms': PRESSURE_INTERVAL_MS,
                   'exit_code': exit_code, 'elapsed_seconds': time.monotonic() - started,
                   'capacity_after': self.capacity.capacity(), 'stop_reason': stop_reason,
                   'owned_process_limits': process_limits}
        if execution_error is not None:
            outcome['execution_error'] = {'type': type(execution_error).__name__,
                                          'message': str(execution_error)}
        report = None
        try:
            report = json.loads(report_path.read_text())
            outcome['report_sha256'] = digest(report_path)
            self.all_processes.extend(report['memory_diagnostic']['processes'])
            outcome['reduced'] = self.capacity.reduce_report(report, SOURCE_COMMIT)
            self.reviewer.validate_report(report, case, backend, SOURCE_COMMIT)
            assert (exit_code == 0) == outcome['reduced']['operational_pass']
        except Exception as error:
            outcome['review_error'] = repr(error)
            if report is not None and 'workload' in report:
                outcome['available_unvalidated_counters'] = report['workload']
            outcome['source_validation_at_failure'] = self.validation['source_commit']
        self.outcomes.append(outcome)
        self.capacity.write_json(self.run_directory / (case + '-status.json'), outcome)
        self.capacity.write_json(self.run_directory / 'summary.json', self.outcomes)
        if stop_reason is not None or outcome.get('review_error') or not outcome.get('reduced', {}).get('operational_pass'):
            self.capacity.kill_group(group_path)
        if execution_error is not None:
            raise AssertionError({'case': case, 'execution_error': outcome['execution_error']}) from execution_error
        if stop_reason is not None:
            raise AssertionError({'case': case, 'stop_reason': stop_reason})
        if outcome.get('review_error'):
            raise AssertionError({'case': case, 'review_error': outcome['review_error']})
        assert outcome['reduced']['operational_pass'] is True, outcome
        assert outcome['report_sha256']
        assert process_limits and all(row['limits']['soft'] == NOFILE_SOFT_TARGET for row in process_limits)
        assert not any(self.capacity.process_identity_live(identity) for identity in self.all_processes)
        return report, outcome

    def run_pair(self, pair_spec):
        reports = {}
        outcomes = {}
        for backend in pair_spec['backends']:
            report, outcome = self.run_cell(
                pair_spec['case'] + ('-blink' if backend == 'blink' else '-shared'),
                pair_spec['mode'], backend, pair_spec['registered'], pair_spec['active'],
                pair_spec['idle'], pair_spec['duration_ms'])
            reports[backend] = report
            outcomes[backend] = outcome
        pair_key = pair_spec['case']
        backend_order = pair_spec['backends']
        comparison = self.capacity.compare_pair(reports[backend_order[0]], reports[backend_order[1]])
        pair_result = {'pair': pair_key, 'mode': pair_spec['mode'],
                       'cases': [outcomes[backend]['case'] for backend in backend_order],
                       'backend_order': backend_order, 'comparison': comparison,
                       'slo': {backend: outcomes[backend]['reduced']['slo_pass'] for backend in backend_order}}
        self.pair_reports[pair_key] = reports
        self.capacity.write_json(self.run_directory / (pair_key + '-pair.json'), pair_result)
        return reports, outcomes

    def compare_phase_pair(self, earlier_case, later_case, backend):
        earlier_report = self.pair_reports[earlier_case][backend]
        later_report = self.pair_reports[later_case][backend]
        comparison = self.reviewer.compare_phases(earlier_report, later_report)
        key = earlier_case + '--' + later_case + '-' + backend
        self.phase_comparisons[key] = comparison
        self.capacity.write_json(self.run_directory / (key + '-phase-comparison.json'), comparison)

    def execute(self):
        specs = list(cell_sequence())
        tiny_slo_gate = True
        for pair_spec in specs[:2]:
            _, outcomes = self.run_pair(pair_spec)
            tiny_slo_gate = tiny_slo_gate and all(
                outcome['reduced']['slo_pass'] for outcome in outcomes.values())
        self.compare_phase_pair('tiny-synchronized', 'tiny-staggered', 'blink')
        self.compare_phase_pair('tiny-synchronized', 'tiny-staggered', 'minimal-native-shared')
        assert tiny_slo_gate, {'reason': 'tiny SLO gate failed',
                               'slo': {outcome['case']: outcome['reduced']['slo_pass']
                                       for outcome in self.outcomes}}
        for pair_spec in specs[2:]:
            self.run_pair(pair_spec)
        for repetition in range(1, 4):
            sync_case = f'control-n28-r{repetition}-synchronized'
            staggered_case = f'control-n28-r{repetition}-staggered'
            if repetition == 2:
                sync_case, staggered_case = staggered_case, sync_case
            for backend in ['blink', 'minimal-native-shared']:
                self.compare_phase_pair(sync_case, staggered_case, backend)
        self.assert_final_source_and_limits()
        final = {'completed': True, 'source_commit': SOURCE_COMMIT,
                 'plan_commit': self.plan_commit, 'cases': len(self.outcomes),
                 'phase_comparisons': self.phase_comparisons,
                 'capacity_after': self.capacity.capacity(),
                 'processes': [{'pid': identity['pid'], 'role': identity['role'],
                                'start_time_ticks': identity['start_time_ticks'],
                                'same_live_process': self.capacity.process_identity_live(identity)}
                               for identity in self.all_processes]}
        assert len(self.outcomes) == 16
        assert len(final['processes']) >= 16
        assert all(not identity['same_live_process'] for identity in final['processes'])
        self.capacity.write_json(self.run_directory / 'completion.json', final)
        return final

    def assert_final_source_and_limits(self):
        current = subprocess.check_output(
            ['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), 'rev-parse', 'HEAD'], text=True
        ).strip()
        status = subprocess.check_output(
            ['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), 'status', '--porcelain'], text=True
        ).strip()
        assert current == SOURCE_COMMIT and not status
        assert parse_proc_limits(os.getpid())['soft'] == NOFILE_SOFT_TARGET
        assert digest(Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')) == EXPECTED_NATIVE_SHA256
        assert all(parse_proc_limits(identity['pid'])['soft'] == NOFILE_SOFT_TARGET
                   for identity in self.all_processes if self.capacity.process_identity_live(identity))


def main():
    parser = argparse.ArgumentParser(description='Run the frozen E2 arrival-phase controls')
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    if os.geteuid() != 0:
        raise AssertionError('arrival controls require the designated root OCI harness')
    nofile_identity = set_controller_nofile_limit()
    sys.path.insert(0, str(ROOT))
    validation_module = load_module(
        'oci_arrival_validate',
        Path(__file__).with_name('oci_arrival_validate.py'))
    validation = read_completed_validation(validation_module)
    try:
        nofile_identity['preflight_child'] = verify_preflight_child_limits(nofile_identity['proc_hard'])
    except Exception as error:
        failure_directory = ROOT / 'arrival-phase'
        failure_directory.mkdir(parents=True, exist_ok=True)
        write_json(failure_directory / (RUN_NAME + '-preflight-failure.json'), {
            'source_commit': SOURCE_COMMIT, 'plan_commit': arguments.plan_commit,
            'runtime_limits': nofile_identity,
            'validation_sha256': validation['summary_sha256'],
            'failure': {'type': type(error).__name__, 'message': str(error)},
        })
        raise
    capacity_module = load_module('oci_capacity', capacity_controller_path())
    reviewer_module = load_module(
        'arrival_review', Path(__file__).with_name('arrival_review.py'))
    plan_commit = arguments.plan_commit
    experiment = ArrivalExperiment(
        plan_commit, capacity_module, reviewer_module, validation, nofile_identity,
        validation['default_binaries'])
    def stop_controller(signum, frame):
        experiment.stop_reason = f'parent_signal_{signum}'

    signal.signal(signal.SIGTERM, stop_controller)
    signal.signal(signal.SIGINT, stop_controller)
    try:
        result = experiment.execute()
    except Exception as error:
        failure = {'completed': False, 'source_commit': SOURCE_COMMIT,
                   'plan_commit': plan_commit, 'cases_attempted': len(experiment.outcomes),
                   'failure': {'type': type(error).__name__, 'message': str(error)},
                   'capacity_after': capacity_module.capacity(),
                   'outcomes': experiment.outcomes,
                   'processes': [{'pid': identity['pid'], 'role': identity['role'],
                                  'start_time_ticks': identity['start_time_ticks'],
                                  'same_live_process': capacity_module.process_identity_live(identity)}
                                 for identity in experiment.all_processes]}
        try:
            validation_module.validate_source_and_native()
            failure['source_validation'] = {'source_commit': validation_module.EXPECTED_SOURCE,
                                            'clean': True,
                                            'native_archive_sha256': digest(validation_module.NATIVE_ARCHIVE)}
        except Exception as source_error:
            failure['source_validation'] = {'clean': False,
                                            'error': f'{type(source_error).__name__}:{source_error}'}
        capacity_module.write_json(experiment.run_directory / 'failure-receipt.json', failure)
        raise
    print(json.dumps({'completed': result['completed'], 'cases': result['cases'],
                      'source_commit': result['source_commit'],
                      'plan_commit': result['plan_commit']}, sort_keys=True), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
