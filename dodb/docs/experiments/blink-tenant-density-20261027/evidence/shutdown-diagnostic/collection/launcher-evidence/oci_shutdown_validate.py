import json
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
EXPECTED_SOURCE = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
BASELINE_BUILD = 'observations-clean'
SUMMARY_NAME = 'shutdown-validation-summary.json'
NATIVE_ARCHIVE = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')
EXPECTED_NATIVE_ARCHIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
sys.path.insert(0, str(ROOT))
from oci_harness_build import build, digest


BASE = ['--locked', '--offline', '--release', '-p', 'dodb-rocksdb']
BINS = ['--bin', 'tenant_density_workload', '--bin', 'memory_diagnostic_worker']
CHECKS = [
    ('shutdown-clean-tests', ['test', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--', '--test-threads=1']),
    ('shutdown-clean-clippy', ['clippy', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--no-deps']),
    ('shutdown-clean-a1-link', ['build', *BASE, *BINS, '--features', 'dodb-storage/blink-writer-pin-reuse']),
    ('shutdown-clean-default-link', ['build', *BASE, *BINS]),
]


def command(arguments):
    return subprocess.check_output(arguments, cwd=SOURCE, text=True).strip()


def test_counts(log_text):
    counts = [int(match.group(1)) for match in re.finditer(r'test result: ok\. (\d+) passed;', log_text)]
    assert counts
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
        assert path is not None, line
        if '/dodb/' in path:
            path = path[path.index('dodb/'):]
        diagnostics.append({'message': match.group(1), 'path': path})
    return sorted(diagnostics, key=lambda item: (item['path'], item['message']))


def write_summary(summary_path, summary):
    serialized = json.dumps(summary, indent=2) + '\n'
    if summary_path.exists():
        summary_path.write_text(serialized)
    else:
        with summary_path.open('x') as output:
            output.write(serialized)


def validated_existing_check(name, arguments, revision):
    build_directory = ROOT / 'builds' / name
    status_path = build_directory / 'status.json'
    provenance_path = build_directory / 'provenance.json'
    assert build_directory.is_dir() and status_path.is_file() and provenance_path.is_file()
    provenance = json.loads(provenance_path.read_text())
    status = json.loads(status_path.read_text())
    assert provenance['source_commit'] == revision
    assert provenance['command'][4:] == arguments
    assert (build_directory / 'cargo.log').is_file()
    return status['exit_code']


def main():
    revision = command(['git', 'rev-parse', 'HEAD'])
    assert revision == EXPECTED_SOURCE
    assert not command(['git', 'status', '--porcelain'])
    target = ROOT / 'target'
    assert target.is_symlink()
    target_link = target.readlink()
    target_directory = target.resolve(strict=True)
    native_bytes = NATIVE_ARCHIVE.stat().st_size
    native_sha256 = digest(NATIVE_ARCHIVE)
    assert native_sha256 == EXPECTED_NATIVE_ARCHIVE_SHA256
    baseline_tests = ROOT / 'builds' / (BASELINE_BUILD + '-tests') / 'cargo.log'
    baseline_clippy = ROOT / 'builds' / (BASELINE_BUILD + '-clippy') / 'cargo.log'
    assert baseline_tests.is_file() and baseline_clippy.is_file()
    baseline_test_counts = test_counts(baseline_tests.read_text())
    baseline_diagnostics = clippy_diagnostics(baseline_clippy.read_text())
    assert sum(baseline_test_counts) == 74
    assert len(baseline_diagnostics) == 17
    summary_path = ROOT / SUMMARY_NAME
    existing_summary = json.loads(summary_path.read_text()) if summary_path.exists() else None
    assert CHECKS[-1][0] == 'shutdown-clean-default-link'
    outcomes = existing_summary['outcomes'] if existing_summary is not None else []
    if existing_summary is not None:
        assert existing_summary['source_commit'] == revision
        assert [outcome['name'] for outcome in outcomes] == [name for name, _ in CHECKS[:len(outcomes)]]
        assert len(outcomes) <= len(CHECKS)
        assert all(outcome['exit_code'] == 0 for outcome in outcomes)
        assert existing_summary['baseline_passing_tests'] == sum(baseline_test_counts)
        assert existing_summary['baseline_clippy_diagnostics'] == baseline_diagnostics
        assert existing_summary['native_archive_sha256_before'] == native_sha256
        assert existing_summary['native_archive_bytes_before'] == native_bytes
        assert existing_summary['target_symlink_before'] == str(target_link)
        assert existing_summary['resolved_target_directory_before'] == str(target_directory)
    summary = {
        'source_commit': revision,
        'baseline_passing_tests': sum(baseline_test_counts),
        'baseline_test_group_counts': baseline_test_counts,
        'baseline_clippy_diagnostics': baseline_diagnostics,
        'native_archive_sha256_before': native_sha256,
        'native_archive_bytes_before': native_bytes,
        'target_symlink_before': str(target_link),
        'resolved_target_directory_before': str(target_directory),
        'outcomes': outcomes,
    }
    if existing_summary is not None:
        for summary_key in [
            'native_archive_sha256_after', 'native_archive_bytes_after',
            'target_symlink_after', 'resolved_target_directory_after',
            'passing_tests', 'same_17_existing_clippy_diagnostics', 'default_link_was_last',
        ]:
            if summary_key in existing_summary:
                summary[summary_key] = existing_summary[summary_key]
    for check_index, (name, arguments) in enumerate(CHECKS):
        build_directory = ROOT / 'builds' / name
        if build_directory.exists():
            exit_code = validated_existing_check(name, arguments, revision)
            reused_completed_check = True
        else:
            assert check_index >= len(outcomes)
            exit_code = build(name, arguments)
            reused_completed_check = False
        log_path = ROOT / 'builds' / name / 'cargo.log'
        assert log_path.is_file()
        outcome = {'name': name, 'exit_code': exit_code, 'command': arguments,
                   'reused_completed_check': reused_completed_check}
        if name == 'shutdown-clean-tests' and exit_code == 0:
            current_test_counts = test_counts(log_path.read_text())
            outcome['test_group_counts'] = current_test_counts
            outcome['passing_tests'] = sum(current_test_counts)
            assert outcome['passing_tests'] == 79
        if name == 'shutdown-clean-clippy' and exit_code == 0:
            current_diagnostics = clippy_diagnostics(log_path.read_text())
            outcome['clippy_diagnostics'] = current_diagnostics
            outcome['same_17_existing_clippy_diagnostics'] = current_diagnostics == baseline_diagnostics
            assert outcome['same_17_existing_clippy_diagnostics']
        if existing_summary is None:
            outcomes.append(outcome)
        else:
            if check_index < len(outcomes):
                assert outcomes[check_index]['exit_code'] == exit_code
                assert outcomes[check_index]['command'] == arguments
                outcomes[check_index] = outcome
            else:
                outcomes.append(outcome)
        summary['outcomes'] = outcomes
        write_summary(summary_path, summary)
        print(json.dumps(outcome), flush=True)
        assert exit_code == 0, outcome
    assert outcomes[-1]['name'] == 'shutdown-clean-default-link'
    assert sum(outcome.get('passing_tests', 0) for outcome in outcomes) == 79
    assert outcomes[1]['same_17_existing_clippy_diagnostics']
    assert command(['git', 'rev-parse', 'HEAD']) == EXPECTED_SOURCE
    assert not command(['git', 'status', '--porcelain'])
    assert target.is_symlink() and target.readlink() == target_link
    assert target.resolve(strict=True) == target_directory
    native_after_bytes = NATIVE_ARCHIVE.stat().st_size
    assert digest(NATIVE_ARCHIVE) == native_sha256
    assert native_after_bytes == native_bytes
    summary.update({
        'native_archive_sha256_after': native_sha256,
        'native_archive_bytes_after': native_after_bytes,
        'target_symlink_after': str(target.readlink()),
        'resolved_target_directory_after': str(target.resolve(strict=True)),
        'passing_tests': 79,
        'same_17_existing_clippy_diagnostics': True,
        'default_link_was_last': True,
    })
    if existing_summary is not None and 'native_archive_sha256_after' in existing_summary:
        assert existing_summary['native_archive_sha256_after'] == native_sha256
        assert existing_summary['native_archive_bytes_after'] == native_after_bytes
        assert existing_summary['target_symlink_after'] == str(target.readlink())
        assert existing_summary['resolved_target_directory_after'] == str(target.resolve(strict=True))
    else:
        write_summary(summary_path, summary)
    print(json.dumps({key: summary[key] for key in [
        'source_commit', 'passing_tests', 'baseline_passing_tests',
        'same_17_existing_clippy_diagnostics', 'default_link_was_last',
        'native_archive_sha256_after', 'resolved_target_directory_after']}), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
