import json
from pathlib import Path
import statistics
import sys


ROOT = Path(__file__).resolve().parent
CAPACITY = ROOT.parent / 'capacity'
sys.path.insert(0, str(CAPACITY))

import capacity_review
import frontier
import operational_review


BASE_REDUCE_REPORT = capacity_review.reduce_report
assert operational_review.baseline_reduce_report is BASE_REDUCE_REPORT


BASELINE = 'aaa40565529ca92150df5d94e19b02ca93859f71'
CANDIDATE = '023b18bd8921b593f243f59b9eaa1576125c6572'
CONTROL = ROOT / 'runtime/observations-control-aaa405655-r1'
VARIANT = ROOT / 'runtime/observations-023b18bd8-r1'
STOPPED_CASE = 'confirm-e3-n5-r3-blink'
STOPPED_FAILURE = 'diagnostic worker shutdown timed out and was killed'


def read_json(file_path):
    return json.loads(file_path.read_text())


def reduce_stopped_measurement(report, revision, directory):
    assert revision == CANDIDATE
    assert report == read_json(directory / (STOPPED_CASE + '.json'))
    status = read_json(directory / (STOPPED_CASE + '-status.json'))
    assert status['case'] == STOPPED_CASE and status['exit_code'] == 1 and status['stop_reason'] is None
    assert STOPPED_FAILURE in status['review_error']
    assert report['failure_reason'] == STOPPED_FAILURE
    assert STOPPED_FAILURE in (directory / (STOPPED_CASE + '.log')).read_text()
    assert report['status'] == 'failed_or_inconclusive'
    assert report['partial_workload_counters'] == {'status': 'unavailable_after_early_abort'}
    assert report['data_directory_preserved']
    configuration = report['effective_configuration']
    assert configuration['backend'] == 'blink' and configuration['active_tenants'] == 5
    assert configuration['duration_ms'] == 60000 and configuration['backend_memory_budget']['sample_interval_ms'] == 1000
    diagnostic = report['memory_diagnostic']
    assert diagnostic['source_revision'] == CANDIDATE
    provenance = read_json(directory / 'provenance.json')
    binaries = provenance['binaries']
    assert diagnostic['harness_binary_sha256'] == binaries['tenant_density_workload']['sha256']
    lifecycle = diagnostic['worker_lifecycle']
    assert [event['event'] for event in lifecycle] == ['started', 'stopped', 'started']
    assert all(event['source_revision'] == CANDIDATE for event in lifecycle)
    assert all(event['harness_binary_sha256'] == binaries['tenant_density_workload']['sha256'] for event in lifecycle)
    assert all(event['worker_binary_sha256'] == binaries['memory_diagnostic_worker']['sha256'] for event in lifecycle)
    setup_pid = lifecycle[0]['pid']
    measurement_pid = lifecycle[2]['pid']
    assert lifecycle[1]['pid'] == setup_pid and measurement_pid != setup_pid
    backend_identities = [identity for identity in diagnostic['processes'] if identity['role'] == 'backend-process']
    assert {identity['pid'] for identity in backend_identities} == {setup_pid, measurement_pid}
    assert all(identity['pid'] in {setup_pid, measurement_pid} or identity['role'] == 'harness-process'
               for identity in diagnostic['processes'])
    health_reports = diagnostic['engine_health_reports']
    assert len(health_reports) == 1
    setup_health = health_reports[0]
    assert setup_health['shutdown_complete'] and setup_health['compiled_source_revision'] == CANDIDATE
    assert setup_health['worker_pid'] == setup_pid
    pressure = report['backend_memory_pressure']
    assert pressure['enabled'] and pressure['groups_preserved'] and not pressure['samples_truncated']
    samples = pressure['samples']
    events = {(sample['epoch'], sample['event']) for sample in samples}
    assert {('setup', 'previous-worker-exited'), ('measurement', 'before-worker-spawn'),
            ('measurement', 'offer-start'), ('measurement', 'after-drain'),
            ('measurement', 'after-worker-exit')} <= events
    drain_time = next(sample['snapshot']['timestamp_unix_ms'] for sample in samples
                      if sample['epoch'] == 'measurement' and sample['event'] == 'after-drain')
    exit_time = next(sample['snapshot']['timestamp_unix_ms'] for sample in samples
                     if sample['epoch'] == 'measurement' and sample['event'] == 'after-worker-exit')
    shutdown_elapsed_ms = exit_time - drain_time
    assert shutdown_elapsed_ms == 20039
    assert not any(event['event'] == 'stopped' and event['pid'] == measurement_pid for event in lifecycle)
    assert setup_health['periodic_pressure']['enabled'] and not setup_health['periodic_pressure']['samples_truncated']
    setup_samples = setup_health['periodic_pressure']['samples']
    setup_sample_keys = set(setup_samples[0]) if setup_samples else None
    assert all(type(sample) is dict and set(sample) == setup_sample_keys for sample in setup_samples)
    setup_encoded_bytes = sum(len(json.dumps(sample, ensure_ascii=False, separators=(',', ':')).encode())
                              for sample in setup_samples)
    assert setup_health['periodic_pressure']['serialized_sample_bytes'] == setup_encoded_bytes
    assert setup_health['periodic_pressure']['sample_limit'] == 4096
    assert setup_health['periodic_pressure']['serialized_sample_byte_limit'] == 8 * 1024**2
    assert setup_encoded_bytes <= setup_health['periodic_pressure']['serialized_sample_byte_limit']
    setup_timestamps = [sample['timestamp_unix_ms'] for sample in setup_samples]
    assert setup_timestamps == sorted(setup_timestamps)
    final = pressure['cell']['final']
    assert final['cgroup.events']['populated'] == 0 and final['cgroup.procs'] == []
    assert final['memory.peak'] == 231751680
    assert final['memory.events']['high'] == 8264
    assert all(final['memory.events'][name] == 0 for name in ['max', 'oom', 'oom_group_kill', 'oom_kill'])
    epoch_peaks = {name: epoch['final']['memory.peak'] for name, epoch in pressure['epochs'].items()}
    epoch_events = {name: epoch['local_event_delta'] for name, epoch in pressure['epochs'].items()}
    assert epoch_peaks == {'measurement': 231706624, 'oracle': 0, 'reopen-oracle': 0, 'setup': 212492288}
    assert epoch_events['measurement']['high'] == 2797
    assert all(epoch_events[name]['oom_kill'] == 0 for name in epoch_events)
    for snapshot in [final, *[epoch['final'] for epoch in pressure['epochs'].values()]]:
        assert snapshot['memory.max'] == 256 * 1024**2
        assert snapshot['memory.high'] == 192 * 1024**2
        assert snapshot['memory.swap.max'] == snapshot['memory.swap.current'] == 0
        assert snapshot['cgroup.procs'] == []
    return {
        'backend': configuration['backend'], 'registered': configuration['registered_tenants'],
        'active': configuration['active_tenants'], 'initial_rows': configuration['initial_rows_per_tenant'],
        'duration_ms': configuration['duration_ms'],
        'sample_interval_ms': configuration['backend_memory_budget']['sample_interval_ms'],
        'status': report['status'], 'cell_peak_bytes': final['memory.peak'],
        'cell_events': pressure['cell']['event_delta'], 'epoch_peaks_bytes': epoch_peaks,
        'epoch_events': epoch_events, 'operational_pass': False, 'slo_pass': False,
        'resource_failure': False, 'operational_failure': True, 'operational_timeout': True,
        'failure_kind': 'measurement_worker_shutdown_deadline', 'failure_epochs': ['measurement'],
        'failure_reason': report['failure_reason'], 'parent_worker_stop_timeout_seconds': 20,
        'worker_internal_shutdown_timeout_seconds': 15, 'measurement_shutdown_elapsed_ms': shutdown_elapsed_ms,
        'offered_load_started': True, 'successful': None, 'scheduled': None,
        'offer_throughput_completed_per_second': None,
        'scope': 'Measurement worker shutdown deadline after offer and drain; workload counters and full-population oracles are unavailable; exact shutdown cause is unknown',
    }


