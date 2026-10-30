import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time


POLL_INTERVAL_SECONDS = 0.1
CAPTURE_INTERVAL_SECONDS = 1.0
SHUTDOWN_PENDING_SECONDS = 2.0
MAX_CAPTURES_PER_WORKER = 24
MAX_WORKERS = 16
MAX_THREADS_PER_WORKER = 32
MAX_FILE_BYTES = 1024
MAX_LINE_BYTES = 256 * 1024
MAX_WORKER_OUTPUT_BYTES = 2 * 1024 * 1024
MAX_TOTAL_OUTPUT_BYTES = 16 * 1024 * 1024
MAX_WORKER_CAPTURED_FILE_BYTES = 4 * 1024 * 1024
MAX_ERRORS_PER_WORKER = 128
MAX_TRACE_LINE_BYTES = 4096
MAX_OBSERVER_WALL_SECONDS = 1230
MAX_OBSERVER_ERRORS = 256
SOURCE_REVISION_LIMIT = 128
TRACE_FIELDS = {
    'timestamp_unix_ns', 'process_id', 'process_start_time_ticks',
    'source_revision', 'stage',
}
SHUTDOWN_START_STAGE = 'async_server_shutdown_start'
SHUTDOWN_TERMINAL_STAGES = {
    'async_server_shutdown_end', 'async_server_shutdown_error',
}
WAIT_FIELDS = ('wchan', 'syscall', 'stack')
THREAD_FIELDS = ('stat', 'status', 'wchan', 'syscall', 'stack', 'schedstat')
PROCESS_STATUS_FIELDS = {
    'State', 'VmRSS', 'VmHWM', 'Threads', 'voluntary_ctxt_switches',
    'nonvoluntary_ctxt_switches',
}
CGROUP_FIELDS = ('memory.current', 'memory.events', 'cpu.stat', 'io.stat')


def process_start_time_ticks(stat_text):
    command_end = stat_text.rfind(')')
    if command_end < 0:
        raise ValueError('process stat omitted command delimiter')
    fields = stat_text[command_end + 1:].split()
    if len(fields) <= 19:
        raise ValueError('process stat omitted start-time field')
    return int(fields[19])


def parse_trace_line(line, expected_source):
    if len(line.encode('utf-8')) > MAX_TRACE_LINE_BYTES:
        raise ValueError('trace line exceeds byte limit')
    try:
        event = json.loads(line)
    except (json.JSONDecodeError, UnicodeError):
        return None
    if type(event) is not dict or 'stage' not in event:
        return None
    if set(event) != TRACE_FIELDS:
        raise ValueError('trace event fields do not match the frozen schema')
    if event['source_revision'] != expected_source:
        raise ValueError('trace event source revision mismatch')
    if type(event['timestamp_unix_ns']) is not int or event['timestamp_unix_ns'] < 0:
        raise ValueError('trace event timestamp is invalid')
    if type(event['process_id']) is not int or event['process_id'] <= 0:
        raise ValueError('trace event process id is invalid')
    if type(event['process_start_time_ticks']) is not int or event['process_start_time_ticks'] < 0:
        raise ValueError('trace event process start time is invalid')
    if type(event['stage']) is not str or not event['stage'] or len(event['stage']) > 96:
        raise ValueError('trace event stage is invalid')
    return event


class CaptureBudget:
    def __init__(self):
        self.file_bytes = 0
        self.errors = 0
        self.errors_truncated = False

    def add_error(self, field_name, error):
        self.errors += 1
        if self.errors > MAX_ERRORS_PER_WORKER:
            self.errors_truncated = True
            return None
        return {'field': field_name, 'error': str(error)[:192]}


class BoundedErrors:
    def __init__(self):
        self.values = []
        self.count = 0
        self.truncated = False

    def append(self, value):
        self.count += 1
        if len(self.values) < MAX_OBSERVER_ERRORS:
            self.values.append(str(value)[:192])
        else:
            self.truncated = True

    def __getitem__(self, item):
        return self.values[item]


