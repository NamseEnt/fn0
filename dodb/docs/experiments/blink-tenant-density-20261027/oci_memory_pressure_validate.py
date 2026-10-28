import json
from pathlib import Path
import sys

sys.path.insert(0, '/home/opc/tenant-density-harness-20261027')
from oci_harness_build import build

ROOT = Path('/home/opc/tenant-density-harness-20261027')
BASE = ['--locked', '--offline', '--release', '-p', 'dodb-rocksdb']
BINS = ['--bin', 'tenant_density_workload', '--bin', 'memory_diagnostic_worker']
CHECKS = [
    ('pressure-clean-tests', ['test', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--', '--test-threads=1']),
    ('pressure-clean-clippy', ['clippy', *BASE, '--lib', *BINS, '--test', 'read_contract', '--test', 'transaction_contract', '--no-deps']),
    ('pressure-clean-a1-link', ['build', *BASE, *BINS, '--features', 'dodb-storage/blink-writer-pin-reuse']),
    ('pressure-clean-default-link', ['build', *BASE, *BINS]),
]

outcomes = []
for name, arguments in CHECKS:
    outcome = {'name': name, 'exit_code': build(name, arguments)}
    outcomes.append(outcome)
    (ROOT / 'pressure-clean-validation-summary.json').write_text(json.dumps(outcomes, indent=2) + '\n')
    print(json.dumps(outcome), flush=True)
    if outcome['exit_code'] != 0:
        raise SystemExit(outcome['exit_code'])
