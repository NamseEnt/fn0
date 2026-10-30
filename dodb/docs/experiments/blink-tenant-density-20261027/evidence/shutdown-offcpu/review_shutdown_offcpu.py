import argparse
import hashlib
import json
from pathlib import Path
import re
import sys


SOURCE_COMMIT = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
EXPECTED_RUN_NAME = 'shutdown-offcpu-b22638bdc-r1'


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def parse_trace(path, expected_source):
    events = []
    errors = []
    with Path(path).open('r', errors='replace') as source:
        for line_number, line in enumerate(source, start=1):
            if '"timestamp_unix_ns"' not in line or '"process_start_time_ticks"' not in line:
                continue
            if len(line.encode('utf-8')) > 4096:
                errors.append({'line': line_number, 'error': 'trace_line_byte_limit'})
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError as error:
                errors.append({'line': line_number, 'error': f'invalid_json:{error.msg}'})
                continue
            expected_fields = {
                'timestamp_unix_ns', 'process_id', 'process_start_time_ticks',
                'source_revision', 'stage',
            }
            if type(event) is not dict or set(event) != expected_fields:
                errors.append({'line': line_number, 'error': 'trace_schema_mismatch'})
                continue
            if event['source_revision'] != expected_source:
                errors.append({'line': line_number, 'error': 'trace_source_mismatch'})
                continue
            events.append(event)
    return events, errors


def read_jsonl(path):
    records = []
    errors = []
    with Path(path).open('r', errors='replace') as source:
        for line_number, line in enumerate(source, start=1):
            if len(line.encode('utf-8')) > 256 * 1024:
                errors.append({'line': line_number, 'error': 'jsonl_line_byte_limit'})
                continue
            try:
                records.append(json.loads(line))
            except json.JSONDecodeError as error:
                errors.append({'line': line_number, 'error': f'invalid_json:{error.msg}'})
    return records, errors


def trace_timeline(events, worker_identity):
    identity_key = (worker_identity['process_id'], worker_identity['process_start_time_ticks'])
    worker_events = [
        event for event in events
        if (event['process_id'], event['process_start_time_ticks']) == identity_key
    ]
    worker_events.sort(key=lambda event: event['timestamp_unix_ns'])
    shutdown_start = next(
        (event for event in worker_events if event['stage'] == 'async_server_shutdown_start'),
        None,
    )
    timeline = []
    for event in worker_events:
        elapsed_ms = None
        if shutdown_start is not None:
            elapsed_ms = (event['timestamp_unix_ns'] - shutdown_start['timestamp_unix_ns']) / 1_000_000
        timeline.append({'stage': event['stage'], 'elapsed_from_async_shutdown_start_ms': elapsed_ms})
    return worker_events, shutdown_start, timeline


def field_status_summary(records):
    counts = {}
    useful_waits = []
    for record in records:
        if record.get('record_type') != 'thread_snapshot':
            continue
        capture = record.get('capture', {})
        for thread in capture.get('threads', []):
            tid = thread.get('tid')
            fields = thread.get('fields', {})
            for name, entry in fields.items():
                status = entry.get('status', 'missing')
                counts[name + ':' + status] = counts.get(name + ':' + status, 0) + 1
            wchan = fields.get('wchan', {}).get('value')
            syscall = fields.get('syscall', {}).get('value')
            stack = fields.get('stack', {}).get('value')
            thread_stat = fields.get('stat', {}).get('selected') or {}
            if wchan and wchan.strip() != '0' or syscall or stack:
                useful_waits.append({
                    'capture_number': record.get('capture_number'),
                    'timestamp_unix_ns': record.get('timestamp_unix_ns'),
                    'since_shutdown_marker_ms': record.get('since_shutdown_marker_ms'),
                    'tid': tid,
                    'state': thread_stat.get('state'),
                    'wchan': wchan,
                    'syscall': syscall,
                    'stack': stack,
                })
    return counts, useful_waits