def read_field(path, field_name, budget, max_bytes=MAX_FILE_BYTES):
    started = time.monotonic_ns()
    remaining_bytes = MAX_WORKER_CAPTURED_FILE_BYTES - budget.file_bytes
    allowed_bytes = min(max_bytes, max(remaining_bytes, 0))
    if allowed_bytes == 0:
        return {
            'status': 'limit', 'value': None, 'bytes_read': 0,
            'truncated': True, 'read_duration_ns': time.monotonic_ns() - started,
        }
    try:
        with path.open('rb') as source:
            content = source.read(allowed_bytes + 1)
        truncated = len(content) > allowed_bytes
        content = content[:allowed_bytes]
        budget.file_bytes += len(content)
        value = content.decode('utf-8', errors='replace')
        return {
            'status': 'truncated' if truncated else 'read',
            'value': value.rstrip('\n'),
            'bytes_read': len(content),
            'truncated': truncated,
            'read_duration_ns': time.monotonic_ns() - started,
        }
    except FileNotFoundError:
        status = 'unsupported'
        error = 'not_found'
    except PermissionError as exception:
        status = 'denied'
        error = f'errno_{exception.errno}'
        budget_error = budget.add_error(field_name, exception)
        return {
            'status': status, 'value': None, 'bytes_read': 0,
            'truncated': False, 'read_duration_ns': time.monotonic_ns() - started,
            'error': budget_error,
        }
    except OSError as exception:
        status = 'error'
        error = f'errno_{exception.errno}'
        budget_error = budget.add_error(field_name, exception)
        return {
            'status': status, 'value': None, 'bytes_read': 0,
            'truncated': False, 'read_duration_ns': time.monotonic_ns() - started,
            'error': budget_error,
        }
    budget.add_error(field_name, error)
    return {
        'status': status, 'value': None, 'bytes_read': 0,
        'truncated': False, 'read_duration_ns': time.monotonic_ns() - started,
    }


def selected_status_fields(value):
    selected = {}
    for line in value.splitlines():
        name, separator, field_value = line.partition(':')
        if separator and name in PROCESS_STATUS_FIELDS:
            selected[name] = field_value.strip()[:96]
    return selected


def selected_stat_fields(value):
    command_start = value.find('(')
    command_end = value.rfind(')')
    if command_start < 0 or command_end <= command_start:
        raise ValueError('task stat omitted command delimiters')
    fields = value[command_end + 1:].split()
    if len(fields) <= 19:
        raise ValueError('task stat omitted required fields')
    return {
        'command': value[command_start + 1:command_end][:128],
        'state': fields[0],
        'user_ticks': int(fields[11]),
        'system_ticks': int(fields[12]),
        'start_time_ticks': int(fields[19]),
    }


def selected_schedstat_fields(value):
    fields = value.split()
    if len(fields) < 3:
        raise ValueError('schedstat omitted required counters')
    return {
        'runtime_ns': int(fields[0]),
        'runqueue_wait_ns': int(fields[1]),
        'timeslices': int(fields[2]),
    }


def selected_cgroup_fields(path, budget):
    path = Path(path)
    snapshots = {}
    for field_name in CGROUP_FIELDS:
        field = read_field(path / field_name, field_name, budget, max_bytes=4096)
        value = field.pop('value')
        if field['status'] in ('read', 'truncated'):
            if field_name == 'memory.events':
                value = dict(
                    line.split(maxsplit=1) for line in value.splitlines()
                    if len(line.split(maxsplit=1)) == 2
                )
            elif field_name == 'cpu.stat':
                value = dict(
                    line.split(maxsplit=1) for line in value.splitlines()
                    if len(line.split(maxsplit=1)) == 2
                )
            elif field_name == 'memory.current':
                try:
                    value = int(value)
                except ValueError:
                    field['status'] = 'invalid'
                    budget.add_error(field_name, 'non-integer memory.current')
                    value = None
        field['value'] = value
        snapshots[field_name] = field
    return snapshots