def review_rows_from_local(directory, local_review):
    rows = []
    for case, reduced in local_review['cases'].items():
        report = read_json(directory / (case + '.json'))
        row = {name: reduced.get(name) for name in [
            'backend', 'registered', 'active', 'initial_rows', 'duration_ms', 'sample_interval_ms', 'status',
            'operational_pass', 'slo_pass', 'resource_failure', 'operational_timeout', 'failure_kind',
            'failure_epochs', 'successful', 'dropped', 'offer_throughput_completed_per_second',
            'cell_peak_bytes', 'epoch_peaks_bytes', 'cell_events',
            'worst_tenant_p99_upper_exclusive_ms', 'backend_offer_cpu_usec',
            'maximum_measurement_sample_gap_ms', 'parent_worker_stop_timeout_seconds',
            'worker_internal_shutdown_timeout_seconds', 'measurement_shutdown_elapsed_ms', 'scope']}
        row['case'] = case
        row['runtime_origin'] = directory.name
        row['offered_load_started'] = any(sample['event'] == 'offer-start'
                                          for sample in report['backend_memory_pressure']['samples'])
        if reduced['operational_pass']:
            configuration = report['effective_configuration']
            boundary = report['workload']['offer_boundary']['backend_memory_pressure_after']['measurement']
            row['offer_boundary_epoch_peak_bytes'] = boundary['memory.peak']
            row['offer_boundary_current_bytes'] = boundary['memory.current']
            row['post_offer_epoch_peak_increase_bytes'] = reduced['epoch_peaks_bytes']['measurement'] - boundary['memory.peak']
            count = configuration['duration_ms'] * configuration['per_tenant_rate'] // 1000
            initial_rows = configuration['initial_rows_per_tenant']
            distinct_get_rows = len({(operation * 0x9e37 + 1) % initial_rows
                                      for operation in range(count) if operation * 37 % 100 < 40})
            row['offered_distinct_seed_get_rows_per_tenant'] = distinct_get_rows
            row['offered_seed_get_coverage_fraction'] = distinct_get_rows / initial_rows
            row['seed_read_coverage_scope'] = 'Get recipe only; Query reads first sixteen rows; realized Get coverage matches only if no drops'
            row['backend_sampled_rss_max_kib'] = reduced['process_sampling']['backend']['fields']['rss_kb']['observed_max']
            row['driver_sampled_rss_max_kib'] = reduced['process_sampling']['driver']['fields']['rss_kb']['observed_max']
            row['measurement_native_serialized_bytes'] = reduced['native_serialized_bytes']
            row['filesystem_after_measurement'] = reduced['filesystem_snapshots']['after_measurement_worker_shutdown']
            objectives = reduced['per_tenant_objective']
            row['tenants_failing_objective'] = sum(not tenant['passed'] for tenant in objectives)
            row['tenants_failing_latency_by_operation'] = {
                name: sum(tenant['p99'][name]['bucket'] > limit for tenant in objectives)
                for name, limit in capacity_review.LIMIT_BUCKETS.items()}
            health = report['memory_diagnostic']['engine_health_reports'][1]['periodic_pressure']
            row['native_measurement_sample_count'] = len(health['samples'])
            row['mean_serialized_native_frame_bytes'] = health['serialized_sample_bytes'] / len(health['samples']) if health['samples'] else None
        rows.append(row)
    return rows


