import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
RUN_NAME = 'arrival-controls-cb0e04482-r1'
CONTROL_LAUNCHER_PREFIX = 'arrival-controls-cb0e04482-r'
VALIDATION_RUN = 'arrival-cb0e04482-r1'
DEFAULT_ATTEMPT = 'arrival-control-collection-cb0e04482-r1'
SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
ACTUAL_CONTROLLER_COMMIT = '95c3b8c5323415dfc493bdfcb32c7dc5e057e8b8'
FROZEN_PLAN_COMMIT = 'e02a1fbdde81fd6fc363a26e5341c8c61000023e'
EXPECTED_NATIVE_COMMIT = 'abeebd9630f11bd08c28b7bd43c7bdfc62050654'
EXPECTED_NATIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
NATIVE_ARCHIVE = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')
DATABASE_ROOT = Path('/bench/zfs/db/tenant-density-harness-20261027') / RUN_NAME
GROUPING_ROOT = Path('/sys/fs/cgroup') / f'dodb-density-{RUN_NAME}'
VALIDATION_CHECKS = ['tests', 'clippy', 'a1-link', 'default-link']
EPOCHS = ['setup', 'measurement', 'oracle', 'reopen-oracle']
MEMORY_MAX_BYTES = 256 * 1024**2
MEMORY_HIGH_BYTES = 192 * 1024**2
LAUNCHER_GRACE_SECONDS = 20