def cgroup_leaf_for_worker(pid, proc_root, cgroup_root, cgroup_mount):
    try:
        membership_lines = (proc_root / str(pid) / 'cgroup').read_text().splitlines()
    except OSError as error:
        return None, f'process_cgroup_unavailable:{type(error).__name__}'
    unified = [line[3:] for line in membership_lines if line.startswith('0::/')]
    if len(unified) != 1:
        return None, 'process_unified_cgroup_membership_invalid'
    leaf = Path(cgroup_mount) / unified[0].lstrip('/')
    try:
        relative_leaf = leaf.relative_to(Path(cgroup_root))
    except ValueError:
        return None, 'process_not_in_owned_case_cgroup'
    if len(relative_leaf.parts) != 1 or relative_leaf == Path('.'):
        return None, 'process_cgroup_leaf_not_direct_owned_child'
    try:
        process_ids = (leaf / 'cgroup.procs').read_text().split()
    except OSError as error:
        return None, f'leaf_cgroup_procs_unavailable:{type(error).__name__}'
    if str(pid) not in process_ids:
        return None, 'process_not_in_reported_leaf_cgroup'
    return leaf, None


def verify_worker_identity(pid, start_time_ticks, source_revision, proc_root, cgroup_root,
                           cgroup_mount=Path('/sys/fs/cgroup')):
    stat_path = proc_root / str(pid) / 'stat'
    try:
        current_start_time = process_start_time_ticks(stat_path.read_text())
    except (OSError, ValueError) as error:
        return {'valid': False, 'reason': f'process_identity_unavailable:{type(error).__name__}'}
    if current_start_time != start_time_ticks:
        return {'valid': False, 'reason': 'process_start_time_mismatch'}
    leaf, cgroup_error = cgroup_leaf_for_worker(
        pid, proc_root, cgroup_root, cgroup_mount,
    )
    if cgroup_error:
        return {'valid': False, 'reason': cgroup_error}
    return {
        'valid': True,
        'process_id': pid,
        'process_start_time_ticks': start_time_ticks,
        'source_revision': source_revision,
        'cgroup_root': str(cgroup_root),
        'leaf_cgroup': str(leaf),
    }


