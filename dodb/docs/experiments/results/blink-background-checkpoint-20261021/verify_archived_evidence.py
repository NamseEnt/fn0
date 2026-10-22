import hashlib
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parent
artifacts = root / 'evidence' / 'oci-concurrent'
records = [json.loads(line) for line in (artifacts / 'blink-background-checkpoint-archive-summary.jsonl').read_text().splitlines()]
results = []
for record in records:
    archive = artifacts / Path(record['archive']).name
    assert archive.stat().st_size == record['bytes'], archive
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == record['sha256'], archive
    with tarfile.open(archive, 'r:gz') as contents, tempfile.TemporaryDirectory(prefix='dodb-checkpoint-review-') as temporary:
        members = contents.getmembers()
        assert len(members) == record['files'], archive
        files = {}
        for member in members:
            path = Path(member.name)
            assert member.isfile() and not path.is_absolute() and '..' not in path.parts, member.name
            data = contents.extractfile(member).read()
            files[member.name] = hashlib.sha256(data).hexdigest()
            target = Path(temporary) / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        inventories = [name for name in files if name.endswith('/evidence-file-sha256.json')]
        report = {'archive': archive.name, 'sha256_verified': True, 'files': len(members)}
        for name in inventories:
            run_root = Path(temporary) / Path(name).parent
            inventory = json.loads((Path(temporary) / name).read_text())
            prefix = str(Path(name).parent) + '/'
            assert set(files) == {prefix + filename for filename in inventory} | {name}
            assert all(files[prefix + filename] == digest for filename, digest in inventory.items())
            original_summary = (run_root / 'matrix-summary.json').read_bytes()
            summary = json.loads(original_summary)
            extracted = artifacts / ('final-matrix-summary.json' if summary['source_sha'].startswith('d240416f0') else 'initial-matrix-summary.json')
            assert extracted.read_bytes() == original_summary
            result = subprocess.run([sys.executable, str(root / 'summarize_matrix.py'), str(run_root)], capture_output=True, text=True)
            assert result.returncode == 0, result.stderr
            assert (run_root / 'matrix-summary.json').read_bytes() == original_summary
            report.update({'file_inventory_verified': True, 'offline_summary_exit_code': result.returncode, 'offline_summary_matches': True, 'cells': len(summary['cells']), 'source_sha': summary['source_sha']})
        results.append(report)
(artifacts / 'archive-verification.json').write_text(json.dumps(results, indent=2) + '\n')
(artifacts / 'SHA256SUMS').write_text(''.join(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.name + '\n' for path in sorted(artifacts.iterdir()) if path.is_file() and path.name != 'SHA256SUMS'))
print(json.dumps(results, indent=2))
