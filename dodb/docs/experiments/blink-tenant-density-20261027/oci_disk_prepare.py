import json
import os
from pathlib import Path
import shutil
import subprocess
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
OLD_TARGET = Path('/bench/zfs/db/dodb-rocksdb-r1b3a-551471d5/target')
CACHE = Path('/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/sccache')
BOOT_TARGET = Path('/home/opc/dodb/target/debug')
NATIVE = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')


def capacity():
    result = {}
    for name, path in [('boot', ROOT), ('benchmark', OLD_TARGET)]:
        filesystem = os.statvfs(path)
        result[name] = filesystem.f_bavail * filesystem.f_frsize
    return result


def identity(path):
    metadata = path.stat()
    return {'path': str(path), 'bytes': metadata.st_size, 'inode': metadata.st_ino,
            'device': metadata.st_dev, 'mtime_ns': metadata.st_mtime_ns}


def main():
    evidence = ROOT / 'disk-preparation-20261028'
    evidence.mkdir(exist_ok=False)
    candidates = [base / name for base in [OLD_TARGET / 'debug', OLD_TARGET / 'release', BOOT_TARGET]
                  for name in ['deps', 'build', '.fingerprint', 'incremental']]
    candidates.append(CACHE)
    candidates = [path for path in candidates if path.exists()]
    for path in candidates:
        assert path.resolve() == path and path.is_dir() and not path.is_symlink()
    processes = subprocess.check_output(['ps', '-eo', 'pid,comm,args'], text=True)
    (evidence / 'processes-before.txt').write_text(processes)
    for line in processes.splitlines():
        fields = line.split(None, 2)
        if len(fields) < 3:
            continue
        assert fields[1] not in ['cargo', 'rustc', 'cc1', 'cc1plus', 'sccache'], line
        assert not any(str(path) in fields[2] for path in candidates), line
    preserved = [NATIVE]
    for directory in [OLD_TARGET / 'debug', OLD_TARGET / 'release', BOOT_TARGET]:
        preserved.extend(path for path in directory.iterdir()
                         if path.is_file() and path.stat().st_mode & 0o111)
    identities = [identity(path) for path in preserved]
    inventory = []
    for path in candidates:
        size = int(subprocess.check_output(['du', '-sx', '-B1', str(path)], text=True).split()[0])
        inventory.append({'path': str(path), 'allocated_bytes_before': size,
                          'reason': 'Regenerable compiler cache or standard Cargo intermediate directory'})
    plan = {'started_unix_seconds': time.time(), 'capacity_before': capacity(),
            'targets': inventory, 'preserved_runtime_and_native_files': identities,
            'scope': 'Only explicit regenerable caches/intermediates; source, databases, raw reports/logs, runtime binaries and pinned native archive retained'}
    (evidence / 'cleanup-plan.json').write_text(json.dumps(plan, indent=2) + '\n')
    outcomes = []
    for path in candidates:
        shutil.rmtree(path)
        outcomes.append({'path': str(path), 'absent_after': not path.exists(),
                         'capacity_after': capacity(), 'ended_unix_seconds': time.time()})
        (evidence / 'cleanup-progress.json').write_text(json.dumps(outcomes, indent=2) + '\n')
        print(json.dumps(outcomes[-1]), flush=True)
    assert identities == [identity(path) for path in preserved]
    for _attempt_number in range(10):
        subprocess.run(['sync'], check=True)
        current = capacity()
        if current['benchmark'] > plan['capacity_before']['benchmark'] + 20 * 1024**3:
            break
        time.sleep(1)
    result = {'capacity_before': plan['capacity_before'], 'capacity_after': capacity(),
              'preserved_files_unchanged': True, 'deleted_paths': len(outcomes),
              'ended_unix_seconds': time.time()}
    (evidence / 'cleanup-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result), flush=True)


if __name__ == '__main__':
    main()
