import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'target'
DESTINATION = Path('/bench/zfs/db/tenant-density-build-cache-20261029/target')
PREVIOUS = ROOT / 'target-on-boot-before-20261029'
RECORDS = ROOT / 'observations-cache-relocation-20261029'


def write_json(name, value):
    (RECORDS / name).write_text(json.dumps(value, indent=2) + '\n')


def digest(file_path):
    checksum = hashlib.sha256()
    with file_path.open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024**2), b''):
            checksum.update(chunk)
    return checksum.hexdigest()


def capacity():
    result = {}
    for name, directory in [('boot', ROOT), ('benchmark', Path('/bench/zfs/db'))]:
        filesystem = os.statvfs(directory)
        result[name + '_free_bytes'] = filesystem.f_bavail * filesystem.f_frsize
    return result


def identities():
    paths = [SOURCE / 'release' / name for name in ['tenant_density_workload', 'memory_diagnostic_worker']]
    paths.append(Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a'))
    paths.extend(ROOT.glob('capacity/*/bin/tenant_density_workload'))
    paths.extend(ROOT.glob('capacity/*/bin/memory_diagnostic_worker'))
    return {str(file_path): {'sha256': digest(file_path), 'bytes': file_path.stat().st_size} for file_path in paths}


def main():
    assert SOURCE.is_dir() and not SOURCE.is_symlink()
    assert not DESTINATION.exists() and not PREVIOUS.exists()
    RECORDS.mkdir(exist_ok=False)
    processes = subprocess.check_output(['ps', '-eo', 'pid,comm'], text=True)
    assert not any(line.split()[-1] in ['cargo', 'rustc', 'cc1plus'] for line in processes.splitlines()[1:])
    cache_bytes = int(subprocess.check_output(['du', '-sb', str(SOURCE)], text=True).split()[0])
    before = capacity()
    assert before['benchmark_free_bytes'] - cache_bytes >= 12 * 1024**3
    before_identities = identities()
    write_json('plan.json', {'source': str(SOURCE), 'destination': str(DESTINATION),
                            'previous_duplicate': str(PREVIOUS), 'cache_apparent_bytes': cache_bytes,
                            'capacity_before': before, 'identities_before': before_identities,
                            'processes_before': processes, 'controller_sha256': digest(Path(__file__)),
                            'scope': 'Relocate the complete regenerable build cache; preserve runtime binaries, logs, source, native archive, databases, and failed evidence'})
    DESTINATION.parent.mkdir(exist_ok=False)
    started = time.monotonic()
    with (RECORDS / 'copy.log').open('x') as output:
        subprocess.run(['rsync', '-aH', '--numeric-ids', str(SOURCE) + '/', str(DESTINATION) + '/'],
                       stdout=output, stderr=subprocess.STDOUT, check=True)
    verification = subprocess.run(['rsync', '-aHnci', '--numeric-ids', str(SOURCE) + '/', str(DESTINATION) + '/'],
                                  capture_output=True, text=True, check=True)
    (RECORDS / 'verification.log').write_text(verification.stdout + verification.stderr)
    assert not verification.stdout.strip() and not verification.stderr.strip()
    assert capacity()['benchmark_free_bytes'] >= 12 * 1024**3
    write_json('verified.json', {'all_cache_files_checksum_verified': True,
                                'elapsed_seconds': time.monotonic() - started, 'capacity': capacity()})
    SOURCE.rename(PREVIOUS)
    SOURCE.symlink_to(DESTINATION, target_is_directory=True)
    assert SOURCE.resolve() == DESTINATION
    assert identities() == before_identities
    write_json('activated.json', {'logical_target_path_unchanged': True,
                                 'runtime_and_native_identities_unchanged': True, 'capacity': capacity()})
    assert PREVIOUS.parent == ROOT and PREVIOUS.is_dir() and not PREVIOUS.is_symlink()
    shutil.rmtree(PREVIOUS)
    write_json('result.json', {'completed': True, 'source_symlink': str(SOURCE),
                              'cache_destination': str(DESTINATION), 'capacity_after': capacity(),
                              'identities_after': identities(), 'elapsed_seconds': time.monotonic() - started,
                              'removed_only_checksum_verified_duplicate': str(PREVIOUS)})
    print(json.dumps(json.loads((RECORDS / 'result.json').read_text())), flush=True)


if __name__ == '__main__':
    main()
