import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

from capacity_review import compare_pair, digest, reduce_report


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
DATABASE_ROOT = Path('/bench/zfs/db/tenant-density-harness-20261027')
BASE_ARGUMENTS = [
    '--seed', '20261027', '--client-count', '1', '--value-bytes', '512',
    '--topology', 'fixed-per-tenant', '--per-tenant-rate', '50',
    '--queue-bound', '8', '--max-in-flight', '8', '--request-timeout-ms', '5000',
    '--drain-timeout-ms', '15000', '--profile', 'mixed', '--width', 'mixed',
    '--backend-memory-max-bytes', str(256 * 1024**2),
    '--backend-memory-high-bytes', str(192 * 1024**2),
]
BACKEND_ARGUMENTS = {
    'blink': ['--blink-wal-layout', 'segmented-bases-coalesced',
              '--blink-checkpoint-execution', 'background',
              '--blink-wal-segment-bytes', str(256 * 1024),
              '--blink-checkpoint-wal-bytes', str(1024**2)],
    'minimal-native-shared': ['--rocksdb-shared-cache-bytes', str(64 * 1024**2),
                              '--rocksdb-shared-wbm-bytes', str(32 * 1024**2),
                              '--rocksdb-write-buffer-bytes', str(1024**2),
                              '--rocksdb-flush-threads', '1', '--rocksdb-compaction-threads', '1',
                              '--rocksdb-max-open-files', '64'],
}


def write_json(file_path, value):
    temporary = file_path.with_suffix(file_path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, indent=2) + '\n')
    temporary.replace(file_path)


def command(arguments):
    return subprocess.check_output(arguments, text=True).strip()


def capacity():
    memory = {}
    for line in Path('/proc/meminfo').read_text().splitlines():
        name, value = line.split(':', 1)
        if name in ['MemTotal', 'MemAvailable', 'SwapFree', 'SwapTotal', 'Dirty', 'Writeback']:
            memory[name] = int(value.split()[0]) * 1024
    arc = {}
    for line in Path('/proc/spl/kstat/zfs/arcstats').read_text().splitlines()[2:]:
        fields = line.split()
        if fields[0] in ['size', 'c', 'c_min', 'c_max', 'memory_throttle_count']:
            arc[fields[0]] = int(fields[2])
    filesystem = os.statvfs(DATABASE_ROOT)
    boot = os.statvfs(ROOT)
    return {'timestamp_unix_seconds': time.time(),
            'zfs_free_bytes': filesystem.f_bavail * filesystem.f_frsize,
            'boot_free_bytes': boot.f_bavail * boot.f_frsize,
            'memory_bytes': memory, 'arc': arc,
            'device_stat': Path('/sys/class/block/sda4/stat').read_text().strip(),
            'cpu_stat': Path('/proc/stat').read_text().splitlines()[:3]}


def guard(state, before=False):
    assert state['zfs_free_bytes'] >= (12 if before else 10) * 1024**3, state
    assert state['memory_bytes']['MemAvailable'] >= 1500 * 1024**2, state


def prepare_group(group_path):
    group_path.mkdir(exist_ok=False)
    (group_path / 'cgroup.subtree_control').write_text('+memory')
    assert not (group_path / 'cgroup.procs').read_text().strip()


def kill_group(group_path):
    kill_file = group_path / 'cgroup.kill'
    if kill_file.exists():
        kill_file.write_text('1')


def process_identity_live(identity):
    try:
        raw = (Path('/proc') / str(identity['pid']) / 'stat').read_text()
        fields = raw[raw.rfind(')') + 2:].split()
        return int(fields[19]) == identity['start_time_ticks']
    except FileNotFoundError:
        return False


