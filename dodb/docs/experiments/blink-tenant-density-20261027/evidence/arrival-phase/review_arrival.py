import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import sys
import tarfile


SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
CONTROLLER_COMMIT = '95c3b8c5323415dfc493bdfcb32c7dc5e057e8b8'
PLAN_COMMIT = 'e02a1fbdde81fd6fc363a26e5341c8c61000023e'
RUN_NAME = 'arrival-controls-cb0e04482-r1'
VALIDATION_RUN = 'arrival-cb0e04482-r1'
EXPECTED_NATIVE_COMMIT = 'abeebd9630f11bd08c28b7bd43c7bdfc62050654'
EXPECTED_NATIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
EXPECTED_TEST_GROUP_COUNTS = [42, 9, 25, 5, 4]
EXPECTED_VALIDATION_CHECKS = ['tests', 'clippy', 'a1-link', 'default-link']
EXPECTED_REPORT_CASES = [
    'tiny-synchronized-blink', 'tiny-synchronized-shared',
    'tiny-staggered-shared', 'tiny-staggered-blink',
    'control-n28-r1-synchronized-blink', 'control-n28-r1-synchronized-shared',
    'control-n28-r1-staggered-shared', 'control-n28-r1-staggered-blink',
    'control-n28-r2-staggered-blink', 'control-n28-r2-staggered-shared',
    'control-n28-r2-synchronized-shared', 'control-n28-r2-synchronized-blink',
    'control-n28-r3-synchronized-blink', 'control-n28-r3-synchronized-shared',
    'control-n28-r3-staggered-shared', 'control-n28-r3-staggered-blink',
]
EXPECTED_PAIRS = [
    'tiny-synchronized', 'tiny-staggered',
    'control-n28-r1-synchronized', 'control-n28-r1-staggered',
    'control-n28-r2-staggered', 'control-n28-r2-synchronized',
    'control-n28-r3-synchronized', 'control-n28-r3-staggered',
]
PHASE_PAIRS = [
    ('tiny-synchronized', 'tiny-staggered'),
    ('control-n28-r1-synchronized', 'control-n28-r1-staggered'),
    ('control-n28-r2-synchronized', 'control-n28-r2-staggered'),
    ('control-n28-r3-synchronized', 'control-n28-r3-staggered'),
]
EXPECTED_HELPER_ARCHIVE_PATHS = {
    'arrival-phase/oci_arrival_validate.py',
    'arrival-phase/oci_arrival_controls.py',
    'arrival-phase/arrival_review.py',
    'arrival-phase/oci_arrival_collect.py',
    'capacity/oci_capacity.py',
    'capacity/capacity_review.py',
}
NATIVE_CACHE_BYTES = 64 * 1024**2
NATIVE_WBM_BYTES = 32 * 1024**2
BACKEND_MEMORY_MAX_BYTES = 256 * 1024**2
BACKEND_MEMORY_HIGH_BYTES = 192 * 1024**2


def sha256_file(path):
    result = hashlib.sha256()
    with Path(path).open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def load_module(name, path):
    module_path = Path(path)
    if not module_path.is_file():
        raise AssertionError(f'required review helper is missing: {module_path}')
    sys.path.insert(0, str(module_path.parent))
    specification = importlib.util.spec_from_file_location(name, module_path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[name] = module
    specification.loader.exec_module(module)
    return module


def safe_archive_path(value):
    if not isinstance(value, str) or not value or '\\' in value:
        raise AssertionError(f'invalid archive path: {value!r}')
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in {'', '.', '..'} for part in path.parts):
        raise AssertionError(f'unsafe archive path: {value!r}')
    if str(path) != value:
        raise AssertionError(f'archive path is not canonical: {value!r}')
    if path.parts[0] not in {'arrival-phase', 'builds', 'capacity', 'launcher'}:
        raise AssertionError(f'archive path has an unexpected prefix: {value!r}')
    return path


def manifest_index(manifest, terminal_status):
    if not isinstance(manifest, list) or not manifest:
        raise AssertionError('collection file manifest is empty or malformed')
    indexed = {}
    for record in manifest:
        archive_path = record.get('relative_archive_path')
        safe_archive_path(archive_path)
        if archive_path in indexed:
            raise AssertionError(f'duplicate archive manifest path: {archive_path}')
        if not isinstance(record.get('bytes'), int) or record['bytes'] < 0:
            raise AssertionError(f'invalid byte count in archive manifest: {archive_path}')
        if not isinstance(record.get('sha256'), str) or not re.fullmatch(r'[0-9a-f]{64}', record['sha256']):
            raise AssertionError(f'invalid SHA-256 in archive manifest: {archive_path}')
        indexed[archive_path] = {'bytes': record['bytes'], 'sha256': record['sha256']}
    terminal_name = {
        'completed': 'completion.json',
        'failed': 'failure-receipt.json',
        'initialization_failed': 'initialization-failure-receipt.json',
    }.get(terminal_status)
    if terminal_name is None:
        raise AssertionError(f'unknown controls terminal state: {terminal_status}')
    required = {
        f'arrival-phase/{RUN_NAME}/{terminal_name}',
        f'arrival-phase/{VALIDATION_RUN}/arrival-validation-summary.json',
        *EXPECTED_HELPER_ARCHIVE_PATHS,
    }
    missing = required - set(indexed)
    if missing:
        raise AssertionError(f'archive manifest is missing required evidence: {sorted(missing)}')
    return indexed


def verify_archive_members(archive_path, expected):
    observed = {}
    with tarfile.open(archive_path, mode='r:gz') as archive:
        for member in archive:
            archive_name = member.name
            safe_archive_path(archive_name)
            if archive_name in observed:
                raise AssertionError(f'duplicate archive member: {archive_name}')
            if not member.isfile() or member.issym() or member.islnk():
                raise AssertionError(f'archive contains a non-regular member: {archive_name}')
            member_stream = archive.extractfile(member)
            if member_stream is None:
                raise AssertionError(f'archive member cannot be read: {archive_name}')
            digest = hashlib.sha256()
            byte_count = 0
            with member_stream:
                for chunk in iter(lambda: member_stream.read(1024 * 1024), b''):
                    digest.update(chunk)
                    byte_count += len(chunk)
            observed[archive_name] = {'bytes': byte_count, 'sha256': digest.hexdigest()}
    if observed != expected:
        missing = sorted(set(expected) - set(observed))
        extra = sorted(set(observed) - set(expected))
        changed = sorted(name for name in set(expected) & set(observed)
                         if expected[name] != observed[name])
        raise AssertionError({'archive_manifest_mismatch': True,
                              'missing': missing, 'extra': extra, 'changed': changed})
    return {'member_count': len(observed), 'archive_manifest_matches': True}


def verify_extracted_tree(runtime_directory, expected):
    runtime_root = Path(runtime_directory)
    if not runtime_root.is_dir() or runtime_root.is_symlink():
        raise AssertionError('extracted runtime directory is missing or is a symlink')
    observed = {}
    for path in runtime_root.rglob('*'):
        if path.is_symlink():
            raise AssertionError(f'extracted runtime tree contains a symlink: {path}')
        if path.is_dir():
            continue
        if not path.is_file():
            raise AssertionError(f'extracted runtime tree contains a non-regular path: {path}')
        relative = path.relative_to(runtime_root).as_posix()
        safe_archive_path(relative)
        if relative in observed:
            raise AssertionError(f'duplicate extracted runtime path: {relative}')
        observed[relative] = {'bytes': path.stat().st_size, 'sha256': sha256_file(path)}
    if observed != expected:
        missing = sorted(set(expected) - set(observed))
        extra = sorted(set(observed) - set(expected))
        changed = sorted(name for name in set(expected) & set(observed)
                         if expected[name] != observed[name])
        raise AssertionError({'extracted_tree_manifest_mismatch': True,
                              'missing': missing, 'extra': extra, 'changed': changed})
    return {'file_count': len(observed), 'extracted_tree_matches': True}


