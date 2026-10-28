import argparse
import hashlib
import json
from pathlib import Path


LIMIT_BUCKETS = {'Get': 23, 'Query': 24, 'Write': 26}


def digest(file_path):
    return hashlib.sha256(Path(file_path).read_bytes()).hexdigest()


def percentile_bucket(histogram, numerator=99, denominator=100):
    counts = histogram['bucket_counts']
    observations = histogram['observation_count']
    assert sum(counts) == observations
    if not observations:
        return None
    rank = (observations * numerator + denominator - 1) // denominator
    cumulative = 0
    for bucket_number, count in enumerate(counts):
        cumulative += count
        if cumulative >= rank:
            return {'bucket': bucket_number,
                    'lower_inclusive_ns': 0 if bucket_number == 0 else 2 ** (bucket_number - 1),
                    'upper_exclusive_ns': 1 if bucket_number == 0 else 2 ** bucket_number}
    raise AssertionError('percentile rank missing from histogram')


def reduce_report(report, source_commit):
    configuration = report['effective_configuration']
    pressure = report['backend_memory_pressure']
    final = pressure['cell']['final']
    result = {
        'backend': configuration['backend'],
        'registered': configuration['registered_tenants'],
        'active': configuration['active_tenants'],
        'initial_rows': configuration['initial_rows_per_tenant'],
        'duration_ms': configuration['duration_ms'],
        'sample_interval_ms': configuration['backend_memory_budget']['sample_interval_ms'],
        'status': report['status'],
        'cell_peak_bytes': final['memory.peak'],
        'cell_events': pressure['cell']['event_delta'],
        'epoch_peaks_bytes': {name: epoch['final']['memory.peak'] for name, epoch in pressure['epochs'].items()},
        'epoch_events': {name: epoch['local_event_delta'] for name, epoch in pressure['epochs'].items()},
        'operational_pass': False,
        'slo_pass': False,
        'resource_failure': final['memory.events']['oom_kill'] > 0,
        'failure_reason': report.get('failure_reason'),
    }
    assert pressure['enabled'] and final['cgroup.events']['populated'] == 0
    for snapshot in [final, *[epoch['final'] for epoch in pressure['epochs'].values()]]:
        assert snapshot['memory.max'] == 256 * 1024**2
        assert snapshot['memory.high'] == 192 * 1024**2
        assert snapshot['memory.swap.max'] == snapshot['memory.swap.current'] == 0
        assert snapshot['cgroup.procs'] == []
    if result['resource_failure']:
        result['failure_epochs'] = [name for name, events in result['epoch_events'].items() if events['oom_kill']]
        return result
    assert report['status'] == 'completed', result
    assert report['compiled_source_revision'] == report['requested_source_revision'] == source_commit
    assert not pressure['samples_truncated']
    workload = report['workload']
    assert workload['cell_valid'] and workload['valid_for_oracle']
    assert all(workload['accounting'].values())
    assert all(workload[field] == 0 for field in ['rejected_conflicts', 'other_error', 'timed_out', 'queued_at_end', 'in_flight_at_end'])
    assert workload['completed'] == workload['admitted'] == workload['successful']
    expected_per_tenant = configuration['duration_ms'] * configuration['per_tenant_rate'] // 1000
    assert workload['scheduled'] == expected_per_tenant * configuration['active_tenants']
    for oracle_name in ['post_measurement_full_population_oracle', 'reopened_full_population_oracle']:
        assert report[oracle_name]['passed']
        assert report[oracle_name]['verified_seed_rows'] == configuration['registered_tenants'] * configuration['initial_rows_per_tenant']
    health_reports = report['memory_diagnostic']['engine_health_reports']
    assert len(health_reports) == 4
    identities = report['memory_diagnostic']['processes']
    for epoch_name, health in zip(['setup', 'measurement', 'oracle', 'reopen-oracle'], health_reports):
        assert health['shutdown_complete'] and health['compiled_source_revision'] == source_commit
        membership = health['backend_memory_cgroup']
        assert membership['verified'] and membership['before_runtime_and_backend']
        assert membership['pid'] == health['worker_pid']
        assert membership['path'] == pressure['configuration']['cgroup_root'] + '/' + epoch_name
        assert membership['pid'] in membership['snapshot']['cgroup.procs']
        matching = [identity for identity in identities if identity['pid'] == health['worker_pid'] and identity['role'] == 'backend-process']
        assert len(matching) == 1 and matching[0]['start_time_ticks'] == health['worker_process_start_time_ticks']
        assert health['periodic_pressure']['enabled'] and not health['periodic_pressure']['samples_truncated']
        if configuration['backend'] == 'minimal-native-shared':
            snapshots = health['shared_resource_snapshots']
            names = ['before_shutdown'] + (['measurement_start', 'measurement_end'] if epoch_name == 'measurement' else [])
            for snapshot_name in names:
                native = snapshots[snapshot_name]
                opened = configuration['active_tenants'] + configuration['opened_idle_tenants'] if epoch_name == 'measurement' else configuration['registered_tenants']
                assert native['opened_tenants'] == native['tenants_verified_using_shared_resources'] == opened
                assert native['usage']['cache_capacity_bytes'] == 64 * 1024**2
                assert native['usage']['wbm_limit_bytes'] == 32 * 1024**2
        else:
            assert health['blink_checkpoint_metrics']['checkpoints_failed'] == 0
            assert health['blink_checkpoint_metrics']['omitted_checkpoint_events'] == 0
    assert report['measurement_worker_process_sampling']['same_backend_identity_at_offer_boundaries']
    health = health_reports[1]
    native_samples = health['periodic_pressure']['samples']
    measurement_samples = [sample for sample in pressure['samples'] if sample['epoch'] == 'measurement' and sample['event'] == 'periodic']
    timestamps = [sample['snapshot']['timestamp_unix_ms'] for sample in measurement_samples]
    result['measurement_sample_count'] = len(measurement_samples)
    result['maximum_measurement_sample_gap_ms'] = max((later - earlier for earlier, later in zip(timestamps, timestamps[1:])), default=None)
    result['native_measurement_sample_count'] = len(native_samples)
    result['native_serialized_bytes'] = health['periodic_pressure']['serialized_sample_bytes']
    result['native_samples_truncated'] = health['periodic_pressure']['samples_truncated']
    before = report['offer_boundary_process_before']
    after = report['offer_boundary_process_after']
    result['process_sampling'] = report['measurement_worker_process_sampling']
    offer_start = next(sample for sample in pressure['samples'] if sample['event'] == 'offer-start')
    offer_end = workload['offer_boundary']['backend_memory_pressure_after']['measurement']
    result['backend_offer_cpu_usec'] = offer_end['cpu.stat']['usage_usec'] - offer_start['snapshot']['cpu.stat']['usage_usec']
    result['backend_offer_io_delta'] = {name: report['offer_boundary_process_io_after']['counters'][name] - value for name, value in report['offer_boundary_process_io_before']['counters'].items()}
    host_before = report['offer_boundary_host_device_before']
    host_after = report['offer_boundary_host_device_after']
    result['whole_device_offer_write_bytes'] = host_after['write_bytes'] - host_before['write_bytes'] if host_before['status'] == host_after['status'] == 'sampled' else None
    result['filesystem_snapshots'] = report['filesystem_snapshots']
    result['offer_elapsed_ms'] = workload['offer_window_elapsed_ms']
    result['scheduled'] = workload['scheduled']
    result['successful'] = workload['successful']
    result['dropped'] = workload['dropped']
    result['completed_at_offer_end'] = workload['completed_at_offer_end']
    result['offer_throughput_completed_per_second'] = workload['completed_at_offer_end'] * 1000 / workload['offer_window_elapsed_ms']
    result['aggregate_p99'] = {name: percentile_bucket(histogram) for name, histogram in workload['operation_latency_histograms'].items()}
    result['per_tenant_objective'] = []
    offer_counts = {tenant['tenant']: tenant for tenant in workload['offer_boundary_accounting']}
    for tenant in workload['per_tenant']:
        histograms = tenant['operation_latency_histograms']
        assert all(sum(histogram['bucket_counts']) == histogram['observation_count'] for histogram in histograms.values())
        percentiles = {name: percentile_bucket(histogram) for name, histogram in histograms.items()}
        counts_pass = tenant['scheduled'] == tenant['admitted'] == tenant['completed'] == tenant['successful'] == expected_per_tenant and tenant['dropped'] == 0
        rate_pass = offer_counts[tenant['tenant']]['completed'] * 100 >= expected_per_tenant * 99
        latency_pass = all(histograms[name]['observation_count'] >= 100 and percentiles[name]['bucket'] <= limit for name, limit in LIMIT_BUCKETS.items())
        result['per_tenant_objective'].append({'tenant': tenant['tenant'], 'counts_pass': counts_pass,
            'offer_rate_pass': rate_pass, 'latency_pass': latency_pass,
            'p99': percentiles, 'completed_at_offer_end': offer_counts[tenant['tenant']]['completed'],
            'passed': counts_pass and rate_pass and latency_pass})
    result['worst_tenant_p99_upper_exclusive_ms'] = {name: max(tenant['p99'][name]['upper_exclusive_ns'] for tenant in result['per_tenant_objective']) / 1_000_000 for name in LIMIT_BUCKETS}
    result['operational_pass'] = True
    result['slo_pass'] = all(tenant['passed'] for tenant in result['per_tenant_objective'])
    result['checkpoint_metrics'] = health['blink_checkpoint_metrics']
    result['native_first_last'] = [native_samples[0], native_samples[-1]] if native_samples else []
    result['offer_boundary_process_identities'] = {'before': before, 'after': after}
    return result


