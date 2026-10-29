import argparse
import json
from pathlib import Path
import re
import sys


HELPER_DIRECTORY = Path(__file__).resolve().parent
EVIDENCE_DIRECTORY = HELPER_DIRECTORY.parent
CAPACITY_DIRECTORY = EVIDENCE_DIRECTORY / 'capacity'
REPOSITORY_ROOT = HELPER_DIRECTORY.parents[5]
sys.path.insert(0, str(CAPACITY_DIRECTORY))
import capacity_review


SOURCE_COMMIT = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
TRACE_SOURCE = REPOSITORY_ROOT / 'dodb/crates/dodb-rocksdb/src/bin/support/shutdown_trace.rs'
TRACE_FIELDS = {'timestamp_unix_ns', 'process_id', 'process_start_time_ticks', 'source_revision', 'stage'}
TRACE_STAGES = {
    'command_received_start_measurement', 'command_received_stop_measurement', 'command_received_shutdown',
    'final_sampler_capture_start', 'final_sampler_capture_end', 'final_sampler_capture_error',
    'final_property_capture_start', 'final_property_capture_end',
    'async_server_shutdown_start', 'async_server_shutdown_end', 'async_server_shutdown_error',
    'final_checkpoint_metrics_capture_start', 'final_checkpoint_metrics_capture_end',
    'server_drop_start', 'server_drop_end', 'backend_drop_start', 'backend_drop_end',
    'health_report_write_start', 'health_report_write_end', 'health_report_write_error',
    'async_run_return_success', 'async_run_return_error',
    'runtime_destruction_start', 'runtime_destruction_end',
}
TRACE_INTERVALS = [
    ('final_sampler_capture', 'final_sampler_capture_start',
     {'final_sampler_capture_end', 'final_sampler_capture_error'}),
    ('final_property_capture', 'final_property_capture_start',
     {'final_property_capture_end'}),
    ('async_server_shutdown', 'async_server_shutdown_start',
     {'async_server_shutdown_end', 'async_server_shutdown_error'}),
    ('final_checkpoint_metrics_capture', 'final_checkpoint_metrics_capture_start',
     {'final_checkpoint_metrics_capture_end'}),
    ('server_drop', 'server_drop_start', {'server_drop_end'}),
    ('backend_drop', 'backend_drop_start', {'backend_drop_end'}),
    ('health_report_write', 'health_report_write_start',
     {'health_report_write_end', 'health_report_write_error'}),
    ('runtime_destruction', 'runtime_destruction_start', {'runtime_destruction_end'}),
]
REQUIRED_NORMAL_SHUTDOWN_STAGES = [
    'command_received_shutdown',
    'final_property_capture_start', 'final_property_capture_end',
    'async_server_shutdown_start', 'async_server_shutdown_end',
    'final_checkpoint_metrics_capture_start', 'final_checkpoint_metrics_capture_end',
    'server_drop_start', 'server_drop_end',
    'backend_drop_start', 'backend_drop_end',
    'health_report_write_start', 'health_report_write_end',
    'async_run_return_success', 'runtime_destruction_start', 'runtime_destruction_end',
]
MAX_TRACE_EVENT_BYTES = 4096
MAX_TRACE_EVENTS = 256


def read_json(file_path):
    return json.loads(file_path.read_text())


def write_review_output(output_path, output):
    serialized = json.dumps(output, indent=2) + '\n'
    if output_path.exists():
        assert output_path.read_text() == serialized, 'existing shutdown review differs from the deterministic result'
        return
    with output_path.open('x') as output_file:
        output_file.write(serialized)


def worker_identities(report):
    diagnostic = report['memory_diagnostic']
    processes = {
        (identity['pid'], identity['start_time_ticks']): identity
        for identity in diagnostic['processes']
    }
    lifecycle = diagnostic['worker_lifecycle']
    for event in lifecycle:
        assert event.get('source_revision') == SOURCE_COMMIT
    return processes, lifecycle