def verify_archive_and_tree(evidence_directory, receipt):
    evidence_directory = Path(evidence_directory)
    archive_path = evidence_directory / 'runtime.tar.gz'
    runtime_directory = evidence_directory / 'runtime'
    archive_record = receipt.get('archive', {})
    if not archive_path.is_file() or archive_path.is_symlink():
        raise AssertionError('runtime.tar.gz is missing or is a symlink')
    actual_archive = {'bytes': archive_path.stat().st_size, 'sha256': sha256_file(archive_path)}
    expected_archive = {'bytes': archive_record.get('bytes'), 'sha256': archive_record.get('sha256')}
    if actual_archive != expected_archive:
        raise AssertionError({'archive_receipt_mismatch': True,
                              'expected': expected_archive, 'actual': actual_archive})
    expected = manifest_index(receipt.get('file_manifest'), receipt.get('controls_terminal_status'))
    archive_review = verify_archive_members(archive_path, expected)
    tree_review = verify_extracted_tree(runtime_directory, expected)
    return {'path': str(archive_path), **actual_archive, **archive_review,
            'extracted_runtime_path': str(runtime_directory), **tree_review,
            'manifest_file_count': len(expected)}


def local_helper_paths(evidence_directory):
    evidence_directory = Path(evidence_directory)
    capacity_directory = evidence_directory.parent / 'capacity'
    return {
        'arrival-phase/oci_arrival_validate.py': evidence_directory / 'oci_arrival_validate.py',
        'arrival-phase/oci_arrival_controls.py': evidence_directory / 'oci_arrival_controls.py',
        'arrival-phase/arrival_review.py': evidence_directory / 'arrival_review.py',
        'arrival-phase/oci_arrival_collect.py': evidence_directory / 'oci_arrival_collect.py',
        'capacity/oci_capacity.py': capacity_directory / 'oci_capacity.py',
        'capacity/capacity_review.py': capacity_directory / 'capacity_review.py',
    }


def verify_helper_provenance(evidence_directory, runtime_directory, receipt):
    evidence_directory = Path(evidence_directory)
    runtime_directory = Path(runtime_directory)
    receipt_helpers = receipt.get('helpers', [])
    helper_records = {record.get('archive_path'): record for record in receipt_helpers}
    if len(helper_records) != len(receipt_helpers) or set(helper_records) != EXPECTED_HELPER_ARCHIVE_PATHS:
        raise AssertionError('receipt helper set is missing, duplicated, or unexpected')
    local_paths = local_helper_paths(evidence_directory)
    validation_directory = runtime_directory / 'arrival-phase' / VALIDATION_RUN
    controls_directory = runtime_directory / 'arrival-phase' / RUN_NAME
    validation = read_json(validation_directory / 'arrival-validation-summary.json')
    provenance_path = controls_directory / 'provenance.json'
    provenance = read_json(provenance_path) if provenance_path.is_file() else {}
    expected_hashes = {
        'arrival-phase/oci_arrival_validate.py': validation.get('controller_sha256'),
        'arrival-phase/oci_arrival_controls.py': provenance.get('controller_sha256'),
        'arrival-phase/arrival_review.py': provenance.get('reviewer_sha256'),
        'arrival-phase/oci_arrival_collect.py': None,
        'capacity/oci_capacity.py': provenance.get('capacity_controller_sha256'),
        'capacity/capacity_review.py': provenance.get('capacity_reviewer_sha256'),
    }
    results = {}
    for archive_path, local_path in local_paths.items():
        record = helper_records[archive_path]
        extracted_path = runtime_directory / PurePosixPath(archive_path)
        if not local_path.is_file() or local_path.is_symlink():
            raise AssertionError(f'local frozen helper is missing or is a symlink: {local_path}')
        local_hash = sha256_file(local_path)
        archive_hash = sha256_file(extracted_path)
        receipt_hash = record.get('sha256')
        expected_hash = expected_hashes[archive_path]
        if not (local_hash == archive_hash == receipt_hash):
            raise AssertionError(f'local, receipt, and extracted helper hashes differ: {archive_path}')
        if expected_hash is not None and local_hash != expected_hash:
            raise AssertionError(f'helper hash differs from run provenance: {archive_path}')
        if record.get('hash_matches_provenance') is not None and record['hash_matches_provenance'] is not True:
            raise AssertionError(f'collector helper provenance check failed: {archive_path}')
        results[archive_path] = {'sha256': local_hash, 'local_matches_archive': True,
                                 'matches_run_provenance': None if expected_hash is None else local_hash == expected_hash}
    frozen_match = all(record['local_matches_archive'] for record in results.values())
    provenance_matches = all(record['matches_run_provenance'] is not False for record in results.values())
    return {'helpers': results, 'all_frozen_helper_hashes_match': frozen_match,
            'all_run_provenance_hashes_match': provenance_matches,
            'validation_summary': validation, 'controls_provenance': provenance}


def validate_validation_summary(summary, expected_native):
    outcomes = summary.get('outcomes', [])
    if summary.get('status') != 'completed' or summary.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('validation summary status or source differs from the frozen source')
    if [outcome.get('name') for outcome in outcomes] != EXPECTED_VALIDATION_CHECKS:
        raise AssertionError('validation checks are incomplete or out of order')
    if not all(outcome.get('exit_code') == 0 for outcome in outcomes):
        raise AssertionError('one or more required validation checks did not pass')
    if outcomes[0].get('passing_tests') != 85 or outcomes[0].get('test_group_counts') != EXPECTED_TEST_GROUP_COUNTS:
        raise AssertionError('validation test count differs from the frozen 85-test baseline')
    if outcomes[1].get('same_17_existing_clippy_diagnostics') is not True:
        raise AssertionError('validation Clippy diagnostics differ from the frozen 17-warning baseline')
    if len(outcomes[1].get('clippy_diagnostics', [])) != 17:
        raise AssertionError('validation does not preserve all 17 Clippy diagnostics')
    if outcomes[2].get('name') != 'a1-link' or outcomes[3].get('name') != 'default-link':
        raise AssertionError('A1/default compatibility links are missing or reordered')
    if summary.get('default_link_was_last') is not True:
        raise AssertionError('default compatibility link was not recorded last')
    if summary.get('native_commit') != EXPECTED_NATIVE_COMMIT:
        raise AssertionError('validation native source commit differs from the pin')
    if summary.get('native_archive_sha256_before') != expected_native or summary.get('native_archive_sha256_after') != expected_native:
        raise AssertionError('validation native archive hash differs from the pin')
    if summary.get('native_archive_bytes_before') != summary.get('native_archive_bytes_after'):
        raise AssertionError('native archive size changed during validation')
    if summary.get('source_status') != '':
        raise AssertionError('validation source tree was not clean')
    if summary.get('target_symlink_before') != summary.get('target_symlink_after'):
        raise AssertionError('frozen build target symlink changed during validation')
    if summary.get('resolved_target_directory_before') != summary.get('resolved_target_directory_after'):
        raise AssertionError('frozen build target directory changed during validation')
    return True


