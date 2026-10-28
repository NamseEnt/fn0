import argparse
import json
from pathlib import Path
import shutil

from capacity_review import digest
import oci_capacity
from oci_capacity import Experiment, ROOT, process_identity_live, write_json
from operational_review import reduce_report


class Continuation(Experiment):
    def __init__(self, original_run, run_name, plan_commit):
        original_directory = ROOT / 'capacity' / original_run
        original_provenance = json.loads((original_directory / 'provenance.json').read_text())
        assert not (original_directory / 'completion.json').exists()
        assert (original_directory / 'controller-failure.json').exists()
        super().__init__(run_name, original_provenance['source_commit'], plan_commit)
        for file_path in original_directory.iterdir():
            if file_path.is_file():
                destination_name = 'original-' + file_path.name if file_path.name in ['provenance.json', 'controller-failure.json'] else file_path.name
                shutil.copy2(file_path, self.run_directory / destination_name)
        provenance = json.loads((self.run_directory / 'provenance.json').read_text())
        assert provenance['binaries'] == original_provenance['binaries']
        provenance.update({'plan_commit': plan_commit, 'continuation_from': original_run,
                           'continuation_controller_sha256': digest(Path(__file__)),
                           'operational_failure_reducer_sha256': digest(Path(__file__).with_name('operational_review.py')),
                           'original_plan_commit': original_provenance['plan_commit'],
                           'scope': 'Composite evidence view; existing cells reused byte-for-byte, new cells continue the original frozen sequence'})
        write_json(self.run_directory / 'provenance.json', provenance)
        self.replayed = {}
        original_outcomes = json.loads((original_directory / 'summary.json').read_text())
        input_review = []
        for original in original_outcomes:
            outcome = dict(original)
            case = outcome['case']
            report_path = self.run_directory / (case + '.json')
            assert digest(report_path) == outcome['report_sha256']
            report = json.loads(report_path.read_text())
            self.all_processes.extend(report['memory_diagnostic']['processes'])
            assert not any(process_identity_live(identity) for identity in self.all_processes)
            outcome['reduced'] = reduce_report(report, self.source_commit, (self.run_directory / (case + '.log')).read_text())
            if 'review_error' in outcome:
                outcome['original_review_error'] = outcome.pop('review_error')
            outcome['reused_from_run'] = original_run
            write_json(self.run_directory / (case + '-status.json'), outcome)
            self.replayed[case] = outcome
            input_review.append({'case': case, 'report_sha256': digest(report_path),
                                 'original_status_sha256': digest(original_directory / (case + '-status.json')),
                                 'operational_pass': outcome['reduced']['operational_pass'],
                                 'failure_kind': outcome['reduced'].get('failure_kind'),
                                 'runtime_replayed': False})
        self.outcomes = list(self.replayed.values())
        write_json(self.run_directory / 'summary.json', self.outcomes)
        write_json(self.run_directory / 'continuation-input-review.json', input_review)
        oci_capacity.reduce_report = self.review_current_report

    def review_current_report(self, report, source_commit):
        case = Path(report['effective_configuration']['backend_memory_budget']['cgroup_root']).name
        result = reduce_report(report, source_commit, (self.run_directory / (case + '.log')).read_text())
        if result.get('operational_timeout'):
            self.operational_failures.add(('E2' if result['initial_rows'] == 2048 else 'E3', result['backend'], result['registered']))
        return result

    def run_cell(self, case, stage, backend, tenants, duration_ms, interval_ms=1000, active=None, idle=0):
        if case in self.replayed:
            outcome = self.replayed[case]
            assert outcome['backend'] == backend and outcome['stage'] == stage
            assert outcome['tenants'] == tenants and outcome['active'] == (tenants if active is None else active)
            assert outcome['duration_ms'] == duration_ms and outcome['sample_interval_ms'] == interval_ms
            reduced = outcome['reduced']
            if reduced['resource_failure'] or reduced.get('operational_timeout'):
                self.operational_failures.add((stage, backend, tenants))
            return outcome
        return super().run_cell(case, stage, backend, tenants, duration_ms, interval_ms, active, idle)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--original-run', required=True)
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    experiment = Continuation(arguments.original_run, arguments.run_name, arguments.plan_commit)
    try:
        experiment.execute()
        completion_path = experiment.run_directory / 'completion.json'
        completion = json.loads(completion_path.read_text())
        completion['reused_existing_cases'] = list(experiment.replayed)
        completion['executed_new_cases'] = [outcome['case'] for outcome in experiment.outcomes if outcome['case'] not in experiment.replayed]
        write_json(completion_path, completion)
    except Exception as error:
        write_json(experiment.run_directory / 'continuation-failure.json', {'reason': repr(error)})
        raise