def review_run(directory, source_commit, controller_key, controller_name):
    provenance = read_json(directory / 'provenance.json')
    assert provenance['source_commit'] == source_commit
    assert provenance[controller_key] == capacity_review.digest(ROOT / controller_name)
    if 'operational_failure_reducer_sha256' in provenance:
        assert provenance['operational_failure_reducer_sha256'] == capacity_review.digest(CAPACITY / 'operational_review.py')
    for script_name, provenance_key in [('oci_capacity.py', 'controller_sha256'), ('capacity_review.py', 'reducer_sha256')]:
        assert capacity_review.digest(CAPACITY / script_name) == provenance[provenance_key]
    if 'continuation_controller_sha256' in provenance:
        assert capacity_review.digest(CAPACITY / 'oci_capacity_continue.py') == provenance['continuation_controller_sha256']
    if 'supplement_controller_sha256' in provenance:
        assert capacity_review.digest(CAPACITY / 'oci_capacity_supplement.py') == provenance['supplement_controller_sha256']
    stopped_report_path = directory / (STOPPED_CASE + '.json')
    stopped_report = read_json(stopped_report_path) if stopped_report_path.exists() else None

    def run_reducer(report, revision):
        if stopped_report is not None and report == stopped_report:
            return reduce_stopped_measurement(report, revision, directory)
        worker_log = directory / (Path(report['effective_configuration']['backend_memory_budget']['cgroup_root']).name + '.log')
        return operational_review.reduce_report(report, revision, worker_log.read_text())

    capacity_review.reduce_report = run_reducer
    if (directory / 'completion.json').exists():
        assert (directory / 'summary.json').exists() and not (directory / 'controller-failure.json').exists()
        frontier.main(directory)
        result = read_json(directory / 'frontier.json')
    else:
        assert source_commit == CANDIDATE and directory == VARIANT
        assert (directory / 'controller-failure.json').exists()
        local_review = capacity_review.review_directory(directory)
        result = {'source_commit': source_commit, 'runtime_script_hashes_verified': True,
                  'cells': review_rows_from_local(directory, local_review),
                  'controller_failure': read_json(directory / 'controller-failure.json'),
                  'scope': 'Stopped-run report reduction; no synthetic completion, workload result, or full-population oracle'}
    detailed = read_json(directory / 'local-review.json')['cases']
    result['observation_controller_sha256_verified'] = True
    for row in result['cells']:
        report = read_json(directory / (row['case'] + '.json'))
        status = read_json(directory / (row['case'] + '-status.json'))
        if row['case'] == STOPPED_CASE and directory == VARIANT:
            assert status['stop_reason'] is None and STOPPED_FAILURE in status['review_error']
            assert row['failure_kind'] == 'measurement_worker_shutdown_deadline'
            assert row['operational_timeout'] and not row['resource_failure']
            assert row['successful'] is None and row['offer_throughput_completed_per_second'] is None
        else:
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
            assert len(samples) <= pressure['sample_limit']
            sample_keys = set(samples[0]) if samples else None
            assert all(type(sample) is dict and set(sample) == sample_keys for sample in samples)
            encoded_bytes = sum(len(json.dumps(sample, ensure_ascii=False, separators=(',', ':')).encode()) for sample in samples)
            assert pressure['serialized_sample_bytes'] == encoded_bytes
            assert encoded_bytes <= pressure['serialized_sample_byte_limit']
            timestamps = [sample['timestamp_unix_ms'] for sample in samples]
            assert timestamps == sorted(timestamps)
        if row['operational_pass']:
            objectives = detailed[row['case']]['per_tenant_objective']
            row['tenants_failing_objective'] = sum(not tenant['passed'] for tenant in objectives)
            row['tenants_failing_latency_by_operation'] = {
                name: sum(tenant['p99'][name]['bucket'] > limit for tenant in objectives)
                for name, limit in capacity_review.LIMIT_BUCKETS.items()}
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
                    old_sample_keys = set(old_samples[0])
                    new_sample_keys = set(new_samples[0])
                    assert old_sample_keys == new_sample_keys
                    assert all(set(sample) == old_sample_keys for sample in old_samples)
                    assert all(set(sample) == new_sample_keys for sample in new_samples)
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
                       'median_backend_offer_cpu_usec': statistics.median(row['backend_offer_cpu_usec'] for row in complete) if complete else None,
                       'median_sampled_backend_rss_max_kib': statistics.median(row['backend_sampled_rss_max_kib'] for row in complete) if complete else None,
                       'median_sampled_driver_rss_max_kib': statistics.median(row['driver_sampled_rss_max_kib'] for row in complete) if complete else None,
                       'median_serialized_measurement_native_bytes': statistics.median(row['measurement_native_serialized_bytes'] for row in complete) if complete else None,
                       'median_measurement_native_samples': statistics.median(row['native_measurement_sample_count'] for row in complete) if complete else None,
                       'median_after_measurement_file_allocated_bytes': statistics.median(row['filesystem_after_measurement']['allocated_file_bytes'] for row in complete) if complete else None,
                       'median_after_measurement_file_logical_bytes': statistics.median(row['filesystem_after_measurement']['logical_file_bytes'] for row in complete) if complete else None})
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
              'operational_timeout_cells': sum(bool(row.get('operational_timeout')) for row in rows),
              'setup_shutdown_deadline_cells': sum(row.get('failure_kind') == 'setup_worker_shutdown_deadline' for row in rows),
              'measurement_shutdown_deadline_cells': sum(row.get('failure_kind') == 'measurement_worker_shutdown_deadline' for row in rows),
              'successful_offered_requests_in_complete_reports': sum(row['successful'] for row in rows if row['successful'] is not None),
              'passing_full_population_oracles': 2 * sum(row['operational_pass'] for row in rows),
              'preserved_unique_databases': len(databases), 'exited_process_identities': len(identities),
              'minimum_sampled_zfs_free_bytes': minimum_free, 'minimum_sampled_host_available_bytes': minimum_available,
              'validation': validation, 'runtime_archive': transfer,
              'scope': 'Instrumented diagnostic worker; new current-host cadence control plus separately dated original large-population cohorts; no engine-only or total-machine ranking'}
    (ROOT / 'final-review.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({key: value for key, value in output.items() if key.endswith('_cells') or key in ['successful_offered_requests_in_complete_reports', 'passing_full_population_oracles', 'preserved_unique_databases', 'exited_process_identities']}, indent=2))


if __name__ == '__main__':
    main()
