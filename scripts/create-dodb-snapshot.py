import argparse
import datetime
import hashlib
import json
import os
import pathlib
import re
import shutil
import sqlite3
import subprocess
import tempfile
import time


FORMAT_VERSION = 1
PROJECT_ID = re.compile(r"^[0-9a-z]{8}$")
HTTP_FAILURE = re.compile(r"\b(?:HTTP\s+|status(?:\s+code)?\s+)?5\d{2}\b", re.IGNORECASE)


class RetryableExportError(RuntimeError):
    pass


def fail(message):
    raise RuntimeError(message)


def digest_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def inspect_database(path):
    connection = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
    try:
        integrity = connection.execute("PRAGMA integrity_check").fetchone()
        if integrity != ("ok",):
            fail(f"SQLite integrity check failed for {path.name}: {integrity!r}")
        table = connection.execute("SELECT sql FROM sqlite_master WHERE type='table' AND name='docs'").fetchone()
        if table is None:
            return 0, 0
        columns = {row[1] for row in connection.execute("PRAGMA table_info(docs)")}
        if not {"pk", "sk", "data", "version"}.issubset(columns):
            fail(f"{path.name} docs table has unexpected columns")
        count, data_bytes = connection.execute("SELECT COUNT(*), COALESCE(SUM(length(data)), 0) FROM docs").fetchone()
        return count, data_bytes
    finally:
        connection.close()


