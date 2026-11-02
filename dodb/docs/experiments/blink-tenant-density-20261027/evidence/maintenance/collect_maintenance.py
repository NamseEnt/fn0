import argparse
import json
import os
from pathlib import Path
import tarfile
import time

import oci_maintenance as maintenance


def collect():
    if os.geteuid() != 0:
        raise AssertionError('collection requires the designated OCI root harness')
    frontier, reviewer, capacity_review, helper_paths = maintenance.dependencies()
    controls = frontier.validate_controls_review(
        maintenance.ROOT / 'resident-frontier' / '714320187' / 'final-review.json',
        maintenance.CONTROLS_REVIEW_SHA256)
    frozen_paths = frontier.controls_helper_paths(controls, Path(__file__).resolve().parent)
    capacity_module = maintenance.load_module('collection_capacity', frozen_paths['oci_capacity.py'])
    validation_module = maintenance.load_module('collection_validation', frozen_paths['oci_arrival_validate.py'])
    source_native = frontier.validate_current_source_native(validation_module)
    if (source_native['source_commit'] != maintenance.SOURCE_COMMIT or source_native['source_status'] != ''
            or source_native['native_archive_sha256_before'] != frontier.EXPECTED_NATIVE_SHA256):
        raise AssertionError('collection source/native provenance differs')
    run_directory = maintenance.RUN_DIRECTORY
    if not any((run_directory / name).is_file() for name in ['completion.json', 'failure-receipt.json']):
        raise AssertionError('workload cohort has no terminal receipt')
    maintenance.configure_reviewer(reviewer)
    review = maintenance.review_run(run_directory, reviewer, capacity_review, capacity_module)
    review_path = run_directory / 'review-r2.json'
    if review_path.exists():
        raise AssertionError('corrected review already exists; refusing overwrite')
    maintenance.write_json(review_path, review)
    provenance = maintenance.read_json(run_directory / 'provenance.json')
    for binary_name, identity in provenance['validated_binaries'].items():
        binary = run_directory / 'bin' / binary_name
        if binary.stat().st_size != identity['bytes'] or maintenance.digest(binary) != identity['sha256']:
            raise AssertionError(f'preserved binary changed: {binary_name}')
    groups = []
    for group in [maintenance.CGROUP_ROOT, *sorted(maintenance.CGROUP_ROOT.rglob('*'))]:
        if group.is_dir():
            processes = (group / 'cgroup.procs').read_text().strip()
            if processes:
                raise AssertionError(f'collected cgroup still populated: {group}')
            groups.append({'path': str(group), 'processes_empty': True,
                           'memory_max': (group / 'memory.max').read_text().strip(),
                           'memory_high': (group / 'memory.high').read_text().strip(),
                           'memory_swap_max': (group / 'memory.swap.max').read_text().strip()})
    databases = []
    for command_path in sorted(run_directory.glob('*-command.json')):
        case = command_path.name.removesuffix('-command.json')
        command = maintenance.read_json(command_path)['command']
        data_path = Path(command[command.index('--data-dir') + 1])
        if data_path != maintenance.DATABASE_ROOT / case:
            raise AssertionError('attempted database is outside the frozen namespace')
        files = [path for path in data_path.rglob('*') if path.is_file()]
        if not files:
            raise AssertionError(f'attempted database is not preserved: {case}')
        databases.append({'case': case, 'path': str(data_path), 'file_count': len(files),
                          'total_file_bytes': sum(path.stat().st_size for path in files), 'preserved': True})
    capacity = capacity_module.capacity()
    capacity_module.guard(capacity)
    release = Path(__file__).resolve().parents[2]
    archive_roots = [run_directory, maintenance.ROOT / 'maintenance' / '476c69218', release]
    paths = {path for directory in archive_roots for path in directory.rglob('*') if path.is_file()}
    paths.update(frozen_paths.values())
    paths.update(helper_paths.values())
    manifest = []
    for path in sorted(paths):
        if path.is_symlink():
            raise AssertionError(f'archive input is a symlink: {path}')
        manifest.append({'path': str(path.relative_to(maintenance.ROOT)),
                         'bytes': path.stat().st_size, 'sha256': maintenance.digest(path)})
    archive_path = maintenance.ROOT / 'maintenance' / (maintenance.RUN_NAME + '.tar.gz')
    with tarfile.open(archive_path, 'x:gz') as archive:
        for record in manifest:
            archive.add(maintenance.ROOT / record['path'], arcname=record['path'], recursive=False)
    receipt = {'timestamp_unix_seconds': time.time(), 'source_native': source_native,
               'collector_sha256': maintenance.digest(__file__),
               'reviewer_sha256': maintenance.digest(Path(maintenance.__file__)),
               'review_path': str(review_path), 'review_sha256': maintenance.digest(review_path),
               'completed_run_evidence_valid': review['completed_run_evidence_valid'],
               'accepted_run': review['accepted_run'], 'all_recorded_processes_exited': review['all_recorded_processes_exited'],
               'process_count': len(review['process_records']), 'cgroups': groups, 'databases': databases,
               'capacity_after': capacity, 'file_manifest': manifest,
               'archive': {'path': str(archive_path), 'bytes': archive_path.stat().st_size,
                           'sha256': maintenance.digest(archive_path)}}
    receipt_path = archive_path.with_name(maintenance.RUN_NAME + '-collection.json')
    if receipt_path.exists():
        raise AssertionError('collection receipt already exists')
    maintenance.write_json(receipt_path, receipt)
    print(json.dumps({key: value for key, value in receipt.items()
                      if key not in ['file_manifest', 'cgroups', 'databases', 'source_native']}, sort_keys=True))