def validate_build_artifacts(runtime_directory, summary):
    runtime_directory = Path(runtime_directory)
    helper_hash = summary.get('build_helper_sha256')
    local_build_helper = Path(__file__).resolve().parents[2] / 'oci_harness_build.py'
    if not local_build_helper.is_file() or local_build_helper.is_symlink():
        raise AssertionError('local frozen build helper is missing or is a symlink')
    local_build_helper_hash = sha256_file(local_build_helper)
    if helper_hash != local_build_helper_hash:
        raise AssertionError('validation build helper hash differs from the local frozen helper')
    check_results = []
    for outcome in summary['outcomes']:
        name = outcome['name']
        build_directory = runtime_directory / 'builds' / f'{VALIDATION_RUN}-{name}'
        cargo_log = build_directory / 'cargo.log'
        provenance_path = build_directory / 'provenance.json'
        status_path = build_directory / 'status.json'
        if not all(path.is_file() for path in [cargo_log, provenance_path, status_path]):
            raise AssertionError(f'validation build artifacts are incomplete: {name}')
        provenance = read_json(provenance_path)
        status = read_json(status_path)
        if status.get('exit_code') != outcome['exit_code']:
            raise AssertionError(f'validation build exit status differs from summary: {name}')
        if sha256_file(cargo_log) != outcome.get('cargo_log_sha256'):
            raise AssertionError(f'validation build log hash differs from summary: {name}')
        if sha256_file(provenance_path) != outcome.get('provenance_sha256'):
            raise AssertionError(f'validation build provenance hash differs from summary: {name}')
        if (provenance.get('source_commit') != SOURCE_COMMIT
                or provenance.get('source_status') != ''
                or provenance.get('native_commit') != EXPECTED_NATIVE_COMMIT
                or provenance.get('native_archive_sha256') != EXPECTED_NATIVE_SHA256
                or provenance.get('controller_sha256') != helper_hash):
            raise AssertionError(f'validation build provenance differs from the frozen source/native/helper: {name}')
        check_results.append({'name': name, 'exit_code': status['exit_code'],
                              'cargo_log_sha256': sha256_file(cargo_log),
                              'provenance_sha256': sha256_file(provenance_path)})
    return {'checks': check_results, 'all_build_artifacts_valid': True,
            'build_helper_path': str(local_build_helper),
            'build_helper_sha256': helper_hash,
            'local_build_helper_matches_validation': True}


def validate_cell_configuration(report, case, backend, arrival_reviewer):
    expected = arrival_reviewer.expected_cell(case)
    configuration = report['effective_configuration']
    if (configuration.get('profile') != 'mixed'
            or configuration.get('profile_weights_get_query_write') != [40, 20, 40]
            or configuration.get('width') != 'mixed 80/15/5 for 1/4/16'
            or configuration.get('value_bytes') != 512
            or configuration.get('topology') != 'fixed-per-tenant'):
        raise AssertionError(f'workload recipe differs from the frozen controls: {case}')
    if configuration.get('backend_memory_budget', {}).get('max_bytes') != BACKEND_MEMORY_MAX_BYTES:
        raise AssertionError(f'backend memory max differs from the frozen controls: {case}')
    if configuration.get('backend_memory_budget', {}).get('high_bytes') != BACKEND_MEMORY_HIGH_BYTES:
        raise AssertionError(f'backend memory high differs from the frozen controls: {case}')
    if configuration.get('backend_memory_budget', {}).get('sample_interval_ms') != 5000:
        raise AssertionError(f'pressure cadence differs from the frozen controls: {case}')
    if configuration.get('backend') != backend:
        raise AssertionError(f'backend differs from the frozen controls: {case}')
    if expected['active'] != 28 and expected['registered'] != 4:
        raise AssertionError(f'cell shape is not part of the frozen tiny/N28 matrix: {case}')
    if backend == 'minimal-native-shared':
        resources = configuration.get('rocksdb_shared_resources', {})
        configured = resources.get('configured', {})
        expected_native = {'cache_bytes': NATIVE_CACHE_BYTES, 'wbm_bytes': NATIVE_WBM_BYTES,
                           'write_buffer_bytes': 1024**2, 'flush_threads': 1,
                           'compaction_threads': 1, 'max_open_files': 64}
        if configured != expected_native:
            raise AssertionError(f'shared native resource limits differ from the frozen controls: {case}')
    else:
        layout = configuration.get('blink_layout', {})
        expected_layout = {'layout': 'segmented-bases-coalesced', 'execution': 'background',
                           'segment_bytes': 256 * 1024}
        if any(layout.get(key) != value for key, value in expected_layout.items()):
            raise AssertionError(f'B-link WAL/checkpoint configuration differs from the frozen controls: {case}')
        if configuration.get('checkpoint_policy') != 'WalBytesThreshold { threshold_bytes: 1048576 }':
            raise AssertionError(f'B-link checkpoint threshold differs from the frozen controls: {case}')
    return True


def verify_runtime_binaries(runtime_directory, validation_summary, receipt):
    runtime_directory = Path(runtime_directory)
    names = ['tenant_density_workload', 'memory_diagnostic_worker']
    control_binary_records = receipt.get('controls', {}).get('control_binaries', {})
    identities = {}
    validation_ok = True
    controls_ok = True
    for binary_name in names:
        expected = validation_summary.get('default_binaries', {}).get(binary_name)
        validation_path = runtime_directory / 'arrival-phase' / VALIDATION_RUN / 'bin' / binary_name
        control_path = runtime_directory / 'arrival-phase' / RUN_NAME / 'bin' / binary_name
        if not expected or not validation_path.is_file():
            validation_ok = False
            validation_identity = None
        else:
            validation_identity = {'sha256': sha256_file(validation_path),
                                   'bytes': validation_path.stat().st_size}
            validation_ok = validation_ok and validation_identity == expected
        if not control_path.is_file():
            controls_ok = False
            control_identity = None
        else:
            control_identity = {'sha256': sha256_file(control_path),
                                'bytes': control_path.stat().st_size}
            controls_ok = controls_ok and control_identity == expected
        receipt_control = control_binary_records.get(binary_name, {})
        if control_identity is not None:
            controls_ok = controls_ok and receipt_control.get('matches_validation') is True
            controls_ok = controls_ok and receipt_control.get('sha256') == control_identity['sha256']
            controls_ok = controls_ok and receipt_control.get('bytes') == control_identity['bytes']
        identities[binary_name] = {'validation': validation_identity, 'controls': control_identity,
                                   'matches_validation': validation_identity == expected if validation_identity else False,
                                   'controls_matches_validation': control_identity == expected if control_identity else False}
    return {'binaries': identities, 'validation_binaries_valid': validation_ok,
            'runtime_binaries_match_validation': controls_ok}