def turso_database_is_listed(database):
    result = subprocess.run(
        ["turso", "db", "list"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", "replace").strip()
        fail(f"could not confirm Turso database absence: {detail}")
    rows = result.stdout.decode("utf-8", "replace").splitlines()
    return any(row.split() and row.split()[0] == database for row in rows[1:])


def create_empty_database(path):
    connection = sqlite3.connect(path)
    try:
        connection.execute(
            "CREATE TABLE docs (pk TEXT NOT NULL, sk TEXT NOT NULL, data BLOB NOT NULL, version INTEGER NOT NULL, PRIMARY KEY (pk, sk))"
        )
        connection.commit()
    finally:
        connection.close()


def export_database(database, output_path, retries):
    last_error = None
    for attempt in range(retries):
        with tempfile.TemporaryDirectory(prefix="fn0-dodb-dump-") as temporary:
            temporary_path = pathlib.Path(temporary)
            sqlite_path = temporary_path / "database.sqlite"
            try:
                result = subprocess.run(
                    ["turso", "db", "export", database, "--output-file", str(sqlite_path)],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=False,
                )
                if result.returncode != 0:
                    detail = result.stderr.decode("utf-8", "replace").strip()
                    message = f"turso database export failed for {database}: {detail}"
                    if HTTP_FAILURE.search(detail) or any(token in detail.lower() for token in ("timeout", "temporarily unavailable", "connection reset")):
                        raise RetryableExportError(message)
                    if re.search(rf"\bdatabase\s+{re.escape(database)}\s+not found\b", detail, re.IGNORECASE):
                        if turso_database_is_listed(database):
                            raise RuntimeError(f"Turso export reported {database} missing, but database list includes it")
                        create_empty_database(sqlite_path)
                        inspect_database(sqlite_path)
                        os.replace(sqlite_path, output_path)
                        return "missing_database"
                    raise RuntimeError(message)
                if not sqlite_path.is_file() or sqlite_path.stat().st_size == 0:
                    raise RuntimeError(f"turso database export was empty for {database}")
                result = subprocess.run(["sqlite3", str(sqlite_path), "PRAGMA wal_checkpoint(TRUNCATE); PRAGMA integrity_check;"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
                if result.returncode != 0:
                    raise RuntimeError(f"SQLite checkpoint or integrity check failed for {database}: {result.stderr.decode('utf-8', 'replace').strip()}")
                if result.stdout.strip().splitlines()[-1:] != [b"ok"]:
                    raise RuntimeError(f"SQLite integrity check returned an unexpected result for {database}")
                for suffix in ("-wal", ".db-wal"):
                    sidecar = pathlib.Path(str(sqlite_path) + suffix)
                    if sidecar.is_file() and sidecar.stat().st_size != 0:
                        raise RuntimeError(f"SQLite export left uncheckpointed WAL data for {database}")
                for suffix in ("-wal", "-shm", ".db-wal", ".db-shm"):
                    sidecar = pathlib.Path(str(sqlite_path) + suffix)
                    sidecar.unlink(missing_ok=True)
                inspect_database(sqlite_path)
                os.replace(sqlite_path, output_path)
                return "present"
            except RetryableExportError as error:
                last_error = error
            except (OSError, RuntimeError):
                raise
        if attempt + 1 < retries:
            time.sleep(min(0.5 * (2 ** attempt), 4.0))
    fail(f"snapshot export failed for {database} after {retries} attempts: {last_error}")


def discover_projects(control_path):
    connection = sqlite3.connect(f"file:{control_path}?mode=ro", uri=True)
    projects = set()
    try:
        if connection.execute("SELECT 1 FROM sqlite_master WHERE type='table' AND name='docs'").fetchone() is None:
            return []
        for pk, sk, data in connection.execute("SELECT pk, sk, data FROM docs WHERE pk LIKE 'ProjectDoc/%' ORDER BY pk, sk"):
            try:
                document = json.loads(data)
            except (TypeError, UnicodeDecodeError, json.JSONDecodeError) as error:
                fail(f"malformed ProjectDoc at ({pk!r}, {sk!r}): {error}")
            project_id = document.get("project_id") if isinstance(document, dict) else None
            if project_id == "fn0-control":
                continue
            if not isinstance(project_id, str) or not PROJECT_ID.fullmatch(project_id):
                fail(f"invalid or noncanonical project ID at ({pk!r}, {sk!r}): {project_id!r}")
            if project_id in projects:
                fail(f"duplicate project ID in fn0-control snapshot: {project_id}")
            projects.add(project_id)
    finally:
        connection.close()
    return sorted(projects)


def _write_snapshot(output, retries):
    control_path = output / "fn0-control.sqlite"
    export_database("fn0-control", control_path, retries)
    project_ids = discover_projects(control_path)
    database_ids = ["fn0-control", *project_ids]
    entries = []
    for database in database_ids:
        filename = f"{database}.sqlite"
        path = output / filename
        source_state = "present"
        if database != "fn0-control":
            source_state = export_database(database, path, retries)
        rows, data_bytes = inspect_database(path)
        os.chmod(path, 0o600)
        entries.append({
            "project_id": database,
            "filename": filename,
            "source_state": source_state,
            "row_count": rows,
            "data_bytes": data_bytes,
            "sha256": digest_file(path),
        })
    manifest = {
        "format_version": FORMAT_VERSION,
        "captured_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "project_ids": database_ids,
        "active_project_ids": project_ids,
        "databases": entries,
    }
    manifest_path = output / "manifest.json"
    with manifest_path.open("x", encoding="utf-8") as destination:
        json.dump(manifest, destination, indent=2, sort_keys=True)
        destination.write("\n")
    os.chmod(manifest_path, 0o600)


def create_snapshot(output, retries):
    output = output.expanduser().absolute()
    repository_root = pathlib.Path(__file__).resolve().parents[1]
    try:
        output.relative_to(repository_root)
    except ValueError:
        pass
    else:
        fail("snapshot output must be outside the repository")
    if output.exists():
        fail(f"snapshot output already exists: {output}")
    output.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    staging = pathlib.Path(tempfile.mkdtemp(prefix=f".{output.name}.partial-", dir=output.parent))
    os.chmod(staging, 0o700)
    try:
        _write_snapshot(staging, retries)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=pathlib.Path)
    parser.add_argument("--retries", type=int, default=5)
    arguments = parser.parse_args()
    if not 1 <= arguments.retries <= 5:
        fail("retries must be between 1 and 5")
    create_snapshot(arguments.output, arguments.retries)


if __name__ == "__main__":
    main()
