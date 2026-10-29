import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
NATIVE = Path('/bench/zfs/db/product-build/rocksdb-source')
RUSTUP = '/home/opc/.cargo/bin/rustup'
BASE = 'a6fc9db7b15b4c1845381989c3f5cd5be6c9f87c'


def command(arguments, cwd=None):
    return subprocess.check_output(arguments, cwd=cwd, text=True).strip()


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def capacity():
    memory = {}
    for line in Path('/proc/meminfo').read_text().splitlines():
        name, value = line.split(':', 1)
        if name in ['MemTotal', 'MemAvailable', 'SwapFree']:
            memory[name] = int(value.split()[0]) * 1024
    free_bytes = {}
    for name, path in [('boot', ROOT), ('benchmark', Path('/bench/zfs/db'))]:
        filesystem = os.statvfs(path)
        free_bytes[name] = filesystem.f_bavail * filesystem.f_frsize
    return {'memory_bytes': memory, 'filesystem_free_bytes': free_bytes}


def bootstrap():
    if not SOURCE.exists():
        subprocess.run(['git', 'clone', '--shared', '--no-checkout', '/bench/zfs/db/backend-hotpaths-20261026/source', str(SOURCE)], check=True)
        subprocess.run(['git', 'fetch', str(ROOT / 'base.bundle'), 'HEAD'], cwd=SOURCE, check=True)
        subprocess.run(['git', 'sparse-checkout', 'init', '--no-cone'], cwd=SOURCE, check=True)
        subprocess.run(['git', 'sparse-checkout', 'set', '--no-cone', '/*', '!/dodb/docs/'], cwd=SOURCE, check=True)
        subprocess.run(['git', 'checkout', '--detach', BASE], cwd=SOURCE, check=True)
    assert command(['git', 'rev-parse', 'HEAD'], SOURCE) == BASE
    assert not command(['git', 'status', '--porcelain'], SOURCE)
    print(json.dumps({'source': str(SOURCE), 'source_commit': BASE, 'capacity': capacity()}))


def build(run_name, cargo_arguments):
    run_directory = ROOT / 'builds' / run_name
    run_directory.mkdir(parents=True, exist_ok=False)
    temporary = ROOT / 'tmp'
    temporary.mkdir(exist_ok=True)
    preflight = capacity()
    assert preflight['filesystem_free_bytes']['benchmark'] >= 10 * 1024**3
    assert preflight['filesystem_free_bytes']['boot'] >= 1024**3
    assert preflight['memory_bytes']['MemAvailable'] >= 1500 * 1024**2
    native_commit = command(['git', '-C', str(NATIVE), 'rev-parse', 'HEAD'])
    assert native_commit == 'abeebd9630f11bd08c28b7bd43c7bdfc62050654'
    environment = os.environ.copy()
    environment.update({
        'DODB_ROCKSDB_DIR': str(NATIVE),
        'CARGO_TARGET_DIR': str(ROOT / 'target'),
        'CARGO_INCREMENTAL': '0',
        'CARGO_BUILD_JOBS': '2',
        'CARGO_PROFILE_RELEASE_DEBUG': '0',
        'CARGO_PROFILE_RELEASE_STRIP': 'debuginfo',
        'TMPDIR': str(temporary),
    })
    environment.pop('RUSTC_WRAPPER', None)
    invocation = [RUSTUP, 'run', '1.97.1', 'cargo', *cargo_arguments]
    provenance = {
        'controller_sha256': digest(Path(__file__).resolve()),
        'command': invocation,
        'cwd': str(SOURCE),
        'source_commit': command(['git', 'rev-parse', 'HEAD'], SOURCE),
        'source_status': command(['git', 'status', '--porcelain'], SOURCE),
        'native_commit': native_commit,
        'native_archive_sha256': digest(NATIVE / 'librocksdb.a'),
        'toolchain': command([RUSTUP, 'run', '1.97.1', 'rustc', '-Vv']).splitlines(),
        'environment': {name: environment[name] for name in ['DODB_ROCKSDB_DIR', 'CARGO_TARGET_DIR', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'CARGO_PROFILE_RELEASE_DEBUG', 'CARGO_PROFILE_RELEASE_STRIP', 'TMPDIR']},
        'preflight': preflight,
        'started_unix_seconds': time.time(),
        'scope': 'Build and correctness validation only; no performance comparison.',
    }
    (run_directory / 'provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    with (run_directory / 'cargo.log').open('w') as output:
        result = subprocess.run(invocation, cwd=SOURCE, env=environment, stdout=output, stderr=subprocess.STDOUT)
    status = {'exit_code': result.returncode, 'ended_unix_seconds': time.time(), 'capacity': capacity()}
    (run_directory / 'status.json').write_text(json.dumps(status, indent=2) + '\n')
    return result.returncode


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['bootstrap', 'build', 'start'])
    parser.add_argument('--run-name', default='bootstrap-base-worker')
    parser.add_argument('cargo_arguments', nargs=argparse.REMAINDER)
    options = parser.parse_args()
    if options.mode == 'bootstrap':
        bootstrap()
        return 0
    arguments = options.cargo_arguments
    if arguments[:1] == ['--']:
        arguments = arguments[1:]
    if options.mode == 'build':
        return build(options.run_name, arguments)
    wrapper_output = ROOT / f'{options.run_name}-launcher.log'
    with wrapper_output.open('x') as output:
        child = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), '--run-name', options.run_name, 'build', '--', *arguments], stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
    (ROOT / f'{options.run_name}-pid.txt').write_text(str(child.pid) + '\n')
    print(json.dumps({'pid': child.pid, 'run_name': options.run_name, 'launcher_log': str(wrapper_output)}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