def validate_receipt_provenance(receipt, runtime_directory, helper_review, terminal):
    controls = receipt.get('controls', {})
    control_validity = receipt.get('controls_validity', {})
    source = receipt.get('source', {})
    native = receipt.get('native', {})
    if receipt.get('collection_status') != 'completed':
        raise AssertionError('collection receipt is not complete')
    if source.get('commit') != SOURCE_COMMIT or source.get('expected_commit') != SOURCE_COMMIT or source.get('clean') is not True or source.get('valid') is not True:
        raise AssertionError('collection source provenance is invalid')
    if native.get('commit') != EXPECTED_NATIVE_COMMIT or native.get('archive_sha256') != EXPECTED_NATIVE_SHA256 or native.get('expected_archive_sha256') != EXPECTED_NATIVE_SHA256 or native.get('valid') is not True:
        raise AssertionError('collection native pin provenance is invalid')
    if native.get('unchanged_during_collection') is not True:
        raise AssertionError('native archive changed during evidence collection')
    if receipt.get('controls_terminal_status') not in {'completed', 'failed', 'initialization_failed'}:
        raise AssertionError('collection receipt has an unknown controls terminal state')
    if controls.get('actual_controller_commit') != CONTROLLER_COMMIT:
        raise AssertionError('actual controller commit differs from the frozen run')
    if controls.get('frozen_plan_commit') != PLAN_COMMIT or controls.get('plan_commit') != PLAN_COMMIT:
        raise AssertionError('frozen plan commit differs')
    if control_validity.get('actual_controller_commit_recorded') != CONTROLLER_COMMIT:
        raise AssertionError('actual controller commit was not recorded by collection')
    source_validation = controls.get('source_validation') or terminal.get('source_validation') or {}
    if source_validation.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('runtime source validation is missing or mismatched')
    if receipt['controls_terminal_status'] == 'completed' and source_validation.get('source_status') != '':
        raise AssertionError('completed runtime source tree was not clean')
    if (source_validation.get('native_commit') not in {None, EXPECTED_NATIVE_COMMIT}
            or source_validation.get('native_archive_sha256_before') not in {None, EXPECTED_NATIVE_SHA256}
            or source_validation.get('native_archive_sha256_after') not in {None, EXPECTED_NATIVE_SHA256}
            or source_validation.get('native_archive_sha256') not in {None, EXPECTED_NATIVE_SHA256}):
        raise AssertionError('runtime native source or archive validation differs from the pin')
    recorded_validation_sha = controls.get('validation_sha256_recorded') or terminal.get('validation_sha256')
    if recorded_validation_sha != receipt.get('validation', {}).get('summary_sha256'):
        raise AssertionError('validation summary hash recorded by controls differs')
    runtime_limits = controls.get('runtime_limits') or terminal.get('runtime_limits')
    if runtime_limits is None:
        raise AssertionError('runtime open-file limit provenance is missing')
    controls_provenance = helper_review['controls_provenance']
    if controls_provenance:
        if controls_provenance.get('source_commit') != SOURCE_COMMIT:
            raise AssertionError('runtime controls provenance source differs')
        if controls_provenance.get('run_name') != RUN_NAME:
            raise AssertionError('runtime controls provenance run name differs')
        if controls_provenance.get('plan_commit') != PLAN_COMMIT:
            raise AssertionError('runtime controls provenance plan commit differs')
        if controls_provenance.get('build_helper_sha256') != helper_review['validation_summary'].get('build_helper_sha256'):
            raise AssertionError('runtime provenance build helper hash differs from validation')
    elif receipt['controls_terminal_status'] != 'initialization_failed':
        raise AssertionError('controls provenance is missing outside initialization failure')
    if helper_review['validation_summary'].get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('archived validation source differs')
    validate_validation_summary(helper_review['validation_summary'], EXPECTED_NATIVE_SHA256)
    controls_dir = Path(runtime_directory) / 'arrival-phase' / RUN_NAME
    expected_terminal = {
        'completed': 'completion.json',
        'failed': 'failure-receipt.json',
        'initialization_failed': 'initialization-failure-receipt.json',
    }[receipt['controls_terminal_status']]
    if not (controls_dir / expected_terminal).is_file():
        raise AssertionError('controls terminal receipt does not exist in the archived runtime')
    if receipt['controls_terminal_status'] == 'completed':
        if control_validity.get('completed_run_evidence_valid') is not True:
            raise AssertionError('collector did not validate the completed run evidence')
        if (control_validity.get('all_sixteen_cells_operationally_valid') is not True
                or control_validity.get('status_cases_match_terminal_outcomes') is not True
                or control_validity.get('helper_hashes_match_run_provenance') is not True
                or control_validity.get('all_cgroup_process_lists_empty') is not True
                or control_validity.get('all_completed_cell_databases_preserved') is not True):
            raise AssertionError('collector completed evidence gates are inconsistent')
    elif control_validity.get('completed_run_evidence_valid') is True:
        raise AssertionError('incomplete controls receipt was promoted to complete evidence')
    if not control_validity.get('validation_source_native_and_binaries_valid'):
        raise AssertionError('collector did not validate source, native pin, and frozen binaries')
    return {'source_valid': True, 'native_pin_valid': True,
            'actual_controller_commit': CONTROLLER_COMMIT,
            'plan_commit': PLAN_COMMIT,
            'terminal_status': receipt['controls_terminal_status'],
            'completed_evidence_valid': control_validity.get('completed_run_evidence_valid') is True,
            'accepted_run': control_validity.get('accepted_run') is True}


def validate_runtime_limits(provenance):
    limits = provenance.get('runtime_limits') if 'runtime_limits' in provenance else provenance
    if not isinstance(limits, dict):
        raise AssertionError('runtime RLIMIT_NOFILE evidence is missing')
    if limits.get('original_soft') != 1024 or limits.get('original_hard') != 524288:
        raise AssertionError('original controller RLIMIT_NOFILE differs from 1024/524288')
    if limits.get('new_soft') != 65536 or limits.get('new_hard') != 524288:
        raise AssertionError('controller RLIMIT_NOFILE differs from 65536/524288')
    before = limits.get('proc_limits_before', {})
    after = limits.get('controller_proc_limits', {})
    if before.get('soft') != 1024 or before.get('hard') != 524288:
        raise AssertionError('recorded original /proc limits differ')
    if after.get('soft') != 65536 or after.get('hard') != 524288:
        raise AssertionError('recorded updated /proc limits differ')
    preflight = limits.get('preflight_child', {})
    if (preflight.get('role') != 'preflight-limit-child'
            or preflight.get('limits') != {'soft': 65536, 'hard': 524288, 'units': 'files'}
            or preflight.get('exit_code') != 0
            or preflight.get('same_live_process_after_exit') is not False
            or not isinstance(preflight.get('pid'), int)
            or not isinstance(preflight.get('start_time_ticks'), int)):
        raise AssertionError('preflight child did not prove inherited limits and exit')
    return {'original_soft': 1024, 'new_soft': 65536, 'hard': 524288,
            'preflight_child_exited': True, 'preflight_child_pid': preflight['pid'],
            'preflight_child_start_time_ticks': preflight['start_time_ticks']}


def command_argument(command, flag):
    arguments = command.get('command', [])
    try:
        return arguments[arguments.index(flag) + 1]
    except (ValueError, IndexError):
        raise AssertionError(f'cell command is missing {flag}')


def command_record_map(controls_directory):
    commands = {}
    for path in Path(controls_directory).glob('*-command.json'):
        case = path.name.removesuffix('-command.json')
        if case in commands:
            raise AssertionError(f'duplicate cell command: {case}')
        commands[case] = read_json(path)
    return commands


def status_record_map(controls_directory):
    statuses = {}
    for path in Path(controls_directory).glob('*-status.json'):
        record = read_json(path)
        case = record.get('case')
        if not isinstance(case, str) or case in statuses:
            raise AssertionError('cell status has missing or duplicate case identity')
        statuses[case] = record
    return statuses


