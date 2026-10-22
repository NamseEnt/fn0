import hashlib
import json
import os
import tarfile
from pathlib import Path

base = Path('/bench/zfs/db')
roots = [base / 'blink-background-checkpoint-20261022', base / 'blink-background-checkpoint-20261022-threshold']
excluded = {'source', 'bin', 'databases'}
for root in roots:
    assert (root / 'matrix-complete.txt').exists(), root
    paths = []
    for directory, subdirectories, filenames in os.walk(root, followlinks=False):
        subdirectories[:] = [name for name in subdirectories if name not in excluded and not (Path(directory) / name).is_symlink()]
        paths.extend(Path(directory) / name for name in filenames if name != 'evidence-file-sha256.json' and not (Path(directory) / name).is_symlink())
    paths.sort()
    inventory = {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}
    inventory_path = root / 'evidence-file-sha256.json'
    inventory_path.write_text(json.dumps(inventory, indent=2) + '\n')
    paths.append(inventory_path)
    archive = base / (root.name + '-evidence.tar.gz')
    with tarfile.open(archive, 'w:gz') as output:
        for path in paths:
            output.add(path, arcname=str(path.relative_to(base)), recursive=False)
    print(json.dumps({'archive': str(archive), 'bytes': archive.stat().st_size, 'files': len(paths), 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest()}), flush=True)
run_root = base / 'dodb-rocksdb-build-runs'
paths = [path for directory in sorted(run_root.glob('background-checkpoint-*')) for path in directory.rglob('*') if path.is_file() and not path.is_symlink()]
archive = base / 'blink-background-checkpoint-build-provenance.tar.gz'
with tarfile.open(archive, 'w:gz') as output:
    for path in sorted(paths):
        output.add(path, arcname=str(path.relative_to(base)), recursive=False)
print(json.dumps({'archive': str(archive), 'bytes': archive.stat().st_size, 'files': len(paths), 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest()}), flush=True)
