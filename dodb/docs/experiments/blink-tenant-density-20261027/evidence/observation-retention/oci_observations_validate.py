import json
from pathlib import Path
import subprocess
import sys


ROOT = Path('/home/opc/tenant-density-harness-20261027')
sys.path.insert(0, str(ROOT))
from oci_harness_build import build


BASE = ['--locked', '--offline', '--release', '-p', 'dodb-rocksdb']
BINS = ['--bin', 'tenant_density_workload', '--bin', 'memory_diagnostic_worker']
CHECKS = [
    ('observations-clean-tests', ['test', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--', '--test-threads=1']),
    ('observations-clean-clippy', ['clippy', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--no-deps']),
    ('observations-clean-a1-link', ['build', *BASE, *BINS, '--features', 'dodb-storage/blink-writer-pin-reuse']),
    ('observations-clean-default-link', ['build', *BASE, *BINS]),
]


def main():
    source = ROOT / 'source'
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip()
    assert revision == '023b18bd8921b593f243f59b9eaa1576125c6572'
    assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=source, text=True).strip()
    assert (ROOT / 'target').is_symlink()
    outcomes = []
    for name, arguments in CHECKS:
        status_path = ROOT / 'builds' / name / 'status.json'
        if status_path.exists():
            provenance = json.loads(status_path.with_name('provenance.json').read_text())
            assert provenance['source_commit'] == revision and provenance['command'][4:] == arguments
            outcome = {'name': name, 'exit_code': json.loads(status_path.read_text())['exit_code'], 'reused_completed_check': True}
        else:
            outcome = {'name': name, 'exit_code': build(name, arguments), 'reused_completed_check': False}
        outcomes.append(outcome)
        (ROOT / 'observations-clean-validation-summary.json').write_text(json.dumps({
            'outcomes': outcomes, 'resolved_target_directory': str((ROOT / 'target').resolve()),
            'source_commit': revision}, indent=2) + '\n')
        print(json.dumps(outcome), flush=True)
        if outcome['exit_code'] != 0:
            return outcome['exit_code']
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
