import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

import offcpu_observer as observer


SOURCE = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'


def stat_line(process_id, start_time_ticks, command='worker'):
    fields = ['S'] + ['0'] * 19
    fields[11] = '12'
    fields[12] = '4'
    fields[19] = str(start_time_ticks)
    return f'{process_id} ({command}) ' + ' '.join(fields) + '\n'


def write_worker_fixture(proc_root, cgroup_root, process_id=777, start_time_ticks=123,
                         thread_ids=(777,)):
    process_root = Path(proc_root) / str(process_id)
    task_root = process_root / 'task'
    task_root.mkdir(parents=True)
    (process_root / 'stat').write_text(stat_line(process_id, start_time_ticks))
    (process_root / 'status').write_text(
        'Name:\tworker\nState:\tS (sleeping)\nVmRSS:\t1024 kB\nVmHWM:\t2048 kB\n'
        'Threads:\t1\nvoluntary_ctxt_switches:\t5\nnonvoluntary_ctxt_switches:\t2\n'
        'Uid:\t0\t0\t0\t0\n'
    )
    for thread_id in thread_ids:
        thread_root = task_root / str(thread_id)
        thread_root.mkdir()
        (thread_root / 'stat').write_text(stat_line(thread_id, start_time_ticks))
        (thread_root / 'status').write_text(
            'Name:\tworker\nState:\tS (sleeping)\nVmRSS:\t1024 kB\n'
            'Threads:\t1\nvoluntary_ctxt_switches:\t5\n'
            'nonvoluntary_ctxt_switches:\t2\nUid:\t0\t0\t0\t0\n'
        )
        (thread_root / 'wchan').write_text('hrtimer_nanosleep\n')
        (thread_root / 'syscall').write_text('230 0x0 0x1\n')
        (thread_root / 'stack').write_text('[<0000>] hrtimer_nanosleep+0x1\n')
        (thread_root / 'schedstat').write_text('100 50 3\n')
    Path(cgroup_root).mkdir(parents=True)
    (Path(cgroup_root) / 'cgroup.procs').write_text('')
    leaf = Path(cgroup_root) / 'setup'
    leaf.mkdir()
    (leaf / 'cgroup.procs').write_text(str(process_id) + '\n')
    for cgroup in (Path(cgroup_root), leaf):
        (cgroup / 'memory.current').write_text('4096\n')
        (cgroup / 'memory.events').write_text('low 0\nhigh 1\nmax 0\n')
        (cgroup / 'cpu.stat').write_text('usage_usec 10\nuser_usec 5\n')
        (cgroup / 'io.stat').write_text('8:0 rbytes=10 wbytes=20\n')
    (Path(proc_root) / str(process_id) / 'cgroup').write_text('0::/offcpu/case/setup\n')
    cgroup_mount = Path(cgroup_root).parents[1]
    assert leaf == cgroup_mount / 'offcpu/case/setup'


def trace_event(stage, pid=777, start_time_ticks=123):
    return {
        'timestamp_unix_ns': time.time_ns(),
        'process_id': pid,
        'process_start_time_ticks': start_time_ticks,
        'source_revision': SOURCE,
        'stage': stage,
    }