def capture_snapshot(pid, start_time_ticks, source_revision, proc_root, cgroup_root,
                     cgroup_mount=Path('/sys/fs/cgroup')):
    started = time.monotonic_ns()
    identity = verify_worker_identity(
        pid, start_time_ticks, source_revision, proc_root, cgroup_root, cgroup_mount,
    )
    if not identity['valid']:
        return {
            'status': 'identity_invalid', 'identity': identity,
            'capture_started_monotonic_ns': started,
            'read_duration_ns': time.monotonic_ns() - started,
            'threads': [], 'threads_truncated': False,
            'process': None, 'cgroup': {}, 'errors': [],
            'error_count': 1, 'errors_truncated': False,
        }

    budget = CaptureBudget()
    process_directory = proc_root / str(pid)
    process_stat = read_field(process_directory / 'stat', 'process.stat', budget)
    process_status = read_field(process_directory / 'status', 'process.status', budget)
    process_stat_selected = None
    if process_stat['value'] is not None:
        try:
            process_stat_selected = selected_stat_fields(process_stat['value'])
        except (ValueError, IndexError) as error:
            process_stat['status'] = 'invalid'
            budget.add_error('process.stat', error)
    process_stat['value'] = None
    process = {
        'stat': process_stat,
        'selected_stat': process_stat_selected,
        'status': {key: value for key, value in process_status.items() if key != 'value'},
        'selected_status': selected_status_fields(process_status['value'])
        if process_status['value'] is not None else None,
    }
    process_status['value'] = None
    task_directory = process_directory / 'task'
    try:
        thread_ids = sorted(
            (int(path.name) for path in task_directory.iterdir() if path.name.isdecimal())
        )
        thread_listing_status = 'read'
    except FileNotFoundError:
        thread_ids = []
        thread_listing_status = 'unsupported'
    except OSError as error:
        thread_ids = []
        thread_listing_status = 'error'
        budget.add_error('task_directory', error)
    threads_truncated = len(thread_ids) > MAX_THREADS_PER_WORKER
    observed_thread_ids = thread_ids[:MAX_THREADS_PER_WORKER]
    threads = []
    for thread_id in observed_thread_ids:
        directory = task_directory / str(thread_id)
        fields = {
            name: read_field(directory / name, f'tid_{thread_id}.{name}', budget)
            for name in THREAD_FIELDS
        }
        for name, selector in (
            ('stat', selected_stat_fields),
            ('schedstat', selected_schedstat_fields),
        ):
            value = fields[name]['value']
            if value is not None:
                try:
                    fields[name]['selected'] = selector(value)
                except (ValueError, IndexError) as error:
                    fields[name]['status'] = 'invalid'
                    budget.add_error(f'tid_{thread_id}.{name}', error)
        fields['status']['selected'] = selected_status_fields(fields['status']['value']) \
            if fields['status']['value'] is not None else None
        fields['status']['value'] = None
        if fields['stat']['value'] is not None:
            fields['stat']['value'] = None
        if fields['schedstat']['value'] is not None:
            fields['schedstat']['value'] = None
        threads.append({'tid': thread_id, 'fields': fields})
    case_cgroup = selected_cgroup_fields(cgroup_root, budget)
    worker_leaf_cgroup = selected_cgroup_fields(identity['leaf_cgroup'], budget)
    identity_after = verify_worker_identity(
        pid, start_time_ticks, source_revision, proc_root, cgroup_root, cgroup_mount,
    )
    identity['valid_at_capture_end'] = identity_after['valid']
    identity['capture_end_failure_reason'] = identity_after.get('reason')
    errors = []
    for field_name in [*('process.' + name for name in ('stat', 'status')),
                       *(f'tid_{thread_id}.{name}' for thread_id in observed_thread_ids
                         for name in THREAD_FIELDS), *CGROUP_FIELDS]:
        field_entries = [process.get(field_name.removeprefix('process.'))] \
            if field_name.startswith('process.') else None
        if field_entries is None and field_name in worker_leaf_cgroup:
            field_entries = [worker_leaf_cgroup[field_name]]
        if field_entries is None and field_name.startswith('tid_'):
            tid_name, name = field_name.split('.', 1)
            tid = int(tid_name.removeprefix('tid_'))
            matching_thread = next((thread for thread in threads if thread['tid'] == tid), None)
            field_entries = [matching_thread['fields'][name]] if matching_thread else []
        for entry in field_entries or []:
            if entry and entry.get('error'):
                errors.append(entry['error'])
    return {
        'status': 'captured' if identity_after['valid'] else 'identity_changed_during_capture',
        'identity': identity, 'identity_after_capture': identity_after,
        'capture_started_monotonic_ns': started,
        'read_duration_ns': time.monotonic_ns() - started,
        'thread_listing_status': thread_listing_status,
        'thread_count': len(thread_ids),
        'threads_observed': len(threads),
        'threads_truncated': threads_truncated,
        'threads': threads,
        'process': process,
        'cgroup': {'case_root': case_cgroup, 'worker_leaf': worker_leaf_cgroup},
        'captured_file_bytes': budget.file_bytes,
        'errors': errors[:MAX_ERRORS_PER_WORKER],
        'error_count': budget.errors,
        'errors_truncated': budget.errors_truncated or len(errors) > MAX_ERRORS_PER_WORKER,
    }


