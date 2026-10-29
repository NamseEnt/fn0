import hashlib
import json
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parent
MAIN = ROOT / 'runtime/capacity-aaa405655-r2'
ORIGINAL = ROOT / 'runtime/capacity-aaa405655-r1'
SUPPLEMENT = ROOT / 'runtime/capacity-supplement-aaa405655-r1'


def digest(file_path):
    return hashlib.sha256(file_path.read_bytes()).hexdigest()


def main():
    archives = {'main-runtime.tar.gz': '08bda865f29e4472535dbfb6974b3864d03862433c29d238c419cc0010f2db15',
                'supplement-runtime.tar.gz': '7cb98a232110de005259bf4c851e9d86d4951ae4245f2a21ef3224e3870aa58a'}
    for name, expected in archives.items():
        assert digest(ROOT / name) == expected
    for directory in [MAIN, SUPPLEMENT]:
        subprocess.run([sys.executable, str(ROOT / 'frontier.py'), str(directory)], check=True, capture_output=True, text=True)
    frontiers = [json.loads((directory / 'frontier.json').read_text()) for directory in [MAIN, SUPPLEMENT]]
    assert all(frontier['runtime_script_hashes_verified'] for frontier in frontiers)
    assert {frontier['source_commit'] for frontier in frontiers} == {'aaa40565529ca92150df5d94e19b02ca93859f71'}
    reused = frontiers[0]['completion']['reused_existing_cases']
    new = frontiers[0]['completion']['executed_new_cases']
    assert len(reused) == 36 and len(new) == 16 and not set(reused) & set(new)
    for case in reused:
        assert digest(MAIN / (case + '.json')) == digest(ORIGINAL / (case + '.json'))
    input_review = json.loads((MAIN / 'continuation-input-review.json').read_text())
    for record in input_review:
        assert digest(ORIGINAL / (record['case'] + '-status.json')) == record['original_status_sha256']
    rows = [row for frontier in frontiers for row in frontier['cells']]
    assert len(rows) == 57
    assert sum(row['operational_pass'] for row in rows) == 55
    assert sum(row['slo_pass'] for row in rows) == 52
    assert sum(row['resource_failure'] for row in rows) == 1
    assert sum(bool(row.get('operational_timeout')) for row in rows) == 1
    assert sum(row.get('successful') or 0 for row in rows) == 2_304_000
    oracles = 0
    sample_states = []
    driver_rss_kib = []
    for directory in [MAIN, SUPPLEMENT]:
        for status_path in directory.glob('*-status.json'):
            status = json.loads(status_path.read_text())
            case = status['case']
            report = json.loads((directory / (case + '.json')).read_text())
            assert digest(directory / (case + '.json')) == status['report_sha256']
            if status['reduced']['operational_pass']:
                for name in ['post_measurement_full_population_oracle', 'reopened_full_population_oracle']:
                    assert report[name]['passed']
                    oracles += 1
            sample_states.extend(json.loads((directory / (case + '-host-samples.json')).read_text()))
            for sample in report['memory_diagnostic']['process_memory_samples']:
                driver_rss_kib.extend(process['rss_kb'] for process in sample['processes']
                                      if process['role'] == 'harness-process' and 'rss_kb' in process)
    assert oracles == 110
    minimum_free = min(state['zfs_free_bytes'] for state in sample_states)
    minimum_available = min(state['memory_bytes']['MemAvailable'] for state in sample_states)
    assert minimum_free >= 10 * 1024**3 and minimum_available >= 1500 * 1024**2
    databases = set()
    processes = set()
    binaries = []
    postflights = []
    for file_path in sorted((ROOT / 'collection').glob('*-postflight.json')):
        postflight = json.loads(file_path.read_text())
        assert postflight['source_clean'] and postflight['source_commit'] == frontiers[0]['source_commit']
        assert postflight['collector_sha256'] == digest(ROOT / 'oci_capacity_collect.py')
        postflights.append({'file': str(file_path.relative_to(ROOT)), 'capacity': postflight['capacity']})
        for run in postflight['runs'].values():
            binaries.append(run['binaries'])
            for database in run['databases'].values():
                assert database['preserved']
                databases.add(database['path'])
            for process in run['processes']:
                assert not process['same_live_process']
                processes.add((process['pid'], process['start_time_ticks']))
            assert all(not pids for group in run['cgroup_processes'].values() for pids in group.values())
    assert len(databases) == 57 and len(processes) == 279
    assert all(binary == binaries[0] for binary in binaries)
    validation = json.loads((ROOT / 'local-validation-review.json').read_text())
    assert validation['passing_tests'] == 68 and validation['same_17_existing_clippy_diagnostics']
    output = {'source_commit': frontiers[0]['source_commit'], 'reviewer_sha256': digest(Path(__file__)),
              'unique_runtime_cells': len(rows), 'completed_cells': 55, 'slo_passing_cells': 52,
              'completed_slo_misses': 3, 'kernel_oom_cells': 1,
              'setup_shutdown_deadline_cells': 1, 'successful_offered_requests_in_complete_reports': 2_304_000,
              'passing_full_population_oracles': oracles, 'reused_cases_without_reexecution': len(reused),
              'new_continuation_cells': len(new), 'supplemental_cells': len(frontiers[1]['cells']),
              'preserved_unique_databases': len(databases), 'recorded_exited_process_identities': len(processes),
              'minimum_sampled_zfs_free_bytes': minimum_free,
              'minimum_sampled_host_available_bytes': minimum_available,
              'maximum_sampled_driver_lifetime_rss_kib': max(driver_rss_kib),
              'archive_sha256': archives, 'frozen_binaries': binaries[0], 'postflights': postflights,
              'confirmation_groups': [group for frontier in frontiers for group in frontier['confirmation_groups']],
              'scope': 'Instrumented loopback/cgroup evidence only; cohort durations/cadences and operational failures remain distinct'}
    (ROOT / 'final-review.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps({name: output[name] for name in ['unique_runtime_cells', 'completed_cells', 'kernel_oom_cells', 'setup_shutdown_deadline_cells', 'successful_offered_requests_in_complete_reports', 'passing_full_population_oracles', 'preserved_unique_databases', 'recorded_exited_process_identities']}, indent=2))


if __name__ == '__main__':
    main()
