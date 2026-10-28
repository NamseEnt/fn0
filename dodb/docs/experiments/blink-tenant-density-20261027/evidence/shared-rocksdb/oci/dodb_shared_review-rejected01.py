import argparse
import hashlib
import json
from pathlib import Path
import time


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--run-directory', type=Path, required=True)
    parser.add_argument('--source-commit', required=True)
    options = parser.parse_args()
    directory = options.run_directory
    cases = ['native-all-active', 'native-sparse', 'shared-all-active', 'shared-sparse']
    summary = json.loads((directory / 'summary.json').read_text())
    assert {row['case'] for row in summary} == set(cases)
    assert all(row['exit_code'] == 0 and row['validation']['passed'] for row in summary)
    reports = {name: json.loads((directory / f'{name}.json').read_text()) for name in cases}
    review = {'scope': 'Correctness and shared-resource acceptance only; no performance, SLO, density, or enforced process-memory result',
        'source_commit': options.source_commit, 'reviewed_unix_seconds': time.time(),
        'review_script_sha256': digest(Path(__file__)), 'cases': {}, 'matched_pairs': {}}
    for name, report in reports.items():
        assert report['compiled_source_revision'] == options.source_commit
        diagnostic = report['memory_diagnostic']
        still_live = []
        for process in diagnostic['processes']:
            stat_path = Path('/proc') / str(process['pid']) / 'stat'
            if stat_path.exists():
                start_ticks = int(stat_path.read_text().rsplit(')', 1)[1].split()[19])
                if start_ticks == process['start_time_ticks']:
                    still_live.append(process)
        assert not still_live, f'{name} has remaining processes: {still_live}'
        workload = report['workload']
        sampling = report['measurement_worker_process_sampling']
        item = {'report_sha256': digest(directory / f'{name}.json'),
            'scheduled': workload['scheduled'], 'completed': workload['completed'],
            'successful': workload['successful'], 'completed_at_offer_end': workload['completed_at_offer_end'],
            'drain_completions': workload['completed'] - workload['completed_at_offer_end'],
            'request_fingerprint': workload['request_fingerprint'],
            'scheduled_fingerprint': workload['scheduled_fingerprint'],
            'per_tenant_completions': [row['completed'] for row in workload['per_tenant']],
            'verified_seed_rows_per_oracle': report['post_measurement_full_population_oracle']['verified_seed_rows'],
            'verified_extra_keys_per_oracle': report['post_measurement_full_population_oracle']['verified_mutated_keys'],
            'measurement_process_sampling_status': sampling['status'],
            'periodic_offer_sample_count': sampling['periodic_offer_sample_count'],
            'process_samples_truncated': sampling['process_samples_truncated'],
            'same_backend_identity': sampling['same_backend_identity_at_offer_boundaries'],
            'recorded_processes_still_live': still_live}
        if name.startswith('shared'):
            item['native_resource_snapshots'] = diagnostic['engine_health_reports'][1]['shared_resource_snapshots']
        review['cases'][name] = item
    for suffix in ['all-active', 'sparse']:
        control = reports[f'native-{suffix}']['workload']
        shared = reports[f'shared-{suffix}']['workload']
        matched = {field: control[field] == shared[field] for field in
            ['deterministic_fingerprints', 'request_count', 'request_fingerprint', 'scheduled_fingerprint',
             'scheduled', 'admitted', 'completed', 'successful', 'operation_counts', 'transaction_counts', 'write_width_counts']}
        for field in ['tenant', 'request_count', 'request_fingerprint', 'scheduled_fingerprint', 'scheduled', 'admitted', 'completed', 'successful']:
            matched[f'per_tenant_{field}'] = [row[field] for row in control['per_tenant']] == [row[field] for row in shared['per_tenant']]
        assert all(matched.values()), matched
        review['matched_pairs'][suffix] = matched
    review['minimum_observed_zfs_free_bytes'] = min([row['capacity_after']['zfs_free_bytes'] for row in summary] +
        [json.loads((directory / f'{name}-command.json').read_text())['preflight']['zfs_free_bytes'] for name in cases])
    assert review['minimum_observed_zfs_free_bytes'] >= 10 * 1024**3
    review['passed'] = True
    (directory / 'acceptance-review.json').write_text(json.dumps(review, indent=2) + '\n')
    print(json.dumps({'passed': True, 'total_completed': sum(row['completed'] for row in review['cases'].values()),
        'minimum_observed_zfs_free_bytes': review['minimum_observed_zfs_free_bytes']}))


if __name__ == '__main__':
    main()