class JsonlSink:
    def __init__(self, output_path):
        self.output_path = Path(output_path)
        self.output = self.output_path.open('xb')
        self.total_bytes = 0
        self.worker_bytes = {}
        self.truncated_workers = set()
        header = self.encode({
            'record_type': 'observer_header',
            'limits': {
                'poll_interval_ms': int(POLL_INTERVAL_SECONDS * 1000),
                'capture_interval_ms': int(CAPTURE_INTERVAL_SECONDS * 1000),
                'shutdown_pending_ms': int(SHUTDOWN_PENDING_SECONDS * 1000),
                'max_captures_per_worker': MAX_CAPTURES_PER_WORKER,
                'max_workers': MAX_WORKERS,
                'max_threads_per_worker': MAX_THREADS_PER_WORKER,
                'max_file_bytes': MAX_FILE_BYTES,
                'max_line_bytes': MAX_LINE_BYTES,
                'max_worker_output_bytes': MAX_WORKER_OUTPUT_BYTES,
                'max_total_output_bytes': MAX_TOTAL_OUTPUT_BYTES,
                'max_worker_captured_file_bytes': MAX_WORKER_CAPTURED_FILE_BYTES,
                'max_errors_per_worker': MAX_ERRORS_PER_WORKER,
                'max_observer_errors': MAX_OBSERVER_ERRORS,
                'max_observer_wall_seconds': MAX_OBSERVER_WALL_SECONDS,
            },
        })
        self.output.write(header)
        self.total_bytes += len(header)

    @staticmethod
    def encode(value):
        line = json.dumps(value, sort_keys=True, separators=(',', ':')).encode('utf-8') + b'\n'
        if len(line) > MAX_LINE_BYTES:
            raise ValueError('JSONL record exceeds line byte limit')
        return line

    def write(self, value, worker_key=None):
        line = self.encode(value)
        worker_bytes = self.worker_bytes.get(worker_key, 0) if worker_key is not None else 0
        if worker_key in self.truncated_workers:
            return False
        if self.total_bytes + len(line) > MAX_TOTAL_OUTPUT_BYTES or (
            worker_key is not None and worker_bytes + len(line) > MAX_WORKER_OUTPUT_BYTES
        ):
            self.truncated_workers.add(worker_key)
            marker = self.encode({
                'record_type': 'truncation', 'reason': 'output_byte_limit',
                'worker_identity': worker_key,
                'worker_bytes_before_marker': worker_bytes,
                'total_bytes_before_marker': self.total_bytes,
                'truncated': True,
            })
            if self.total_bytes + len(marker) <= MAX_TOTAL_OUTPUT_BYTES and (
                worker_key is None or worker_bytes + len(marker) <= MAX_WORKER_OUTPUT_BYTES
            ):
                self.output.write(marker)
                self.output.flush()
                self.total_bytes += len(marker)
                if worker_key is not None:
                    self.worker_bytes[worker_key] = worker_bytes + len(marker)
            return False
        self.output.write(line)
        self.output.flush()
        self.total_bytes += len(line)
        if worker_key is not None:
            self.worker_bytes[worker_key] = worker_bytes + len(line)
        return True

    def close(self):
        self.output.flush()
        self.output.close()


