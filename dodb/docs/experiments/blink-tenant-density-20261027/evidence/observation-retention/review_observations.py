import json
from pathlib import Path
import statistics
import sys


ROOT = Path(__file__).resolve().parent
CAPACITY = ROOT.parent / 'capacity'
sys.path.insert(0, str(CAPACITY))

import capacity_review
import frontier
from operational_review import reduce_report


BASELINE = 'aaa40565529ca92150df5d94e19b02ca93859f71'
CANDIDATE = '023b18bd8921b593f243f59b9eaa1576125c6572'
CONTROL = ROOT / 'runtime/observations-control-aaa405655-r1'
VARIANT = ROOT / 'runtime/observations-023b18bd8-r1'


def read_json(file_path):
    return json.loads(file_path.read_text())


def review_run(directory, source_commit, controller_key, controller_name):
    provenance = read_json(directory / 'provenance.json')
    assert provenance['source_commit'] == source_commit
    assert provenance[controller_key] == capacity_review.digest(ROOT / controller_name)
    if 'operational_failure_reducer_sha256' in provenance:
        assert provenance['operational_failure_reducer_sha256'] == capacity_review.digest(CAPACITY / 'operational_review.py')
    capacity_review.reduce_report = lambda report, revision: reduce_report(
        report, revision, (directory / (Path(report['effective_configuration']['backend_memory_budget']['cgroup_root']).name + '.log')).read_text())
    frontier.main(directory)
    result = read_json(directory / 'frontier.json')
    result['observation_controller_sha256_verified'] = True
    for row in result['cells']:
        report = read_json(directory / (row['case'] + '.json'))
        status = read_json(directory / (row['case'] + '-status.json'))
        assert status['stop_reason'] is None and 'review_error' not in status
        assert (status['exit_code'] == 0) == row['operational_pass']
        assert report['memory_diagnostic']['source_revision'] == source_commit
        if row['operational_pass']:
            assert report['compiled_source_revision'] == report['requested_source_revision'] == source_commit
        for event in report['memory_diagnostic']['worker_lifecycle']:
            assert event['source_revision'] == source_commit
            assert event['harness_binary_sha256'] == provenance['binaries']['tenant_density_workload']['sha256']
            assert event['worker_binary_sha256'] == provenance['binaries']['memory_diagnostic_worker']['sha256']
        for health in report['memory_diagnostic']['engine_health_reports']:
            pressure = health['periodic_pressure']
            assert pressure['enabled'] and not pressure['samples_truncated']
            assert pressure['sample_limit'] == 4096
            assert pressure['serialized_sample_byte_limit'] == 8 * 1024**2
            samples = pressure['samples']
            assert all(isinstance(sample, dict) for sample in samples)
            encoded_bytes = sum(len(json.dumps(sample, ensure_ascii=False, separators=(',', ':')).encode()) for sample in samples)
            assert pressure['serialized_sample_bytes'] == encoded_bytes
            timestamps = [sample['timestamp_unix_ms'] for sample in samples]
            assert timestamps == sorted(timestamps)
        if row['operational_pass']:
            pressure = report['memory_diagnostic']['engine_health_reports'][1]['periodic_pressure']
            row['native_measurement_sample_count'] = len(pressure['samples'])
            row['mean_serialized_native_frame_bytes'] = pressure['serialized_sample_bytes'] / len(pressure['samples']) if pressure['samples'] else None
    return result


def comparable_configuration(report):
    configuration = json.loads(json.dumps(report['effective_configuration']))
    del configuration['backend_memory_budget']['cgroup_root']
    return configuration


def compare_sources(reference_directory, variant_directory, cases):
    comparisons = []
    for case in cases:
        reference = read_json(reference_directory / (case + '.json'))
        variant = read_json(variant_directory / (case + '.json'))
        assert comparable_configuration(reference) == comparable_configuration(variant)
        realized = capacity_review.compare_pair(reference, variant)
        if reference['status'] == variant['status'] == 'completed':
            for old_health, new_health in zip(reference['memory_diagnostic']['engine_health_reports'], variant['memory_diagnostic']['engine_health_reports']):
                assert set(old_health) == set(new_health)
                assert old_health['effective_backend_configuration'] == new_health['effective_backend_configuration']
                assert old_health['retained_heap_bytes'] == new_health['retained_heap_bytes']
                assert set(old_health['periodic_pressure']) == set(new_health['periodic_pressure'])
                old_samples = old_health['periodic_pressure']['samples']
                new_samples = new_health['periodic_pressure']['samples']
                if old_samples and new_samples:
                    assert set(old_samples[0]) == set(new_samples[0])
        comparisons.append({'case': case, 'configuration_matches_excluding_cgroup_path': True,
                            'parsed_worker_report_keys_match_on_complete_pairs': reference['status'] == variant['status'] == 'completed',
                            'workload': realized})
    return comparisons


