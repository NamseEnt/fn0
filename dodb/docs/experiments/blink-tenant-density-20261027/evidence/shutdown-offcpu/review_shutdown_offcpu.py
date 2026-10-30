import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tarfile


SOURCE_COMMIT = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
EXPECTED_RUN_NAME = 'shutdown-offcpu-b22638bdc-r1'
EXPECTED_NATIVE_ARCHIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
EVIDENCE_DIRECTORY = Path(__file__).resolve().parent
REPOSITORY_ROOT = EVIDENCE_DIRECTORY.parents[5]
DEFAULT_RUN_DIRECTORY = EVIDENCE_DIRECTORY / 'runtime' / EXPECTED_RUN_NAME
DEFAULT_COLLECTION = EVIDENCE_DIRECTORY / 'collection' / 'collection.json'
DEFAULT_ARCHIVE = EVIDENCE_DIRECTORY / 'runtime.tar.gz'
DEFAULT_TRANSFER_STATUS = EVIDENCE_DIRECTORY / 'collection' / 'transfer-status.json'
DEFAULT_VALIDATION_SUMMARY = EVIDENCE_DIRECTORY / 'collection' / 'shutdown-validation-summary.json'
DEFAULT_PLANNED_HELPERS = EVIDENCE_DIRECTORY / 'planned-helpers'
CAPACITY_REVIEW_PATH = EVIDENCE_DIRECTORY.parent / 'capacity' / 'capacity_review.py'
TRACE_SOURCE_PATH = REPOSITORY_ROOT / 'dodb/crates/dodb-rocksdb/src/bin/support/shutdown_trace.rs'
HELPER_FILES = {
    'controller': 'oci_shutdown_offcpu.py',
    'observer': 'offcpu_observer.py',
    'collector': 'oci_shutdown_offcpu_collect.py',
    'reviewer': 'review_shutdown_offcpu.py',
}
PROVENANCE_HASH_FIELDS = {
    'controller': 'offcpu_controller_sha256',
    'observer': 'offcpu_observer_sha256',
    'collector': 'offcpu_collector_sha256',
    'reviewer': 'offcpu_reviewer_sha256',
}


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def digest_stream(source):
    value = hashlib.sha256()
    for chunk in iter(lambda: source.read(1024 * 1024), b''):
        value.update(chunk)
    return value.hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def load_capacity_review(path=CAPACITY_REVIEW_PATH):
    spec = importlib.util.spec_from_file_location('shutdown_offcpu_capacity_review', path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def archive_member_digest(archive, member_name):
    try:
        member = archive.getmember(member_name)
    except KeyError as error:
        raise AssertionError(f'archive member missing: {member_name}') from error
    source = archive.extractfile(member)
    if source is None:
        raise AssertionError(f'archive member is not a file: {member_name}')
    with source:
        return digest_stream(source)


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
    wait_samples = []
    status_truncated_count = 0
    status_unsupported_count = 0
    thread_truncated_count = 0
    for record in records:
        if record.get('record_type') != 'thread_snapshot':
            continue
        capture = record.get('capture', {})
        thread_truncated_count += int(capture.get('threads_truncated') is True)
        identity = capture.get('identity', {})
        if capture.get('status') != 'captured' or not identity.get('valid') \
                or not identity.get('valid_at_capture_end'):
            continue
        for thread in capture.get('threads', []):
            tid = thread.get('tid')
            fields = thread.get('fields', {})
            for name, entry in fields.items():
                status = entry.get('status', 'missing')
                counts[name + ':' + status] = counts.get(name + ':' + status, 0) + 1
                if status == 'truncated':
                    status_truncated_count += 1
                elif status == 'unsupported':
                    status_unsupported_count += 1
            wchan = fields.get('wchan', {}).get('value')
            syscall = fields.get('syscall', {}).get('value')
            stack = fields.get('stack', {}).get('value')
            stat = fields.get('stat', {}).get('selected') or {}
            if (wchan and wchan.strip() != '0') or syscall or stack:
                wait_samples.append({
                    'capture_number': record.get('capture_number'),
                    'timestamp_unix_ns': record.get('timestamp_unix_ns'),
                    'since_shutdown_marker_ms': record.get('since_shutdown_marker_ms'),
                    'process_id': identity.get('process_id'),
                    'process_start_time_ticks': identity.get('process_start_time_ticks'),
                    'tid': tid,
                    'tid_start_time_ticks': stat.get('start_time_ticks'),
                    'state': stat.get('state'),
                    'wchan': wchan,
                    'syscall': syscall,
                    'stack': stack,
                })
    return {
        'field_status_counts': counts,
        'truncated_field_count': status_truncated_count,
        'unsupported_field_count': status_unsupported_count,
        'thread_list_truncated_capture_count': thread_truncated_count,
    }, wait_samples


def identity_valid_snapshot(record, expected_source, expected_cgroup_root, worker_identity):
    capture = record.get('capture', {})
    before = capture.get('identity', {})
    after = capture.get('identity_after_capture', {})
    worker_key = (worker_identity['pid'], worker_identity['start_time_ticks'])
    record_key = (
        record.get('worker_identity', {}).get('process_id'),
        record.get('worker_identity', {}).get('process_start_time_ticks'),
    )
    before_key = (before.get('process_id'), before.get('process_start_time_ticks'))
    after_key = (after.get('process_id'), after.get('process_start_time_ticks'))
    expected_root = str(expected_cgroup_root)
    leaf = before.get('leaf_cgroup')
    after_leaf = after.get('leaf_cgroup')
    try:
        leaf_parts = Path(leaf).relative_to(expected_root).parts
        after_leaf_parts = Path(after_leaf).relative_to(expected_root).parts
    except (TypeError, ValueError):
        return False
    return (
        capture.get('status') == 'captured'
        and before.get('valid') is True
        and before.get('valid_at_capture_end') is True
        and after.get('valid') is True
        and record_key == worker_key
        and before_key == worker_key
        and after_key == worker_key
        and record.get('worker_identity', {}).get('source_revision') == expected_source
        and before.get('source_revision') == expected_source
        and after.get('source_revision') == expected_source
        and before.get('cgroup_root') == expected_root
        and after.get('cgroup_root') == expected_root
        and len(leaf_parts) == 1
        and len(after_leaf_parts) == 1
        and leaf_parts == after_leaf_parts
    )


def summarize_wait_evidence(records, valid_snapshot_records):
    grouped = {}
    for record in valid_snapshot_records:
        capture_number = record.get('capture_number')
        elapsed = record.get('since_shutdown_marker_ms')
        for thread in record.get('capture', {}).get('threads', []):
            fields = thread.get('fields', {})
            thread_stat = fields.get('stat', {}).get('selected') or {}
            wchan = fields.get('wchan', {}).get('value')
            stack = fields.get('stack', {}).get('value') or ''
            stack_symbols = re.findall(r'\b([A-Za-z_][A-Za-z0-9_]*)\+', stack)
            if not wchan and not stack_symbols:
                continue
            identity = record.get('worker_identity', {})
            primary_symbol = stack_symbols[0] if stack_symbols else wchan
            key = (
                identity.get('process_id'), identity.get('process_start_time_ticks'),
                thread.get('tid'), thread_stat.get('start_time_ticks'),
                thread_stat.get('state'), wchan, primary_symbol, stack,
            )
            sample = grouped.setdefault(key, {
                'process_id': key[0], 'process_start_time_ticks': key[1],
                'tid': key[2], 'tid_start_time_ticks': key[3], 'state': key[4],
                'wchan': key[5], 'primary_kernel_symbol': key[6],
                'stack_symbols': stack_symbols, 'stack': stack,
                'capture_numbers': [], 'elapsed_ms': [],
            })
            sample['capture_numbers'].append(capture_number)
            sample['elapsed_ms'].append(elapsed)
    samples = []
    for sample in grouped.values():
        elapsed_values = [value for value in sample.pop('elapsed_ms') if value is not None]
        capture_numbers = sample['capture_numbers']
        sample['count'] = len(capture_numbers)
        sample['first_capture_number'] = min(capture_numbers)
        sample['last_capture_number'] = max(capture_numbers)
        sample['first_elapsed_ms'] = min(elapsed_values) if elapsed_values else None
        sample['last_elapsed_ms'] = max(elapsed_values) if elapsed_values else None
        samples.append(sample)
    samples.sort(key=lambda sample: (sample['process_id'], sample['tid'], sample['primary_kernel_symbol']))
    valid_keys = {
        (record.get('worker_identity', {}).get('process_id'),
         record.get('worker_identity', {}).get('process_start_time_ticks'))
        for record in valid_snapshot_records
    }
    invalid_after_exit = [
        record for record in records
        if record.get('record_type') == 'thread_snapshot'
        and record.get('capture', {}).get('identity', {}).get('valid') is False
        and 'FileNotFoundError' in record.get('capture', {}).get('identity', {}).get('reason', '')
        and (record.get('worker_identity', {}).get('process_id'),
             record.get('worker_identity', {}).get('process_start_time_ticks')) in valid_keys
    ]
    valid_durations = [
        record['capture']['read_duration_ns'] for record in valid_snapshot_records
        if type(record.get('capture', {}).get('read_duration_ns')) is int
    ]
    valid_elapsed = [
        record['since_shutdown_marker_ms'] for record in valid_snapshot_records
        if type(record.get('since_shutdown_marker_ms')) is int
    ]
    return {
        'capture_count': sum(record.get('record_type') == 'thread_snapshot' for record in records),
        'identity_valid_capture_count': len(valid_snapshot_records),
        'unavailable_after_exit_capture_count': len(invalid_after_exit),
        'usable_wait_evidence': bool(samples),
        'wait_sample_count': sum(sample['count'] for sample in samples),
        'wait_samples_by_kernel_symbol_tid_and_start_ticks': samples,
        'valid_capture_elapsed_span_ms': [min(valid_elapsed), max(valid_elapsed)] if valid_elapsed else None,
        'capture_read_duration_ns': {
            'count': len(valid_durations),
            'min': min(valid_durations) if valid_durations else None,
            'max': max(valid_durations) if valid_durations else None,
            'sum': sum(valid_durations),
        },
    }


def case_files(case):
    return {
        'report': case + '.json',
        'status': case + '-status.json',
        'log': case + '.log',
        'observer_jsonl': case + '-offcpu.jsonl',
        'observer_status': case + '-offcpu-status.json',
    }


def verify_case_hashes(run_directory, case, collection_case, archive):
    files = case_files(case)
    hashes = {
        'report': collection_case['report_sha256'],
        'log': collection_case['log_sha256'],
        'observer_jsonl': collection_case['observer_jsonl_sha256'],
        'observer_status': collection_case['observer_status_sha256'],
    }
    for name, relative_path in files.items():
        path = Path(run_directory) / relative_path
        assert path.is_file(), f'{name} missing for {case}'
        actual_hash = digest(path)
        if name in hashes:
            assert actual_hash == hashes[name], f'{name} collection hash mismatch for {case}'
        archive_hash = archive_member_digest(archive, f'{EXPECTED_RUN_NAME}/{relative_path}')
        assert archive_hash == actual_hash, f'{name} archive hash mismatch for {case}'
    return hashes


def review_case(run_directory, case, collection_case, expected_source, reducer, archive):
    run_directory = Path(run_directory)
    paths = case_files(case)
    report_path = run_directory / paths['report']
    status_path = run_directory / paths['status']
    log_path = run_directory / paths['log']
    observer_path = run_directory / paths['observer_jsonl']
    observer_status_path = run_directory / paths['observer_status']
    report = read_json(report_path)
    status = read_json(status_path)
    observer_status = read_json(observer_status_path)
    verify_case_hashes(run_directory, case, collection_case, archive)
    trace_events, trace_errors = parse_trace(log_path, expected_source)
    records, jsonl_errors = read_jsonl(observer_path)
    worker_reports = [
        identity for identity in report.get('memory_diagnostic', {}).get('processes', [])
        if identity.get('role') == 'backend-process'
    ]
    source_valid = report.get('memory_diagnostic', {}).get('source_revision') == expected_source
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

    cgroup_root = report.get('effective_configuration', {}).get(
        'backend_memory_budget', {},
    ).get('cgroup_root')
    expected_worker_keys = {
        (worker['pid'], worker['start_time_ticks']) for worker in worker_reports
    }
    snapshot_records = [record for record in records if record.get('record_type') == 'thread_snapshot']
    valid_snapshot_records = [
        record for record in snapshot_records
        if cgroup_root
        and (
            record.get('worker_identity', {}).get('process_id'),
            record.get('worker_identity', {}).get('process_start_time_ticks'),
        ) in expected_worker_keys
        and identity_valid_snapshot(record, expected_source, cgroup_root, {
            'pid': record.get('worker_identity', {}).get('process_id'),
            'start_time_ticks': record.get('worker_identity', {}).get('process_start_time_ticks'),
        })
    ]
    status_summary, wait_samples = field_status_summary(records)
    wait_evidence = summarize_wait_evidence(records, valid_snapshot_records)
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
    expected_worker_keys = {
        (worker['pid'], worker['start_time_ticks']) for worker in worker_reports
    }
    worker_identity_checks = []
    for worker_key in expected_worker_keys:
        marker = marker_records.get(worker_key, {})
        terminal = terminal_records.get(worker_key, {})
        observer_worker = observer_workers.get(worker_key, {})
        marker_valid = marker.get('identity_verification', {}).get('valid') is True
        fast_finish = (
            observer_worker.get('observation_status') == 'finished_before_sampling'
            and observer_worker.get('captures') == 0
            and terminal.get('marker', {}).get('stage') == 'async_server_shutdown_end'
        )
        worker_identity_checks.append(marker_valid or fast_finish)
    all_expected_workers_seen = set(marker_records) == expected_worker_keys
    all_expected_workers_terminated = set(terminal_records) == expected_worker_keys
    observer_valid = (
        source_valid
        and all_expected_workers_seen
        and all_expected_workers_terminated
        and all(worker_identity_checks)
        and len(valid_snapshot_records) == len(snapshot_records)
        and not observer_status.get('observer_failure')
        and not observer_status.get('observer_errors')
        and observer_status.get('observer_error_count', 0) == 0
        and not jsonl_errors
        and not trace_errors
    )
    assert observer_valid == collection_case['observer_valid']
    assert observer_status.get('observer_failure') == collection_case.get('observer_failure')
    assert observer_status.get('observer_errors', []) == collection_case.get('observer_errors', [])

    failure_reason = report.get('failure_reason')
    watchdog_failure = failure_reason == 'diagnostic worker shutdown timed out and was killed'
    partial_counters = report.get('partial_workload_counters') or {}
    workload_counters_unavailable = partial_counters.get('status') == 'unavailable_after_early_abort'
    report_status = report.get('status')
    reduction = None
    reducer_valid = False
    if report_status == 'completed':
        reduction = reducer.reduce_report(report, expected_source)
        reducer_valid = reduction == status.get('reduced')
        assert reducer_valid
        assert status.get('exit_code') == 0
        assert collection_case.get('operational_pass') is True
    else:
        assert watchdog_failure
        assert status.get('exit_code') == 1
        assert status.get('reduced') is None
        assert collection_case.get('operational_pass') is None
        assert workload_counters_unavailable
        assert report.get('compiled_source_revision') is None
        assert report.get('requested_source_revision') is None

    setup_pressure = report.get('backend_memory_pressure', {}).get('epochs', {}).get('setup', {})
    setup_final = setup_pressure.get('final', {})
    setup_events = setup_pressure.get('local_event_delta', {})
    budget = report.get('effective_configuration', {}).get('backend_memory_budget', {})
    pressure_valid = (
        report.get('backend_memory_pressure', {}).get('enabled') is True
        and budget.get('max_bytes') == 268435456
        and budget.get('high_bytes') == 201326592
        and setup_final.get('memory.max') == 268435456
        and setup_final.get('memory.high') == 201326592
        and setup_final.get('memory.swap.max') == 0
        and setup_final.get('memory.swap.current') == 0
        and setup_events.get('high', 0) > 0
        and setup_events.get('oom_kill', 0) == 0
        and setup_final.get('memory.events', {}).get('oom_kill', 0) == 0
    )
    source_valid = source_valid and report.get('memory_diagnostic', {}).get('source_revision') == expected_source
    for record in valid_snapshot_records:
        source_valid = source_valid and record['capture']['identity']['source_revision'] == expected_source

    return {
        'case': case,
        'source_revision_valid': source_valid,
        'report_status': report_status,
        'failure_reason': failure_reason,
        'parent_watchdog_observed': watchdog_failure,
        'operational_pass': status.get('reduced', {}).get('operational_pass') is True,
        'valid_operational_cell': report_status == 'completed' and reducer_valid,
        'valid_observation': observer_valid,
        'observer_valid': observer_valid,
        'observer_failure': observer_status.get('observer_failure'),
        'observer_errors': observer_status.get('observer_errors', []),
        'observer_error_count': observer_status.get('observer_error_count', 0),
        'observer_worker_count': len(observer_status.get('workers', [])),
        'observer_workers': observer_status.get('workers', []),
        'observer_record_count': len(records),
        'observer_jsonl_errors': jsonl_errors,
        'trace_parse_errors': trace_errors,
        'shutdown_trace_timelines': timelines,
        'expected_backend_workers': len(expected_worker_keys),
        'shutdown_markers_seen': len(set(marker_records) & expected_worker_keys),
        'shutdown_terminals_seen': len(set(terminal_records) & expected_worker_keys),
        'proc_field_status_counts': status_summary['field_status_counts'],
        'truncated_field_count': status_summary['truncated_field_count'],
        'unsupported_field_count': status_summary['unsupported_field_count'],
        'thread_list_truncated_capture_count': status_summary['thread_list_truncated_capture_count'],
        'wait_evidence': wait_evidence,
        'sampling_gaps_acknowledged': True,
        'causal_claim': None,
        'interpretation': (
            'Valid snapshots show sampled thread state and kernel wait symbols only. '
            'The post-exit unavailable sample does not invalidate earlier identity-verified captures. '
            'Sampling gaps and read overhead remain; the observations do not prove a unique causal origin '
            'or retroactively explain the prior source-023 measurement failure.'
        ),
        'workload_counters_status': (
            'unavailable_after_early_abort' if workload_counters_unavailable
            else 'validated_complete_report' if reduction is not None else 'unavailable_or_unvalidated'
        ),
        'workload_completed_calls': report.get('workload', {}).get('completed')
            if reduction is not None else None,
        'full_population_oracles': {
            'post_measurement': report.get('post_measurement_full_population_oracle', {}).get('passed') is True,
            'reopened': report.get('reopened_full_population_oracle', {}).get('passed') is True,
        },
        'reducer_valid': reducer_valid,
        'reducer_summary': reduction,
        'setup_pressure_valid': pressure_valid,
        'setup_pressure': {
            'memory_max': setup_final.get('memory.max'),
            'memory_high': setup_final.get('memory.high'),
            'memory_swap_max': setup_final.get('memory.swap.max'),
            'memory_swap_current': setup_final.get('memory.swap.current'),
            'memory_peak': setup_final.get('memory.peak'),
            'memory_events': setup_final.get('memory.events'),
            'local_event_delta': setup_events,
        },
        'partial_workload_counters': partial_counters,
        'original_review_error': status.get('review_error'),
        'database_preserved': collection_case.get('database', {}).get('preserved') is True
            and report.get('data_directory_preserved', True) is True,
        'report_sha256': digest(report_path),
        'report_status_sha256_recorded': status.get('report_sha256'),
        'log_sha256': digest(log_path),
        'observer_jsonl_sha256': digest(observer_path),
        'observer_status_sha256': digest(observer_status_path),
    }


def verify_runtime_metadata(run_directory, collection_path, archive_path,
                            transfer_status_path, validation_summary_path,
                            planned_helpers_directory, collection, transfer, validation):
    run_directory = Path(run_directory)
    provenance = read_json(run_directory / 'provenance.json')
    assert provenance['source_commit'] == SOURCE_COMMIT
    assert provenance['run_name'] == EXPECTED_RUN_NAME
    source_identity = provenance['source_identity']
    assert source_identity['source_clean'] is True
    assert source_identity['source_status_before'] == ''
    assert source_identity['source_commit'] == SOURCE_COMMIT
    assert source_identity['trace_source_sha256'] == digest(TRACE_SOURCE_PATH)
    assert validation['source_commit'] == SOURCE_COMMIT
    assert validation['passing_tests'] == 79
    assert validation['same_17_existing_clippy_diagnostics'] is True
    assert validation['default_link_was_last'] is True
    assert len(validation['outcomes']) == 4
    assert all(outcome['exit_code'] == 0 for outcome in validation['outcomes'])
    assert validation['outcomes'][-1]['name'] == 'shutdown-clean-default-link'
    validation_sha256 = digest(validation_summary_path)
    assert validation_sha256 == provenance['accepted_validation_sha256']
    assert validation_sha256 == source_identity['accepted_validation_sha256']
    assert source_identity['accepted_validation_passing_tests'] == validation['passing_tests']
    assert source_identity['accepted_validation_same_17_existing_clippy_diagnostics'] is True
    assert source_identity['native_archive_sha256_before'] == EXPECTED_NATIVE_ARCHIVE_SHA256
    assert collection['native_archive_sha256_before'] == EXPECTED_NATIVE_ARCHIVE_SHA256
    assert collection['native_archive_sha256_after'] == EXPECTED_NATIVE_ARCHIVE_SHA256
    assert collection['native_archive_sha256_before'] == collection['native_archive_sha256_after']
    assert collection['source_commit'] == SOURCE_COMMIT
    assert collection['source_clean'] is True
    assert collection['planned_collector_sha256'] == provenance['offcpu_collector_sha256']
    assert collection['planned_collector_matches_actual'] is False

    planned_hashes = {}
    actual_hashes = {}
    for helper_name, filename in HELPER_FILES.items():
        planned_path = Path(planned_helpers_directory) / filename
        actual_path = EVIDENCE_DIRECTORY / filename
        planned_hashes[helper_name] = digest(planned_path)
        actual_hashes[helper_name] = digest(actual_path)
        assert planned_hashes[helper_name] == provenance[PROVENANCE_HASH_FIELDS[helper_name]]
    assert actual_hashes['controller'] == provenance['offcpu_controller_sha256']
    assert actual_hashes['observer'] == provenance['offcpu_observer_sha256']
    assert actual_hashes['collector'] == collection['actual_collector_sha256']
    assert planned_hashes['collector'] == collection['planned_collector_sha256']
    assert actual_hashes['reviewer'] != planned_hashes['reviewer']
    actual_hashes['capacity_controller'] = digest(EVIDENCE_DIRECTORY.parent / 'capacity' / 'oci_capacity.py')
    actual_hashes['capacity_reducer'] = digest(CAPACITY_REVIEW_PATH)
    assert actual_hashes['capacity_controller'] == provenance['controller_sha256']
    assert actual_hashes['capacity_reducer'] == provenance['reducer_sha256']

    binary_summary = {}
    for binary_name, identity in provenance['binaries'].items():
        binary_path = run_directory / 'bin' / binary_name
        actual_binary_hash = digest(binary_path)
        collection_identity = collection['binaries'][binary_name]
        assert actual_binary_hash == identity['sha256'] == collection_identity['sha256']
        assert binary_path.stat().st_size == identity['bytes'] == collection_identity['bytes']
        assert collection_identity['matches_provenance'] is True
        binary_summary[binary_name] = {
            'sha256': actual_binary_hash, 'bytes': binary_path.stat().st_size,
            'matches_provenance': True,
        }

    groups = collection['groups']
    assert all(group['group_empty'] for group in groups.values())
    group_validation = collection['cgroup_group_validation']
    assert group_validation['grouping_root_path'] == '.'
    assert group_validation['grouping_root_empty'] is True
    assert group_validation['grouping_root_memory_limits'] == {
        'memory.max': 'max', 'memory.high': 'max', 'memory.swap.max': 'max',
    }
    assert len(group_validation['budgeted_case_and_epoch_groups']) == len(groups) - 1
    assert all(
        group == {
            'memory.max': '268435456',
            'memory.high': '201326592',
            'memory.swap.max': '0',
        }
        for group in group_validation['budgeted_case_and_epoch_groups'].values()
    )
    assert len(collection['processes']) == 12
    assert all(not process['same_live_process'] for process in collection['processes'])
    assert len(groups) == 16
    assert len(collection['databases']) == 3
    assert all(database['preserved'] for database in collection['databases'].values())

    archive_sha256 = digest(archive_path)
    assert archive_sha256 == collection['runtime_archive_sha256']
    assert transfer['scp_completed'] is True
    assert transfer['sha256_matches_collection'] is True
    assert transfer['archive_sha256'] == archive_sha256
    assert transfer['archive_bytes'] == Path(archive_path).stat().st_size
    assert collection['valid_diagnostic_run'] is False

    capacity_review = load_capacity_review()
    collection_by_case = {entry['case']: entry for entry in collection['cases']}
    assert len(collection_by_case) == 3
    with tarfile.open(archive_path, 'r:gz') as archive:
        assert archive_member_digest(archive, f'{EXPECTED_RUN_NAME}/provenance.json') == \
            digest(run_directory / 'provenance.json')
        assert archive_member_digest(archive, f'{EXPECTED_RUN_NAME}/capability-probe.json') == \
            digest(run_directory / 'capability-probe.json') == collection['capability_probe_sha256']
        binary_archive_hashes = {}
        for binary_name, binary in binary_summary.items():
            binary_archive_hashes[binary_name] = archive_member_digest(
                archive, f'{EXPECTED_RUN_NAME}/bin/{binary_name}',
            )
            assert binary_archive_hashes[binary_name] == binary['sha256']

        run_outcome = read_json(run_directory / 'controller-failure.json')
        assert not (run_directory / 'completion.json').exists()
        assert run_outcome['status'] == 'failed_or_invalid'
        assert run_outcome['attempted_cells'] == 3
        assert run_outcome['completed_cells'] == 2
        assert run_outcome['cells'] == 2
        assert collection['attempted_cells'] == 3
        assert collection['completed_cells'] == 2
        assert collection['failure_preserved'] is True
        assert collection['completion_preserved'] is False
        process_identity_summary = collection['processes']
        case_results = []
        for repetition in range(1, 4):
            case = f'offcpu-e3-n5-r{repetition}-blink'
            collection_case = collection_by_case[case]
            case_results.append(review_case(
                run_directory, case, collection_case, SOURCE_COMMIT, capacity_review, archive,
            ))
        for relative_path in (
            'controller-failure.json', 'summary.json',
        ):
            assert archive_member_digest(
                archive, f'{EXPECTED_RUN_NAME}/{relative_path}',
            ) == digest(run_directory / relative_path)
    assert sum(case['valid_operational_cell'] for case in case_results) == 2
    assert sum(case['workload_completed_calls'] or 0 for case in case_results) == 30000
    assert sum(sum(case['full_population_oracles'].values()) for case in case_results) == 4
    assert all(case['database_preserved'] for case in case_results)
    assert all(case['source_revision_valid'] for case in case_results)
    failed_case = case_results[2]
    assert failed_case['parent_watchdog_observed'] is True
    assert failed_case['valid_operational_cell'] is False
    assert failed_case['valid_observation'] is False
    assert failed_case['workload_counters_status'] == 'unavailable_after_early_abort'
    assert failed_case['setup_pressure_valid'] is True
    assert failed_case['wait_evidence']['identity_valid_capture_count'] == 18
    assert failed_case['wait_evidence']['unavailable_after_exit_capture_count'] == 1
    assert failed_case['wait_evidence']['usable_wait_evidence'] is True
    assert failed_case['observer_error_count'] == 1
    assert failed_case['observer_errors'] == [
        'worker_capture_identity_invalid:process_identity_unavailable:FileNotFoundError',
    ]
    assert case_results[0]['observer_valid'] is True and case_results[1]['observer_valid'] is True
    assert failed_case['observer_valid'] is False

    return {
        'provenance': provenance,
        'validation_summary_sha256': validation_sha256,
        'archive': {
            'path': str(archive_path), 'sha256': archive_sha256,
            'bytes': Path(archive_path).stat().st_size,
            'transfer_status': transfer,
        },
        'collection_sha256': digest(collection_path),
        'process_count': len(process_identity_summary),
        'empty_cgroup_count': len(groups),
        'all_cgroups_empty': all(group['group_empty'] for group in groups.values()),
        'grouping_root_limits': group_validation['grouping_root_memory_limits'],
        'budgeted_cgroup_count': len(group_validation['budgeted_case_and_epoch_groups']),
        'cgroup_group_validation': group_validation,
        'process_identities': process_identity_summary,
        'all_processes_exited': all(not process['same_live_process'] for process in process_identity_summary),
        'databases_preserved': all(database['preserved'] for database in collection['databases'].values()),
        'binaries': binary_summary,
        'binary_archive_sha256': binary_archive_hashes,
        'native_archive_sha256_before': collection['native_archive_sha256_before'],
        'native_archive_sha256_after': collection['native_archive_sha256_after'],
        'helper_hashes': {
            'planned': planned_hashes,
            'actual': actual_hashes,
        },
        'accepted_validation': {
            'passing_tests': validation['passing_tests'],
            'same_17_existing_clippy_diagnostics': validation['same_17_existing_clippy_diagnostics'],
            'default_link_was_last': validation['default_link_was_last'],
        },
        'attempted_cells': len(case_results),
        'completed_operational_cells': sum(case['valid_operational_cell'] for case in case_results),
        'completed_calls': sum(case['workload_completed_calls'] or 0 for case in case_results),
        'passed_full_population_oracles': sum(sum(case['full_population_oracles'].values()) for case in case_results),
        'cells': case_results,
        'all_cells_valid': all(
            case['valid_operational_cell'] and case['valid_observation'] for case in case_results
        ),
        'usable_wait_evidence': failed_case['wait_evidence']['usable_wait_evidence'],
        'bounded_stall_not_reproduced': False,
        'capacity_or_performance_ranking': None,
        'actual_reviewer_sha256': actual_hashes['reviewer'],
        'planned_reviewer_sha256': planned_hashes['reviewer'],
    }


def write_idempotent_json(output_path, value):
    output_path = Path(output_path)
    if output_path.exists():
        existing = read_json(output_path)
        if existing != value:
            raise RuntimeError(f'existing derived review differs: {output_path}')
        return 'reused_identical'
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open('x') as output:
        json.dump(value, output, indent=2, sort_keys=True)
        output.write('\n')
    return 'written'


def main():
    parser = argparse.ArgumentParser(description='Verify and review preserved shutdown off-CPU evidence')
    parser.add_argument('--run-directory', type=Path, default=DEFAULT_RUN_DIRECTORY)
    parser.add_argument('--collection', type=Path, default=DEFAULT_COLLECTION)
    parser.add_argument('--archive', type=Path, default=DEFAULT_ARCHIVE)
    parser.add_argument('--transfer-status', type=Path, default=DEFAULT_TRANSFER_STATUS)
    parser.add_argument('--validation-summary', type=Path, default=DEFAULT_VALIDATION_SUMMARY)
    parser.add_argument('--planned-helpers', type=Path, default=DEFAULT_PLANNED_HELPERS)
    parser.add_argument('--output', type=Path)
    arguments = parser.parse_args()
    collection = read_json(arguments.collection)
    transfer = read_json(arguments.transfer_status)
    validation = read_json(arguments.validation_summary)
    summary = verify_runtime_metadata(
        arguments.run_directory, arguments.collection, arguments.archive,
        arguments.transfer_status, arguments.validation_summary,
        arguments.planned_helpers, collection, transfer, validation,
    )
    output_status = write_idempotent_json(arguments.output, summary) if arguments.output else None
    print(json.dumps({
        'run_name': EXPECTED_RUN_NAME,
        'attempted_cells': summary['attempted_cells'],
        'completed_operational_cells': summary['completed_operational_cells'],
        'all_cells_valid': summary['all_cells_valid'],
        'usable_wait_evidence': summary['usable_wait_evidence'],
        'identity_valid_capture_count': summary['cells'][2]['wait_evidence']['identity_valid_capture_count'],
        'unavailable_after_exit_capture_count': summary['cells'][2]['wait_evidence']['unavailable_after_exit_capture_count'],
        'output_status': output_status,
    }), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