def review_case(run_directory, case, expected_source=SOURCE_COMMIT):
    run_directory = Path(run_directory)
    report_path = run_directory / (case + '.json')
    status_path = run_directory / (case + '-status.json')
    log_path = run_directory / (case + '.log')
    observer_path = run_directory / (case + '-offcpu.jsonl')
    observer_status_path = run_directory / (case + '-offcpu-status.json')
    report = read_json(report_path)
    status = read_json(status_path)
    observer_status = read_json(observer_status_path)
    trace_events, trace_errors = parse_trace(log_path, expected_source)
    records, jsonl_errors = read_jsonl(observer_path)
    worker_reports = [
        identity for identity in report.get('memory_diagnostic', {}).get('processes', [])
        if identity.get('role') == 'backend-process'
    ]
    source_valid = report.get('memory_diagnostic', {}).get('source_revision') == expected_source
    observed_identities = {
        (record.get('worker_identity', {}).get('process_id'),
         record.get('worker_identity', {}).get('process_start_time_ticks'))
        for record in records
        if record.get('record_type') == 'shutdown_marker'
    }
    timelines = []
    for worker_identity in worker_reports:
        worker_events, shutdown_start, timeline = trace_timeline(trace_events, {
            'process_id': worker_identity['pid'],
            'process_start_time_ticks': worker_identity['start_time_ticks'],
        })
        timelines.append({
            'process_id': worker_identity['pid'],
            'process_start_time_ticks': worker_identity['start_time_ticks'],
            'shutdown_start_observed': shutdown_start is not None,
            'stages': timeline,
            'trace_event_count': len(worker_events),
        })
    field_statuses, wait_samples = field_status_summary(records)
    is_completed = report.get('status') == 'completed'
    oracles = {
        'post_measurement': report.get('post_measurement_full_population_oracle', {}).get('passed') is True,
        'reopened': report.get('reopened_full_population_oracle', {}).get('passed') is True,
    }
    validated = is_completed and all(oracles.values()) and status.get('exit_code') == 0
    if validated:
        workload_status = 'validated_complete_report'
        workload = report.get('workload')
        throughput = {
            'status': 'present_in_validated_report',
            'offered_requests_per_second': workload.get('offered_requests_per_second'),
        }
    else:
        partial = report.get('partial_workload_counters')
        if partial and partial.get('status') == 'available_unvalidated_after_later_failure':
            workload_status = 'available_but_unvalidated'
        else:
            workload_status = 'unavailable_or_unvalidated'
        workload = None
        throughput = {
            'status': 'unavailable',
            'value': None,
            'interpretation': 'A failed or oracle-free report is not a zero-throughput result',
        }
    marker_records = {
        (record.get('marker', {}).get('process_id'),
         record.get('marker', {}).get('process_start_time_ticks')): record
        for record in records if record.get('record_type') == 'shutdown_marker'
    }
    terminal_records = {
        (record.get('marker', {}).get('process_id'),
         record.get('marker', {}).get('process_start_time_ticks')): record
        for record in records if record.get('record_type') == 'shutdown_terminal_marker'
    }
    observer_workers = {
        (worker.get('process_id'), worker.get('process_start_time_ticks')): worker
        for worker in observer_status.get('workers', [])
    }
    worker_identity_checks = []
    expected_worker_keys = {
        (worker['pid'], worker['start_time_ticks']) for worker in worker_reports
    }
    for worker_key in expected_worker_keys:
        marker = marker_records.get(worker_key, {})
        terminal = terminal_records.get(worker_key, {})
        observer_worker = observer_workers.get(worker_key, {})
        fast_finish = (
            observer_worker.get('observation_status') == 'finished_before_sampling'
            and observer_worker.get('captures') == 0
            and terminal.get('marker', {}).get('stage') == 'async_server_shutdown_end'
        )
        marker_valid = marker.get('identity_verification', {}).get('valid') is True
        worker_identity_checks.append(marker_valid or fast_finish)
    capture_records_valid = all(
        record.get('capture', {}).get('status') == 'captured'
        and record.get('capture', {}).get('identity', {}).get('valid') is True
        and record.get('capture', {}).get('identity', {}).get('valid_at_capture_end') is True
        for record in records if record.get('record_type') == 'thread_snapshot'
    )
    observer_valid = (
        source_valid
        and set(marker_records) == expected_worker_keys
        and set(terminal_records) == expected_worker_keys
        and all(worker_identity_checks)
        and capture_records_valid
        and not observer_status.get('observer_failure')
        and not observer_status.get('observer_errors')
        and not jsonl_errors
        and not trace_errors
    )
    shutdown_durations = [
        item['since_shutdown_marker_ms'] for item in wait_samples
        if item.get('since_shutdown_marker_ms') is not None
    ]
    return {
        'case': case,
        'source_commit': expected_source,
        'report_source_valid': source_valid,
        'report_status': report.get('status'),
        'failure_reason': report.get('failure_reason'),
        'operational_pass': status.get('reduced', {}).get('operational_pass') is True,
        'valid_operational_cell': validated,
        'valid_observation': observer_valid,
        'full_population_oracles': oracles,
        'workload_counters_status': workload_status,
        'workload_counters': workload,
        'throughput': throughput,
        'parent_watchdog_observed': report.get('failure_reason') ==
            'diagnostic worker shutdown timed out and was killed',
        'shutdown_trace_timelines': timelines,
        'observer_workers': observer_status.get('workers', []),
        'observer_failure': observer_status.get('observer_failure'),
        'observer_errors': observer_status.get('observer_errors', []),
        'observer_error_count': observer_status.get('observer_error_count', 0),
        'observer_errors_truncated': observer_status.get('observer_errors_truncated', False),
        'observer_worker_identity_checks': worker_identity_checks,
        'observer_record_count': len(records),
        'observer_jsonl_errors': jsonl_errors,
        'trace_parse_errors': trace_errors,
        'proc_field_status_counts': field_statuses,
        'observed_kernel_wait_samples': wait_samples,
        'wait_sample_time_span_ms': [min(shutdown_durations), max(shutdown_durations)]
            if shutdown_durations else None,
        'sampling_gaps_acknowledged': True,
        'causal_claim': None,
        'interpretation': (
            'Recorded task state and kernel wait fields are observations at sample times; '
            'they do not identify the allocation or I/O cause.'
        ),
    }


