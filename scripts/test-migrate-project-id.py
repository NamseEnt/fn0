import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).parent / "lib" / "project-id-migrate.py"
SPEC = importlib.util.spec_from_file_location("project_id_migrate", MODULE_PATH)
migration = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(migration)


OLD_ID = "namse-mottomite"
NEW_ID = "c9qxk46r"


def record(pk, data, sk="", version=7):
    return {"pk": pk, "sk": sk, "data": migration.json_bytes(data), "version": version}


class FakeTurso:
    def __init__(self, databases):
        self.databases = databases

    def db_url(self, project_id):
        return project_id

    def fetch_all(self, url):
        return [dict(row) for row in self.databases.get(url, [])]

    def pipeline(self, url, statements):
        for sql, args in statements:
            if sql.startswith("CREATE TABLE"):
                self.databases.setdefault(url, [])
            elif sql.startswith("INSERT INTO docs"):
                values = [argument.get("value") for argument in args]
                data = args[2].get("base64")
                row = {"pk": values[0], "sk": values[1], "data": __import__("base64").b64decode(data), "version": int(values[3])}
                if any((existing["pk"], existing["sk"]) == (row["pk"], row["sk"]) for existing in self.databases[url]):
                    raise migration.MigrationError("duplicate key")
                self.databases[url].append(row)


class FakeManagementClient:
    def __init__(self):
        self.posted = False

    def request(self, url, method="GET", headers=None, body=None):
        if method == "POST":
            self.posted = True
            return 400, b'{"error":"database with same name already exists"}'
        if not self.posted:
            return 200, b'{"databases":[]}'
        return 200, ('{"databases":[{"Name":"' + NEW_ID + '","group":"forte-db"}]}').encode()