def verify_local(evidence_directory):
    evidence_directory = Path(evidence_directory)
    receipt = maintenance.read_json(evidence_directory / 'collection.json')
    archive_path = evidence_directory / 'runtime.tar.gz'
    if maintenance.digest(archive_path) != receipt['archive']['sha256'] or archive_path.stat().st_size != receipt['archive']['bytes']:
        raise AssertionError('collected archive identity differs')
    runtime = evidence_directory / 'runtime'
    for record in receipt['file_manifest']:
        path = runtime / record['path']
        if path.is_symlink() or not path.is_file() or path.stat().st_size != record['bytes'] or maintenance.digest(path) != record['sha256']:
            raise AssertionError(f'extracted archive file differs: {record["path"]}')
    run_directory = runtime / 'arrival-phase' / maintenance.RUN_NAME
    review_path = run_directory / 'review-r2.json'
    if maintenance.digest(review_path) != receipt['review_sha256']:
        raise AssertionError('corrected review hash differs')
    review = maintenance.read_json(review_path)
    _, reviewer, capacity_review, _ = maintenance.dependencies()
    maintenance.configure_reviewer(reviewer)
    reports = {}
    for case, cell in review['cells'].items():
        command = maintenance.read_json(run_directory / (case + '-command.json'))
        reviewer.parse_command(command, case)
        report = maintenance.read_json(run_directory / (case + '.json'))
        reviewer.validate_report(report, case, cell['backend'])
        reduced = capacity_review.reduce_report(report, maintenance.SOURCE_COMMIT)
        if (maintenance.digest(run_directory / (case + '.json')) != cell['report_sha256']
                or reduced['slo_pass'] != cell['slo_pass']
                or maintenance.maintenance_metrics(report, capacity_review) != cell['maintenance']):
            raise AssertionError(f'local maintenance reduction differs: {case}')
        reports[case] = report
    for repetition, pair in review['pairs'].items():
        if capacity_review.compare_pair(reports[f'maintenance-n24-r{repetition}-blink'],
                                        reports[f'maintenance-n24-r{repetition}-shared']) != pair:
            raise AssertionError('local paired fingerprints differ')
    if not receipt['all_recorded_processes_exited'] or not all(group['processes_empty'] for group in receipt['cgroups']):
        raise AssertionError('collection did not preserve clean process/group exit evidence')
    if not all(database['preserved'] for database in receipt['databases']):
        raise AssertionError('collection did not preserve all attempted databases')
    output = {'archive_sha256': receipt['archive']['sha256'], 'file_count': len(receipt['file_manifest']),
              'locally_reduced_cells': len(reports), 'matched_pairs': len(review['pairs']),
              'completed_run_evidence_valid': review['completed_run_evidence_valid'],
              'accepted_run': review['accepted_run'], 'slo_passes_by_backend': review['slo_passes_by_backend'],
              'successful_requests': review['successful_requests'], 'oracle_count': review['oracle_count'],
              'verification_passed': True}
    maintenance.write_json(evidence_directory / 'collection-verification.json', output)
    print(json.dumps(output, sort_keys=True))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Collect or locally verify maintenance evidence')
    parser.add_argument('--verify-local', type=Path)
    arguments = parser.parse_args()
    if arguments.verify_local:
        verify_local(arguments.verify_local)
    else:
        collect()