def validate_runtime_command(case, command, controls_directory, arrival_reviewer):
    expected = None
    for candidate in EXPECTED_REPORT_CASES:
        if case == candidate:
            expected = candidate
            break
    if expected is None:
        raise AssertionError(f'cell is outside the frozen matrix: {case}')
    expected_cell = arrival_reviewer.expected_cell(case)
    backend = 'blink' if case.endswith('-blink') else 'minimal-native-shared'
    command_vector = command.get('command', [])
    if command_argument(command, '--backend') != backend:
        raise AssertionError(f'backend differs from frozen matrix: {case}')
    if command_argument(command, '--tenant-arrival-phase') != expected_cell['mode']:
        raise AssertionError(f'arrival mode differs from frozen matrix: {case}')
    expected_data = f'/bench/zfs/db/tenant-density-harness-20261027/{RUN_NAME}/{case}'
    expected_cgroup = f'/sys/fs/cgroup/dodb-density-{RUN_NAME}/{case}'
    expected_report = f'/home/opc/tenant-density-harness-20261027/arrival-phase/{RUN_NAME}/{case}.json'
    if command_argument(command, '--data-dir') != expected_data:
        raise AssertionError(f'database path differs from frozen run namespace: {case}')
    if command_argument(command, '--backend-cgroup-root') != expected_cgroup:
        raise AssertionError(f'backend cgroup path differs from frozen run namespace: {case}')
    if command_argument(command, '--report') != expected_report:
        raise AssertionError(f'report path differs from frozen run namespace: {case}')
    expected_arguments = {
        '--seed': 20261027,
        '--registered-tenants': expected_cell['registered'],
        '--active-tenants': expected_cell['active'],
        '--opened-idle-tenants': expected_cell['idle'],
        '--initial-rows': expected_cell['rows'],
        '--duration-ms': expected_cell['duration_ms'],
        '--pressure-sample-interval-ms': 5000,
        '--client-count': 1,
        '--value-bytes': 512,
        '--topology': 'fixed-per-tenant',
        '--per-tenant-rate': 50,
        '--queue-bound': 8,
        '--max-in-flight': 8,
        '--request-timeout-ms': 5000,
        '--drain-timeout-ms': 15000,
        '--profile': 'mixed',
        '--width': 'mixed',
        '--backend-memory-max-bytes': BACKEND_MEMORY_MAX_BYTES,
        '--backend-memory-high-bytes': BACKEND_MEMORY_HIGH_BYTES,
    }
    for flag, expected_value in expected_arguments.items():
        if command_argument(command, flag) != str(expected_value):
            raise AssertionError(f'cell argument differs from frozen matrix: {case} {flag}')
    if '--features' in command_vector or 'blink-writer-pin-reuse' in command_vector:
        raise AssertionError('runtime command enabled the A1 feature')
    if '--operation-count' in command_vector:
        raise AssertionError('runtime command changed the frozen duration-based request recipe')
    if backend == 'blink':
        expected_blink_options = {
            '--blink-wal-layout': 'segmented-bases-coalesced',
            '--blink-checkpoint-execution': 'background',
            '--blink-wal-segment-bytes': 256 * 1024,
            '--blink-checkpoint-wal-bytes': 1024**2,
        }
        for flag, expected_value in expected_blink_options.items():
            if command_argument(command, flag) != str(expected_value):
                raise AssertionError(f'B-link runtime argument differs from frozen controls: {case} {flag}')
    else:
        expected_native_options = {
            '--rocksdb-shared-cache-bytes': NATIVE_CACHE_BYTES,
            '--rocksdb-shared-wbm-bytes': NATIVE_WBM_BYTES,
            '--rocksdb-write-buffer-bytes': 1024**2,
            '--rocksdb-flush-threads': 1,
            '--rocksdb-compaction-threads': 1,
            '--rocksdb-max-open-files': 64,
        }
        for flag, expected_value in expected_native_options.items():
            if command_argument(command, flag) != str(expected_value):
                raise AssertionError(f'native runtime argument differs from frozen controls: {case} {flag}')
    if command.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError(f'cell command source differs: {case}')
    return {**command, 'case': case, 'backend': backend, 'mode': expected_cell['mode'],
            'registered': expected_cell['registered'], 'active': expected_cell['active'],
            'idle': expected_cell['idle'], 'duration_ms': expected_cell['duration_ms']}


def validate_collection_database_and_cgroups(receipt, controls_directory, command_records):
    completed = receipt.get('controls_terminal_status') == 'completed'
    controls = receipt.get('controls', {})
    cgroups = controls.get('cgroups', {})
    grouping_root_valid = cgroups.get('grouping_root_path_matches_expected') is True
    root_limit_valid = cgroups.get('grouping_root_memory_max_is_unbounded') is True
    epoch_budgets_valid = cgroups.get('cell_epoch_budgets_match') is True
    process_groups_empty = cgroups.get('all_cgroup_process_lists_empty') is True
    groups = cgroups.get('groups', {})
    root_path = f'/sys/fs/cgroup/dodb-density-{RUN_NAME}'
    grouping_root_valid = grouping_root_valid and cgroups.get('grouping_root') == root_path
    root_group = groups.get(root_path)
    root_limit_valid = root_limit_valid and bool(root_group and root_group.get('memory.max') == 'max')
    observed_cell_budgets_valid = True
    for case in command_records:
        cell_path = f'{root_path}/{case}'
        for path in [cell_path, *[cell_path + '/' + epoch for epoch in ['setup', 'measurement', 'oracle', 'reopen-oracle']]]:
            group = groups.get(path)
            if not group:
                observed_cell_budgets_valid = False
                continue
            if (group.get('memory.max') != str(BACKEND_MEMORY_MAX_BYTES)
                    or group.get('memory.high') != str(BACKEND_MEMORY_HIGH_BYTES)
                    or group.get('memory.swap.max') != '0'
                    or group.get('cgroup.procs')):
                observed_cell_budgets_valid = False
    epoch_budgets_valid = epoch_budgets_valid and observed_cell_budgets_valid
    if cgroups.get('cell_count') != len(command_records):
        epoch_budgets_valid = False
    if completed and cgroups.get('group_count') != 1 + 16 * 5:
        epoch_budgets_valid = False
    databases = controls.get('databases', {})
    database_cases = databases.get('cases', [])
    database_index = {record.get('case'): record for record in database_cases}
    database_inventory_valid = len(database_index) == len(database_cases)
    for case in command_records:
        record = database_index.get(case)
        if (not record or record.get('path') != f'/bench/zfs/db/tenant-density-harness-20261027/{RUN_NAME}/{case}'
                or (completed and (record.get('file_count', 0) < 1 or record.get('total_file_bytes', 0) < 1))):
            database_inventory_valid = False
    preserved_database_count = sum(record.get('preserved') is True for record in database_cases)
    databases_preserved = database_inventory_valid and preserved_database_count == len(command_records)
    command_count_valid = controls.get('actual_cell_command_count') == len(command_records)
    status_count_valid = controls.get('actual_cell_status_count') == len(status_record_map(controls_directory))
    if completed and not all([grouping_root_valid, root_limit_valid, epoch_budgets_valid,
                              process_groups_empty, database_inventory_valid, databases_preserved,
                              command_count_valid, status_count_valid]):
        raise AssertionError('completed-run cgroup, database, or cell inventory validation failed')
    return {'cell_count': len(command_records), 'database_count': len(database_index),
            'database_preserved_count': preserved_database_count,
            'database_inventory_valid': database_inventory_valid,
            'all_attempted_databases_preserved': databases_preserved,
            'cgroup_count': cgroups.get('group_count'),
            'grouping_root_valid': grouping_root_valid,
            'epoch_budgets_valid': epoch_budgets_valid,
            'all_cgroup_process_lists_empty': process_groups_empty,
            'command_count_valid': command_count_valid,
            'status_count_valid': status_count_valid}


def process_keys(terminal, controls_directory, reports):
    keys = set()
    for process in terminal.get('processes', []):
        if isinstance(process, dict) and 'pid' in process and 'start_time_ticks' in process:
            keys.add((int(process['pid']), int(process['start_time_ticks'])))
    for status in status_record_map(controls_directory).values():
        for process in status.get('owned_process_limits', []):
            if isinstance(process, dict) and 'pid' in process and 'start_time_ticks' in process:
                keys.add((int(process['pid']), int(process['start_time_ticks'])))
    for report in reports.values():
        for process in report.get('memory_diagnostic', {}).get('processes', []):
            if isinstance(process, dict) and 'pid' in process and 'start_time_ticks' in process:
                keys.add((int(process['pid']), int(process['start_time_ticks'])))
    return keys


