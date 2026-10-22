import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path('/bench/zfs/db/blink-background-checkpoint-20261022-threshold')
source = root / 'source'

def capture(arguments, cwd=None):
    result = subprocess.run(arguments, cwd=cwd, text=True, capture_output=True, check=True)
    return result.stdout

assert (root / 'matrix-complete.txt').exists()
source_sha = capture(['git', 'rev-parse', 'HEAD'], source).strip()
assert source_sha == 'd240416f0e0efafa7c08bb1e868387ebd0d894bc'
status = capture(['git', 'status', '--porcelain'], source)
assert not status, status
archive = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')
evidence = {
    'captured_at_utc': datetime.now(timezone.utc).isoformat(),
    'source_sha': source_sha,
    'source_alias': str(source),
    'source_canonical': str(source.resolve()),
    'source_status_porcelain': status,
    'host': capture(['uname', '-a']),
    'cpu': capture(['lscpu']),
    'filesystem': capture(['findmnt', '-T', '/bench/zfs/db']),
    'zfs_properties': capture(['zfs', 'get', '-H', '-o', 'property,value', 'sync,compression,recordsize,logbias', 'dodbbench/db']),
    'memory': Path('/proc/meminfo').read_text(),
    'rust': capture(['rustup', 'run', '1.97.1', 'rustc', '-Vv']),
    'cargo': capture(['rustup', 'run', '1.97.1', 'cargo', '-V']),
    'rocksdb_source': capture(['git', 'rev-parse', 'HEAD'], archive.parent).strip(),
    'rocksdb_archive_sha256': hashlib.sha256(archive.read_bytes()).hexdigest(),
    'binaries': json.loads((root / 'binary-provenance.json').read_text()),
    'shared_target_canonical': str(Path('/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release').resolve()),
}
(root / 'final-source-environment.json').write_text(json.dumps(evidence, indent=2) + '\n')
print(json.dumps({'source_sha': source_sha, 'clean': not status, 'zfs_properties': evidence['zfs_properties']}))