def digest(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def git(arguments):
    return subprocess.check_output(
        ['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), *arguments], text=True
    ).strip()


def load_json(path):
    return json.loads(Path(path).read_text())


def process_identity_live(identity):
    try:
        raw_stat = (Path('/proc') / str(identity['pid']) / 'stat').read_text()
        fields = raw_stat[raw_stat.rfind(')') + 2:].split()
        return int(fields[19]) == identity['start_time_ticks']
    except (FileNotFoundError, ProcessLookupError, KeyError, IndexError, ValueError):
        return False


def process_command_line(pid):
    try:
        return (Path('/proc') / str(pid) / 'cmdline').read_bytes().replace(b'\0', b' ').decode(errors='replace').strip()
    except FileNotFoundError:
        return ''


def launcher_state(name, launcher_directory, timeout_seconds=LAUNCHER_GRACE_SECONDS):
    log_path = launcher_directory / f'{name}-launcher.log'
    pid_path = launcher_directory / f'{name}-pid.txt'
    if not log_path.is_file() or not pid_path.is_file():
        raise AssertionError(f'launcher evidence is missing for {name}')
    pid_text = pid_path.read_text().strip()
    if not pid_text.isdecimal():
        raise AssertionError(f'launcher PID is invalid for {name}')
    pid = int(pid_text)
    command_line = ''
    deadline = time.monotonic() + timeout_seconds
    expected_tokens = ['oci_arrival_controls.py' if name.startswith('arrival-controls-') else 'oci_arrival_validate.py']
    while True:
        command_line = process_command_line(pid)
        if not command_line or not any(token in command_line for token in expected_tokens):
            return {
                'name': name,
                'pid': pid,
                'launcher_log': str(log_path),
                'launcher_pid_file': str(pid_path),
                'launcher_sha256': digest(log_path),
                'pid_file_sha256': digest(pid_path),
                'process_running': False,
                'process_command_line': command_line,
            }
        if time.monotonic() >= deadline:
            raise AssertionError(f'launcher process is still running for {name}: {pid}')
        time.sleep(0.1)


def validation_artifacts():
    namespace = ROOT / 'arrival-phase' / VALIDATION_RUN
    summary_path = namespace / 'arrival-validation-summary.json'
    summary = load_json(summary_path)
    outcomes = summary.get('outcomes', [])
    successful = (
        summary.get('status') == 'completed'
        and summary.get('source_commit') == SOURCE_COMMIT
        and len(outcomes) == len(VALIDATION_CHECKS)
        and [outcome.get('name') for outcome in outcomes] == VALIDATION_CHECKS
        and all(outcome.get('exit_code') == 0 for outcome in outcomes)
        and outcomes[0].get('passing_tests') == 85
        and outcomes[1].get('same_17_existing_clippy_diagnostics') is True
        and outcomes[-1].get('name') == 'default-link'
        and outcomes[0].get('test_group_counts') == [42, 9, 25, 5, 4]
        and summary.get('default_link_was_last') is True
        and summary.get('native_archive_sha256_before') == EXPECTED_NATIVE_SHA256
        and summary.get('native_archive_sha256_after') == EXPECTED_NATIVE_SHA256
    )
    binary_records = summary.get('default_binaries', {})
    binary_directory = namespace / 'bin'
    binary_hashes = {}
    for binary_name in ['tenant_density_workload', 'memory_diagnostic_worker']:
        binary_path = binary_directory / binary_name
        identity = binary_records.get(binary_name)
        if not binary_path.is_file() or not isinstance(identity, dict):
            successful = False
            continue
        actual = {'sha256': digest(binary_path), 'bytes': binary_path.stat().st_size}
        binary_hashes[binary_name] = actual
        if actual != identity:
            successful = False
    if set(binary_hashes) != {'tenant_density_workload', 'memory_diagnostic_worker'}:
        successful = False
    return {
        'path': str(namespace),
        'summary_path': str(summary_path),
        'summary': summary,
        'summary_sha256': digest(summary_path),
        'valid': successful,
        'binary_hashes': binary_hashes,
    }


def control_binary_hashes(controls_directory, validation_summary):
    expected_binaries = validation_summary.get('default_binaries', {})
    binaries = {}
    for binary_name in ['tenant_density_workload', 'memory_diagnostic_worker']:
        binary_path = controls_directory / 'bin' / binary_name
        if not binary_path.is_file():
            binaries[binary_name] = {'present': False, 'matches_validation': False}
            continue
        actual = {'sha256': digest(binary_path), 'bytes': binary_path.stat().st_size}
        binaries[binary_name] = {
            **actual,
            'present': True,
            'matches_validation': actual == expected_binaries.get(binary_name),
        }
    return binaries


def controls_terminal(controls_directory):
    completion_path = controls_directory / 'completion.json'
    failure_path = controls_directory / 'failure-receipt.json'
    initialization_failure_path = controls_directory / 'initialization-failure-receipt.json'
    terminal_paths = [
        path for path in [completion_path, failure_path, initialization_failure_path]
        if path.is_file()
    ]
    if len(terminal_paths) != 1:
        raise AssertionError('controls namespace must have exactly one terminal receipt')
    terminal_path = terminal_paths[0]
    terminal = load_json(terminal_path)
    if terminal.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('controls terminal receipt does not match its terminal filename or source')
    if terminal_path == completion_path:
        terminal_status = 'completed'
        if terminal.get('completed') is not True or terminal.get('cases') != 16:
            raise AssertionError('completed controls receipt does not contain all sixteen cells')
    elif terminal_path == failure_path:
        terminal_status = 'failed'
        if terminal.get('completed') is not False:
            raise AssertionError('failure receipt has an invalid completed flag')
        if terminal.get('cases_attempted') != len(terminal.get('outcomes', [])):
            raise AssertionError('failure receipt attempted-cell count differs from its outcomes')
    else:
        terminal_status = 'initialization_failed'
        if terminal.get('completed') is not False or not terminal.get('initialization_failure'):
            raise AssertionError('initialization failure receipt has an invalid terminal state')
    if terminal_status == 'completed' and terminal.get('cases') != 16:
        raise AssertionError('completed controls receipt does not contain all sixteen cells')
    return terminal_path, terminal, terminal_status


def control_statuses(controls_directory, terminal):
    statuses = []
    for status_path in sorted(controls_directory.glob('*-status.json')):
        status = load_json(status_path)
        statuses.append({
            'path': str(status_path),
            'sha256': digest(status_path),
            'case': status.get('case'),
            'exit_code': status.get('exit_code'),
            'stop_reason': status.get('stop_reason'),
            'review_error': status.get('review_error'),
            'operational_pass': (status.get('reduced') or {}).get('operational_pass'),
            'slo_pass': (status.get('reduced') or {}).get('slo_pass'),
        })
    outcome_cases = {outcome.get('case') for outcome in terminal.get('outcomes', [])}
    status_cases = {status['case'] for status in statuses if status['case'] is not None}
    if terminal.get('completed') is True:
        command_cases = {
            path.name.removesuffix('-command.json')
            for path in controls_directory.glob('*-command.json')
        }
        cases_match = status_cases == command_cases and len(command_cases) == 16
    else:
        cases_match = outcome_cases == status_cases if outcome_cases else None
    recorded_cells_operationally_valid = bool(statuses) and all(
        status['exit_code'] == 0
        and status['stop_reason'] is None
        and status['review_error'] is None
        and status['operational_pass'] is True
        for status in statuses
    )
    all_sixteen_operationally_valid = (
        len(statuses) == 16 and recorded_cells_operationally_valid and cases_match is True
    )
    all_sixteen_slo_pass = len(statuses) == 16 and all(
        status['slo_pass'] is True for status in statuses
    )
    if terminal.get('completed') is True:
        all_sixteen_operationally_valid = all_sixteen_operationally_valid and cases_match is True
    return {
        'statuses': statuses,
        'status_count': len(statuses),
        'terminal_outcome_count': len(terminal.get('outcomes', [])),
        'status_cases_match_terminal_outcomes': cases_match,
        'recorded_cells_operationally_valid': recorded_cells_operationally_valid,
        'all_sixteen_cells_operationally_valid': all_sixteen_operationally_valid,
        'all_sixteen_cells_slo_pass': all_sixteen_slo_pass,
        'slo_miss_cases': [status['case'] for status in statuses if status['slo_pass'] is False],
    }


def add_process_identity(identities, identity, origin):
    if not isinstance(identity, dict) or 'pid' not in identity or 'start_time_ticks' not in identity:
        return
    key = (int(identity['pid']), int(identity['start_time_ticks']))
    record = identities.setdefault(key, {'pid': key[0], 'start_time_ticks': key[1], 'origins': []})
    if origin not in record['origins']:
        record['origins'].append(origin)
    if 'role' in identity and identity['role'] not in record.get('roles', []):
        record.setdefault('roles', []).append(identity['role'])
    if 'same_live_process' in identity:
        record['recorded_same_live_process'] = bool(identity['same_live_process'])


def collect_processes(controls_directory, terminal):
    identities = {}
    for identity in terminal.get('processes', []):
        add_process_identity(identities, identity, 'terminal_receipt')
    for status_path in sorted(controls_directory.glob('*-status.json')):
        status = load_json(status_path)
        for process in status.get('owned_process_limits', []):
            add_process_identity(identities, process, status_path.name)
        report_path = controls_directory / f"{status.get('case', status_path.name.removesuffix('-status.json'))}.json"
        if report_path.is_file():
            report = load_json(report_path)
            for process in report.get('memory_diagnostic', {}).get('processes', []):
                add_process_identity(identities, process, report_path.name)
    processes = []
    for identity in identities.values():
        identity['same_live_process_at_collection'] = process_identity_live(identity)
        processes.append(identity)
    live = [identity for identity in processes if identity['same_live_process_at_collection']]
    if live:
        raise AssertionError(f'owned process identities are still live: {len(live)}')
    return processes


def command_value(command_record, option):
    arguments = command_record.get('command', [])
    try:
        return arguments[arguments.index(option) + 1]
    except (ValueError, IndexError):
        raise AssertionError(f'cell command is missing {option}')


def collect_cgroups(controls_directory, terminal):
    command_paths = sorted(controls_directory.glob('*-command.json'))
    cell_paths = {}
    for command_path in command_paths:
        record = load_json(command_path)
        case = command_path.name.removesuffix('-command.json')
        cell_path = Path(command_value(record, '--backend-cgroup-root'))
        if cell_path != GROUPING_ROOT / case:
            raise AssertionError(f'cell cgroup path differs from the frozen run namespace: {case}')
        cell_paths[case] = cell_path
    grouping_roots = {cell_path.parent for cell_path in cell_paths.values()}
    if len(grouping_roots) > 1:
        raise AssertionError('cell cgroup paths do not share one grouping root')
    grouping_root = next(iter(grouping_roots)) if grouping_roots else GROUPING_ROOT
    grouping_root_path_matches = grouping_root == GROUPING_ROOT
    groups = {}
    group_paths = {grouping_root}
    if grouping_root.is_dir():
        group_paths.update(path.parent for path in grouping_root.rglob('cgroup.procs'))
    for case, cell_path in cell_paths.items():
        if cell_path.parent != grouping_root:
            raise AssertionError('cell cgroup escaped the common grouping root')
        group_paths.add(cell_path)
        group_paths.update(cell_path / epoch for epoch in EPOCHS if (cell_path / epoch).is_dir())
    for group_path in sorted(group_paths):
        if not group_path.is_dir():
            continue
        key = str(group_path)
        record = {
            'path': key,
            'relative_to_grouping_root': '.' if group_path == grouping_root else str(group_path.relative_to(grouping_root)),
            'memory.max': (group_path / 'memory.max').read_text().strip(),
            'memory.high': (group_path / 'memory.high').read_text().strip(),
            'memory.swap.max': (group_path / 'memory.swap.max').read_text().strip(),
            'memory.current': (group_path / 'memory.current').read_text().strip(),
            'cgroup.procs': (group_path / 'cgroup.procs').read_text().split(),
        }
        groups[key] = record
    expected_budget = {
        'memory.max': str(MEMORY_MAX_BYTES),
        'memory.high': str(MEMORY_HIGH_BYTES),
        'memory.swap.max': '0',
    }
    root_group = groups.get(str(grouping_root))
    root_budget = {key: root_group[key] for key in expected_budget} if root_group else None
    grouping_root_valid = bool(root_group and root_group['memory.max'] == 'max')
    budget_groups_valid = all(
        all(record[key] == value for key, value in expected_budget.items())
        for record in groups.values()
        if record['relative_to_grouping_root'] != '.'
    )
    process_groups_empty = all(not record['cgroup.procs'] for record in groups.values())
    expected_cells = {status.get('case') for status in terminal.get('outcomes', [])}
    if terminal.get('completed') is True:
        expected_cells = {path.name.removesuffix('-command.json') for path in command_paths}
        if len(command_paths) != 16 or len(expected_cells) != 16:
            raise AssertionError('completed controls namespace does not have sixteen cell command records')
    observed_cells = set(cell_paths)
    cell_groups_present = (
        all(str(cell_path) in groups for cell_path in cell_paths.values()) if cell_paths else None
    )
    epochs_present = (
        all(
            all(str(cell_path / epoch) in groups for epoch in EPOCHS)
            for cell_path in cell_paths.values()
        ) if cell_paths else None
    )
    expected_cells_present = expected_cells.issubset(observed_cells) if expected_cells else None
    return {
        'grouping_root': str(grouping_root),
        'grouping_root_path_matches_expected': grouping_root_path_matches,
        'grouping_root_budget': root_budget,
        'grouping_root_memory_max_is_unbounded': grouping_root_valid,
        'cell_epoch_budgets_match': (
            budget_groups_valid and cell_groups_present and epochs_present
            if cell_paths else None
        ),
        'cell_groups_present': cell_groups_present,
        'all_four_epochs_present_for_each_cell': epochs_present,
        'all_cgroup_process_lists_empty': process_groups_empty if groups else None,
        'expected_cell_count': len(expected_cells),
        'all_expected_cells_have_commands': expected_cells_present,
        'groups': groups,
        'cell_count': len(cell_paths),
        'group_count': len(groups),
    }


def database_inventory(controls_directory):
    inventory = []
    for command_path in sorted(controls_directory.glob('*-command.json')):
        case = command_path.name.removesuffix('-command.json')
        command_record = load_json(command_path)
        path = Path(command_value(command_record, '--data-dir'))
        if path != DATABASE_ROOT / case:
            raise AssertionError(f'database path differs from frozen run namespace: {case}')
        exists = path.is_dir()
        file_count = 0
        total_bytes = 0
        if exists:
            for file_path in path.rglob('*'):
                if file_path.is_file():
                    file_count += 1
                    total_bytes += file_path.stat().st_size
        inventory.append({
            'case': case,
            'path': str(path),
            'preserved': exists,
            'file_count': file_count,
            'total_file_bytes': total_bytes,
        })
    if not inventory and DATABASE_ROOT.is_dir():
        for path in sorted(DATABASE_ROOT.iterdir()):
            if path.is_dir():
                file_count = 0
                total_bytes = 0
                for file_path in path.rglob('*'):
                    if file_path.is_file():
                        file_count += 1
                        total_bytes += file_path.stat().st_size
                inventory.append({
                    'case': path.name,
                    'path': str(path),
                    'preserved': True,
                    'file_count': file_count,
                    'total_file_bytes': total_bytes,
                    'inventory_source': 'filesystem_enumeration_without_cell_commands',
                })
    return {
        'root_path': str(DATABASE_ROOT),
        'root_exists': DATABASE_ROOT.is_dir(),
        'cases': inventory,
        'preserved_case_count': sum(item['preserved'] for item in inventory),
    }


def runtime_helper(primary_path, fallback_path):
    primary = Path(primary_path)
    if primary.is_file():
        return primary
    fallback = Path(fallback_path)
    if fallback.is_file():
        return fallback
    raise AssertionError('required helper file is missing')


def actual_helpers(controls_provenance, validation_summary):
    arrival_directory = ROOT / 'arrival-phase'
    evidence_directory = Path(__file__).resolve().parent
    capacity_directory = evidence_directory.parent / 'capacity'
    validation_path = runtime_helper(arrival_directory / 'oci_arrival_validate.py', evidence_directory / 'oci_arrival_validate.py')
    controls_path = runtime_helper(arrival_directory / 'oci_arrival_controls.py', evidence_directory / 'oci_arrival_controls.py')
    review_path = runtime_helper(arrival_directory / 'arrival_review.py', evidence_directory / 'arrival_review.py')
    capacity_path = runtime_helper(ROOT / 'oci_capacity.py', capacity_directory / 'oci_capacity.py')
    capacity_review_path = runtime_helper(
        capacity_path.with_name('capacity_review.py'),
        capacity_directory / 'capacity_review.py',
    )
    collector_path = Path(__file__).resolve()
    helpers = [
        ('arrival-phase/oci_arrival_validate.py', validation_path, validation_summary.get('controller_sha256')),
        ('arrival-phase/oci_arrival_controls.py', controls_path, controls_provenance.get('controller_sha256')),
        ('arrival-phase/arrival_review.py', review_path, controls_provenance.get('reviewer_sha256')),
        ('arrival-phase/oci_arrival_collect.py', collector_path, None),
        ('capacity/oci_capacity.py', capacity_path, controls_provenance.get('capacity_controller_sha256')),
        ('capacity/capacity_review.py', capacity_review_path, controls_provenance.get('capacity_reviewer_sha256')),
    ]
    return [
        {'archive_path': archive_path, 'path': str(path), 'sha256': digest(path),
         'expected_sha256': expected,
         'hash_matches_provenance': None if expected is None else digest(path) == expected}
        for archive_path, path, expected in helpers
    ]


def collect_launcher_evidence():
    validation_launcher = launcher_state(VALIDATION_RUN, ROOT)
    controls_logs = sorted(ROOT.glob(f'{CONTROL_LAUNCHER_PREFIX}*-launcher.log'))
    if not controls_logs:
        raise AssertionError('arrival controls launcher evidence is missing')
    controls_launchers = [
        launcher_state(path.name.removesuffix('-launcher.log'), ROOT)
        for path in controls_logs
    ]
    return [validation_launcher, *controls_launchers]


def preflight_launch_attempts(launcher_records):
    return [
        {
            'launcher_name': record['name'],
            'classification': 'preflight_only_plan_commit_mismatch',
            'actual_workload_cells': 0,
            'launcher_log_sha256': record['launcher_sha256'],
            'pid_file_sha256': record['pid_file_sha256'],
        }
        for record in launcher_records
        if record['name'] == RUN_NAME
    ]


def collect_space():
    records = {}
    for name, path in [('harness_root', ROOT), ('benchmark_root', Path('/bench/zfs/db'))]:
        filesystem = os.statvfs(path)
        records[name] = {
            'path': str(path),
            'available_bytes': filesystem.f_bavail * filesystem.f_frsize,
            'total_bytes': filesystem.f_blocks * filesystem.f_frsize,
        }
    memory = {}
    for line in Path('/proc/meminfo').read_text().splitlines():
        name, value = line.split(':', 1)
        if name in ['MemAvailable', 'SwapFree', 'SwapTotal']:
            memory[name] = int(value.split()[0]) * 1024
    return {'filesystems': records, 'memory_bytes': memory}


def archive_inputs(controls_directory, validation_directory, helper_records, launcher_records):
    members = {}

    def add_tree(base_path, archive_prefix):
        for file_path in sorted(base_path.rglob('*')):
            if file_path.is_symlink():
                raise AssertionError(f'symlinks are not allowed in archived evidence namespaces: {file_path}')
            if file_path.is_file():
                relative = file_path.relative_to(base_path)
                members[str(Path(archive_prefix) / relative)] = file_path

    add_tree(controls_directory, Path('arrival-phase') / RUN_NAME)
    add_tree(validation_directory, Path('arrival-phase') / VALIDATION_RUN)
    for check_name in VALIDATION_CHECKS:
        build_directory = ROOT / 'builds' / f'{VALIDATION_RUN}-{check_name}'
        if not build_directory.is_dir():
            raise AssertionError(f'validation build directory is missing: {build_directory}')
        for required_name in ['cargo.log', 'provenance.json', 'status.json']:
            if not (build_directory / required_name).is_file():
                raise AssertionError(f'validation build evidence is missing: {build_directory / required_name}')
        add_tree(build_directory, Path('builds') / build_directory.name)
    for helper in helper_records:
        members[helper['archive_path']] = Path(helper['path'])
    for launcher in launcher_records:
        members[str(Path('launcher') / Path(launcher['launcher_log']).name)] = Path(launcher['launcher_log'])
        members[str(Path('launcher') / Path(launcher['launcher_pid_file']).name)] = Path(launcher['launcher_pid_file'])
    for archive_path, source_path in members.items():
        if not source_path.is_file() or source_path.is_symlink():
            raise AssertionError(f'archive input is not a regular file: {source_path}')
    return members


def file_manifest(members):
    return [
        {'relative_archive_path': archive_path, 'bytes': source_path.stat().st_size, 'sha256': digest(source_path)}
        for archive_path, source_path in sorted(members.items())
    ]


def verify_archive_manifest(archive_path, manifest):
    expected = {record['relative_archive_path']: record for record in manifest}
    observed = {}
    with tarfile.open(archive_path, mode='r:gz') as archive:
        for member in archive:
            if not member.isfile():
                raise AssertionError(f'archive contains a non-file member: {member.name}')
            source_file = archive.extractfile(member)
            if source_file is None:
                raise AssertionError(f'archive member cannot be read: {member.name}')
            result = hashlib.sha256()
            size = 0
            with source_file:
                for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
                    result.update(chunk)
                    size += len(chunk)
            observed[member.name] = {'relative_archive_path': member.name,
                                     'bytes': size, 'sha256': result.hexdigest()}
    if observed != expected:
        raise AssertionError('archive contents differ from the recorded file manifest')
    return True


def collect(attempt_name=DEFAULT_ATTEMPT):
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_-]{0,95}', attempt_name):
        raise AssertionError('collection attempt name is invalid')
    namespace = ROOT / attempt_name
    namespace.mkdir(parents=False, exist_ok=False)
    try:
        source_revision = git(['rev-parse', 'HEAD'])
        source_status = git(['status', '--porcelain'])
        source_valid = source_revision == SOURCE_COMMIT and not source_status
        native_bytes = NATIVE_ARCHIVE.stat().st_size
        native_sha256 = digest(NATIVE_ARCHIVE)
        native_commit = subprocess.check_output(
            ['git', '-C', str(NATIVE_ARCHIVE.parent), 'rev-parse', 'HEAD'], text=True
        ).strip()
        validation = validation_artifacts()
        controls_directory = ROOT / 'arrival-phase' / RUN_NAME
        validation_directory = ROOT / 'arrival-phase' / VALIDATION_RUN
        terminal_path, terminal, terminal_status = controls_terminal(controls_directory)
        provenance_path = controls_directory / 'provenance.json'
        controls_provenance = load_json(provenance_path) if provenance_path.is_file() else {}
        if terminal_status != 'initialization_failed' and not controls_provenance:
            raise AssertionError('controls provenance is missing outside an initialization failure')
        if controls_provenance and controls_provenance.get('source_commit') != SOURCE_COMMIT:
            raise AssertionError('controls provenance source commit differs')
        if controls_provenance and controls_provenance.get('run_name') != RUN_NAME:
            raise AssertionError('controls provenance run name differs')
        controls_plan_commit = controls_provenance.get('plan_commit', terminal.get('plan_commit'))
        plan_commit_valid = controls_plan_commit == FROZEN_PLAN_COMMIT
        recorded_validation_sha256 = controls_provenance.get('validation_sha256', terminal.get('validation_sha256'))
        validation_sha_matches = recorded_validation_sha256 == validation['summary_sha256']
        launcher_records = collect_launcher_evidence()
        processes = collect_processes(controls_directory, terminal)
        cgroups = collect_cgroups(controls_directory, terminal)
        if cgroups['all_cgroup_process_lists_empty'] is False:
            raise AssertionError('one or more control cgroups still contain processes')
        statuses = control_statuses(controls_directory, terminal)
        databases = database_inventory(controls_directory)
        helpers = actual_helpers(controls_provenance, validation['summary'])
        control_binaries = control_binary_hashes(controls_directory, validation['summary'])
        preflight_attempts = preflight_launch_attempts(launcher_records)
        members = archive_inputs(controls_directory, validation_directory, helpers, launcher_records)
        manifest = file_manifest(members)
        capacity_before = collect_space()
        archive_path = namespace / 'runtime.tar.gz'
        with tarfile.open(archive_path, mode='x:gz') as archive:
            for archive_name, source_path in sorted(members.items()):
                archive.add(source_path, arcname=archive_name, recursive=False)
        archive_bytes = archive_path.stat().st_size
        archive_sha256 = digest(archive_path)
        manifest_unchanged = all(
            record['bytes'] == members[record['relative_archive_path']].stat().st_size
            and record['sha256'] == digest(members[record['relative_archive_path']])
            for record in manifest
        )
        if not manifest_unchanged:
            raise AssertionError('an evidence file changed while the archive was being created')
        archive_manifest_valid = verify_archive_manifest(archive_path, manifest)
        if git(['rev-parse', 'HEAD']) != source_revision or git(['status', '--porcelain']) != source_status:
            raise AssertionError('source Git state changed during evidence collection')
        if digest(NATIVE_ARCHIVE) != native_sha256 or NATIVE_ARCHIVE.stat().st_size != native_bytes:
            raise AssertionError('native archive changed during evidence collection')
        completion = terminal_status == 'completed'
        native_after_sha256 = digest(NATIVE_ARCHIVE)
        native_after_bytes = NATIVE_ARCHIVE.stat().st_size
        native_valid = native_commit == EXPECTED_NATIVE_COMMIT and native_sha256 == EXPECTED_NATIVE_SHA256
        native_unchanged = native_after_sha256 == native_sha256 and native_after_bytes == native_bytes
        all_attempted_cell_databases_preserved = (
            databases['preserved_case_count'] == cgroups['cell_count']
            and len(databases['cases']) == cgroups['cell_count']
            and all(record['preserved'] for record in databases['cases'])
            if cgroups['cell_count'] else None
        )
        completed_database_set_valid = (
            completion
            and cgroups['cell_count'] == 16
            and all_attempted_cell_databases_preserved is True
        )
        controls_provenance_available = bool(controls_provenance)
        helper_hashes_match = all(
            helper['hash_matches_provenance'] is True
            for helper in helpers if helper['archive_path'] != 'arrival-phase/oci_arrival_collect.py'
        ) if controls_provenance_available else False
        completed_run_evidence_valid = (
            completion
            and statuses['all_sixteen_cells_operationally_valid']
            and statuses['status_cases_match_terminal_outcomes'] is True
            and cgroups['grouping_root_memory_max_is_unbounded']
            and cgroups['grouping_root_path_matches_expected']
            and cgroups['cell_epoch_budgets_match'] is True
            and cgroups['all_expected_cells_have_commands'] is True
            and cgroups['all_cgroup_process_lists_empty']
            and plan_commit_valid and source_valid and native_valid and native_unchanged
            and validation['valid'] and validation_sha_matches
            and all(binary.get('matches_validation') is True for binary in control_binaries.values())
            and completed_database_set_valid and helper_hashes_match
            and len(processes) >= 16
            and all(not process['same_live_process_at_collection'] for process in processes)
            and manifest_unchanged and archive_manifest_valid
        )
        accepted_run = completed_run_evidence_valid and statuses['all_sixteen_cells_slo_pass']
        receipt = {
            'collection_status': 'completed',
            'attempt_name': attempt_name,
            'controls_terminal_status': terminal_status,
            'controls_success_preserved': completion,
            'controls_terminal_path': str(terminal_path),
            'controls_terminal_sha256': digest(terminal_path),
            'controls_validity': {
                'completion_receipt_present': completion,
                'failure_receipt_present': terminal_status == 'failed',
                'initialization_failure_receipt_present': terminal_status == 'initialization_failed',
                'source_commit_matches': terminal.get('source_commit') == SOURCE_COMMIT,
                'frozen_plan_commit_matches': plan_commit_valid,
                'actual_controller_commit_recorded': ACTUAL_CONTROLLER_COMMIT,
                'sixteen_cells_required_only_for_completion': completion,
                'all_sixteen_cells_operationally_valid': statuses['all_sixteen_cells_operationally_valid'],
                'all_sixteen_cells_slo_pass': statuses['all_sixteen_cells_slo_pass'],
                'slo_miss_cases': statuses['slo_miss_cases'],
                'status_cases_match_terminal_outcomes': statuses['status_cases_match_terminal_outcomes'],
                'grouping_root_memory_max_is_unbounded': cgroups['grouping_root_memory_max_is_unbounded'],
                'cell_epoch_budgets_match': cgroups['cell_epoch_budgets_match'],
                'all_cgroup_process_lists_empty': cgroups['all_cgroup_process_lists_empty'],
                'validation_source_native_and_binaries_valid': validation['valid'],
                'validation_summary_hash_matches_receipt': validation_sha_matches,
                'control_binaries_match_validation': all(
                    binary.get('matches_validation') is True for binary in control_binaries.values()
                ),
                'all_attempted_cell_databases_preserved': all_attempted_cell_databases_preserved,
                'all_completed_cell_databases_preserved': completed_database_set_valid,
                'helper_hashes_match_run_provenance': helper_hashes_match,
                'completed_run_evidence_valid': completed_run_evidence_valid,
                'accepted_run': accepted_run,
            },
            'source': {'path': str(SOURCE), 'commit': source_revision, 'clean': not source_status,
                       'expected_commit': SOURCE_COMMIT, 'valid': source_valid},
            'native': {'commit': native_commit, 'expected_commit': EXPECTED_NATIVE_COMMIT,
                       'archive_path': str(NATIVE_ARCHIVE), 'archive_bytes': native_bytes,
                       'archive_sha256': native_sha256, 'expected_archive_sha256': EXPECTED_NATIVE_SHA256,
                       'archive_sha256_after': native_after_sha256,
                       'archive_bytes_after': native_after_bytes,
                       'unchanged_during_collection': native_unchanged,
                       'valid': native_valid and native_unchanged},
            'validation': {'path': str(validation_directory), 'summary_path': validation['summary_path'],
                           'summary_sha256': validation['summary_sha256'], 'valid': validation['valid'],
                           'passing_tests': validation['summary'].get('outcomes', [{}])[0].get('passing_tests'),
                           'checks': validation['summary'].get('outcomes', []),
                           'default_binaries': validation['binary_hashes']},
            'controls': {'path': str(controls_directory), 'provenance_path': str(provenance_path),
                         'provenance_present': provenance_path.is_file(),
                         'provenance_sha256': digest(provenance_path) if provenance_path.is_file() else None,
                         'plan_commit': controls_plan_commit,
                         'frozen_plan_commit': FROZEN_PLAN_COMMIT,
                         'actual_controller_commit': ACTUAL_CONTROLLER_COMMIT,
                         'source_validation': controls_provenance.get('source_validation', terminal.get('source_validation')),
                         'runtime_limits': controls_provenance.get('runtime_limits', terminal.get('runtime_limits')),
                         'validation_sha256_recorded': recorded_validation_sha256,
                         'launcher_attempt_count': len(launcher_records) - 1,
                         'preflight_launch_attempts': preflight_attempts,
                         'actual_cell_command_count': cgroups['cell_count'],
                         'actual_cell_status_count': statuses['status_count'],
                         'statuses': statuses, 'control_binaries': control_binaries,
                         'processes': processes, 'cgroups': cgroups, 'databases': databases,
                         'launchers': launcher_records},
            'helpers': helpers,
            'space_before_archive': capacity_before,
            'space_after_archive': collect_space(),
            'database_preserved_case_count': databases['preserved_case_count'],
            'archive': {'path': str(archive_path), 'bytes': archive_bytes, 'sha256': archive_sha256},
            'file_manifest_scope': 'regular files included in runtime.tar.gz; collection.json is a sidecar to avoid self-reference',
            'file_manifest': manifest,
            'manifest_unchanged_after_archive': manifest_unchanged,
            'archive_matches_file_manifest': archive_manifest_valid,
        }
        sidecar_path = namespace / 'collection.json'
        with sidecar_path.open('x') as sidecar:
            json.dump(receipt, sidecar, indent=2)
            sidecar.write('\n')
        return receipt
    except Exception as error:
        failure_path = namespace / 'collection-failure.json'
        if not failure_path.exists():
            with failure_path.open('x') as output:
                json.dump({'collection_status': 'failed', 'attempt_name': attempt_name,
                           'failure_type': type(error).__name__, 'failure': str(error)}, output, indent=2)
                output.write('\n')
        raise


def main():
    parser = argparse.ArgumentParser(description='Collect immutable arrival-phase controls evidence')
    parser.add_argument('--attempt-name', default=DEFAULT_ATTEMPT)
    arguments = parser.parse_args()
    receipt = collect(arguments.attempt_name)
    print(json.dumps({
        'collection_status': receipt['collection_status'],
        'controls_terminal_status': receipt['controls_terminal_status'],
        'archive': receipt['archive'],
        'database_preserved_case_count': receipt['database_preserved_case_count'],
        'collection_json': str(ROOT / arguments.attempt_name / 'collection.json'),
    }, sort_keys=True), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