def validate_process_receipt(receipt, terminal, controls_directory, reports, commands):
    controls = receipt.get('controls', {})
    processes = controls.get('processes', [])
    if receipt.get('controls_terminal_status') == 'completed' and len(processes) < 16:
        raise AssertionError('completed run has fewer than sixteen unique owned process identities')
    receipt_keys = {(process['pid'], process['start_time_ticks']) for process in processes}
    expected_keys = process_keys(terminal, controls_directory, reports)
    if receipt_keys != expected_keys:
        raise AssertionError('collector process identity set differs from archived runtime evidence')
    if any(process.get('same_live_process_at_collection') is not False for process in processes):
        raise AssertionError('collector recorded an owned process still live')
    completed = receipt.get('controls_terminal_status') == 'completed'
    owned_child_count = 0
    for case, status in status_record_map(controls_directory).items():
        child_rows = [row for row in status.get('owned_process_limits', [])
                      if row.get('role') == 'owned-workload-process']
        child_identities = {(row['pid'], row['start_time_ticks']) for row in child_rows}
        descendant_rows = [row for row in status.get('owned_process_limits', [])
                           if row.get('role') == 'owned-descendant']
        descendant_identities = {(row['pid'], row['start_time_ticks']) for row in descendant_rows}
        if len(child_identities) > 1 or (completed and len(child_identities) != 1):
            raise AssertionError(f'cell does not have exactly one recorded workload child: {case}')
        if status.get('exit_code') == 0 and len(child_identities) != 1:
            raise AssertionError(f'successful workload cell is missing its owned process identity: {case}')
        if status.get('exit_code') == 0 and not descendant_identities:
            raise AssertionError(f'successful workload cell is missing its owned descendant identity: {case}')
        rows_by_identity = {}
        for row in [*child_rows, *descendant_rows]:
            rows_by_identity.setdefault((row['pid'], row['start_time_ticks']), row)
        for row in rows_by_identity.values():
            limits = row.get('limits', {})
            if limits.get('soft') != 65536 or limits.get('hard') != 524288:
                raise AssertionError(f'owned workload child open-file limits differ: {case}')
        owned_child_count += len(child_identities)
    return {'unique_process_count': len(receipt_keys),
            'all_recorded_processes_exited': True,
            'owned_workload_child_count': owned_child_count}


def load_arrival_module(evidence_directory):
    return load_module('arrival_review_final', Path(evidence_directory) / 'arrival_review.py')


def review_cell_reports(controls_directory, command_records, status_records,
                        source_commit, capacity_reviewer, arrival_reviewer, terminal_status):
    reports = {}
    results = {}
    for case, command in command_records.items():
        status = status_records.get(case)
        if status is None:
            command_path = Path(controls_directory) / f'{case}-command.json'
            results[case] = {'status': 'missing_status', 'valid': False,
                             'available_unvalidated_counters': None,
                             'command_path': str(command_path),
                             'command_sha256': sha256_file(command_path)}
            continue
        report_path = Path(controls_directory) / f'{case}.json'
        if not report_path.is_file():
            results[case] = {'status': 'missing_report', 'valid': False,
                             'available_unvalidated_counters': status.get('available_unvalidated_counters'),
                             'command_path': str(Path(controls_directory) / f'{case}-command.json'),
                             'command_sha256': sha256_file(Path(controls_directory) / f'{case}-command.json'),
                             'status_path': str(Path(controls_directory) / f'{case}-status.json'),
                             'status_sha256': sha256_file(Path(controls_directory) / f'{case}-status.json')}
            continue
        report_sha256 = sha256_file(report_path)
        report = read_json(report_path)
        if not status.get('report_sha256') and terminal_status != 'completed':
            command_path = Path(controls_directory) / f'{case}-command.json'
            status_path = Path(controls_directory) / f'{case}-status.json'
            results[case] = {'valid': False, 'status': 'unvalidated_missing_report_hash',
                             'available_unvalidated_counters': report.get('workload') or status.get('available_unvalidated_counters'),
                             'command_path': str(command_path), 'command_sha256': sha256_file(command_path),
                             'status_path': str(status_path), 'status_sha256': sha256_file(status_path),
                             'report_path': str(report_path), 'report_sha256': report_sha256}
            continue
        if report_sha256 != status.get('report_sha256'):
            raise AssertionError(f'cell report hash differs from its status: {case}')
        command_path = Path(controls_directory) / f'{case}-command.json'
        status_path = Path(controls_directory) / f'{case}-status.json'
        raw_paths = {'command_path': str(command_path), 'command_sha256': sha256_file(command_path),
                     'status_path': str(status_path), 'status_sha256': sha256_file(status_path),
                     'report_path': str(report_path), 'report_sha256': report_sha256}
        reports[case] = report
        try:
            reduced = capacity_reviewer.reduce_report(report, source_commit)
            arrival_reviewer.validate_report(report, case, command['backend'], source_commit)
            validate_cell_configuration(report, case, command['backend'], arrival_reviewer)
            if status.get('exit_code') != 0 or status.get('stop_reason') is not None or status.get('review_error') is not None:
                raise AssertionError('cell status records an operational failure')
            saved_reduced = status.get('reduced') or {}
            if (saved_reduced.get('operational_pass') != reduced.get('operational_pass')
                    or saved_reduced.get('slo_pass') != reduced.get('slo_pass')):
                raise AssertionError('saved cell review flags differ from immutable report reduction')
            results[case] = {**raw_paths, 'valid': reduced.get('operational_pass') is True,
                             'operational_pass': reduced.get('operational_pass'),
                             'slo_pass': reduced.get('slo_pass'), 'reduced': reduced,
                             'successful_requests': report['workload']['successful'],
                             'scheduled_requests': report['workload']['scheduled'],
                             'measurement_pressure_serialized_bytes': sum(
                                 len(json.dumps(sample, separators=(',', ':')).encode())
                                 for sample in report['backend_memory_pressure']['samples']
                                 if sample.get('epoch') == 'measurement' and sample.get('event') == 'periodic'),
                             'oracle_count': 2,
                             'database_path': command_argument(command, '--data-dir')}
        except Exception as error:
            workload = report.get('workload')
            results[case] = {**raw_paths, 'valid': False, 'status': 'unvalidated',
                             'review_error': {'type': type(error).__name__, 'message': str(error)},
                             'available_unvalidated_counters': workload}
    return reports, results


def review_same_mode_pairs(controls_directory, reports, results, capacity_reviewer):
    pairs = {}
    for pair_case in EXPECTED_PAIRS:
        pair_path = Path(controls_directory) / f'{pair_case}-pair.json'
        if not pair_path.is_file():
            pairs[pair_case] = {'valid': False, 'status': 'missing_pair_record'}
            continue
        pair = read_json(pair_path)
        cases = pair.get('cases', [])
        expected_cases = {pair_case + '-blink', pair_case + '-shared'}
        if len(cases) != 2 or set(cases) != expected_cases:
            raise AssertionError(f'same-mode pair members differ from the frozen pair: {pair_case}')
        expected_mode = pair_case.rsplit('-', 1)[-1]
        if pair.get('mode') != expected_mode:
            raise AssertionError(f'same-mode pair label differs from its cells: {pair_case}')
        if not all(case in reports and results[case].get('valid') for case in cases):
            pairs[pair_case] = {'valid': False, 'status': 'reports_not_operationally_valid',
                                'pair_path': str(pair_path), 'pair_sha256': sha256_file(pair_path)}
            continue
        comparison = capacity_reviewer.compare_pair(*(reports[case] for case in cases))
        pairs[pair_case] = {'valid': True, 'mode': pair_case.rsplit('-', 1)[-1],
                            'cases': cases, 'comparison': comparison,
                            'pair_path': str(pair_path), 'pair_sha256': sha256_file(pair_path)}
    return pairs


def review_phase_pairs(reports, results, arrival_reviewer):
    phase_pairs = {}
    for first_case, second_case in PHASE_PAIRS:
        for backend, suffix in [('blink', '-blink'), ('minimal-native-shared', '-shared')]:
            synchronized_case = first_case if first_case.endswith('synchronized') else second_case
            staggered_case = second_case if synchronized_case == first_case else first_case
            synchronized_report_case = synchronized_case + suffix
            staggered_report_case = staggered_case + suffix
            key = synchronized_case + '--' + staggered_case + '-' + backend
            if not all(case in reports and results[case].get('valid')
                       for case in [synchronized_report_case, staggered_report_case]):
                phase_pairs[key] = {'valid': False, 'status': 'reports_not_operationally_valid'}
                continue
            comparison = arrival_reviewer.compare_phases(
                reports[synchronized_report_case], reports[staggered_report_case])
            phase_pairs[key] = {'valid': True, 'cases': [synchronized_report_case, staggered_report_case],
                                'comparison': comparison}
    return phase_pairs