def compare_pair(left, right):
    if 'workload' not in left or 'workload' not in right:
        return {'realized_comparison_available': False, 'reason': 'at least one cell failed before complete workload counters'}
    left_workload = left['workload']
    right_workload = right['workload']
    assert left_workload['deterministic_fingerprints'] == right_workload['deterministic_fingerprints']
    fields = ['request_count', 'request_fingerprint', 'scheduled_fingerprint', 'scheduled', 'admitted', 'completed', 'successful']
    matches = {field: left_workload[field] == right_workload[field] for field in fields}
    for field in ['tenant', *fields, 'operation_counts', 'transaction_counts', 'width_counts']:
        matches['per_tenant_' + field] = [tenant[field] for tenant in left_workload['per_tenant']] == [tenant[field] for tenant in right_workload['per_tenant']]
    if left_workload['dropped'] == right_workload['dropped'] == 0:
        assert all(matches.values()), matches
    return {'realized_comparison_available': True, 'recipe_matches': True, 'realized_matches': matches}


def review_directory(run_directory):
    provenance = json.loads((run_directory / 'provenance.json').read_text())
    results = {}
    for status_path in sorted(run_directory.glob('*-status.json')):
        status = json.loads(status_path.read_text())
        report_path = run_directory / (status['case'] + '.json')
        assert digest(report_path) == status['report_sha256']
        results[status['case']] = reduce_report(json.loads(report_path.read_text()), provenance['source_commit'])
    pairs = {}
    for pair_path in sorted(run_directory.glob('*-pair.json')):
        pair = json.loads(pair_path.read_text())
        reports = [json.loads((run_directory / (name + '.json')).read_text()) for name in pair['cases']]
        pairs[pair_path.stem] = compare_pair(*reports)
    output = {'source_commit': provenance['source_commit'], 'cases': results, 'pairs': pairs}
    (run_directory / 'local-review.json').write_text(json.dumps(output, indent=2) + '\n')
    return output


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('run_directory', type=Path)
    arguments = parser.parse_args()
    review = review_directory(arguments.run_directory)
    print(json.dumps({'source_commit': review['source_commit'], 'reviewed_cases': len(review['cases']), 'reviewed_pairs': len(review['pairs'])}, indent=2))