class OffCpuObserver:
    def __init__(self, log_path, output_path, expected_source, cgroup_root,
                 proc_root=Path('/proc'), cgroup_mount=Path('/sys/fs/cgroup')):
        self.log_path = Path(log_path)
        self.output_path = Path(output_path)
        self.expected_source = expected_source
        self.cgroup_root = Path(cgroup_root)
        self.proc_root = Path(proc_root)
        self.cgroup_mount = Path(cgroup_mount)
        self.stop_event = threading.Event()
        self.finished_event = threading.Event()
        self.result = None
        self.thread = threading.Thread(target=self._run, name='shutdown-offcpu-observer', daemon=True)

    def start(self):
        self.thread.start()

    def stop(self):
        self.stop_event.set()
        self.thread.join(timeout=10)
        if self.thread.is_alive():
            raise RuntimeError('off-CPU observer thread did not stop within its bound')
        return self.result

    def _run(self):
        sink = None
        workers = {}
        errors = BoundedErrors()
        complete_offset = 0
        read_position = 0
        partial_line = bytearray()
        discarding_oversized_line = False
        discarded_line_size = 0
        last_capture_started = {}
        try:
            sink = JsonlSink(self.output_path)
            started = time.monotonic()
            while True:
                if self.log_path.is_file():
                    with self.log_path.open('rb') as log_file:
                        log_file.seek(read_position)
                        while True:
                            chunk = log_file.read(8192)
                            if not chunk:
                                break
                            read_position += len(chunk)
                            for segment in chunk.splitlines(keepends=True):
                                complete_line = segment.endswith(b'\n')
                                if discarding_oversized_line:
                                    discarded_line_size += len(segment)
                                    if complete_line:
                                        complete_offset += discarded_line_size
                                        discarded_line_size = 0
                                        discarding_oversized_line = False
                                    continue
                                if len(partial_line) + len(segment) > MAX_TRACE_LINE_BYTES:
                                    errors.append('trace_line_byte_limit')
                                    discarded_line_size = len(partial_line) + len(segment)
                                    partial_line.clear()
                                    discarding_oversized_line = not complete_line
                                    if complete_line:
                                        complete_offset += discarded_line_size
                                        discarded_line_size = 0
                                    continue
                                partial_line.extend(segment)
                                if not complete_line:
                                    continue
                                complete_offset += len(partial_line)
                                raw_line = bytes(partial_line)
                                partial_line.clear()
                                try:
                                    event = parse_trace_line(
                                        raw_line.decode('utf-8').rstrip('\r\n'),
                                        self.expected_source,
                                    )
                                    if event is not None:
                                        self._observe_event(event, workers, sink, errors)
                                except (UnicodeError, ValueError) as error:
                                    errors.append(str(error)[:192])
                now = time.monotonic()
                for worker_key, worker in list(workers.items()):
                    if not worker['pending'] or worker['captures'] >= MAX_CAPTURES_PER_WORKER:
                        continue
                    if now - worker['detected_monotonic'] < SHUTDOWN_PENDING_SECONDS:
                        continue
                    if now - last_capture_started.get(worker_key, 0) < CAPTURE_INTERVAL_SECONDS:
                        continue
                    if worker['file_bytes'] >= MAX_WORKER_CAPTURED_FILE_BYTES:
                        worker['truncated'] = True
                        worker['pending'] = False
                        sink.write({
                            'record_type': 'truncation', 'reason': 'captured_file_byte_limit',
                            'worker_identity': worker_key, 'truncated': True,
                            'captured_file_bytes': worker['file_bytes'],
                        }, worker_key)
                        continue
                    last_capture_started[worker_key] = now
                    capture = capture_snapshot(
                        worker_key[0], worker_key[1], self.expected_source,
                        self.proc_root, self.cgroup_root, self.cgroup_mount,
                    )
                    worker['captures'] += 1
                    worker['file_bytes'] += capture.get('captured_file_bytes', 0)
                    if capture['status'] != 'captured':
                        worker['identity_valid'] = False
                        worker['pending'] = False
                    record = {
                        'record_type': 'thread_snapshot',
                        'timestamp_unix_ns': time.time_ns(),
                        'shutdown_marker_timestamp_unix_ns': worker['marker_timestamp_unix_ns'],
                        'since_shutdown_marker_ms': max(
                            0, (time.time_ns() - worker['marker_timestamp_unix_ns']) // 1_000_000,
                        ),
                        'worker_identity': {
                            'process_id': worker_key[0],
                            'process_start_time_ticks': worker_key[1],
                            'source_revision': self.expected_source,
                        },
                        'capture_number': worker['captures'],
                        'capture': capture,
                    }
                    if not sink.write(record, worker_key):
                        worker['truncated'] = True
                        worker['pending'] = False
                    if capture['status'] != 'captured':
                        worker['errors'] += 1
                if self.stop_event.is_set():
                    break
                if now - started > MAX_OBSERVER_WALL_SECONDS:
                    errors.append('observer_wall_time_limit')
                    break
                self.stop_event.wait(POLL_INTERVAL_SECONDS)
            summary = {
                'record_type': 'observer_summary',
                'timestamp_unix_ns': time.time_ns(),
                'source_revision': self.expected_source,
                'log_bytes_scanned': read_position,
                'log_complete_line_offset': complete_offset,
                'partial_trace_line_bytes': len(partial_line) + discarded_line_size,
                'partial_trace_line_truncated': discarding_oversized_line,
                'partial_trace_line_pending_at_stop': bool(partial_line) or discarding_oversized_line,
                'observer_errors': errors[:MAX_OBSERVER_ERRORS],
                'observer_error_count': errors.count,
                'observer_errors_truncated': errors.truncated,
                'workers': [
                    {
                        'process_id': key[0], 'process_start_time_ticks': key[1],
                        'captures': value['captures'], 'identity_valid': value['identity_valid'],
                        'pending_at_end': value['pending'], 'truncated': value['truncated'],
                'terminal_stage': value['terminal_stage'],
                        'captured_file_bytes': value['file_bytes'], 'error_count': value['errors'],
                    }
                    for key, value in workers.items()
                ],
                'output_bytes_before_summary': sink.total_bytes,
                'output_truncated_workers': [list(key) for key in sink.truncated_workers],
            }
            sink.write(summary)
            self.result = summary
        except Exception as error:
            self.result = {
                'record_type': 'observer_summary', 'source_revision': self.expected_source,
                'observer_failure': f'{type(error).__name__}:{error}'[:512],
                'workers': [],
            }
            if sink is not None:
                try:
                    sink.write(self.result)
                except Exception:
                    pass
        finally:
            if sink is not None:
                sink.close()
            self.finished_event.set()

    def _observe_event(self, event, workers, sink, errors):
        worker_key = (event['process_id'], event['process_start_time_ticks'])
        stage = event['stage']
        if stage == SHUTDOWN_START_STAGE:
            if worker_key not in workers and len(workers) >= MAX_WORKERS:
                errors.append('worker_count_limit')
                return
            if worker_key in workers:
                errors.append('duplicate_async_shutdown_start')
                return
            identity = verify_worker_identity(
                worker_key[0], worker_key[1], self.expected_source,
                        self.proc_root, self.cgroup_root, self.cgroup_mount,
            )
            worker = {
                'marker_timestamp_unix_ns': event['timestamp_unix_ns'],
                'detected_monotonic': time.monotonic(),
                'pending': identity['valid'], 'identity_valid': identity['valid'],
                'captures': 0, 'file_bytes': 0, 'errors': 0, 'truncated': False,
                'terminal_stage': None,
            }
            workers[worker_key] = worker
            sink.write({
                'record_type': 'shutdown_marker',
                'timestamp_unix_ns': time.time_ns(),
                'marker': event,
                'identity_verification': identity,
                'pending': worker['pending'],
            }, worker_key)
            if not identity['valid']:
                worker['errors'] += 1
                errors.append(f'worker_identity_invalid:{identity["reason"]}')
        elif stage in SHUTDOWN_TERMINAL_STAGES:
            worker = workers.get(worker_key)
            if worker is not None:
                worker['pending'] = False
                worker['terminal_stage'] = stage
                sink.write({
                    'record_type': 'shutdown_terminal_marker',
                    'timestamp_unix_ns': time.time_ns(),
                    'marker': event,
                }, worker_key)