def condition_summaries(results):
    summaries = {}
    for backend in ['blink', 'minimal-native-shared']:
        for mode in ['synchronized', 'staggered']:
            selected = []
            for repetition in [1, 2, 3]:
                case = f'control-n28-r{repetition}-{mode}-' + ('blink' if backend == 'blink' else 'shared')
                if case in results:
                    selected.append(results[case])
            valid = [result for result in selected if result.get('valid')]
            p99_limits = {}
            for operation_name in ['Get', 'Query', 'Write']:
                observed = [result['reduced']['worst_tenant_p99_upper_exclusive_ms'][operation_name]
                            for result in valid]
                p99_limits[operation_name] = max(observed) if observed else None
            measurement_peaks = [result['reduced']['epoch_peaks_bytes']['measurement']
                                 for result in valid]
            native_observation_bytes = [result['reduced'].get('native_serialized_bytes')
                                        for result in valid]
            summaries[f'{backend}/{mode}'] = {
                'expected_repetitions': 3,
                'observed_repetitions': len(selected),
                'operationally_valid_repetitions': len(valid),
                'slo_pass_repetitions': sum(result.get('slo_pass') is True for result in valid),
                'slo_miss_repetitions': sum(result.get('slo_pass') is False for result in valid),
                'worst_tenant_p99_upper_exclusive_ms': p99_limits,
                'measurement_peak_bytes_max': max(measurement_peaks) if measurement_peaks else None,
                'measurement_pressure_samples_total': sum(
                    result['reduced'].get('measurement_sample_count', 0) for result in valid),
                'native_measurement_samples_total': sum(
                    result['reduced'].get('native_measurement_sample_count', 0) for result in valid),
                'native_serialized_observation_bytes_total': sum(
                    value for value in native_observation_bytes if isinstance(value, int))
                    if all(isinstance(value, int) for value in native_observation_bytes) else None,
                'pressure_observation_serialized_bytes_total': sum(
                    result.get('measurement_pressure_serialized_bytes', 0) for result in valid),
            }
    return summaries


def summarize_controls(terminal_status, command_cases, cell_results, pair_results, phase_results):
    valid_cases = [result for result in cell_results.values() if result.get('valid')]
    tiny_cases = [case for case in EXPECTED_REPORT_CASES if case.startswith('tiny-')]
    tiny_acceptance = {
        case: cell_results.get(case, {}).get('valid') is True
        and cell_results.get(case, {}).get('slo_pass') is True
        for case in tiny_cases
    }
    tiny_slo_gate = bool(tiny_acceptance) and all(tiny_acceptance.values())
    pairs_valid = len(pair_results) == 8 and all(pair.get('valid') for pair in pair_results.values())
    phases_valid = len(phase_results) == 8 and all(pair.get('valid') for pair in phase_results.values())
    complete = (terminal_status == 'completed' and len(set(command_cases)) == 16
                and set(command_cases) == set(EXPECTED_REPORT_CASES)
                and len(valid_cases) == 16 and pairs_valid and phases_valid
                and all(result.get('operational_pass') is True for result in valid_cases)
                and tiny_slo_gate)
    all_slo_pass = complete and all(result.get('slo_pass') is True for result in valid_cases)
    return {
        'actual_unique_count': len(set(command_cases)),
        'expected_complete_count': 16,
        'controls_terminal_status': terminal_status,
        'completed': complete,
        'operationally_valid_count': len(valid_cases),
        'operational_failure_count': sum(result.get('valid') is not True for result in cell_results.values()),
        'operational_failure_cases': sorted(case for case, result in cell_results.items()
                                            if result.get('valid') is not True),
        'slo_pass_count': sum(result.get('slo_pass') is True for result in valid_cases),
        'slo_miss_count': sum(result.get('slo_pass') is False for result in valid_cases),
        'slo_miss_cases': sorted(case for case, result in cell_results.items()
                                 if result.get('valid') and result.get('slo_pass') is False),
        'tiny_slo_gate': tiny_slo_gate,
        'tiny_acceptance': tiny_acceptance,
        'same_mode_comparisons_valid': pairs_valid,
        'phase_comparisons_valid': phases_valid,
        'expected_complete_cohort_requests': 1014000 if complete else None,
        'validated_successful_requests': sum(result.get('successful_requests', 0) for result in valid_cases),
        'validated_scheduled_requests': sum(result.get('scheduled_requests', 0) for result in valid_cases),
        'validated_oracle_count': sum(result.get('oracle_count', 0) for result in valid_cases),
        'expected_complete_cohort_oracles': 32 if complete else None,
        'accepted_run': bool(complete and all_slo_pass),
        'all_sixteen_slo_pass': bool(all_slo_pass),
    }


