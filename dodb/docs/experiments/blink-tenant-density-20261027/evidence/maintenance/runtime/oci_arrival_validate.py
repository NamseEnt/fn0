import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
RUN_NAME = 'arrival-cb0e04482-r1'
EXPECTED_SOURCE = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
BASELINE_SOURCE = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
NATIVE = Path('/bench/zfs/db/product-build/rocksdb-source')
NATIVE_ARCHIVE = NATIVE / 'librocksdb.a'
EXPECTED_NATIVE_COMMIT = 'abeebd9630f11bd08c28b7bd43c7bdfc62050654'
EXPECTED_NATIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
SUMMARY_NAME = 'arrival-validation-summary.json'
BASELINE_SUMMARY = ROOT / 'shutdown-validation-summary.json'
BASELINE_RUNS = ('shutdown-clean-tests', 'shutdown-clean-clippy')
RUSTUP = '/home/opc/.cargo/bin/rustup'
TOOLCHAIN = [
    'rustc 1.97.1 (8bab26f4f 2026-07-14)',
    'binary: rustc',
    'commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452',
    'commit-date: 2026-07-14',
    'host: aarch64-unknown-linux-gnu',
    'release: 1.97.1',
    'LLVM version: 22.1.6',
]
BASE = ['--locked', '--offline', '--release', '-p', 'dodb-rocksdb']
BINS = ['--bin', 'tenant_density_workload', '--bin', 'memory_diagnostic_worker']
CHECKS = [
    ('tests', ['test', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--', '--test-threads=1']),
    ('clippy', ['clippy', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--no-deps']),
    ('a1-link', ['build', *BASE, *BINS, '--features', 'dodb-storage/blink-writer-pin-reuse']),
    ('default-link', ['build', *BASE, *BINS]),
]


def digest(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def command(arguments, cwd=None):
    return subprocess.check_output(arguments, cwd=cwd, text=True).strip()


def build(run_name, arguments):
    sys.path.insert(0, str(ROOT))
    from oci_harness_build import build as harness_build

    return harness_build(run_name, arguments)


def test_counts(log_text):
    counts = [int(match.group(1)) for match in re.finditer(r'test result: ok\. (\d+) passed;', log_text)]
    return counts


def clippy_diagnostics(log_text):
    lines = log_text.splitlines()
    diagnostics = []
    for line_index, line in enumerate(lines):
        match = re.match(r'warning: (.+)', line)
        if not match or match.group(1).startswith('`'):
            continue
        path = None
        for following_line in lines[line_index + 1:line_index + 12]:
            location = re.match(r'\s*--> (.+?):\d+:\d+', following_line)
            if location:
                path = location.group(1)
                break
        if path is None:
            raise AssertionError(f'Clippy diagnostic has no source location: {line}')
        if '/dodb/' in path:
            path = path[path.index('dodb/'):]
        diagnostics.append({'message': match.group(1), 'path': path})
    return sorted(diagnostics, key=lambda item: (item['path'], item['message']))


def write_summary(path, summary):
    temporary_path = path.with_name(path.name + '.tmp')
    serialized = json.dumps(summary, indent=2) + '\n'
    temporary_path.write_text(serialized)
    os.replace(temporary_path, path)


def full_build_command(arguments):
    return [RUSTUP, 'run', '1.97.1', 'cargo', *arguments]


def expected_environment():
    return {
        'DODB_ROCKSDB_DIR': str(NATIVE),
        'CARGO_TARGET_DIR': str(ROOT / 'target'),
        'CARGO_INCREMENTAL': '0',
        'CARGO_BUILD_JOBS': '2',
        'CARGO_PROFILE_RELEASE_DEBUG': '0',
        'CARGO_PROFILE_RELEASE_STRIP': 'debuginfo',
        'TMPDIR': str(ROOT / 'tmp'),
    }


def validate_source_and_native():
    revision = command(['git', 'rev-parse', 'HEAD'], cwd=SOURCE)
    status = command(['git', 'status', '--porcelain'], cwd=SOURCE)
    if revision != EXPECTED_SOURCE or status:
        raise AssertionError(f'source must be clean at {EXPECTED_SOURCE}: {revision} {status!r}')
    if not (ROOT / 'target').is_symlink():
        raise AssertionError('the frozen cargo target must remain a symlink')
    target_link = str((ROOT / 'target').readlink())
    target_directory = str((ROOT / 'target').resolve(strict=True))
    native_commit = command(['git', '-C', str(NATIVE), 'rev-parse', 'HEAD'])
    if native_commit != EXPECTED_NATIVE_COMMIT:
        raise AssertionError(f'unexpected native source commit {native_commit}')
    native_bytes = NATIVE_ARCHIVE.stat().st_size
    native_sha256 = digest(NATIVE_ARCHIVE)
    if native_sha256 != EXPECTED_NATIVE_SHA256:
        raise AssertionError(f'unexpected pinned native archive digest {native_sha256}')
    return {
        'source_commit': revision,
        'source_status': status,
        'native_commit': native_commit,
        'native_archive_sha256_before': native_sha256,
        'native_archive_bytes_before': native_bytes,
        'target_symlink_before': target_link,
        'resolved_target_directory_before': target_directory,
    }


def accepted_baseline():
    if not BASELINE_SUMMARY.is_file():
        raise AssertionError(f'missing accepted shutdown validation summary: {BASELINE_SUMMARY}')
    baseline = json.loads(BASELINE_SUMMARY.read_text())
    if baseline.get('source_commit') != BASELINE_SOURCE:
        raise AssertionError('shutdown validation summary has a different baseline source')
    if baseline.get('passing_tests') != 79 or baseline.get('same_17_existing_clippy_diagnostics') is not True:
        raise AssertionError('shutdown validation summary is not the accepted 79-test / 17-diagnostic baseline')
    outcomes = {outcome['name']: outcome for outcome in baseline.get('outcomes', [])}
    if any(name not in outcomes or outcomes[name].get('exit_code') != 0 for name in BASELINE_RUNS):
        raise AssertionError('accepted shutdown test and Clippy outcomes are incomplete')
    baseline_tests_path = ROOT / 'builds' / 'shutdown-clean-tests' / 'cargo.log'
    baseline_clippy_path = ROOT / 'builds' / 'shutdown-clean-clippy' / 'cargo.log'
    if not baseline_tests_path.is_file() or not baseline_clippy_path.is_file():
        raise AssertionError('accepted shutdown test or Clippy log is missing')
    group_counts = test_counts(baseline_tests_path.read_text())
    diagnostics = clippy_diagnostics(baseline_clippy_path.read_text())
    if sum(group_counts) != 79 or group_counts != baseline.get('outcomes')[0].get('test_group_counts'):
        raise AssertionError('accepted shutdown test log does not match its summary')
    if diagnostics != baseline.get('baseline_clippy_diagnostics'):
        raise AssertionError('accepted shutdown Clippy log does not match its summary')
    return {
        'source_commit': BASELINE_SOURCE,
        'passing_tests': sum(group_counts),
        'test_group_counts': group_counts,
        'clippy_diagnostics': diagnostics,
        'same_17_existing_clippy_diagnostics': True,
        'summary_sha256': digest(BASELINE_SUMMARY),
        'test_log_sha256': digest(baseline_tests_path),
        'clippy_log_sha256': digest(baseline_clippy_path),
    }


def validate_existing_build(name, arguments, native_sha256, baseline, build_helper_sha256, allow_failure=False):
    build_directory = ROOT / 'builds' / f'{RUN_NAME}-{name}'
    status_path = build_directory / 'status.json'
    provenance_path = build_directory / 'provenance.json'
    cargo_log = build_directory / 'cargo.log'
    if not build_directory.is_dir() or not status_path.is_file() or not provenance_path.is_file() or not cargo_log.is_file():
        raise AssertionError(f'incomplete prior build must be preserved and cannot be retried: {build_directory}')
    provenance = json.loads(provenance_path.read_text())
    status = json.loads(status_path.read_text())
    expected_command = full_build_command(arguments)
    if provenance.get('source_commit') != EXPECTED_SOURCE or provenance.get('source_status') != '':
        raise AssertionError(f'prior build has mismatched source provenance: {name}')
    if provenance.get('command') != expected_command:
        raise AssertionError(f'prior build command differs: {name}')
    if provenance.get('native_commit') != EXPECTED_NATIVE_COMMIT or provenance.get('native_archive_sha256') != native_sha256:
        raise AssertionError(f'prior build native provenance differs: {name}')
    if provenance.get('toolchain') != TOOLCHAIN or provenance.get('environment') != expected_environment():
        raise AssertionError(f'prior build toolchain or environment differs: {name}')
    if provenance.get('cwd') != str(SOURCE):
        raise AssertionError(f'prior build working directory differs: {name}')
    if provenance.get('controller_sha256') != build_helper_sha256:
        raise AssertionError(f'prior build tool provenance differs: {name}')
    exit_code = status.get('exit_code')
    if not isinstance(exit_code, int):
        raise AssertionError(f'prior build has no integer exit status: {name}')
    if exit_code != 0 and not allow_failure:
        raise AssertionError(f'prior build failed and must not be retried: {name}')
    log_text = cargo_log.read_text()
    outcome = {
        'name': name,
        'exit_code': exit_code,
        'command': arguments,
        'invocation': provenance['command'],
        'build_directory': str(build_directory),
        'provenance_path': str(provenance_path),
        'cargo_log_path': str(cargo_log),
        'status_path': str(status_path),
        'provenance_sha256': digest(provenance_path),
        'cargo_log_sha256': digest(cargo_log),
        'reused_completed_check': True,
    }
    if name == 'tests' and exit_code == 0:
        group_counts = test_counts(log_text)
        expected_group_counts = list(baseline['test_group_counts'])
        if len(expected_group_counts) != 5:
            raise AssertionError('accepted shutdown baseline test groups do not match the frozen check set')
        expected_group_counts[2] += 6
        outcome.update({
            'test_group_counts': group_counts,
            'passing_tests': sum(group_counts),
            'expected_test_group_counts': expected_group_counts,
            'expected_test_count_matches': group_counts == expected_group_counts,
        })
    elif name == 'clippy':
        diagnostics = clippy_diagnostics(log_text)
        outcome.update({
            'clippy_diagnostics': diagnostics,
            'same_17_existing_clippy_diagnostics': exit_code == 0 and diagnostics == baseline['clippy_diagnostics'],
        })
    return outcome


def same_check_result(recorded, validated):
    stable_keys = [
        'name', 'exit_code', 'command', 'invocation', 'provenance_sha256', 'cargo_log_sha256',
        'build_directory', 'provenance_path', 'cargo_log_path', 'status_path',
        'test_group_counts', 'passing_tests', 'clippy_diagnostics', 'same_17_existing_clippy_diagnostics',
        'expected_test_group_counts', 'expected_test_count_matches',
    ]
    return all(recorded.get(key) == validated.get(key) for key in stable_keys)


def copy_default_binaries(namespace, summary):
    binary_directory = namespace / 'bin'
    binary_directory.mkdir(exist_ok=True)
    binaries = {}
    for binary_name in ['tenant_density_workload', 'memory_diagnostic_worker']:
        source_binary = ROOT / 'target' / 'release' / binary_name
        destination = binary_directory / binary_name
        if not source_binary.is_file():
            raise AssertionError(f'default-linked binary is missing: {source_binary}')
        source_identity = {'sha256': digest(source_binary), 'bytes': source_binary.stat().st_size}
        if destination.exists():
            if not destination.is_file() or digest(destination) != source_identity['sha256'] or destination.stat().st_size != source_identity['bytes']:
                raise AssertionError(f'preserved binary differs and will not be overwritten: {destination}')
        else:
            with source_binary.open('rb') as source_file, destination.open('xb') as destination_file:
                shutil.copyfileobj(source_file, destination_file, 1024 * 1024)
            shutil.copystat(source_binary, destination)
        binaries[binary_name] = source_identity
    summary['default_binaries'] = binaries


def run_validation():
    namespace = ROOT / 'arrival-phase' / RUN_NAME
    summary_path = namespace / SUMMARY_NAME
    if namespace.exists() and not summary_path.is_file():
        raise AssertionError(f'incomplete validation namespace is preserved and cannot be reused: {namespace}')
    namespace.mkdir(parents=True, exist_ok=True)
    controller_sha256 = digest(Path(__file__).resolve())
    helper_sha256 = digest(ROOT / 'oci_harness_build.py')
    if summary_path.is_file():
        summary = json.loads(summary_path.read_text())
        if summary.get('run_name') != RUN_NAME or summary.get('controller_sha256') != controller_sha256:
            raise AssertionError('existing validation summary belongs to a different controller')
        if summary.get('status') == 'failed' or summary.get('status') == 'completed':
            raise AssertionError('validation namespace is terminal and cannot be retried')
        outcomes = summary.get('outcomes', [])
        if [outcome.get('name') for outcome in outcomes] != [name for name, _ in CHECKS[:len(outcomes)]]:
            raise AssertionError('existing validation outcomes are not a sequential prefix')
    else:
        summary = {
            'run_name': RUN_NAME,
            'status': 'preflight',
            'controller_sha256': controller_sha256,
            'build_helper_sha256': helper_sha256,
            'outcomes': [],
        }
    try:
        source_native = validate_source_and_native()
        baseline = accepted_baseline()
        if summary.get('build_helper_sha256') != helper_sha256:
            raise AssertionError('existing validation used a different build helper')
        if 'source_commit' in summary and any(summary.get(key) != value for key, value in source_native.items()):
            raise AssertionError('existing validation source/native/target provenance changed')
        if 'baseline' in summary and summary['baseline'] != baseline:
            raise AssertionError('accepted shutdown baseline provenance changed')
        summary.update(source_native)
        summary['baseline'] = baseline
        summary['toolchain'] = TOOLCHAIN
        summary['build_environment'] = expected_environment()
        summary['status'] = 'in_progress'
        write_summary(summary_path, summary)
        outcomes = summary['outcomes']
        if len(outcomes) > len(CHECKS):
            raise AssertionError('existing validation has more outcomes than the frozen matrix')
        for check_index, (name, arguments) in enumerate(CHECKS):
            build_directory = ROOT / 'builds' / f'{RUN_NAME}-{name}'
            if check_index < len(outcomes):
                outcome = validate_existing_build(name, arguments, source_native['native_archive_sha256_before'], baseline, helper_sha256, allow_failure=True)
                if not same_check_result(outcomes[check_index], outcome):
                    raise AssertionError(f'reused check differs from recorded outcome: {name}')
                outcome = outcomes[check_index]
            elif build_directory.exists():
                outcome = validate_existing_build(name, arguments, source_native['native_archive_sha256_before'], baseline, helper_sha256, allow_failure=True)
            else:
                started_at = time.time()
                exit_code = build(f'{RUN_NAME}-{name}', arguments)
                outcome = validate_existing_build(name, arguments, source_native['native_archive_sha256_before'], baseline, helper_sha256, allow_failure=True)
                outcome['build_started_unix_seconds'] = started_at
                outcome['reused_completed_check'] = False
                if exit_code != outcome['exit_code']:
                    raise AssertionError(f'build return code differs from saved status: {name}')
            if check_index >= len(outcomes):
                outcomes.append(outcome)
            summary['outcomes'] = outcomes
            summary['last_completed_check'] = name
            write_summary(summary_path, summary)
            if outcome['exit_code'] != 0:
                summary['status'] = 'failed'
                write_summary(summary_path, summary)
                raise AssertionError(f'validation check failed: {name} exit={outcome["exit_code"]}')
            if name == 'tests' and not outcome.get('expected_test_count_matches'):
                summary['status'] = 'failed'
                write_summary(summary_path, summary)
                raise AssertionError(f'validation test count differs from baseline plus six: {outcome.get("test_group_counts")}')
            if name == 'clippy' and outcome.get('same_17_existing_clippy_diagnostics') is not True:
                summary['status'] = 'failed'
                write_summary(summary_path, summary)
                raise AssertionError('Clippy diagnostics differ from the accepted shutdown baseline')
        if outcomes[-1]['name'] != 'default-link':
            raise AssertionError('default link was not the final validation step')
        expected_test_group_counts = list(baseline['test_group_counts'])
        expected_test_group_counts[2] += 6
        if outcomes[0].get('test_group_counts') != expected_test_group_counts:
            raise AssertionError('unexpected arrival-phase test group counts')
        if outcomes[1].get('same_17_existing_clippy_diagnostics') is not True:
            raise AssertionError('Clippy diagnostics differ from accepted baseline')
        copy_default_binaries(namespace, summary)
        after = validate_source_and_native()
        after_keys = {
            'native_archive_sha256_before': 'native_archive_sha256_after',
            'native_archive_bytes_before': 'native_archive_bytes_after',
            'target_symlink_before': 'target_symlink_after',
            'resolved_target_directory_before': 'resolved_target_directory_after',
        }
        if any(source_native[before_key] != after[before_key] for before_key in after_keys):
            raise AssertionError('source, native archive, or target changed during validation')
        for before_key, after_key in after_keys.items():
            summary[after_key] = after[before_key]
        summary['status'] = 'completed'
        summary['default_link_was_last'] = True
        summary['finished_unix_seconds'] = time.time()
        write_summary(summary_path, summary)
        return summary
    except Exception as error:
        summary['status'] = 'failed'
        summary['failure'] = {'type': type(error).__name__, 'message': str(error), 'failed_unix_seconds': time.time()}
        write_summary(summary_path, summary)
        raise


def main():
    argparse.ArgumentParser(description='Validate the frozen arrival-phase source').parse_args()
    summary = run_validation()
    print(json.dumps({
        'run_name': summary['run_name'],
        'status': summary['status'],
        'passing_tests': summary['outcomes'][0]['passing_tests'],
        'same_17_existing_clippy_diagnostics': summary['outcomes'][1]['same_17_existing_clippy_diagnostics'],
        'default_link_was_last': summary['default_link_was_last'],
        'default_binaries': summary['default_binaries'],
    }, sort_keys=True), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