class OffCpuObserverTests(unittest.TestCase):
    def test_trace_parser_checks_schema_source_and_identity_types(self):
        event = trace_event('async_server_shutdown_start')
        parsed = observer.parse_trace_line(json.dumps(event), SOURCE)
        self.assertEqual(parsed, event)
        with self.assertRaises(ValueError):
            observer.parse_trace_line(json.dumps(event), 'other')
        with self.assertRaises(ValueError):
            observer.parse_trace_line(json.dumps(event | {'unexpected': 1}), SOURCE)
        with self.assertRaises(ValueError):
            observer.parse_trace_line(json.dumps(event | {'process_id': True}), SOURCE)

    def test_capture_verifies_pid_start_time_and_cgroup_ownership(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            identity = observer.verify_worker_identity(
                777, 123, SOURCE, proc_root, cgroup_root, cgroup_mount,
            )
            self.assertTrue(identity['valid'])
            wrong_start = observer.verify_worker_identity(
                777, 124, SOURCE, proc_root, cgroup_root, cgroup_mount,
            )
            self.assertEqual(wrong_start['reason'], 'process_start_time_mismatch')
            (cgroup_root / 'setup' / 'cgroup.procs').write_text('778\n')
            wrong_group = observer.verify_worker_identity(
                777, 123, SOURCE, proc_root, cgroup_root, cgroup_mount,
            )
            self.assertEqual(wrong_group['reason'], 'process_not_in_reported_leaf_cgroup')

    def test_snapshot_keeps_curated_fields_and_enforces_thread_file_limits(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            thread_ids = tuple(range(777, 777 + observer.MAX_THREADS_PER_WORKER + 2))
            write_worker_fixture(proc_root, cgroup_root, thread_ids=thread_ids)
            snapshot = observer.capture_snapshot(
                777, 123, SOURCE, proc_root, cgroup_root, cgroup_mount,
            )
            self.assertEqual(snapshot['status'], 'captured')
            self.assertEqual(snapshot['thread_count'], len(thread_ids))
            self.assertEqual(snapshot['threads_observed'], observer.MAX_THREADS_PER_WORKER)
            self.assertTrue(snapshot['threads_truncated'])
            self.assertLessEqual(snapshot['captured_file_bytes'], observer.MAX_WORKER_CAPTURED_FILE_BYTES)
            thread = snapshot['threads'][0]['fields']
            self.assertEqual(thread['wchan']['value'], 'hrtimer_nanosleep')
            self.assertEqual(thread['stat']['selected']['user_ticks'], 12)
            self.assertIsNone(thread['status']['value'])
            self.assertEqual(thread['status']['selected']['VmRSS'], '1024 kB')
            with tempfile.TemporaryDirectory() as file_temporary:
                large_file = Path(file_temporary) / 'large'
                large_file.write_bytes(b'x' * (observer.MAX_FILE_BYTES * 3))
                field = observer.read_field(large_file, 'fixture', observer.CaptureBudget())
                self.assertEqual(field['status'], 'truncated')
                self.assertEqual(field['bytes_read'], observer.MAX_FILE_BYTES)

    def test_snapshot_reports_unsupported_wait_fields_without_inference(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            (proc_root / '777' / 'task' / '777' / 'wchan').unlink()
            snapshot = observer.capture_snapshot(
                777, 123, SOURCE, proc_root, cgroup_root, cgroup_mount,
            )
            self.assertEqual(snapshot['threads'][0]['fields']['wchan']['status'], 'unsupported')
            self.assertEqual(snapshot['threads'][0]['fields']['wchan']['value'], None)

    def test_status_limit_preserves_context_switches_after_one_kibibyte(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            status_path = proc_root / '777' / 'task' / '777' / 'status'
            status_path.write_text(
                'Name:\tworker\nPadding:\t' + ('x' * 1200) + '\n'
                'State:\tS (sleeping)\nvoluntary_ctxt_switches:\t91\n'
                'nonvoluntary_ctxt_switches:\t17\n'
            )
            snapshot = observer.capture_snapshot(
                777, 123, SOURCE, proc_root, cgroup_root, cgroup_mount,
            )
            status = snapshot['threads'][0]['fields']['status']
            self.assertGreater(status['bytes_read'], observer.MAX_FILE_BYTES)
            self.assertEqual(status['status'], 'read')
            self.assertEqual(status['selected']['voluntary_ctxt_switches'], '91')
            self.assertEqual(status['selected']['nonvoluntary_ctxt_switches'], '17')

    def test_snapshot_marks_identity_reuse_or_exit_during_capture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            valid_identity = {
                'valid': True, 'process_id': 777, 'process_start_time_ticks': 123,
                'source_revision': SOURCE, 'cgroup_root': str(cgroup_root),
                'leaf_cgroup': str(cgroup_root / 'setup'),
            }
            invalid_identity = {'valid': False, 'reason': 'process_start_time_mismatch'}
            with patch.object(observer, 'verify_worker_identity',
                              side_effect=[valid_identity, invalid_identity]):
                snapshot = observer.capture_snapshot(
                    777, 123, SOURCE, proc_root, cgroup_root, cgroup_mount,
                )
            self.assertEqual(snapshot['status'], 'identity_changed_during_capture')
            self.assertFalse(snapshot['identity']['valid_at_capture_end'])

    def test_observer_streams_bounded_samples_for_owned_pending_shutdown(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            log_path = root / 'worker.log'
            output_path = root / 'samples.jsonl'
            log_path.write_text(json.dumps(trace_event('async_server_shutdown_start')) + '\n')
            with patch.object(observer, 'POLL_INTERVAL_SECONDS', 0.01), \
                    patch.object(observer, 'SHUTDOWN_PENDING_SECONDS', 0.0), \
                    patch.object(observer, 'CAPTURE_INTERVAL_SECONDS', 0.01):
                instance = observer.OffCpuObserver(
                    log_path, output_path, SOURCE, cgroup_root, proc_root, cgroup_mount,
                )
                instance.start()
                time.sleep(0.08)
                result = instance.stop()
            records = [json.loads(line) for line in output_path.read_text().splitlines()]
            snapshots = [record for record in records if record['record_type'] == 'thread_snapshot']
            self.assertIsNone(result.get('observer_failure'))
            self.assertTrue(snapshots)
            self.assertLessEqual(len(snapshots), observer.MAX_CAPTURES_PER_WORKER)
            self.assertTrue(all(snapshot['capture']['identity']['valid'] for snapshot in snapshots))
            self.assertLessEqual(output_path.stat().st_size, observer.MAX_TOTAL_OUTPUT_BYTES)

    def test_observer_retains_partial_trace_until_newline_arrives(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            log_path = root / 'worker.log'
            output_path = root / 'samples.jsonl'
            encoded = json.dumps(trace_event('async_server_shutdown_start')).encode() + b'\n'
            log_path.write_bytes(encoded[:-1])
            with patch.object(observer, 'POLL_INTERVAL_SECONDS', 0.01), \
                    patch.object(observer, 'SHUTDOWN_PENDING_SECONDS', 0.0), \
                    patch.object(observer, 'CAPTURE_INTERVAL_SECONDS', 0.02):
                instance = observer.OffCpuObserver(
                    log_path, output_path, SOURCE, cgroup_root, proc_root, cgroup_mount,
                )
                instance.start()
                time.sleep(0.05)
                before_newline = [
                    json.loads(line) for line in output_path.read_text().splitlines()
                ]
                self.assertFalse(any(
                    record['record_type'] == 'shutdown_marker' for record in before_newline
                ))
                with log_path.open('ab') as destination:
                    destination.write(b'\n')
                time.sleep(0.08)
                result = instance.stop()
            records = [json.loads(line) for line in output_path.read_text().splitlines()]
            self.assertTrue(any(record['record_type'] == 'shutdown_marker' for record in records))
            self.assertFalse(result['partial_trace_line_pending_at_stop'])
            self.assertEqual(result['log_complete_line_offset'], len(encoded))

    def test_fast_normal_shutdown_is_classified_without_live_process_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            (proc_root / '777').rename(proc_root / 'gone')
            log_path = root / 'worker.log'
            output_path = root / 'samples.jsonl'
            start = trace_event('async_server_shutdown_start')
            terminal = trace_event('async_server_shutdown_end')
            terminal['timestamp_unix_ns'] = start['timestamp_unix_ns'] + 100_000_000
            log_path.write_text(json.dumps(start) + '\n' + json.dumps(terminal) + '\n')
            with patch.object(observer, 'POLL_INTERVAL_SECONDS', 0.01):
                instance = observer.OffCpuObserver(
                    log_path, output_path, SOURCE, cgroup_root, proc_root, cgroup_mount,
                )
                instance.start()
                time.sleep(0.03)
                result = instance.stop()
            self.assertIsNone(result.get('observer_failure'))
            self.assertFalse(result['observer_errors'])
            self.assertEqual(result['workers'][0]['observation_status'], 'finished_before_sampling')
            self.assertEqual(result['workers'][0]['captures'], 0)
            records = [json.loads(line) for line in output_path.read_text().splitlines()]
            marker = next(record for record in records if record['record_type'] == 'shutdown_marker')
            self.assertFalse(marker['identity_verification']['valid'])

    def test_shutdown_terminal_after_sampling_threshold_without_capture_is_invalid(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            proc_root = root / 'proc'
            cgroup_mount = root / 'sysfs'
            cgroup_root = cgroup_mount / 'offcpu/case'
            write_worker_fixture(proc_root, cgroup_root)
            (proc_root / '777').rename(proc_root / 'gone')
            log_path = root / 'worker.log'
            output_path = root / 'samples.jsonl'
            start = trace_event('async_server_shutdown_start')
            terminal = trace_event('async_server_shutdown_end')
            terminal['timestamp_unix_ns'] = start['timestamp_unix_ns'] + 3_000_000_000
            log_path.write_text(json.dumps(start) + '\n' + json.dumps(terminal) + '\n')
            with patch.object(observer, 'POLL_INTERVAL_SECONDS', 0.01):
                instance = observer.OffCpuObserver(
                    log_path, output_path, SOURCE, cgroup_root, proc_root, cgroup_mount,
                )
                instance.start()
                time.sleep(0.03)
                result = instance.stop()
            self.assertIn('shutdown_terminal_without_required_sample', result['observer_errors'])
            self.assertEqual(
                result['workers'][0]['observation_status'], 'terminal_without_required_sample',
            )

    def test_sink_records_truncation_when_worker_output_bound_is_reached(self):
        with tempfile.TemporaryDirectory() as temporary:
            output_path = Path(temporary) / 'samples.jsonl'
            with patch.object(observer, 'MAX_WORKER_OUTPUT_BYTES', 512):
                sink = observer.JsonlSink(output_path)
                worker_key = (777, 123)
                accepted = sink.write({'record_type': 'x', 'payload': 'z' * 800}, worker_key)
                sink.close()
            self.assertFalse(accepted)
            records = [json.loads(line) for line in output_path.read_text().splitlines()]
            self.assertTrue(any(record['record_type'] == 'truncation' for record in records))


if __name__ == '__main__':
    unittest.main()