def parse_trace_events(log_path, report):
    processes, lifecycle = worker_identities(report)
    events = []
    errors = []
    if not log_path.is_file():
        return events, ['shutdown log is missing']
    previous_timestamps = {}
    with log_path.open() as log_file:
        for line_number, line in enumerate(log_file, start=1):
            if '"timestamp_unix_ns"' not in line:
                continue
            encoded_line = line.encode()
            if len(encoded_line) > MAX_TRACE_EVENT_BYTES:
                errors.append(f'line {line_number}: trace event exceeds byte limit')
                continue
            try:
                event = json.loads(line)
                assert type(event) is dict and set(event) == TRACE_FIELDS
                assert type(event['timestamp_unix_ns']) is int and event['timestamp_unix_ns'] >= 0
                assert type(event['process_id']) is int and event['process_id'] > 0
                assert type(event['process_start_time_ticks']) is int and event['process_start_time_ticks'] >= 0
                assert event['source_revision'] == SOURCE_COMMIT
                assert event['stage'] in TRACE_STAGES
                identity_key = (event['process_id'], event['process_start_time_ticks'])
                assert identity_key in processes, (line_number, identity_key)
                assert any(lifecycle_event['pid'] == event['process_id']
                           for lifecycle_event in lifecycle), (line_number, identity_key)
            except (AssertionError, json.JSONDecodeError, KeyError, TypeError) as error:
                errors.append(f'line {line_number}: {error!r}')
                continue
            previous_timestamp = previous_timestamps.get(identity_key)
            if previous_timestamp is not None:
                if event['timestamp_unix_ns'] < previous_timestamp:
                    errors.append(f'line {line_number}: trace timestamps moved backward for {identity_key}')
            previous_timestamps[identity_key] = event['timestamp_unix_ns']
            events.append(event)
            if len(events) > MAX_TRACE_EVENTS:
                errors.append('trace event count exceeds the bounded parser limit')
                break
    return events, errors


def stage_review(events, measurement_pid=None):
    selected = [event for event in events
                if measurement_pid is None or event['process_id'] == measurement_pid]
    completed = [event for event in selected if event['stage'].endswith('_end')
                 or event['stage'].endswith('_error')
                 or event['stage'] in {'async_run_return_success', 'async_run_return_error'}]
    last_completed = completed[-1] if completed else None
    active_stages = []
    intervals = []
    unmatched_terminal_stages = []
    for stage_name, start_stage, terminal_stages in TRACE_INTERVALS:
        stage_events = [event for event in selected
                        if event['stage'] == start_stage or event['stage'] in terminal_stages]
        pending_start = None
        for event in stage_events:
            if event['stage'] == start_stage:
                assert pending_start is None
                pending_start = event
            elif pending_start is not None:
                elapsed_ns = event['timestamp_unix_ns'] - pending_start['timestamp_unix_ns']
                assert elapsed_ns >= 0
                intervals.append({
                    'stage': stage_name,
                    'start_stage': start_stage,
                    'terminal_stage': event['stage'],
                    'elapsed_ns': elapsed_ns,
                    'status': 'error' if event['stage'].endswith('_error') else 'completed',
                })
                pending_start = None
            else:
                unmatched_terminal_stages.append(event['stage'])
        if pending_start is not None:
            active_stages.append({
                'stage': stage_name,
                'start_stage': start_stage,
                'started_unix_ns': pending_start['timestamp_unix_ns'],
            })
    active_stages.sort(key=lambda item: item['started_unix_ns'])
    return {
        'last_completed_stage': last_completed['stage'] if last_completed else None,
        'last_completed_timestamp_unix_ns': last_completed['timestamp_unix_ns'] if last_completed else None,
        'outstanding_stage': active_stages[-1]['stage'] if active_stages else None,
        'outstanding_stage_start_unix_ns': active_stages[-1]['started_unix_ns'] if active_stages else None,
        'stage_intervals': intervals,
        'unmatched_terminal_stages': unmatched_terminal_stages,
        'trace_event_count': len(selected),
    }


def measurement_worker_pid(report):
    health_reports = report['memory_diagnostic']['engine_health_reports']
    if len(health_reports) > 1:
        return health_reports[1]['worker_pid']
    epochs = {sample.get('epoch') for sample in report.get('backend_memory_pressure', {}).get('samples', [])}
    if report['status'] != 'completed' and 'measurement' not in epochs and 'setup' in epochs:
        return None
    started = [event for event in report['memory_diagnostic']['worker_lifecycle']
               if event['event'] == 'started']
    return started[-1]['pid'] if started else None


def selected_worker(report, measurement_pid):
    if measurement_pid is not None:
        return measurement_pid, 'measurement'
    lifecycle = report['memory_diagnostic']['worker_lifecycle']
    started = [event for event in lifecycle if event['event'] == 'started']
    epochs = {sample.get('epoch') for sample in report.get('backend_memory_pressure', {}).get('samples', [])}
    if report['status'] != 'completed' and 'measurement' not in epochs and 'setup' in epochs:
        return (started[-1]['pid'] if started else None), 'setup'
    return None, None


