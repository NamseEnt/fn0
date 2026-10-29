import argparse
import json
from pathlib import Path
import shutil
import subprocess

from capacity_review import digest
from oci_capacity import Experiment, ROOT, SOURCE, capacity, process_identity_live, write_json


BASELINE = 'aaa40565529ca92150df5d94e19b02ca93859f71'
CANDIDATE = '023b18bd8921b593f243f59b9eaa1576125c6572'


def git(arguments):
    return subprocess.check_output(['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), *arguments], text=True).strip()


class ControlExperiment(Experiment):
    def __init__(self, run_name, plan_commit):
        reference = ROOT / 'capacity/capacity-aaa405655-r2'
        original = json.loads((reference / 'provenance.json').read_text())
        assert original['source_commit'] == BASELINE
        super().__init__(run_name, BASELINE, plan_commit)
        provenance_path = self.run_directory / 'provenance.json'
        provenance = json.loads(provenance_path.read_text())
        binaries = {}
        for name, identity in original['binaries'].items():
            source_binary = reference / 'bin' / name
            assert digest(source_binary) == identity['sha256']
            destination = self.binaries / name
            shutil.copy2(source_binary, destination)
            binaries[name] = {'sha256': digest(destination), 'bytes': destination.stat().st_size}
        assert binaries == original['binaries']
        provenance.update({'binaries': binaries, 'control_controller_sha256': digest(Path(__file__)),
                           'archived_binary_source': str(reference / 'bin'),
                           'resolved_build_target': str((ROOT / 'target').resolve()),
                           'scope': 'New current-host cadence control using unchanged archived baseline binaries; no rebuild or overwrite of prior results'})
        write_json(provenance_path, provenance)

    def compare(self):
        for interval in [1000, 100]:
            outcomes = self.pair(f'sampling-e2-n1-ms{interval}', 'E2', 1, 60000, interval_ms=interval)
            assert all(outcome['reduced']['operational_pass'] for outcome in outcomes.values())
        completion = {'completed': True, 'source_commit': BASELINE, 'cases': len(self.outcomes),
                      'capacity_after': capacity(),
                      'processes': [{'pid': identity['pid'], 'role': identity['role'],
                                     'start_time_ticks': identity['start_time_ticks'],
                                     'same_live_process': process_identity_live(identity)} for identity in self.all_processes]}
        assert all(not identity['same_live_process'] for identity in completion['processes'])
        write_json(self.run_directory / 'completion.json', completion)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    validation = json.loads((ROOT / 'observations-clean-validation-summary.json').read_text())
    assert len(validation['outcomes']) == 4 and all(check['exit_code'] == 0 for check in validation['outcomes'])
    assert git(['rev-parse', 'HEAD']) == CANDIDATE and not git(['status', '--porcelain'])
    git(['checkout', '--detach', BASELINE])
    experiment = None
    try:
        experiment = ControlExperiment(arguments.run_name, arguments.plan_commit)
        experiment.compare()
    except Exception as error:
        if experiment is not None:
            write_json(experiment.run_directory / 'controller-failure.json', {'reason': repr(error), 'capacity': capacity()})
        raise
    finally:
        assert not git(['status', '--porcelain'])
        git(['checkout', '--detach', CANDIDATE])