def field_capability(path, field_name):
    budget = CaptureBudget()
    result = read_field(path, field_name, budget)
    value = result.get('value')
    useful_wait = field_name in WAIT_FIELDS and result['status'] in ('read', 'truncated') \
        and value is not None and value.strip() not in ('', '0')
    selected = None
    if value is not None:
        try:
            if field_name == 'stat':
                selected = selected_stat_fields(value)
            elif field_name == 'status':
                selected = selected_status_fields(value)
            elif field_name == 'schedstat':
                selected = selected_schedstat_fields(value)
        except (ValueError, IndexError):
            selected = None
    return {
        'status': result['status'], 'bytes_read': result['bytes_read'],
        'truncated': result['truncated'], 'read_duration_ns': result['read_duration_ns'],
        'useful_wait_value': useful_wait,
        'selected': selected,
        'value': value if field_name in WAIT_FIELDS else None,
        'error': result.get('error'),
    }


def run_probe(output_path, sleep_seconds=30):
    child = subprocess.Popen(
        [sys.executable, '-c', 'import time; time.sleep(30)'],
        stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    output = Path(output_path)
    try:
        time.sleep(0.05)
        deadline = time.monotonic() + 2
        stat_path = Path('/proc') / str(child.pid) / 'stat'
        while not stat_path.exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        try:
            stat_text = stat_path.read_text()
            start_time = process_start_time_ticks(stat_text)
        except (OSError, ValueError) as error:
            start_time = None
            stat_error = f'{type(error).__name__}:{error}'
        else:
            stat_error = None
        fields = {}
        if start_time is not None:
            task_root = Path('/proc') / str(child.pid) / 'task' / str(child.pid)
            budget = CaptureBudget()
            for field_name in THREAD_FIELDS:
                fields[field_name] = field_capability(task_root / field_name, field_name)
        useful_wait_fields = [
            field_name for field_name in WAIT_FIELDS
            if fields.get(field_name, {}).get('useful_wait_value')
        ]
        result = {
            'probe_kind': 'owned_sleep_child_proc_read_capability',
            'timestamp_unix_ns': time.time_ns(),
            'kernel_release': os.uname().release,
            'child_pid': child.pid,
            'child_start_time_ticks': start_time,
            'child_start_time_error': stat_error,
            'fields': fields,
            'useful_wait_fields': useful_wait_fields,
            'wait_fields_available': bool(useful_wait_fields),
            'probe_child_terminated': False,
        }
    finally:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=2)
    result['probe_child_terminated'] = child.poll() is not None
    with output.open('x') as destination:
        json.dump(result, destination, indent=2)
        destination.write('\n')
    return result