def review(evidence_directory, output_path=None, capacity_reviewer=None, arrival_reviewer=None):
    evidence_directory = Path(evidence_directory).resolve()
    receipt_path = evidence_directory / 'collection' / 'collection.json'
    if not receipt_path.is_file() or receipt_path.is_symlink():
        raise AssertionError('collection/collection.json is missing or is a symlink')
    receipt = read_json(receipt_path)
    archive_review = verify_archive_and_tree(evidence_directory, receipt)
    runtime_directory = evidence_directory / 'runtime'
    helper_review = verify_helper_provenance(evidence_directory, runtime_directory, receipt)
    controls_directory = runtime_directory / 'arrival-phase' / RUN_NAME
    terminal_name = {
        'completed': 'completion.json',
        'failed': 'failure-receipt.json',
        'initialization_failed': 'initialization-failure-receipt.json',
    }[receipt['controls_terminal_status']]
    terminal = read_json(controls_directory / terminal_name)
    provenance_review = validate_receipt_provenance(receipt, runtime_directory, helper_review, terminal)
    validation = helper_review['validation_summary']
    validate_validation_summary(validation, EXPECTED_NATIVE_SHA256)
    validation_summary_path = runtime_directory / 'arrival-phase' / VALIDATION_RUN / 'arrival-validation-summary.json'
    if sha256_file(validation_summary_path) != receipt.get('validation', {}).get('summary_sha256'):
        raise AssertionError('archived validation summary hash differs from collection receipt')
    build_review = validate_build_artifacts(runtime_directory, validation)
    binary_review = verify_runtime_binaries(runtime_directory, validation, receipt)
    if receipt['controls_terminal_status'] == 'completed' and not binary_review['runtime_binaries_match_validation']:
        raise AssertionError('completed runtime binaries do not match validated default binaries')
    binary_hashes = {name: value['validation'] for name, value in binary_review['binaries'].items()}
    controls_provenance = helper_review['controls_provenance']
    runtime_limits_source = receipt.get('controls', {}).get('runtime_limits') or terminal.get('runtime_limits')
    runtime_limits = validate_runtime_limits(runtime_limits_source or {})
    arrival_reviewer = arrival_reviewer or load_arrival_module(Path(__file__).resolve().parent)
    commands = command_record_map(controls_directory)
    statuses = status_record_map(controls_directory)
    command_cases = list(commands)
    if len(command_cases) != len(set(command_cases)):
        raise AssertionError('duplicate actual workload case')
    for case, command in commands.items():
        commands[case] = validate_runtime_command(case, command, controls_directory, arrival_reviewer)
    if terminal.get('source_commit') != SOURCE_COMMIT:
        raise AssertionError('runtime terminal receipt source differs')
    if terminal.get('plan_commit') != PLAN_COMMIT:
        raise AssertionError('runtime terminal receipt plan differs')
    if receipt['controls_terminal_status'] == 'completed':
        if set(command_cases) != set(EXPECTED_REPORT_CASES) or len(command_cases) != 16:
            raise AssertionError('completed controls do not contain exactly the frozen sixteen cells')
        if set(statuses) != set(EXPECTED_REPORT_CASES) or len(statuses) != 16:
            raise AssertionError('completed controls do not contain exactly sixteen cell statuses')
        summary_cases = [record['case'] for record in read_json(controls_directory / 'summary.json')]
        if summary_cases != EXPECTED_REPORT_CASES:
            raise AssertionError('runtime summary cell order differs from the frozen matrix')
    if set(statuses) - set(commands):
        raise AssertionError('runtime contains a cell status without a corresponding command')
    if set(commands) - set(statuses) and receipt['controls_terminal_status'] == 'completed':
        raise AssertionError('completed runtime contains a command without a status')
    collection_counts = validate_collection_database_and_cgroups(receipt, controls_directory, commands)
    capacity_reviewer = capacity_reviewer or load_module(
        'arrival_capacity_review_final', Path(__file__).resolve().parent.parent / 'capacity' / 'capacity_review.py')
    reports, cell_results = review_cell_reports(
        controls_directory, commands, statuses, SOURCE_COMMIT, capacity_reviewer,
        arrival_reviewer, receipt['controls_terminal_status'])
    if receipt['controls_terminal_status'] == 'completed':
        if set(reports) != set(EXPECTED_REPORT_CASES) or len(reports) != 16:
            raise AssertionError('completed runtime did not pass immutable reduction for all sixteen reports')
    pair_results = review_same_mode_pairs(controls_directory, reports, cell_results, capacity_reviewer)
    phase_results = review_phase_pairs(reports, cell_results, arrival_reviewer)
    if receipt['controls_terminal_status'] == 'completed':
        if not all(pair.get('valid') for pair in pair_results.values()):
            raise AssertionError('one or more same-mode backend comparisons failed')
        if not all(pair.get('valid') for pair in phase_results.values()):
            raise AssertionError('one or more cross-mode phase comparisons failed')
    process_results = validate_process_receipt(receipt, terminal, controls_directory, reports, commands)
    control_summary = summarize_controls(
        receipt['controls_terminal_status'], list(commands), cell_results, pair_results, phase_results)
    if receipt['controls_terminal_status'] == 'completed' and not control_summary['completed']:
        raise AssertionError('completed controls receipt did not pass final report and comparison review')
    if receipt['controls_terminal_status'] == 'completed':
        receipt_validity = receipt.get('controls_validity', {})
        if (receipt_validity.get('accepted_run') is not control_summary['accepted_run']
                or receipt_validity.get('all_sixteen_cells_slo_pass') is not control_summary['all_sixteen_slo_pass']):
            raise AssertionError('collector acceptance and per-cell SLO review disagree')
    output = {
        'schema_version': 1,
        'evidence_directory': str(evidence_directory),
        'collection_receipt': {'path': str(receipt_path), 'sha256': sha256_file(receipt_path),
                               'terminal_status': receipt['controls_terminal_status']},
        'terminal_receipt': {'path': str(controls_directory / terminal_name),
                             'sha256': sha256_file(controls_directory / terminal_name)},
        'controls_provenance_file': {
            'path': str(controls_directory / 'provenance.json') if (controls_directory / 'provenance.json').is_file() else None,
            'sha256': sha256_file(controls_directory / 'provenance.json')
            if (controls_directory / 'provenance.json').is_file() else None},
        'archive': archive_review,
        'manifest': {'verified': True, 'regular_file_count': archive_review['manifest_file_count'],
                     'archive_and_extracted_tree_match': True,
                     'files': receipt['file_manifest']},
        'provenance': provenance_review,
        'runtime_provenance': {
            'runtime_uid': controls_provenance.get('runtime_uid'),
            'source_validation': controls_provenance.get('source_validation'),
            'capacity_before': controls_provenance.get('capacity_before'),
            'kernel': controls_provenance.get('kernel'),
            'hostname': controls_provenance.get('hostname'),
            'online_cpus': controls_provenance.get('online_cpus'),
            'filesystem_properties': controls_provenance.get('filesystem_properties'),
            'processes_before': controls_provenance.get('processes_before'),
            'controller_commit': CONTROLLER_COMMIT,
            'plan_commit': PLAN_COMMIT,
            'build_helper_sha256': controls_provenance.get('build_helper_sha256'),
        },
        'validation': {'source_commit': SOURCE_COMMIT, 'passing_tests': 85,
                       'test_group_counts': EXPECTED_TEST_GROUP_COUNTS,
                       'same_17_existing_clippy_diagnostics': True,
                       'default_link_was_last': True,
                       'native_commit': EXPECTED_NATIVE_COMMIT,
                       'native_archive_sha256': EXPECTED_NATIVE_SHA256,
                       'default_binaries': binary_hashes,
                       'validation_binaries_valid': binary_review['validation_binaries_valid'],
                       'runtime_binaries_match_validation': binary_review['runtime_binaries_match_validation']},
        'build_artifacts': build_review,
        'helpers': helper_review['helpers'],
        'runtime_limits': runtime_limits,
        'cells': {**control_summary, 'case_results': cell_results},
        'comparisons': {'same_mode': pair_results, 'cross_mode': phase_results},
        'conditions': condition_summaries(cell_results),
        'requests': {'expected_complete_cohort': control_summary['expected_complete_cohort_requests'],
                     'validated_successful_requests': control_summary['validated_successful_requests'],
                     'validated_tiny_successful_requests': sum(
                         result.get('successful_requests', 0) for case, result in cell_results.items()
                         if case.startswith('tiny-') and result.get('valid')),
                     'validated_n28_successful_requests': sum(
                         result.get('successful_requests', 0) for case, result in cell_results.items()
                         if case.startswith('control-n28-') and result.get('valid')),
                     'validated_scheduled_requests': control_summary['validated_scheduled_requests']},
        'oracles': {'validated_oracle_count': control_summary['validated_oracle_count'],
                    'expected_complete_cohort': control_summary['expected_complete_cohort_oracles']},
        'databases': {'actual_cell_database_count': collection_counts['database_count'],
                      'preserved_database_count': collection_counts['database_preserved_count'],
                      'expected_complete_cohort': 16 if control_summary['completed'] else None},
        'processes': process_results,
        'cgroups': collection_counts,
        'collection_validity': {'completed_run_evidence_valid': control_summary['completed'],
                                'accepted_run': control_summary['accepted_run'],
                                'all_sixteen_slo_pass': control_summary['all_sixteen_slo_pass'],
                                'collector_receipt_accepted_run': receipt.get('controls_validity', {}).get('accepted_run') is True},
        'interpretation_scope': 'current-source E2 arrival-pattern controls; no backend maximum or adoption conclusion',
    }
    serialized = json.dumps(output, indent=2, sort_keys=True) + '\n'
    destination = Path(output_path) if output_path is not None else evidence_directory / 'final-review.json'
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists() or destination.is_symlink():
        if destination.is_symlink() or not destination.is_file():
            raise AssertionError('existing final review output is not a regular file')
        if destination.read_text() != serialized:
            raise AssertionError('existing final review differs from the recomputed review; refusing to overwrite')
        return output
    with destination.open('x') as output_file:
        output_file.write(serialized)
    return output


def main():
    parser = argparse.ArgumentParser(description='Review collected E2 arrival-phase runtime evidence')
    parser.add_argument('--evidence-dir', type=Path, default=Path(__file__).resolve().parent)
    parser.add_argument('--output', type=Path)
    arguments = parser.parse_args()
    result = review(arguments.evidence_dir, arguments.output)
    print(json.dumps({'complete': result['cells']['completed'],
                      'operationally_valid_cells': result['cells']['operationally_valid_count'],
                      'slo_pass_cells': result['cells']['slo_pass_count'],
                      'accepted_run': result['collection_validity']['accepted_run']}, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
