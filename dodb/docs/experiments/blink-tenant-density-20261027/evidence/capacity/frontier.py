import argparse
import json
from pathlib import Path
import statistics

from capacity_review import digest, review_directory


def summarize_series(observations):
    if not observations:
        return {'samples': 0}
    values = [value for _, value in observations]
    tail = observations[len(observations) // 2:]
    first_time = tail[0][0]
    times = [(timestamp - first_time) / 1000 for timestamp, _ in tail]
    mean_time = statistics.mean(times)
    mean_value = statistics.mean(value for _, value in tail)
    denominator = sum((timestamp - mean_time) ** 2 for timestamp in times)
    slope = sum((timestamp - mean_time) * (value - mean_value) for timestamp, (_, value) in zip(times, tail)) / denominator if denominator else None
    return {'samples': len(values), 'first': values[0], 'last': values[-1],
            'minimum': min(values), 'maximum': max(values),
            'latter_half_linear_slope_per_second': slope,
            'scope': 'sampled trend, not a steady-state or causality proof'}


def maintenance_observations(report):
    pressure = report['backend_memory_pressure']
    start = next(sample['snapshot']['timestamp_unix_ms'] for sample in pressure['samples'] if sample['event'] == 'offer-start')
    end = report['workload']['offer_boundary']['backend_memory_pressure_after']['measurement']['timestamp_unix_ms']
    memory = [(sample['snapshot']['timestamp_unix_ms'], sample['snapshot']['memory.current'])
              for sample in pressure['samples'] if sample['epoch'] == 'measurement'
              and start <= sample['snapshot']['timestamp_unix_ms'] <= end]
    health = report['memory_diagnostic']['engine_health_reports'][1]
    samples = [sample for sample in health['periodic_pressure']['samples'] if start <= sample['timestamp_unix_ms'] <= end]
    output = {'measurement_cgroup_current_bytes': summarize_series(memory)}
    if report['effective_configuration']['backend'] == 'blink':
        for name in ['wal_bytes_current', 'retained_image_bytes_normalized_page_images', 'tasks_running', 'checkpoints_completed', 'full_base_snapshots']:
            values = [(sample['timestamp_unix_ms'], sample['blink_checkpoints'][name])
                      for sample in samples if isinstance(sample['blink_checkpoints'].get(name), (int, float))]
            output[name] = summarize_series(values)
        events = health['blink_checkpoint_metrics']['checkpoint_events']
        offered_events = [event for event in events if start * 1_000_000 <= event['started_unix_nanos'] <= end * 1_000_000]
        output['checkpoint_events_started_during_offer'] = len(offered_events)
        output['full_base_events_started_during_offer'] = sum(event['full_base_snapshots'] for event in offered_events)
        output['omitted_final_checkpoint_events'] = health['blink_checkpoint_metrics']['omitted_checkpoint_events']
    else:
        for name in ['cache_usage_bytes', 'wbm_usage_bytes', 'wbm_mutable_bytes']:
            values = [(sample['timestamp_unix_ms'], sample['shared_resources']['usage'][name]) for sample in samples]
            output[name] = summarize_series(values)
        for name in ['estimate-pending-compaction-bytes', 'compaction-pending', 'mem-table-flush-pending', 'num-running-flushes', 'num-running-compactions', 'is-write-stopped', 'live-sst-files-size']:
            values = []
            for sample in samples:
                properties = [database['properties'].get(name, {}) for database in sample['rocksdb_properties']['databases']]
                if properties and all(item.get('status') == 'value' for item in properties):
                    values.append((sample['timestamp_unix_ms'], sum(item['value'] for item in properties)))
            output['tenant_property_sum_' + name] = summarize_series(values)
    return output


def main(run_directory):
    provenance = json.loads((run_directory / 'provenance.json').read_text())
    scripts = Path(__file__).resolve().parent
    assert digest(scripts / 'oci_capacity.py') == provenance['controller_sha256']
    assert digest(scripts / 'capacity_review.py') == provenance['reducer_sha256']
    review = review_directory(run_directory)
    summary = json.loads((run_directory / 'summary.json').read_text())
    assert len(summary) == len(review['cases'])
    compact = []
    maintenance = {}
    for outcome in summary:
        case = outcome['case']
        reduced = review['cases'][case]
        row = {name: reduced.get(name) for name in ['backend', 'registered', 'active', 'initial_rows', 'duration_ms', 'sample_interval_ms', 'operational_pass', 'slo_pass', 'resource_failure', 'failure_epochs', 'successful', 'dropped', 'offer_throughput_completed_per_second', 'cell_peak_bytes', 'epoch_peaks_bytes', 'cell_events', 'worst_tenant_p99_upper_exclusive_ms', 'backend_offer_cpu_usec', 'maximum_measurement_sample_gap_ms']}
        row['case'] = case
        report = json.loads((run_directory / (case + '.json')).read_text())
        if reduced['operational_pass']:
            configuration = report['effective_configuration']
            count = configuration['duration_ms'] * configuration['per_tenant_rate'] // 1000
            rows = configuration['initial_rows_per_tenant']
            distinct_get_rows = len({(operation * 0x9e37 + 1) % rows for operation in range(count) if operation * 37 % 100 < 40})
            row['offered_distinct_seed_get_rows_per_tenant'] = distinct_get_rows
            row['offered_seed_get_coverage_fraction'] = distinct_get_rows / rows
            row['seed_read_coverage_scope'] = 'Get recipe only; Query reads first sixteen rows; realized Get coverage matches only if no drops'
            row['backend_sampled_rss_max_kib'] = reduced['process_sampling']['backend']['fields']['rss_kb']['observed_max']
            row['driver_sampled_rss_max_kib'] = reduced['process_sampling']['driver']['fields']['rss_kb']['observed_max']
            row['measurement_native_serialized_bytes'] = reduced['native_serialized_bytes']
            row['filesystem_after_measurement'] = reduced['filesystem_snapshots']['after_measurement_worker_shutdown']
        compact.append(row)
        if case.startswith('maintenance-') and reduced['operational_pass']:
            maintenance[case] = maintenance_observations(report)
    cohorts = {}
    for prefix in ['screen-', 'refine-', 'confirm-', 'sampling-', 'sparse-', 'maintenance-']:
        selected = [row for row in compact if row['case'].startswith(prefix)]
        cohorts[prefix.rstrip('-')] = {'cells': len(selected), 'operational_passes': sum(row['operational_pass'] for row in selected),
                                       'slo_passes': sum(row['slo_pass'] for row in selected),
                                       'resource_failures': sum(row['resource_failure'] for row in selected),
                                       'successful_requests_with_complete_reports': sum(row['successful'] or 0 for row in selected)}
    confirmations = []
    for stage in ['e2', 'e3']:
        for backend in ['blink', 'minimal-native-shared']:
            selected = [row for row in compact if row['case'].startswith(f'confirm-{stage}-') and row['backend'] == backend]
            for tenants in sorted({row['active'] for row in selected}):
                repetitions = [row for row in selected if row['active'] == tenants]
                completed = [row for row in repetitions if row['operational_pass']]
                confirmations.append({'stage': stage.upper(), 'backend': backend, 'active': tenants,
                    'repetitions': len(repetitions), 'slo_passes': sum(row['slo_pass'] for row in repetitions),
                    'operational_passes': len(completed),
                    'all_three_slo_pass': len(repetitions) == 3 and all(row['slo_pass'] for row in repetitions),
                    'median_measurement_epoch_peak_bytes': statistics.median(row['epoch_peaks_bytes']['measurement'] for row in completed) if completed else None,
                    'median_cell_peak_bytes': statistics.median(row['cell_peak_bytes'] for row in completed) if completed else None,
                    'median_backend_offer_cpu_usec': statistics.median(row['backend_offer_cpu_usec'] for row in completed) if completed else None,
                    'worst_p99_upper_exclusive_ms_across_repetitions': {name: max(row['worst_tenant_p99_upper_exclusive_ms'][name] for row in completed) for name in ['Get', 'Query', 'Write']} if completed else None})
    output = {'source_commit': provenance['source_commit'], 'plan_commit': provenance['plan_commit'],
              'runtime_script_hashes_verified': True, 'cohorts': cohorts, 'cells': compact,
              'confirmation_groups': confirmations,
              'maintenance': maintenance, 'scope': 'Frozen modest capacity bracket; cohorts and memory/SLO outcomes remain separate'}
    (run_directory / 'frontier.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({'cohorts': cohorts, 'reviewed_pairs': len(review['pairs'])}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('run_directory', type=Path)
    arguments = parser.parse_args()
    main(arguments.run_directory)