def parent_watchdog(report):
    events = report.get('backend_memory_pressure', {}).get('samples', [])
    after_drain = [sample for sample in events
                   if sample.get('epoch') == 'measurement' and sample.get('event') == 'after-drain']
    after_exit = [sample for sample in events
                  if sample.get('epoch') == 'measurement' and sample.get('event') == 'after-worker-exit']
    elapsed_ms = None
    if after_drain and after_exit:
        elapsed_ms = after_exit[-1]['snapshot']['timestamp_unix_ms'] - after_drain[-1]['snapshot']['timestamp_unix_ms']
        assert elapsed_ms >= 0
    failure_reason = report.get('failure_reason')
    return {
        'configured_parent_stop_timeout_seconds': 20,
        'worker_internal_shutdown_timeout_seconds': 15,
        'after_drain_to_after_worker_exit_ms': elapsed_ms,
        'parent_worker_shutdown_timeout_observed': failure_reason == 'diagnostic worker shutdown timed out and was killed',
        'failure_reason': failure_reason,
        'causal_interpretation': 'A stage interval locates elapsed time only; it does not establish an internal cause',
    }


def partial_workload_review(report):
    partial = report.get('partial_workload_counters')
    if partial is None:
        return {'status': 'absent', 'validated': False, 'workload': None}
    status = partial['status']
    if status == 'unavailable_after_early_abort':
        assert set(partial) == {'status'}
        return {'status': status, 'validated': False, 'workload': None}
    if status == 'available_unvalidated_after_later_failure':
        assert 'workload' in partial and type(partial['workload']) is dict
        return {'status': status, 'validated': False, 'workload': partial['workload']}
    raise AssertionError({'partial_workload_counters_status': status})


def setup_parent_watchdog_classification(case, source_commit, report, status,
                                         selected_worker_pid, selected_worker_epoch, stages, trace_events):
    if case != 'confirm-e3-n5-r1-blink' or source_commit != SOURCE_COMMIT:
        return None
    diagnostic = report['memory_diagnostic']
    pressure_samples = report['backend_memory_pressure']['samples']
    epochs = {sample['epoch'] for sample in pressure_samples}
    assert report['status'] == 'failed_or_inconclusive'
    assert report['failure_reason'] == 'diagnostic worker shutdown timed out and was killed'
    assert status['exit_code'] == 1
    assert selected_worker_epoch == 'setup' and selected_worker_pid is not None
    assert 'measurement' not in epochs and epochs == {'setup'}
    assert stages['outstanding_stage'] == 'async_server_shutdown'
    worker_events = [event for event in trace_events if event['process_id'] == selected_worker_pid]
    assert [event['stage'] for event in worker_events] == [
        'command_received_shutdown', 'final_property_capture_start', 'final_property_capture_end',
        'async_server_shutdown_start',
    ]
    worker_identity = next(process for process in diagnostic['processes']
                           if process['pid'] == selected_worker_pid)
    assert worker_identity['role'] == 'backend-process'
    pressure_events = [sample['snapshot']['memory.events'] for sample in pressure_samples]
    maximum_counts = {
        key: max(events.get(key, 0) for events in pressure_events)
        for key in ('high', 'max', 'oom', 'oom_kill', 'oom_group_kill')
    }
    assert maximum_counts['high'] > 0
    assert all(maximum_counts[key] == 0 for key in ('max', 'oom', 'oom_kill', 'oom_group_kill'))
    configuration = report['effective_configuration']['backend_memory_budget']
    assert configuration['high_bytes'] == 201326592
    assert configuration['max_bytes'] == 268435456
    assert all(sample['snapshot']['memory.high'] == configuration['high_bytes']
               and sample['snapshot']['memory.max'] == configuration['max_bytes']
               and sample['snapshot']['memory.swap.max'] == 0
               for sample in pressure_samples)
    assert report.get('data_directory_preserved') is True
    assert report.get('partial_workload_counters') == {'status': 'unavailable_after_early_abort'}
    assert 'post_measurement_full_population_oracle' not in report
    assert 'reopened_full_population_oracle' not in report
    return {
        'failure_kind': 'setup_parent_worker_stop_watchdog',
        'operational_timeout': True,
        'kernel_oom': False,
        'resource_failure': False,
        'memory_event_maxima': maximum_counts,
        'configured_setup_memory_limits': {
            'high_bytes': configuration['high_bytes'],
            'max_bytes': configuration['max_bytes'],
            'swap_max_bytes': 0,
        },
    }


