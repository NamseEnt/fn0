import hashlib
import json
import os
import re
import subprocess
from pathlib import Path

root = Path('/bench/zfs/db/blink-background-checkpoint-20261022-threshold')
source = root / 'source'
target = Path('/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release').resolve()
log = Path('/bench/zfs/db/dodb-rocksdb-build-runs/background-checkpoint-d240416f0-tests/cargo.log')
output = root / 'validation'
output.mkdir(exist_ok=True)
selected = {'dodb_storage', 'dodb_server', 'dodb_rocksdb', 'background_checkpoint', 'blink_wal_checkpoint', 'background_checkpoint_sigkill', 'segmented_checkpoint_sigkill', 'segmented_wal_sigkill', 'phase2_recovery', 'process_crash_recovery', 'phase6_quic', 'transaction_contract', 'read_contract', 'contract_history', 'service_workload', 'unconditional_write_workload'}
binaries = {}
for line in log.read_text().splitlines():
    match = re.search(r'Executable .*\(([^)]+)\)', line)
    if match is None:
        continue
    path = Path(match.group(1))
    if not path.is_absolute():
        path = source / path
    path = path.resolve()
    name = path.name.rsplit('-', 1)[0]
    if name in selected:
        assert target in path.parents
        binaries[name] = path
assert selected == set(binaries), selected - set(binaries)
revision = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
assert not subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain'], text=True).strip()
environment = os.environ.copy()
temporary = Path('/bench/zfs/db/bgck-tests-d240')
temporary.mkdir(exist_ok=False)
environment['TMPDIR'] = str(temporary)
environment['DODB_TEST_DATA_DIR'] = str(temporary)
environment['CARGO_BIN_EXE_phase2-crash-child'] = str(target / 'phase2-crash-child')
environment['CARGO_BIN_EXE_process-crash-recovery-child'] = str(target / 'process-crash-recovery-child')
auxiliary = {}
for variable in ['CARGO_BIN_EXE_phase2-crash-child', 'CARGO_BIN_EXE_process-crash-recovery-child']:
    path = Path(environment[variable])
    assert path.is_file(), path
    auxiliary[variable] = {'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
(output / 'auxiliary-binaries.json').write_text(json.dumps(auxiliary, indent=2) + '\n')
previous = output / 'commands.json'
records = json.loads(previous.read_text()) if previous.exists() else []
if records and not (output / 'commands-initial.json').exists():
    (output / 'commands-initial.json').write_bytes(previous.read_bytes())
for name, path in binaries.items():
    successful = [record for record in records if record['name'] == name and record.get('exit_code') == 0]
    if successful:
        assert successful[-1]['source_sha'] == revision
        assert successful[-1]['sha256'] == hashlib.sha256(path.read_bytes()).hexdigest()
        print('PRESERVED', name, successful[-1]['counts'], flush=True)
        continue
    attempt = sum(record['name'] == name for record in records)
    log_name = name + ('.log' if attempt == 0 else f'.retry-{attempt}.log')
    record = {'name': name, 'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'source_sha': revision, 'argv': [str(path), '--test-threads=1'], 'log_name': log_name, 'auxiliary_binaries': auxiliary}
    records.append(record)
    (output / 'commands.json').write_text(json.dumps(records, indent=2) + '\n')
    print('START', name, flush=True)
    with (output / log_name).open('wb') as stream:
        result = subprocess.run(record['argv'], stdout=stream, stderr=subprocess.STDOUT, cwd=source, env=environment)
    record['exit_code'] = result.returncode
    counts = re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', (output / log_name).read_text(errors='replace'))
    if counts:
        record['counts'] = dict(zip(['passed', 'failed', 'ignored'], map(int, counts[-1])))
    (output / 'commands.json').write_text(json.dumps(records, indent=2) + '\n')
    print('END', name, result.returncode, record.get('counts'), flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)
assert not subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain'], text=True).strip()
accepted = [next(record for record in reversed(records) if record['name'] == name and record.get('exit_code') == 0) for name in selected]
summary = {'source_sha': revision, 'binaries': len(accepted), 'passed': sum(record.get('counts', {}).get('passed', 0) for record in accepted), 'failed': 0, 'ignored': sum(record.get('counts', {}).get('ignored', 0) for record in accepted), 'rejected_attempts_preserved': sum(record.get('exit_code', 0) != 0 for record in records), 'completed': True}
(output / 'validation-complete.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary), flush=True)
