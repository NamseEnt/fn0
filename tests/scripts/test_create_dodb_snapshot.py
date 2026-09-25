import importlib.util
import json
import os
import pathlib
import sqlite3
import subprocess
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("create_dodb_snapshot", ROOT / "scripts/create-dodb-snapshot.py")
SNAPSHOT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SNAPSHOT)


class SnapshotCreationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temporary.name)
        self.command_dir = self.root / "bin"
        self.command_dir.mkdir()
        self.call_log = self.root / "calls"
        self.script = self.command_dir / "turso"
        self.script.write_text(
            "#!/usr/bin/env python3\n"
            "import os, pathlib, sys\n"
            "if sys.argv[1:3] == ['db', 'list']:\n"
            " print('NAME GROUP URL\\nfn0-control forte-db url\\n00000001 forte-db url')\n"
            " raise SystemExit(0)\n"
            "database = sys.argv[3]\n"
            "with open(os.environ['CALL_LOG'], 'a') as log: log.write(database + '\\n')\n"
            "if database == os.environ.get('MISSING_DATABASE'):\n"
            " print(f'Error: failed to find database: database {database} not found', file=sys.stderr); raise SystemExit(1)\n"
            "if database == os.environ.get('RETRY_DATABASE') and not pathlib.Path(os.environ['RETRY_MARKER']).exists():\n"
            " pathlib.Path(os.environ['RETRY_MARKER']).touch(); print('HTTP 502', file=sys.stderr); raise SystemExit(1)\n"
            "output = sys.argv[sys.argv.index('--output-file') + 1]\n"
            "pathlib.Path(output).write_bytes(pathlib.Path(os.environ['DUMP_DIR'], database + '.sqlite').read_bytes())\n",
            encoding="utf-8",
        )
        self.script.chmod(0o755)
        dump_dir = self.root / "dumps"
        dump_dir.mkdir()
        self.dumps = dump_dir
        self.write_dump("fn0-control", [
            ("ProjectDoc/fn0-control", "", b'{"project_id":"fn0-control"}'),
            ("ProjectDoc/00000001", "doc", b'{"project_id":"00000001"}'),
        ])
        self.write_dump("00000001", [("binary", "key", bytes([0, 255, 1]))])
        self.environment = {
            **os.environ,
            "PATH": f"{self.command_dir}:{os.environ['PATH']}",
            "CALL_LOG": str(self.call_log),
            "DUMP_DIR": str(dump_dir),
            "RETRY_DATABASE": "00000001",
            "RETRY_MARKER": str(self.root / "retried"),
            "MISSING_DATABASE": "00000002",
        }

    def tearDown(self):
        self.temporary.cleanup()

    def write_dump(self, database, rows):
        path = self.dumps / f"{database}.sqlite"
        path.unlink(missing_ok=True)
        with sqlite3.connect(path) as connection:
            connection.execute("CREATE TABLE docs (pk TEXT NOT NULL, sk TEXT NOT NULL, data BLOB NOT NULL, version INTEGER NOT NULL, PRIMARY KEY (pk, sk))")
            for pk, sk, data in rows:
                connection.execute("INSERT INTO docs VALUES (?, ?, ?, 1)", (pk, sk, data))

    def test_capture_discovers_control_projects_retries_and_protects_files(self):
        output = self.root / "snapshot"
        old_environment = os.environ.copy()
        os.environ.update(self.environment)
        try:
            SNAPSHOT.create_snapshot(output, 5)
        finally:
            os.environ.clear()
            os.environ.update(old_environment)
        manifest = json.loads((output / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest["active_project_ids"], ["00000001"])
        self.assertEqual(manifest["project_ids"], ["fn0-control", "00000001"])
        self.assertEqual(manifest["databases"][1]["row_count"], 1)
        self.assertEqual(manifest["databases"][1]["data_bytes"], 3)
        with sqlite3.connect(f"file:{output / '00000001.sqlite'}?mode=ro", uri=True) as connection:
            self.assertEqual(connection.execute("SELECT pk, sk, data FROM docs ORDER BY pk, sk").fetchone(), ("binary", "key", bytes([0, 255, 1])))
        self.assertEqual(os.stat(output).st_mode & 0o777, 0o700)
        self.assertEqual(os.stat(output / "manifest.json").st_mode & 0o777, 0o600)
        self.assertEqual(os.stat(output / "00000001.sqlite").st_mode & 0o777, 0o600)
        self.assertEqual(self.call_log.read_text(encoding="utf-8").splitlines(), ["fn0-control", "00000001", "00000001"])

    def test_invalid_project_id_fails_before_project_export(self):
        self.write_dump("fn0-control", [("ProjectDoc/not-canonical", "doc", b'{"project_id":"BAD"}')])
        output = self.root / "invalid"
        old_environment = os.environ.copy()
        os.environ.update(self.environment)
        try:
            with self.assertRaisesRegex(RuntimeError, "invalid or noncanonical project ID"):
                SNAPSHOT.create_snapshot(output, 1)
        finally:
            os.environ.clear()
            os.environ.update(old_environment)
        self.assertFalse((output / "BAD.sqlite").exists())
        self.assertFalse(output.exists())

    def test_missing_turso_database_is_recorded_as_an_empty_source(self):
        self.write_dump(
            "fn0-control",
            [
                ("ProjectDoc/fn0-control", "", b'{"project_id":"fn0-control"}'),
                ("ProjectDoc/00000001", "doc", b'{"project_id":"00000001"}'),
                ("ProjectDoc/00000002", "doc", b'{"project_id":"00000002"}'),
            ],
        )
        output = self.root / "missing-source"
        old_environment = os.environ.copy()
        os.environ.update(self.environment)
        try:
            SNAPSHOT.create_snapshot(output, 5)
        finally:
            os.environ.clear()
            os.environ.update(old_environment)
        manifest = json.loads((output / "manifest.json").read_text(encoding="utf-8"))
        missing = next(item for item in manifest["databases"] if item["project_id"] == "00000002")
        self.assertEqual(missing["source_state"], "missing_database")
        self.assertEqual(missing["row_count"], 0)
        self.assertEqual(missing["data_bytes"], 0)
        with sqlite3.connect(f"file:{output / missing['filename']}?mode=ro", uri=True) as connection:
            self.assertEqual(connection.execute("SELECT COUNT(*) FROM docs").fetchone(), (0,))

    def test_snapshot_output_inside_repository_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "outside the repository"):
            SNAPSHOT.create_snapshot(ROOT / "snapshot-not-for-production", 1)


if __name__ == "__main__":
    unittest.main()
