import argparse
import json
from pathlib import Path

from capacity_review import digest
import oci_capacity
from oci_capacity import Experiment, ROOT, capacity, process_identity_live, write_json
from operational_review import reduce_report


class ObservationExperiment(Experiment):
    def __init__(self, run_name, source_commit, plan_commit):
        validation = json.loads((ROOT / 'observations-clean-validation-summary.json').read_text())
        assert validation['source_commit'] == source_commit
        assert len(validation['outcomes']) == 4 and all(check['exit_code'] == 0 for check in validation['outcomes'])
        assert validation['outcomes'][-1]['name'] == 'observations-clean-default-link'
        super().__init__(run_name, source_commit, plan_commit)
        provenance_path = self.run_directory / 'provenance.json'
        provenance = json.loads(provenance_path.read_text())
        provenance.update({'observation_controller_sha256': digest(Path(__file__)),
                           'operational_failure_reducer_sha256': digest(Path(__file__).with_name('operational_review.py')),
                           'resolved_build_target': str((ROOT / 'target').resolve()),
                           'cache_relocation_completed': json.loads((ROOT / 'observations-cache-relocation-20261029/result.json').read_text())['completed'],
                           'representation': 'validated raw JSON frames; streamed worker health report; driver parsing remains outside backend budget'})
        write_json(provenance_path, provenance)
        oci_capacity.reduce_report = self.review_report

    def review_report(self, report, source_commit):
        case = Path(report['effective_configuration']['backend_memory_budget']['cgroup_root']).name
        result = reduce_report(report, source_commit, (self.run_directory / (case + '.log')).read_text())
        if result.get('operational_timeout'):
            stage = 'E2' if result['initial_rows'] == 2048 else 'E3'
            self.operational_failures.add((stage, result['backend'], result['registered']))
        return result

    def compare(self):
        for interval in [1000, 100]:
            outcomes = self.pair(f'sampling-e2-n1-ms{interval}', 'E2', 1, 60000, interval_ms=interval)
            assert all(outcome['reduced']['operational_pass'] for outcome in outcomes.values()), 'cadence operational failure requires review'
        for stage, tenants in [('E2', 28), ('E3', 5)]:
            for repetition in range(1, 4):
                outcomes = self.pair(f'confirm-{stage.lower()}-n{tenants}-r{repetition}', stage, tenants, 60000)
                assert all(outcome['reduced']['operational_pass'] for outcome in outcomes.values()), 'confirmation operational failure requires review'
        pressure_screens = {}
        for tenants in [8, 6]:
            pressure_screens[tenants] = self.pair(f'screen-e3-n{tenants}', 'E3', tenants, 30000, skip_failed=True)
        eligible = []
        for backend in ['blink', 'minimal-native-shared']:
            screen = pressure_screens[8].get(backend)
            failed = any(stage == 'E3' and failure_backend == backend for stage, failure_backend, _ in self.operational_failures)
            if screen and screen['reduced']['operational_pass'] and failed:
                raise AssertionError('non-monotonic operational outcomes require review before further capacity repetitions')
            if screen and screen['reduced']['operational_pass'] and not failed:
                eligible.append(backend)
        write_json(self.run_directory / 'eight-tenant-confirmation-selection.json', {'eligible_routes': eligible})
        if not eligible:
            write_json(self.run_directory / 'eight-tenant-confirmation-omission.json', {'reason': 'no route remained operationally eligible after the frozen pressure screens'})
        for repetition in range(1, 4):
            if len(eligible) == 2:
                outcomes = self.pair(f'confirm-e3-n8-r{repetition}', 'E3', 8, 60000)
            else:
                outcomes = {}
                for backend in eligible:
                    label = 'blink' if backend == 'blink' else 'shared'
                    outcomes[backend] = self.run_cell(f'confirm-e3-n8-r{repetition}-{label}', 'E3', backend, 8, 60000)
            assert all(outcome['reduced']['operational_pass'] for outcome in outcomes.values()), 'survivor operational failure requires review'
        self.pair('maintenance-e2-n28-ms2000', 'E2', 28, 180000, interval_ms=2000)
        completion = {'completed': True, 'source_commit': self.source_commit, 'cases': len(self.outcomes),
                      'eight_tenant_confirmation_eligible': eligible, 'capacity_after': capacity(),
                      'processes': [{'pid': identity['pid'], 'role': identity['role'],
                                     'start_time_ticks': identity['start_time_ticks'],
                                     'same_live_process': process_identity_live(identity)} for identity in self.all_processes]}
        assert all(not identity['same_live_process'] for identity in completion['processes'])
        write_json(self.run_directory / 'completion.json', completion)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    experiment = ObservationExperiment(arguments.run_name, arguments.source_commit, arguments.plan_commit)
    try:
        experiment.compare()
    except Exception as error:
        write_json(experiment.run_directory / 'controller-failure.json', {'reason': repr(error), 'capacity': capacity()})
        raise