def cohort_memory(rows):
    output = []
    keys = sorted({(row['initial_rows'], row['active'], row['duration_ms'], row['sample_interval_ms'], row['backend']) for row in rows})
    for initial_rows, active, duration_ms, interval_ms, backend in keys:
        selected = [row for row in rows if (row['initial_rows'], row['active'], row['duration_ms'], row['sample_interval_ms'], row['backend']) == (initial_rows, active, duration_ms, interval_ms, backend)]
        complete = [row for row in selected if row['operational_pass']]
        output.append({'initial_rows': initial_rows, 'active': active, 'duration_ms': duration_ms,
                       'sample_interval_ms': interval_ms, 'backend': backend, 'cells': len(selected),
                       'operational_passes': len(complete), 'slo_passes': sum(row['slo_pass'] for row in selected),
                       'median_offer_boundary_epoch_peak_bytes': statistics.median(row['offer_boundary_epoch_peak_bytes'] for row in complete) if complete else None,
                       'median_measurement_epoch_peak_bytes': statistics.median(row['epoch_peaks_bytes']['measurement'] for row in complete) if complete else None,
                       'median_backend_offer_cpu_usec': statistics.median(row['backend_offer_cpu_usec'] for row in complete) if complete else None})
    return output


def main():
    control = review_run(CONTROL, BASELINE, 'control_controller_sha256', 'oci_observations_control.py')
    variant = review_run(VARIANT, CANDIDATE, 'observation_controller_sha256', 'oci_observations_compare.py')
    cadence_cases = [row['case'] for row in control['cells']]
    source_comparisons = compare_sources(CONTROL, VARIANT, cadence_cases)
    original_comparisons = []
    for name in ['capacity-aaa405655-r2', 'capacity-supplement-aaa405655-r1']:
        original = CAPACITY / 'runtime' / name
        shared_cases = sorted({row['case'] for row in variant['cells']} & {file_path.name[:-len('-status.json')] for file_path in original.glob('*-status.json')})
        original_comparisons.extend({'reference_run': name, **comparison} for comparison in compare_sources(original, VARIANT, shared_cases))
    rows = [row for run in [control, variant] for row in run['cells']]
    databases = set()
    identities = set()
    for file_path in sorted((ROOT / 'collection').glob('*-postflight.json')):
        postflight = read_json(file_path)
        assert postflight['source_clean'] and postflight['source_commit'] == CANDIDATE
        assert postflight['collector_sha256'] == capacity_review.digest(ROOT / 'oci_observations_collect.py')
        assert postflight['native_archive_sha256'] == '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
        for name, run in postflight['runs'].items():
            provenance = read_json(ROOT / 'runtime' / name / 'provenance.json')
            assert run['source_commit'] == provenance['source_commit'] and run['binaries'] == provenance['binaries']
            for database in run['databases'].values():
                assert database['preserved']
                databases.add(database['path'])
            for identity in run['processes']:
                assert not identity['same_live_process']
                identities.add((identity['pid'], identity['start_time_ticks']))
            assert all(not pids for group in run['cgroup_processes'].values() for pids in group.values())
    assert len(databases) == len(rows)
    host_samples = [sample for directory in [CONTROL, VARIANT] for file_path in directory.glob('*-host-samples.json') for sample in read_json(file_path)]
    minimum_free = min(sample['zfs_free_bytes'] for sample in host_samples)
    minimum_available = min(sample['memory_bytes']['MemAvailable'] for sample in host_samples)
    assert minimum_free >= 10 * 1024**3 and minimum_available >= 1500 * 1024**2
    validation = read_json(ROOT / 'validation-review.json')
    assert validation['source_commit'] == CANDIDATE and validation['passing_tests'] == 74
    assert validation['same_17_existing_clippy_diagnostics']
    assert validation['archive_sha256'] == capacity_review.digest(ROOT / 'validation.tar.gz')
    transfer = read_json(ROOT / 'collection/runtime-transfer.json')
    assert transfer['archive_sha256'] == capacity_review.digest(ROOT / 'runtime.tar.gz')
    assert transfer['bytes'] == (ROOT / 'runtime.tar.gz').stat().st_size
    output = {'reviewer_sha256': capacity_review.digest(Path(__file__)), 'control': control, 'variant': variant,
              'current_host_control_comparisons': source_comparisons, 'original_cohort_comparisons': original_comparisons,
              'control_cohorts': cohort_memory(control['cells']), 'variant_cohorts': cohort_memory(variant['cells']),
              'unique_new_cells': len(rows), 'completed_cells': sum(row['operational_pass'] for row in rows),
              'slo_passing_cells': sum(row['slo_pass'] for row in rows),
              'kernel_oom_cells': sum(row['resource_failure'] for row in rows),
              'setup_shutdown_deadline_cells': sum(bool(row.get('operational_timeout')) for row in rows),
              'successful_offered_requests_in_complete_reports': sum(row['successful'] or 0 for row in rows),
              'passing_full_population_oracles': 2 * sum(row['operational_pass'] for row in rows),
              'preserved_unique_databases': len(databases), 'exited_process_identities': len(identities),
              'minimum_sampled_zfs_free_bytes': minimum_free, 'minimum_sampled_host_available_bytes': minimum_available,
              'validation': validation, 'runtime_archive': transfer,
              'scope': 'Instrumented diagnostic worker; new current-host cadence control plus separately dated original large-population cohorts; no engine-only or total-machine ranking'}
    (ROOT / 'final-review.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({key: value for key, value in output.items() if key.endswith('_cells') or key in ['successful_offered_requests_in_complete_reports', 'passing_full_population_oracles', 'preserved_unique_databases', 'exited_process_identities']}, indent=2))


if __name__ == '__main__':
    main()
