import hashlib
import importlib.util
import json
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parent
RUN = ROOT / 'pressure/memory-pressure-d506d00dd-r3'
SOURCE = 'd506d00dd1f6324bf0074ba2ec0fd0f84792a1f8'
specification = importlib.util.spec_from_file_location('pressure_controller', ROOT / 'oci_memory_pressure_smoke-r3.py')
controller = importlib.util.module_from_spec(specification)
specification.loader.exec_module(controller)


def main():
    acceptance = json.loads((RUN / 'acceptance.json').read_text())
    assert acceptance['passed'] and acceptance['source_commit'] == SOURCE
    provenance = json.loads((RUN / 'provenance.json').read_text())
    assert hashlib.sha256((ROOT / 'oci_memory_pressure_smoke-r3.py').read_bytes()).hexdigest() == provenance['controller_sha256']
    review = {'source_commit': SOURCE, 'cases': {}, 'kernel_probes': {}}
    for name in ['high-probe', 'allocation-probe']:
        probe = json.loads((RUN / f'{name}.json').read_text())
        assert probe['passed']
        review['kernel_probes'][name] = {'passed': True, 'events': probe['after']['memory.events.local'],
                                        'kernel_peak_bytes': int(probe['after']['memory.peak'])}
    for case, backend, active_count in [
        ('blink-all-active', 'blink', 17), ('shared-all-active', 'minimal-native-shared', 17),
        ('blink-sparse', 'blink', 3), ('shared-sparse', 'minimal-native-shared', 3),
    ]:
        path = RUN / f'{case}.json'
        report = json.loads(path.read_text())
        status = json.loads((RUN / f'{case}-status.json').read_text())
        assert hashlib.sha256(path.read_bytes()).hexdigest() == status['report_sha256']
        controller.validate(report, backend, active_count, SOURCE)
        pressure = report['backend_memory_pressure']
        health = report['memory_diagnostic']['engine_health_reports']
        identities = report['memory_diagnostic']['processes']
        for epoch_name, epoch in zip(['setup', 'measurement', 'oracle', 'reopen-oracle'], health):
            membership = epoch['backend_memory_cgroup']
            assert membership['path'] == pressure['configuration']['cgroup_root'] + '/' + epoch_name
            assert membership['pid'] in membership['snapshot']['cgroup.procs']
            matching = [identity for identity in identities if identity['pid'] == epoch['worker_pid'] and identity['role'] == 'backend-process']
            assert len(matching) == 1 and matching[0]['start_time_ticks'] == epoch['worker_process_start_time_ticks']
            if backend == 'minimal-native-shared':
                snapshots = epoch['shared_resource_snapshots']
                names = ['before_shutdown'] + (['measurement_start', 'measurement_end'] if epoch_name == 'measurement' else [])
                for name in names:
                    native = snapshots[name]
                    expected_opened = (17 if active_count == 17 else 5) if epoch_name == 'measurement' else 17
                    assert native['opened_tenants'] == native['tenants_verified_using_shared_resources'] == expected_opened
                    assert native['usage']['cache_capacity_bytes'] == 8 * 1024**2
                    assert native['usage']['wbm_limit_bytes'] == 4 * 1024**2
        assert pressure['cell']['final']['cgroup.events']['populated'] == 0
        assert all(pressure['cell']['event_delta'][field] == 0 for field in ['high', 'max', 'oom_kill'])
        measurement_samples = [sample for sample in pressure['samples'] if sample['epoch'] == 'measurement' and sample['event'] == 'periodic']
        times = [sample['snapshot']['timestamp_unix_ms'] for sample in measurement_samples]
        assert report['measurement_worker_process_sampling']['same_backend_identity_at_offer_boundaries']
        review['cases'][case] = {'successful': report['workload']['successful'],
            'verified_seed_rows_per_oracle': report['reopened_full_population_oracle']['verified_seed_rows'],
            'verified_mutated_keys_raw_field_per_oracle': report['reopened_full_population_oracle']['verified_mutated_keys'],
            'cell_kernel_peak_bytes': pressure['cell']['final']['memory.peak'],
            'measurement_kernel_peak_bytes': pressure['epochs']['measurement']['final']['memory.peak'],
            'measurement_periodic_cgroup_samples': len(measurement_samples),
            'maximum_measurement_periodic_gap_ms': max((later - earlier for earlier, later in zip(times, times[1:])), default=None),
            'psi_status': pressure['cell']['final']['memory.pressure']['status']}
    pairs = json.loads((RUN / 'matched-inputs.json').read_text())
    assert all(all(pair.values()) for pair in pairs.values())
    test_counts = [int(count) for count in re.findall(r'test result: ok\. (\d+) passed;', (ROOT / 'builds/pressure-clean-tests/cargo.log').read_text())]
    assert sum(test_counts) == 68
    assert json.loads((ROOT / 'pressure-clippy-review.json').read_text())['same_message_and_path_diagnostics']
    review['passing_tests'] = sum(test_counts)
    review['successful_requests'] = sum(case['successful'] for case in review['cases'].values())
    review['matched_inputs'] = True
    review['scope'] = 'Accounting/provenance review only; positive correctness cells did not reach high/max; no density or performance ranking'
    (ROOT / 'local-evidence-review.json').write_text(json.dumps(review, indent=2) + '\n')
    print(json.dumps(review, indent=2))


if __name__ == '__main__':
    main()
