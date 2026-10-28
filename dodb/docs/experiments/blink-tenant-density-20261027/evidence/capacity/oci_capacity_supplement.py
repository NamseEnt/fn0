import argparse
import json
from pathlib import Path

from capacity_review import digest
from oci_capacity import Experiment, ROOT, capacity, process_identity_live, write_json


def main(arguments):
    main_directory = ROOT / 'capacity' / arguments.main_run
    completion = json.loads((main_directory / 'completion.json').read_text())
    provenance = json.loads((main_directory / 'provenance.json').read_text())
    assert completion['completed'] and completion['source_commit'] == provenance['source_commit']
    experiment = Experiment(arguments.run_name, provenance['source_commit'], arguments.plan_commit)
    supplementary_provenance = json.loads((experiment.run_directory / 'provenance.json').read_text())
    assert supplementary_provenance['binaries'] == provenance['binaries']
    supplementary_provenance['main_run'] = arguments.main_run
    supplementary_provenance['supplement_controller_sha256'] = digest(Path(__file__))
    write_json(experiment.run_directory / 'provenance.json', supplementary_provenance)
    screen = {}
    for label, backend in [('blink', 'blink'), ('shared', 'minimal-native-shared')]:
        status_path = main_directory / f'screen-e3-n8-{label}-status.json'
        if status_path.exists():
            screen[backend] = json.loads(status_path.read_text())['reduced']
    survivors = [backend for backend, reduced in screen.items() if reduced['operational_pass']]
    oom_failures = [backend for backend, reduced in screen.items() if reduced['resource_failure']]
    selections = {'eight_tenant_e3_survivors': survivors, 'eight_tenant_e3_oom_failures': oom_failures}
    maintenance_count = max(completion['confirmed_common_counts']['E2'], default=0)
    selections['maintenance_count'] = maintenance_count if maintenance_count > 16 else None
    write_json(experiment.run_directory / 'selections.json', selections)
    if len(survivors) == len(oom_failures) == 1:
        backend = survivors[0]
        label = 'blink' if backend == 'blink' else 'shared'
        for repetition in range(1, 4):
            outcome = experiment.run_cell(f'confirm-e3-n8-r{repetition}-{label}', 'E3', backend, 8, 60000)
            assert outcome['reduced']['operational_pass'], outcome['case']
    if maintenance_count > 16:
        experiment.pair(f'maintenance-e2-n{maintenance_count}-ms2000', 'E2', maintenance_count, 180000, interval_ms=2000)
    final = {'completed': True, 'source_commit': provenance['source_commit'],
             'cases': len(experiment.outcomes), 'capacity_after': capacity(), 'selections': selections,
             'scope': 'Conditional independent reference confirmation and actual-boundary maintenance; main sequence unchanged',
             'processes': [{'pid': identity['pid'], 'role': identity['role'],
                            'start_time_ticks': identity['start_time_ticks'],
                            'same_live_process': process_identity_live(identity)} for identity in experiment.all_processes]}
    assert all(not identity['same_live_process'] for identity in final['processes'])
    if not experiment.outcomes:
        write_json(experiment.run_directory / 'summary.json', [])
        write_json(experiment.run_directory / 'omission.json', {'reason': 'neither frozen supplemental condition applies'})
    write_json(experiment.run_directory / 'completion.json', final)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--main-run', required=True)
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    main(arguments)