class ProjectIdMigrationTests(unittest.TestCase):
    def test_project_discovery_marks_canonical_and_noncanonical(self):
        rows = [
            record("ProjectDoc/project_id=abcd1234", {"project_id": "abcd1234"}),
            record("ProjectDoc/project_id=namse-mottomite", {"project_id": OLD_ID}),
            record("ProjectDoc/project_id=fn0-control", {"project_id": "fn0-control"}),
        ]
        discovered = migration.project_ids(rows)
        self.assertEqual([item["project_id"] for item in discovered], ["abcd1234", OLD_ID, "fn0-control"])
        self.assertEqual([item["canonical"] for item in discovered], [True, False, False])
        self.assertTrue(discovered[2]["system"])

    def test_exact_identity_and_resource_substring_are_distinguished(self):
        identity = record(f"ProjectCloudflareConfigDoc/project_id={OLD_ID}", {"project_id": OLD_ID, "public_object_storage_bucket": f"fn0-{OLD_ID}-public"})
        result = migration.classify_row(identity, OLD_ID)
        self.assertEqual(result["classification"], "REKEY")
        self.assertIn({"field": "project_id", "kind": "exact_identity"}, result["occurrences"])
        self.assertIn({"field": "public_object_storage_bucket", "kind": "substring_or_resource"}, result["occurrences"])
        unknown = record("NewUnknownDoc/1", {"owner": OLD_ID})
        self.assertEqual(migration.classify_row(unknown, OLD_ID)["classification"], "UNRECOGNIZED")
        with self.assertRaises(migration.MigrationError):
            migration.control_plan([unknown], OLD_ID, NEW_ID)

    def test_project_database_copy_preserves_binary_data_and_version(self):
        source = [{"pk": "binary", "sk": "one", "data": b"\x00\xff\x80value", "version": 91}]
        fake = FakeTurso({OLD_ID: source})
        result = migration.copy_project_database(fake, OLD_ID, NEW_ID)
        self.assertEqual(fake.databases[NEW_ID], source)
        self.assertEqual(result["copied"], 1)
        self.assertEqual(result["verify"]["different_version"], 0)

    def test_project_database_copy_is_resumable_for_equal_rows(self):
        row = {"pk": "p", "sk": "s", "data": b"\x00raw", "version": 14}
        fake = FakeTurso({OLD_ID: [row], NEW_ID: [dict(row)]})
        result = migration.copy_project_database(fake, OLD_ID, NEW_ID)
        self.assertEqual(result["copied"], 0)
        self.assertEqual(result["skipped"], 1)

    def test_project_database_copy_rejects_different_destination(self):
        source = {"pk": "p", "sk": "s", "data": b"source", "version": 1}
        destination = {"pk": "p", "sk": "s", "data": b"other", "version": 1}
        fake = FakeTurso({OLD_ID: [source], NEW_ID: [destination]})
        with self.assertRaises(migration.MigrationError):
            migration.copy_project_database(fake, OLD_ID, NEW_ID)

    def test_exact_verify_rejects_extra_destination_row(self):
        source = {"pk": "p", "sk": "s", "data": b"source", "version": 1}
        extra = {"pk": "extra", "sk": "s", "data": b"extra", "version": 1}
        fake = FakeTurso({OLD_ID: [source], NEW_ID: [source, extra]})
        with self.assertRaises(migration.MigrationError):
            migration.verify_project_database(fake, OLD_ID, NEW_ID)

    def test_control_row_rekey_preserves_source_version(self):
        row = record(f"ProjectDoc/project_id={OLD_ID}", {"project_id": OLD_ID, "name": "Example"}, version=23)
        plan = migration.classify_row(row, OLD_ID)
        moved = migration.transformed_row(row, OLD_ID, NEW_ID, plan["classification"])
        self.assertEqual(moved["pk"], f"ProjectDoc/project_id={NEW_ID}")
        self.assertEqual(migration.data_json(moved["data"])["project_id"], NEW_ID)
        self.assertEqual(moved["version"], 23)

    def test_user_project_index_identity_is_patched(self):
        row = record("UserDoc/123", {"projects": [{"project_id": OLD_ID, "name": "Example"}]})
        moved = migration.transformed_row(row, OLD_ID, NEW_ID, "PATCH_IDENTITY")
        self.assertEqual(migration.data_json(moved["data"])["projects"][0]["project_id"], NEW_ID)
        self.assertEqual(moved["version"], row["version"] + 1)

    def test_manifest_map_key_is_renamed(self):
        row = record("WorkerManifestDoc", {"project_manifests": {OLD_ID: {"domain": f"{OLD_ID}.example.com"}}})
        moved = migration.transformed_row(row, OLD_ID, NEW_ID, "PATCH_IDENTITY")
        data = migration.data_json(moved["data"])
        self.assertIn(NEW_ID, data["project_manifests"])
        self.assertEqual(data["project_manifests"][NEW_ID]["domain"], f"{OLD_ID}.example.com")

    def test_cloudflare_resource_names_are_preserved(self):
        row = record(f"ProjectCloudflareConfigDoc/project_id={OLD_ID}", {
            "project_id": OLD_ID,
            "private_object_storage_bucket": f"fn0-{OLD_ID}-private",
            "public_object_storage_hostname": f"{OLD_ID}.objects.example.com",
        })
        moved = migration.transformed_row(row, OLD_ID, NEW_ID, "REKEY")
        data = migration.data_json(moved["data"])
        self.assertEqual(data["project_id"], NEW_ID)
        self.assertEqual(data["private_object_storage_bucket"], f"fn0-{OLD_ID}-private")
        self.assertEqual(data["public_object_storage_hostname"], f"{OLD_ID}.objects.example.com")

    def test_transient_rows_are_deleted(self):
        row = record("WebSocketConnectionDoc/connection-1", {"project_id": OLD_ID})
        self.assertEqual(migration.classify_row(row, OLD_ID)["classification"], "DELETE_TRANSIENT")
        self.assertIsNone(migration.transformed_row(row, OLD_ID, NEW_ID, "DELETE_TRANSIENT"))

    def test_websocket_singleton_config_identity_is_rekeyed(self):
        row = record(
            f"WebSocketSingletonConfigDoc/project_id={OLD_ID}",
            {"project_id": OLD_ID, "code_version": 7, "declarations": []},
            "code_version=7",
        )
        classification = migration.classify_row(row, OLD_ID)["classification"]
        moved = migration.transformed_row(row, OLD_ID, NEW_ID, classification)
        self.assertEqual(classification, "REKEY")
        self.assertEqual(moved["pk"], f"WebSocketSingletonConfigDoc/project_id={NEW_ID}")
        self.assertEqual(migration.data_json(moved["data"])["project_id"], NEW_ID)

    def test_apply_requires_explicit_flag(self):
        with self.assertRaises(migration.MigrationError):
            migration.main(["apply", "--old-id", OLD_ID, "--new-id", NEW_ID])

    def test_turso_duplicate_database_response_matches_production_semantics(self):
        client = FakeManagementClient()
        turso = object.__new__(migration.Turso)
        turso.secrets = {"outputs": {}}
        turso.api_token = "mock"
        turso.org_slug = "namse"
        turso.group_name = "forte-db"
        turso.client = client
        self.assertFalse(turso.create_database(NEW_ID))

    def test_turso_duplicate_database_in_another_group_fails(self):
        client = FakeManagementClient()
        client.request = lambda url, method="GET", headers=None, body=None: (
            (400, b'{"error":"database with same name already exists"}')
            if method == "POST"
            else (200, ("{\"databases\":[{\"Name\":\"" + NEW_ID + "\",\"group\":\"other\"}]}").encode())
        )
        turso = object.__new__(migration.Turso)
        turso.secrets = {"outputs": {}}
        turso.api_token = "mock"
        turso.org_slug = "namse"
        turso.group_name = "forte-db"
        turso.client = client
        with self.assertRaises(migration.MigrationError):
            turso.create_database(NEW_ID)


if __name__ == "__main__":
    unittest.main()