def main():
    parser = argparse.ArgumentParser(description='Bounded external observer for shutdown kernel waits')
    subparsers = parser.add_subparsers(dest='command', required=True)
    probe_parser = subparsers.add_parser('probe')
    probe_parser.add_argument('--output', required=True)
    observe_parser = subparsers.add_parser('observe')
    observe_parser.add_argument('--log', required=True)
    observe_parser.add_argument('--output', required=True)
    observe_parser.add_argument('--source-revision', required=True)
    observe_parser.add_argument('--cgroup-root', required=True)
    arguments = parser.parse_args()
    if arguments.command == 'probe':
        result = run_probe(arguments.output)
        print(json.dumps({
            'probe_output': str(Path(arguments.output)),
            'wait_fields_available': result['wait_fields_available'],
            'useful_wait_fields': result['useful_wait_fields'],
            'probe_child_terminated': result['probe_child_terminated'],
        }), flush=True)
        return 0 if result['wait_fields_available'] and result['probe_child_terminated'] else 2
    observer = OffCpuObserver(
        arguments.log, arguments.output, arguments.source_revision,
        arguments.cgroup_root,
    )
    observer.start()
    try:
        while not observer.finished_event.wait(1):
            pass
    except KeyboardInterrupt:
        observer.stop_event.set()
    print(json.dumps(observer.result), flush=True)
    return 0 if observer.result and not observer.result.get('observer_failure') else 1


if __name__ == '__main__':
    raise SystemExit(main())