def review_run(run_directory, output_path=None):
    run_directory = Path(run_directory)
    provenance = read_json(run_directory / 'provenance.json')
    assert provenance['source_commit'] == SOURCE_COMMIT
    assert provenance['run_name'] == EXPECTED_RUN_NAME
    probe = read_json(run_directory / 'capability-probe.json')
    assert probe['probe_child_terminated'] is True
    assert probe['wait_fields_available'] is True
    cases = []
    for status_path in sorted(run_directory.glob('offcpu-e3-n5-r*-blink-status.json')):
        case = read_json(status_path)['case']
        cases.append(review_case(run_directory, case))
    completion_path = run_directory / 'completion.json'
    failure_path = run_directory / 'controller-failure.json'
    assert completion_path.exists() != failure_path.exists()
    if completion_path.exists():
        run_outcome = read_json(completion_path)
        assert run_outcome['attempted_cells'] == len(cases)
        assert run_outcome['completed_cells'] == sum(case['operational_pass'] for case in cases)
    else:
        run_outcome = read_json(failure_path)
        assert run_outcome['attempted_cells'] == len(cases)
        assert run_outcome['completed_cells'] == sum(case['operational_pass'] for case in cases)
    all_valid = len(cases) == 3 and all(
        case['valid_operational_cell'] and case['valid_observation'] for case in cases
    )
    summary = {
        'source_commit': SOURCE_COMMIT,
        'run_name': EXPECTED_RUN_NAME,
        'complete': completion_path.exists(),
        'failed_or_stopped': failure_path.exists(),
        'cell_count': len(cases),
        'attempted_cells': len(cases),
        'completed_cells': sum(case['operational_pass'] for case in cases),
        'all_cells_valid': all_valid,
        'bounded_stall_not_reproduced': all_valid,
        'capacity_or_performance_ranking': None,
        'probe_wait_fields': probe['useful_wait_fields'],
        'cases': cases,
        'reviewer_sha256': digest(Path(__file__)),
    }
    if output_path is not None:
        with Path(output_path).open('x') as output:
            json.dump(summary, output, indent=2)
            output.write('\n')
    return summary


def main():
    parser = argparse.ArgumentParser(description='Review bounded shutdown wait observation evidence')
    parser.add_argument('--run-directory', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    arguments = parser.parse_args()
    summary = review_run(arguments.run_directory, arguments.output)
    print(json.dumps({
        'run_name': summary['run_name'], 'cell_count': summary['cell_count'],
        'all_cells_valid': summary['all_cells_valid'],
        'bounded_stall_not_reproduced': summary['bounded_stall_not_reproduced'],
    }), flush=True)
    return 0 if summary['cell_count'] > 0 else 2


if __name__ == '__main__':
    raise SystemExit(main())
