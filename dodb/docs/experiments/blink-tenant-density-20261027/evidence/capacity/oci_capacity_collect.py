import argparse
import json
from pathlib import Path
import subprocess
import tarfile

from capacity_review import digest
from oci_capacity import ROOT, SOURCE, capacity, process_identity_live, write_json


def main(arguments):
    postflight = {'collector_sha256': digest(Path(__file__)), 'capacity': capacity(), 'runs': {}}
    for run_name in arguments.runs:
        directory = ROOT / 'capacity' / run_name
        assert (directory / 'completion.json').exists() or (directory / 'controller-failure.json').exists()
        provenance = json.loads((directory / 'provenance.json').read_text())
        binaries = {name: {'sha256': digest(directory / 'bin' / name), 'bytes': (directory / 'bin' / name).stat().st_size}
                    for name in provenance['binaries']}
        assert binaries == provenance['binaries']
        processes = {}
        databases = {}
        groups = {}
        for status_path in directory.glob('*-status.json'):
            status = json.loads(status_path.read_text())
            report_path = directory / (status['case'] + '.json')
            assert digest(report_path) == status['report_sha256']
            report = json.loads(report_path.read_text())
            for identity in report['memory_diagnostic']['processes']:
                key = f"{identity['pid']}:{identity['start_time_ticks']}"
                processes[key] = {'pid': identity['pid'], 'start_time_ticks': identity['start_time_ticks'],
                                  'role': identity['role'], 'same_live_process': process_identity_live(identity)}
            command = json.loads((directory / (status['case'] + '-command.json')).read_text())['command']
            data_directory = Path(command[command.index('--data-dir') + 1])
            databases[status['case']] = {'path': str(data_directory), 'preserved': data_directory.is_dir()}
            group = Path(report['backend_memory_pressure']['configuration']['cgroup_root'])
            groups[status['case']] = {str(file_path.relative_to(group)): file_path.read_text().strip()
                                     for file_path in group.rglob('cgroup.procs')}
        assert all(not identity['same_live_process'] for identity in processes.values())
        assert all(record['preserved'] for record in databases.values())
        assert all(not pids for group in groups.values() for pids in group.values())
        postflight['runs'][run_name] = {'binaries': binaries, 'processes': list(processes.values()),
                                       'databases': databases, 'cgroup_processes': groups}
    revision = subprocess.check_output(['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), 'rev-parse', 'HEAD'], text=True).strip()
    status = subprocess.check_output(['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), 'status', '--porcelain'], text=True).strip()
    assert revision == 'aaa40565529ca92150df5d94e19b02ca93859f71' and not status
    postflight['source_commit'] = revision
    postflight['source_clean'] = True
    postflight['scope'] = 'Post-workload evidence collection; frozen binaries and every database remain on the OCI host'
    receipt_path = ROOT / (arguments.archive_name + '-postflight.json')
    write_json(receipt_path, postflight)
    archive_path = ROOT / (arguments.archive_name + '.tar.gz')
    with tarfile.open(archive_path, 'x:gz') as archive:
        archive.add(receipt_path, arcname='collection/' + receipt_path.name)
        for run_name in arguments.runs:
            directory = ROOT / 'capacity' / run_name
            for file_path in sorted(directory.rglob('*')):
                relative = file_path.relative_to(directory)
                if file_path.is_file() and 'bin' not in relative.parts:
                    archive.add(file_path, arcname=str(Path('runtime') / run_name / relative))
        for name in arguments.extra_files:
            file_path = ROOT / name
            archive.add(file_path, arcname='collection/' + name)
    print(json.dumps({'archive': str(archive_path), 'archive_sha256': digest(archive_path),
                      'bytes': archive_path.stat().st_size, 'runs': arguments.runs,
                      'preserved_database_records': sum(len(record['databases']) for record in postflight['runs'].values())}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--runs', nargs='+', required=True)
    parser.add_argument('--archive-name', required=True)
    parser.add_argument('--extra-files', nargs='*', default=[])
    arguments = parser.parse_args()
    main(arguments)