class Experiment:
    def __init__(self, run_name, source_commit, plan_commit):
        assert os.geteuid() == 0
        assert run_name and all(character.isalnum() or character in '-_' for character in run_name)
        self.source_commit = source_commit
        self.run_directory = ROOT / 'capacity' / run_name
        self.run_directory.mkdir(parents=True, exist_ok=False)
        self.database_directory = DATABASE_ROOT / run_name
        self.database_directory.mkdir(exist_ok=False)
        self.group_root = Path('/sys/fs/cgroup') / ('dodb-density-' + run_name)
        prepare_group(self.group_root)
        self.binaries = self.run_directory / 'bin'
        self.binaries.mkdir()
        identities = {}
        for name in ['tenant_density_workload', 'memory_diagnostic_worker']:
            destination = self.binaries / name
            shutil.copy2(ROOT / 'target/release' / name, destination)
            identities[name] = {'sha256': digest(destination), 'bytes': destination.stat().st_size}
        assert command(['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), 'rev-parse', 'HEAD']) == source_commit
        assert not command(['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), 'status', '--porcelain'])
        before = capacity()
        guard(before, before=True)
        self.outcomes = []
        self.pair_number = 0
        self.operational_failures = set()
        self.all_processes = []
        write_json(self.run_directory / 'provenance.json', {
            'source_commit': source_commit, 'plan_commit': plan_commit, 'runtime_uid': os.geteuid(),
            'binaries': identities, 'controller_sha256': digest(Path(__file__)),
            'reducer_sha256': digest(Path(__file__).with_name('capacity_review.py')),
            'capacity_before': before, 'kernel': os.uname().release,
            'hostname': os.uname().nodename, 'online_cpus': os.cpu_count(),
            'filesystem_properties': command(['zfs', 'get', '-Hp', '-o', 'property,value', 'used,available,recordsize,compression,atime,sync', 'dodbbench/db']),
            'processes_before': command(['ps', '-eo', 'pid,ppid,uid,pcpu,rss,args', '--sort=-pcpu']),
            'budget_scope': 'backend cgroups; driver and uncharged host ARC excluded',
        })

    def run_cell(self, case, stage, backend, tenants, duration_ms, interval_ms=1000, active=None, idle=0):
        active = tenants if active is None else active
        before = capacity()
        guard(before, before=True)
        assert not any(process_identity_live(identity) for identity in self.all_processes)
        report_path = self.run_directory / (case + '.json')
        group_path = self.group_root / case
        prepare_group(group_path)
        arguments = [str(self.binaries / 'tenant_density_workload'), '--backend', backend,
                     '--data-dir', str(self.database_directory / case), '--report', str(report_path),
                     '--registered-tenants', str(tenants), '--active-tenants', str(active),
                     '--opened-idle-tenants', str(idle), '--initial-rows', '2048' if stage == 'E2' else '32768',
                     '--duration-ms', str(duration_ms), '--pressure-sample-interval-ms', str(interval_ms),
                     '--backend-cgroup-root', str(group_path), *BASE_ARGUMENTS, *BACKEND_ARGUMENTS[backend]]
        write_json(self.run_directory / (case + '-command.json'), {
            'command': arguments, 'capacity_before': before, 'source_commit': self.source_commit,
            'started_unix_seconds': time.time(), 'stage': stage})
        environment = os.environ.copy()
        environment['DODB_SOURCE_REVISION'] = self.source_commit
        samples = []
        stop_reason = None
        started = time.monotonic()
        with (self.run_directory / (case + '.log')).open('x') as output:
            child = subprocess.Popen(arguments, cwd=SOURCE, env=environment, stdout=output,
                                     stderr=subprocess.STDOUT, start_new_session=True)
            while child.poll() is None:
                state = capacity()
                samples.append(state)
                try:
                    guard(state)
                    assert time.monotonic() - started < 1200, 'controller_timeout'
                except AssertionError as error:
                    stop_reason = repr(error)
                    kill_group(group_path)
                    os.killpg(child.pid, signal.SIGTERM)
                    try:
                        child.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(child.pid, signal.SIGKILL)
                        child.wait(timeout=5)
                    break
                time.sleep(2)
            exit_code = child.wait()
        write_json(self.run_directory / (case + '-host-samples.json'), samples)
        outcome = {'case': case, 'backend': backend, 'stage': stage, 'tenants': tenants,
                   'active': active, 'duration_ms': duration_ms, 'sample_interval_ms': interval_ms,
                   'exit_code': exit_code, 'elapsed_seconds': time.monotonic() - started,
                   'capacity_after': capacity(), 'stop_reason': stop_reason}
        try:
            report = json.loads(report_path.read_text())
            outcome['report_sha256'] = digest(report_path)
            self.all_processes.extend(report['memory_diagnostic']['processes'])
            outcome['reduced'] = reduce_report(report, self.source_commit)
            assert (exit_code == 0) == outcome['reduced']['operational_pass']
            if outcome['reduced']['resource_failure']:
                self.operational_failures.add((stage, backend, tenants))
        except Exception as error:
            outcome['review_error'] = repr(error)
        self.outcomes.append(outcome)
        write_json(self.run_directory / (case + '-status.json'), outcome)
        write_json(self.run_directory / 'summary.json', self.outcomes)
        print(json.dumps({name: outcome[name] for name in ['case', 'exit_code', 'elapsed_seconds', 'stop_reason']} | {
            'operational_pass': outcome.get('reduced', {}).get('operational_pass'),
            'slo_pass': outcome.get('reduced', {}).get('slo_pass'),
            'measurement_peak_bytes': outcome.get('reduced', {}).get('epoch_peaks_bytes', {}).get('measurement'),
            'worst_p99_ms': outcome.get('reduced', {}).get('worst_tenant_p99_upper_exclusive_ms'),
            'review_error': outcome.get('review_error')}), flush=True)
        assert stop_reason is None and 'review_error' not in outcome, outcome
        assert not any(process_identity_live(identity) for identity in self.all_processes)
        return outcome

    def pair(self, name, stage, tenants, duration_ms, interval_ms=1000, active=None, idle=0, skip_failed=False):
        self.pair_number += 1
        backends = ['blink', 'minimal-native-shared']
        if self.pair_number % 2 == 0:
            backends.reverse()
        outcomes = {}
        for backend in backends:
            if skip_failed and any(failure_stage == stage and failure_backend == backend and failure_count <= tenants
                                   for failure_stage, failure_backend, failure_count in self.operational_failures):
                continue
            label = 'blink' if backend == 'blink' else 'shared'
            outcomes[backend] = self.run_cell(name + '-' + label, stage, backend, tenants, duration_ms, interval_ms, active, idle)
        if len(outcomes) == 2:
            cases = [outcomes[backend]['case'] for backend in ['blink', 'minimal-native-shared']]
            reports = [json.loads((self.run_directory / (case + '.json')).read_text()) for case in cases]
            write_json(self.run_directory / (name + '-pair.json'), {'cases': cases, 'comparison': compare_pair(*reports)})
        return outcomes

    def execute(self):
        screens = {}
        selections = {}
        for stage, grid in [('E2', [1, 4, 8, 16, 32]), ('E3', [1, 2, 4, 8])]:
            stage_results = {}
            for tenants in grid:
                pair = self.pair(f'screen-{stage.lower()}-n{tenants}', stage, tenants, 30000, skip_failed=True)
                stage_results[tenants] = pair
                if not pair:
                    break
            common_passes = [tenants for tenants, pair in stage_results.items() if len(pair) == 2 and all(outcome['reduced']['slo_pass'] for outcome in pair.values())]
            low = max(common_passes, default=0)
            failures = [tenants for tenants, pair in stage_results.items() if tenants > low and len(pair) == 2 and any(not outcome['reduced']['slo_pass'] for outcome in pair.values())]
            high = min(failures, default=None)
            for refinement in range(2):
                if not low or high is None or high - low <= 1:
                    break
                midpoint = (low + high) // 2
                pair = self.pair(f'refine-{stage.lower()}-n{midpoint}', stage, midpoint, 30000, skip_failed=True)
                stage_results[midpoint] = pair
                if len(pair) == 2 and all(outcome['reduced']['slo_pass'] for outcome in pair.values()):
                    low = midpoint
                else:
                    high = midpoint
            screens[stage] = stage_results
            selections[stage] = {'common_screen_pass': low, 'next_screen_failure': high,
                                 'confirmation_counts': sorted({count for count in [low or 1, high] if count is not None})}
            write_json(self.run_directory / 'selections.json', selections)
            for tenants in selections[stage]['confirmation_counts']:
                if any(count == tenants and failure_stage == stage for failure_stage, _, count in self.operational_failures):
                    continue
                for repetition in range(1, 4):
                    self.pair(f'confirm-{stage.lower()}-n{tenants}-r{repetition}', stage, tenants, 60000, skip_failed=True)
        for interval in [1000, 100]:
            self.pair(f'sampling-e2-n1-ms{interval}', 'E2', 1, 60000, interval_ms=interval)
        common_operational = [tenants for tenants, pair in screens['E3'].items() if len(pair) == 2 and all(outcome['reduced']['operational_pass'] for outcome in pair.values())]
        sparse_registered = max(common_operational, default=0)
        if sparse_registered >= 3:
            self.pair(f'sparse-e3-registered{sparse_registered}-active1-idle1', 'E3', sparse_registered, 60000, active=1, idle=1)
        else:
            write_json(self.run_directory / 'sparse-omission.json', {'reason': 'no common operationally completed E3 population with at least three tenants'})
        confirmed = {}
        for stage in ['E2', 'E3']:
            candidates = selections[stage]['confirmation_counts']
            confirmed[stage] = []
            for tenants in candidates:
                rows = [outcome for outcome in self.outcomes if outcome['case'].startswith(f'confirm-{stage.lower()}-n{tenants}-')]
                if len(rows) == 6 and all(outcome['reduced']['slo_pass'] for outcome in rows):
                    confirmed[stage].append(tenants)
        maintenance_count = min(max(confirmed['E2'], default=1), 16)
        self.pair(f'maintenance-e2-n{maintenance_count}', 'E2', maintenance_count, 180000)
        final = {'completed': True, 'source_commit': self.source_commit,
                 'cases': len(self.outcomes), 'confirmed_common_counts': confirmed,
                 'maintenance_count': maintenance_count, 'capacity_after': capacity(),
                 'processes': [{'pid': identity['pid'], 'role': identity['role'],
                                'start_time_ticks': identity['start_time_ticks'],
                                'same_live_process': process_identity_live(identity)} for identity in self.all_processes]}
        assert all(not identity['same_live_process'] for identity in final['processes'])
        write_json(self.run_directory / 'completion.json', final)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    experiment = Experiment(arguments.run_name, arguments.source_commit, arguments.plan_commit)
    try:
        experiment.execute()
    except Exception as error:
        write_json(experiment.run_directory / 'controller-failure.json', {'reason': repr(error), 'capacity': capacity()})
        raise