def normal_trace_errors(report, events, parse_errors):
    errors = list(parse_errors)
    diagnostic = report['memory_diagnostic']
    backend_identities = {
        (identity['pid'], identity['start_time_ticks'])
        for identity in diagnostic['processes'] if identity['role'] == 'backend-process'
    }
    lifecycle_ids = {event['pid'] for event in diagnostic['worker_lifecycle']}
    if not backend_identities or {pid for pid, _ in backend_identities} - lifecycle_ids:
        errors.append('backend worker process identities do not match lifecycle identities')
    for identity_key in sorted(backend_identities):
        identity_events = [event for event in events
                           if (event['process_id'], event['process_start_time_ticks']) == identity_key]
        stages = [event['stage'] for event in identity_events]
        cursor = -1
        for required_stage in REQUIRED_NORMAL_SHUTDOWN_STAGES:
            try:
                cursor = stages.index(required_stage, cursor + 1)
            except ValueError:
                errors.append(f'{identity_key}: missing or out-of-order {required_stage}')
                break
        if stages and stages[-1] != 'runtime_destruction_end':
            errors.append(f'{identity_key}: final trace stage was {stages[-1]}')
        if not stages:
            errors.append(f'{identity_key}: no shutdown trace events')
    return errors


def review_case(run_directory, status_path, source_commit, binaries, allow_trace_failure=False):
    status = read_json(status_path)
    case = status['case']
    report_path = run_directory / (case + '.json')
    assert capacity_review.digest(report_path) == status['report_sha256']
    report = read_json(report_path)
    assert report['memory_diagnostic']['source_revision'] == source_commit
    lifecycle = report['memory_diagnostic']['worker_lifecycle']
    assert all(event['source_revision'] == source_commit for event in lifecycle)
    assert all(event['harness_binary_sha256'] == binaries['tenant_density_workload']['sha256']
               for event in lifecycle)
    assert all(event['worker_binary_sha256'] == binaries['memory_diagnostic_worker']['sha256']
               for event in lifecycle)
    events, trace_parse_errors = parse_trace_events(run_directory / (case + '.log'), report)
    measurement_pid = measurement_worker_pid(report)
    selected_worker_pid, selected_worker_epoch = selected_worker(report, measurement_pid)
    stages = stage_review(events, selected_worker_pid)
    trace_errors = normal_trace_errors(report, events, trace_parse_errors) if report['status'] == 'completed' else trace_parse_errors
    if report['status'] == 'completed':
        assert 'partial_workload_counters' not in report
        try:
            reduced = capacity_review.reduce_report(report, source_commit)
        except (AssertionError, KeyError, TypeError) as error:
            assert 'review_error' in status
            reduced = {
                'status': report['status'], 'operational_pass': False, 'slo_pass': None,
                'resource_failure': None, 'failure_reason': report.get('failure_reason'),
                'failure_kind': 'existing_reducer_review_required', 'review_error': repr(error),
            }
            workload = {'status': 'present_but_unvalidated_after_reducer_error',
                        'validated': False, 'workload': None}
        else:
            assert status['exit_code'] == 0 and status.get('stop_reason') is None
            assert 'review_error' not in status and reduced['operational_pass']
            reduced = {
                'status': report['status'],
                'operational_pass': True,
                'slo_pass': reduced['slo_pass'],
                'resource_failure': False,
                'failure_reason': None,
                'full_population_oracles': {
                    'post_measurement': report['post_measurement_full_population_oracle']['passed'],
                    'reopened': report['reopened_full_population_oracle']['passed'],
                },
            }
            workload = {'status': 'validated_complete_report', 'validated': True, 'workload': None}
        if trace_errors and not allow_trace_failure:
            raise AssertionError({'case': case, 'normal_shutdown_trace_errors': trace_errors})
    else:
        assert status['exit_code'] != 0 or status.get('stop_reason') is not None or 'review_error' in status
        classification = setup_parent_watchdog_classification(
            case, source_commit, report, status, selected_worker_pid, selected_worker_epoch, stages, events)
        reduced = {
            'status': report['status'],
            'operational_pass': False,
            'slo_pass': None,
            'resource_failure': classification['resource_failure'] if classification else None,
            'failure_reason': report.get('failure_reason'),
            'failure_kind': classification['failure_kind'] if classification else 'unreviewed_operational_failure',
        }
        if classification:
            reduced.update({key: classification[key] for key in ('operational_timeout', 'kernel_oom',
                                                                  'memory_event_maxima',
                                                                  'configured_setup_memory_limits')})
        workload = partial_workload_review(report)
    return {
        'case': case,
        'status_exit_code': status['exit_code'],
        'status_stop_reason': status.get('stop_reason'),
        'status_review_error_preserved': status.get('review_error'),
        'report_status': report['status'],
        'reduction': reduced,
        'partial_workload_counters': workload,
        'measurement_worker_pid': measurement_pid,
        'selected_worker_pid': selected_worker_pid,
        'selected_worker_epoch': selected_worker_epoch,
        'failed_worker_pid': selected_worker_pid if report['status'] != 'completed' else None,
        'failed_worker_epoch': selected_worker_epoch if report['status'] != 'completed' else None,
        'shutdown_stages': stages,
        'normal_shutdown_trace': {
            'passed': not trace_errors if report['status'] == 'completed' else None,
            'errors': trace_errors,
        },
        'parent_watchdog': parent_watchdog(report),
        'structured_trace_events': events,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('run_directory', type=Path)
    parser.add_argument('--postflight', type=Path, required=True)
    parser.add_argument('--validation-summary', type=Path, required=True)
    arguments = parser.parse_args()
    run_directory = arguments.run_directory.resolve(strict=True)
    postflight = read_json(arguments.postflight)
    validation = read_json(arguments.validation_summary)
    assert validation['source_commit'] == SOURCE_COMMIT and validation['passing_tests'] == 79
    assert validation['same_17_existing_clippy_diagnostics']
    assert postflight['source_commit'] == SOURCE_COMMIT and postflight['source_clean']
    assert postflight['validation_sha256'] == capacity_review.digest(arguments.validation_summary)
    assert postflight['collector_sha256'] == capacity_review.digest(HELPER_DIRECTORY / 'oci_shutdown_collect.py')
    assert postflight['native_archive_sha256'] == '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
    provenance = read_json(run_directory / 'provenance.json')
    assert provenance['source_commit'] == SOURCE_COMMIT
    assert provenance['controller_sha256'] == capacity_review.digest(CAPACITY_DIRECTORY / 'oci_capacity.py')
    assert provenance['reducer_sha256'] == capacity_review.digest(CAPACITY_DIRECTORY / 'capacity_review.py')
    assert provenance['shutdown_controller_sha256'] == capacity_review.digest(HELPER_DIRECTORY / 'oci_shutdown_diagnostic.py')
    assert provenance['shutdown_trace_source_sha256'] == capacity_review.digest(TRACE_SOURCE)
    assert provenance['shutdown_validation_sha256'] == capacity_review.digest(arguments.validation_summary)
    postflight_run = postflight['runs'][run_directory.name]
    assert postflight_run['source_commit'] == SOURCE_COMMIT
    assert postflight_run['binaries'] == provenance['binaries']
    assert not any(process['same_live_process'] for process in postflight_run['processes'])
    assert all(database['preserved'] for database in postflight_run['databases'].values())
    assert all(not pids for group in postflight_run['cgroup_processes'].values() for pids in group.values())
    transfer_receipt = read_json(HELPER_DIRECTORY / 'collection/runtime-transfer.json')
    runtime_archive = HELPER_DIRECTORY / 'runtime.tar.gz'
    assert transfer_receipt['archive_sha256'] == capacity_review.digest(runtime_archive)
    assert transfer_receipt['bytes'] == runtime_archive.stat().st_size
    assert run_directory.name in transfer_receipt['runs']
    summary = read_json(run_directory / 'summary.json')
    status_paths_by_case = {
        read_json(status_path)['case']: status_path
        for status_path in run_directory.glob('*-status.json')
    }
    case_order = [outcome['case'] for outcome in summary]
    assert len(case_order) == len(set(case_order)) == len(status_paths_by_case)
    status_paths = [status_paths_by_case[case] for case in case_order]
    expected_case_order = [
        'tiny-e2-n1-ms1000-blink', 'tiny-e2-n1-ms1000-shared',
        'confirm-e3-n5-r1-shared', 'confirm-e3-n5-r1-blink',
        'confirm-e3-n5-r2-blink', 'confirm-e3-n5-r2-shared',
        'confirm-e3-n5-r3-shared', 'confirm-e3-n5-r3-blink',
    ]
    assert case_order == expected_case_order[:len(case_order)]
    assert len(status_paths) == postflight_run['status_count']
    completion_path = run_directory / 'completion.json'
    failure_path = run_directory / 'controller-failure.json'
    assert completion_path.exists() != failure_path.exists()
    completed = completion_path.exists()
    failure = read_json(failure_path) if not completed else None
    trace_failure_case = None
    trace_failure_pair = None
    if failure and failure.get('failure_kind') == 'shutdown_trace_validation':
        trace_failure = failure['trace_validation']
        trace_failure_case = trace_failure['failing_case']
        assert trace_failure_case in trace_failure['cases']
        trace_failure_pair = trace_failure['pair']
    cells = [
        review_case(run_directory, status_path, SOURCE_COMMIT, provenance['binaries'],
                    allow_trace_failure=read_json(status_path)['case'] == trace_failure_case)
        for status_path in status_paths
    ]
    pair_reviews = []
    for pair_path in sorted(run_directory.glob('*-pair.json')):
        pair = read_json(pair_path)
        reports = [read_json(run_directory / (case + '.json')) for case in pair['cases']]
        comparison = capacity_review.compare_pair(*reports)
        assert pair['comparison'] == comparison
        pair_reviews.append({'pair': pair_path.stem, 'realized_comparison_available': comparison['realized_comparison_available']})
    if completed:
        completion = read_json(completion_path)
        assert completion['completed'] and completion['source_commit'] == SOURCE_COMMIT
        assert completion['cases'] == len(cells) == 8
        assert all(cell['reduction']['operational_pass'] for cell in cells)
        assert len(pair_reviews) == 4 and all(pair['realized_comparison_available'] for pair in pair_reviews)
    else:
        assert failure is not None
        assert failure['completed_cells'] == len(cells)
        if failure.get('failure_kind') == 'shutdown_trace_validation':
            assert failure['last_operational_pass']
            assert trace_failure_pair == 'tiny-e2-n1-ms1000'
            assert len(cells) == 2
            assert {cell['case'] for cell in cells} == set(failure['trace_validation']['cases'])
            assert all(cell['reduction']['operational_pass'] for cell in cells)
            failed_trace_cell = next(cell for cell in cells
                                     if cell['case'] == failure['trace_validation']['failing_case'])
            assert not failed_trace_cell['normal_shutdown_trace']['passed']
            assert not any(cell['case'].startswith('confirm-e3-') for cell in cells)
        else:
            assert cells[-1]['reduction']['operational_pass'] is False
    output_path = run_directory / 'shutdown-review.json'
    output = {
        'source_commit': SOURCE_COMMIT,
        'run_name': run_directory.name,
        'completed_sequence': completed,
        'attempted_cells': len(cells),
        'completed_cells': sum(cell['reduction']['operational_pass'] for cell in cells),
        'pairs': pair_reviews,
        'stopped_case': None if completed or trace_failure_pair else cells[-1]['case'],
        'stopped_after_pair': trace_failure_pair,
        'cells': cells,
        'completion': read_json(completion_path) if completed else None,
        'controller_failure': None if completed else read_json(failure_path),
        'scope': 'Shutdown trace intervals and preserved run classification; timings do not prove causality or establish a performance ranking',
    }
    write_review_output(output_path, output)
    failure_cell = None
    if not completed:
        failure_case = failure.get('trace_validation', {}).get('failing_case')
        failure_cell = next((cell for cell in cells if cell['case'] == failure_case), cells[-1])
    summary = {
        'source_commit': SOURCE_COMMIT,
        'run_name': run_directory.name,
        'completed_sequence': completed,
        'attempted_cells': len(cells),
        'completed_cells': output['completed_cells'],
        'stopped_case': output['stopped_case'],
        'stopped_after_pair': trace_failure_pair,
        'failure_kind': failure_cell['reduction'].get('failure_kind') if failure_cell else None,
        'shutdown_stage': failure_cell['shutdown_stages'] if failure_cell else None,
        'parent_watchdog': failure_cell['parent_watchdog'] if failure_cell and not trace_failure_pair else None,
    }
    print(json.dumps(summary, indent=2), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
