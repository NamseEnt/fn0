import importlib.util
import hashlib
import json
from pathlib import Path
import tarfile
import tempfile
import unittest


REVIEW_PATH = Path(__file__).with_name('review_arrival.py')
SPEC = importlib.util.spec_from_file_location('review_arrival_under_test', REVIEW_PATH)
REVIEW = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REVIEW)


def synthetic_cell_results(slo_miss_case=None):
    results = {}
    for case in REVIEW.EXPECTED_REPORT_CASES:
        tiny = case.startswith('tiny-')
        results[case] = {
            'valid': True,
            'operational_pass': True,
            'slo_pass': case != slo_miss_case,
            'successful_requests': 1500 if tiny else 84000,
            'scheduled_requests': 1500 if tiny else 84000,
            'oracle_count': 2,
            'reduced': {
                'worst_tenant_p99_upper_exclusive_ms': {
                    'Get': 8.0, 'Query': 16.0, 'Write': 64.0,
                },
                'epoch_peaks_bytes': {'measurement': 220 * 1024**2},
                'measurement_sample_count': 12,
                'native_measurement_sample_count': 6,
                'native_serialized_bytes': 4096,
            },
        }
    return results


class ArrivalFinalReviewTests(unittest.TestCase):
    def test_manifest_rejects_duplicate_paths_and_traversal(self):
        duplicate = [
            {'relative_archive_path': 'arrival-phase/a', 'bytes': 0, 'sha256': '0' * 64},
            {'relative_archive_path': 'arrival-phase/a', 'bytes': 0, 'sha256': '0' * 64},
        ]
        with self.assertRaises(AssertionError):
            REVIEW.manifest_index(duplicate, 'failed')
        with self.assertRaises(AssertionError):
            REVIEW.safe_archive_path('arrival-phase/../outside')
        with self.assertRaises(AssertionError):
            REVIEW.safe_archive_path('/arrival-phase/outside')

    def test_archive_and_extracted_tree_are_checked_against_one_manifest(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            evidence_directory = Path(temporary_directory)
            runtime_directory = evidence_directory / 'runtime'
            runtime_directory.mkdir()
            relative_path = f'arrival-phase/{REVIEW.RUN_NAME}/completion.json'
            content = b'{"completed":true}\n'
            extracted = runtime_directory / relative_path
            extracted.parent.mkdir(parents=True)
            extracted.write_bytes(content)
            archive_path = evidence_directory / 'runtime.tar.gz'
            with tarfile.open(archive_path, 'w:gz') as archive:
                archive.add(extracted, arcname=relative_path, recursive=False)
            record = {'relative_archive_path': relative_path,
                      'bytes': len(content),
                      'sha256': hashlib.sha256(content).hexdigest()}
            receipt = {
                'controls_terminal_status': 'completed',
                'archive': {'bytes': archive_path.stat().st_size,
                            'sha256': REVIEW.sha256_file(archive_path)},
                'file_manifest': [record],
            }
            expected = {relative_path: {'bytes': len(content),
                                        'sha256': hashlib.sha256(content).hexdigest()}}
            self.assertEqual(REVIEW.verify_archive_members(archive_path, expected)['member_count'], 1)
            self.assertEqual(REVIEW.verify_extracted_tree(runtime_directory, expected)['file_count'], 1)
            extracted.write_bytes(b'tampered\n')
            with self.assertRaises(AssertionError):
                REVIEW.verify_extracted_tree(runtime_directory, expected)
            extracted.write_bytes(content)
            receipt['archive']['bytes'] += 1
            with self.assertRaises(AssertionError):
                REVIEW.verify_archive_and_tree(evidence_directory, receipt)

    def test_archive_tamper_stops_before_any_report_reduction(self):
        class ReducerSpy:
            def __init__(self):
                self.calls = 0

            def reduce_report(self, report, source_commit):
                self.calls += 1
                raise AssertionError('tampered evidence reached the reducer')

        class ArrivalSpy:
            def __init__(self):
                self.calls = 0

            def validate_report(self, report, case, backend, source_commit):
                self.calls += 1
                raise AssertionError('tampered evidence reached the arrival validator')

        with tempfile.TemporaryDirectory() as temporary_directory:
            evidence_directory = Path(temporary_directory)
            collection_directory = evidence_directory / 'collection'
            collection_directory.mkdir()
            archive_path = evidence_directory / 'runtime.tar.gz'
            archive_path.write_bytes(b'tampered archive')
            receipt = {
                'archive': {'bytes': 1, 'sha256': '0' * 64},
                'controls_terminal_status': 'completed',
            }
            (collection_directory / 'collection.json').write_text(json.dumps(receipt))
            capacity_spy = ReducerSpy()
            arrival_spy = ArrivalSpy()
            with self.assertRaises(AssertionError):
                REVIEW.review(evidence_directory, capacity_reviewer=capacity_spy,
                              arrival_reviewer=arrival_spy)
            self.assertEqual(capacity_spy.calls, 0)
            self.assertEqual(arrival_spy.calls, 0)

    def test_operational_completion_and_slo_miss_remain_separate(self):
        results = synthetic_cell_results('control-n28-r1-staggered-shared')
        pair_results = {case: {'valid': True} for case in REVIEW.EXPECTED_PAIRS}
        phase_results = {f'{first}--{second}-{backend}': {'valid': True}
                         for first, second in REVIEW.PHASE_PAIRS for backend in ['blink', 'shared']}
        summary = REVIEW.summarize_controls(
            'completed', REVIEW.EXPECTED_REPORT_CASES, results, pair_results, phase_results)
        self.assertTrue(summary['completed'])
        self.assertEqual(summary['operationally_valid_count'], 16)
        self.assertEqual(summary['slo_pass_count'], 15)
        self.assertEqual(summary['slo_miss_cases'], ['control-n28-r1-staggered-shared'])
        self.assertEqual(summary['validated_successful_requests'], 1_014_000)
        self.assertEqual(summary['expected_complete_cohort_requests'], 1_014_000)
        self.assertEqual(summary['validated_oracle_count'], 32)
        self.assertFalse(summary['accepted_run'])
        self.assertEqual(summary['slo_pass_count'], 15)

    def test_partial_failure_does_not_promote_available_cells(self):
        results = synthetic_cell_results()
        failed_case = 'control-n28-r2-staggered-blink'
        results[failed_case] = {
            'valid': False,
            'available_unvalidated_counters': {'scheduled': 84000, 'successful': 83900},
        }
        summary = REVIEW.summarize_controls(
            'failed', REVIEW.EXPECTED_REPORT_CASES, results,
            {case: {'valid': True} for case in REVIEW.EXPECTED_PAIRS},
            {f'{first}--{second}-{backend}': {'valid': True}
             for first, second in REVIEW.PHASE_PAIRS for backend in ['blink', 'shared']})
        self.assertFalse(summary['completed'])
        self.assertEqual(summary['operational_failure_count'], 1)
        self.assertEqual(summary['operational_failure_cases'], [failed_case])
        self.assertEqual(summary['operationally_valid_count'], 15)
        self.assertIsNone(summary['expected_complete_cohort_requests'])
        self.assertIsNone(summary['expected_complete_cohort_oracles'])
        self.assertFalse(summary['accepted_run'])

    def test_condition_summary_keeps_three_repetition_slo_count_and_evidence_metrics(self):
        results = synthetic_cell_results('control-n28-r2-staggered-shared')
        conditions = REVIEW.condition_summaries(results)
        staggered_native = conditions['minimal-native-shared/staggered']
        self.assertEqual(staggered_native['expected_repetitions'], 3)
        self.assertEqual(staggered_native['slo_pass_repetitions'], 2)
        self.assertEqual(staggered_native['slo_miss_repetitions'], 1)
        self.assertEqual(staggered_native['worst_tenant_p99_upper_exclusive_ms']['Query'], 16.0)
        self.assertEqual(staggered_native['measurement_peak_bytes_max'], 220 * 1024**2)
        self.assertEqual(staggered_native['native_serialized_observation_bytes_total'], 3 * 4096)

    def test_phase_comparison_normalizes_reversed_repetition_two_order(self):
        reports = {}
        results = {}
        for first_case, second_case in REVIEW.PHASE_PAIRS:
            for case in [first_case, second_case]:
                for suffix in ['-blink', '-shared']:
                    report_case = case + suffix
                    reports[report_case] = {'case': report_case}
                    results[report_case] = {'valid': True}

        class ArrivalComparisonSpy:
            def __init__(self):
                self.calls = []

            def compare_phases(self, synchronized_report, staggered_report):
                self.calls.append((synchronized_report['case'], staggered_report['case']))
                return {'recipe_matches': True}

        comparison_spy = ArrivalComparisonSpy()
        comparisons = REVIEW.review_phase_pairs(reports, results, comparison_spy)
        r2_blink_key = 'control-n28-r2-synchronized--control-n28-r2-staggered-blink'
        self.assertTrue(comparisons[r2_blink_key]['valid'])
        self.assertEqual(
            ('control-n28-r2-synchronized-blink', 'control-n28-r2-staggered-blink'),
            comparison_spy.calls[4],
        )
        self.assertEqual(len(comparison_spy.calls), 8)

    def test_process_limit_samples_are_deduplicated_by_pid_and_start_time(self):
        workload_identity = {'pid': 1201, 'start_time_ticks': 4001}
        descendant_identity = {'pid': 1202, 'start_time_ticks': 4002}
        workload_row = {
            **workload_identity,
            'role': 'owned-workload-process',
            'limits': {'soft': 65536, 'hard': 524288, 'units': 'files'},
        }
        descendant_row = {
            **descendant_identity,
            'role': 'owned-descendant',
            'limits': {'soft': 65536, 'hard': 524288, 'units': 'files'},
        }
        with tempfile.TemporaryDirectory() as temporary_directory:
            controls_directory = Path(temporary_directory)
            status = {
                'case': 'tiny-synchronized-blink',
                'exit_code': 0,
                'owned_process_limits': [workload_row, workload_row, descendant_row, descendant_row],
            }
            (controls_directory / 'tiny-synchronized-blink-status.json').write_text(json.dumps(status))
            receipt_processes = [
                {**workload_identity, 'same_live_process_at_collection': False},
                {**descendant_identity, 'same_live_process_at_collection': False},
            ]
            result = REVIEW.validate_process_receipt(
                {'controls_terminal_status': 'failed', 'controls': {'processes': receipt_processes}},
                {'processes': receipt_processes}, controls_directory, {}, {},
            )
        self.assertEqual(result['unique_process_count'], 2)
        self.assertEqual(result['owned_workload_child_count'], 1)


if __name__ == '__main__':
    unittest.main()
