import argparse
import base64
import copy
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request


CANONICAL_ID = re.compile(r"^[0-9a-z]{8}$")
EXTERNAL_FIELDS = {
    "ProjectCloudflareConfigDoc": {
        "frontend_asset_hostname",
        "public_object_storage_hostname",
        "private_object_storage_bucket",
        "public_object_storage_bucket",
        "frontend_asset_bucket",
        "rendered_html_cache_bucket",
    },
    "WorkerManifestDoc": {
        "domain",
        "private_object_storage_bucket",
        "public_object_storage_bucket",
        "public_object_storage_base_url",
    },
    "WorkerCertManifestDoc": {"cert_pem"},
}
PRESERVED_VALUE_FIELDS = {
    "ProjectDoc": {"name"},
    "UserDoc": {"github_login"},
    "CronConfigDoc": {"function"},
}
IDENTITY_FIELD_DOCS = {
    "ProjectDoc",
    "TelemetryPolicyOutboxDoc",
    "TelemetryPolicyMigrationExceptionDoc",
    "ProjectDeletionDoc",
    "ProjectEgressQuotaDoc",
    "ProjectEgressUsageDoc",
    "ProjectOperationsUsageDoc",
    "ProjectStorageSnapshotDoc",
    "CompiledBundleDoc",
    "CronConfigDoc",
    "ProjectCloudflareConfigDoc",
    "WebSocketSingletonConfigDoc",
    "WebSocketConnectionDoc",
}
REKEY_DOCS = {
    "ProjectDoc",
    "TelemetryPolicyOutboxDoc",
    "TelemetryPolicyMigrationExceptionDoc",
    "ProjectDeletionDoc",
    "ProjectEgressQuotaDoc",
    "ProjectEgressUsageDoc",
    "ProjectOperationsUsageDoc",
    "ProjectStorageSnapshotDoc",
    "CompiledBundleDoc",
    "CronConfigDoc",
    "ProjectCloudflareConfigDoc",
    "WebSocketSingletonConfigDoc",
    "WebSocketSingletonRuntimeDoc",
}
TRANSIENT_DOCS = {
    "WebSocketConnectionDoc",
    "WebSocketSingletonRuntimeDoc",
    "WebSocketSingletonReconcileCursorDoc",
}
KNOWN_DOCS = IDENTITY_FIELD_DOCS | REKEY_DOCS | TRANSIENT_DOCS | {
    "UserDoc",
    "WorkerManifestDoc",
    "WorkerCertManifestDoc",
}


class MigrationError(RuntimeError):
    pass


def load_secrets(path=None):
    selected_path = path or os.environ.get("FN0_PROJECT_ID_MIGRATION_SECRETS")
    if not selected_path:
        raise MigrationError("migration credentials are unavailable")
    with open(selected_path, encoding="utf-8") as source:
        return json.load(source)


class HttpClient:
    def request(self, url, method="GET", headers=None, body=None):
        request_headers = {"User-Agent": "curl/8.7.1"}
        request_headers.update(headers or {})
        request = urllib.request.Request(url, data=body, headers=request_headers, method=method)
        try:
            with urllib.request.urlopen(request, timeout=45) as response:
                return response.status, response.read()
        except urllib.error.HTTPError as error:
            return error.code, error.read()
        except urllib.error.URLError as error:
            raise MigrationError(f"network request failed: {error.reason}") from None


class Turso:
    def __init__(self, secrets, client=None):
        self.secrets = secrets
        outputs = secrets["outputs"]
        self.token = outputs.get("forteDbGroupToken")
        self.host_suffix = outputs.get("forteDbHostSuffix")
        self.control_url = outputs.get("controlDbUrl", "").replace("libsql://", "https://").rstrip("/")
        self.api_token = secrets["api_token"]
        self.org_slug = secrets["org_slug"]
        self.group_name = secrets["group_name"]
        self.client = client or HttpClient()
        if not all((self.token, self.host_suffix, self.control_url, self.api_token, self.org_slug, self.group_name)):
            raise MigrationError("required Turso configuration is missing")

    def db_url(self, project_id):
        return f"https://{project_id}{self.host_suffix}"

    def pipeline(self, url, statements):
        requests = []
        for sql, args in statements:
            statement = {"sql": sql}
            if args:
                statement["args"] = args
            requests.append({"type": "execute", "stmt": statement})
        requests.append({"type": "close"})
        status, raw = self.client.request(
            f"{url}/v2/pipeline",
            method="POST",
            headers={
                "Authorization": f"Bearer {self.token}",
                "Content-Type": "application/json",
            },
            body=json.dumps({"requests": requests}, separators=(",", ":")).encode(),
        )
        if status != 200:
            raise MigrationError(f"Turso pipeline returned HTTP {status}")
        try:
            response = json.loads(raw)
        except json.JSONDecodeError:
            raise MigrationError("Turso returned invalid pipeline JSON") from None
        errors = [result for result in response.get("results", []) if result.get("type") == "error"]
        if errors:
            message = errors[0].get("error", {}).get("message", "unknown database error")
            raise MigrationError(f"Turso pipeline failed: {message}")
        return response

    def query(self, url, sql, args=()):
        encoded_args = [self.sql_arg(value) for value in args]
        response = self.pipeline(url, [(sql, encoded_args)])
        rows = response["results"][0]["response"]["result"].get("rows", [])
        columns = response["results"][0]["response"]["result"].get("cols", [])
        return [self.row_values(columns, row) for row in rows]

    @staticmethod
    def sql_arg(value):
        if isinstance(value, bytes):
            return {"type": "blob", "base64": base64.b64encode(value).decode("ascii")}
        if isinstance(value, int):
            return {"type": "integer", "value": str(value)}
        return {"type": "text", "value": str(value)}

    @staticmethod
    def row_values(columns, row):
        names = [column.get("name", str(index)) for index, column in enumerate(columns)]
        result = {}
        for name, cell in zip(names, row):
            if cell.get("type") == "blob":
                encoded = cell.get("base64", "")
                encoded += "=" * (-len(encoded) % 4)
                result[name] = base64.b64decode(encoded, validate=True)
            elif cell.get("type") == "null":
                result[name] = None
            else:
                value = cell.get("value")
                if cell.get("type") == "integer":
                    value = int(value)
                result[name] = value
        return result

    def fetch_all(self, url):
        rows = []
        cursor = None
        while True:
            if cursor is None:
                page = self.query(url, "SELECT pk, sk, data, version FROM docs ORDER BY pk, sk LIMIT ?", (500,))
            else:
                page = self.query(
                    url,
                    "SELECT pk, sk, data, version FROM docs WHERE pk > ? OR (pk = ? AND sk > ?) ORDER BY pk, sk LIMIT ?",
                    (cursor[0], cursor[0], cursor[1], 500),
                )
            rows.extend(page)
            if len(page) < 500:
                return rows
            cursor = (page[-1]["pk"], page[-1]["sk"])

    def list_databases(self):
        url = f"https://api.turso.tech/v1/organizations/{urllib.parse.quote(self.org_slug, safe='')}/databases"
        status, raw = self.client.request(url, headers={"Authorization": f"Bearer {self.api_token}"})
        if status != 200:
            raise MigrationError(f"Turso database listing returned HTTP {status}")
        try:
            parsed = json.loads(raw)
        except json.JSONDecodeError:
            raise MigrationError("Turso returned invalid database listing JSON") from None
        databases = parsed.get("databases", parsed.get("items", []))
        if not isinstance(databases, list):
            raise MigrationError("Turso database listing had an unexpected shape")
        return databases

    def database_info(self, project_id):
        matches = [
            item for item in self.list_databases()
            if isinstance(item, dict) and item.get("Name", item.get("name")) == project_id
        ]
        if len(matches) != 1:
            return None
        return matches[0]

    def create_database(self, project_id):
        existing = self.database_info(project_id)
        if existing is not None:
            if existing.get("group") != self.group_name:
                raise MigrationError("existing destination database is not in the configured Turso group")
            return False
        url = f"https://api.turso.tech/v1/organizations/{urllib.parse.quote(self.org_slug, safe='')}/databases"
        status, raw = self.client.request(
            url,
            method="POST",
            headers={"Authorization": f"Bearer {self.api_token}", "Content-Type": "application/json"},
            body=json.dumps({"name": project_id, "group": self.group_name}, separators=(",", ":")).encode(),
        )
        if 200 <= status < 300:
            return True
        try:
            error = json.loads(raw).get("error")
        except (json.JSONDecodeError, AttributeError):
            error = None
        if status == 409 or (status == 400 and error == "database with same name already exists"):
            existing = self.database_info(project_id)
            if existing is not None and existing.get("group") == self.group_name:
                return False
        raise MigrationError(f"Turso database creation returned HTTP {status}")


def data_json(data):
    if isinstance(data, bytes):
        return json.loads(data.decode("utf-8"))
    return json.loads(data)


def json_bytes(data):
    return json.dumps(data, ensure_ascii=False, separators=(",", ":")).encode("utf-8")


def walk_strings(value, path=()):
    found = []
    if isinstance(value, dict):
        for key, child in value.items():
            if isinstance(key, str) and "namse-mottomite" in key:
                found.append((path + ("<map-key>", key), key))
            found.extend(walk_strings(child, path + (key,)))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            found.extend(walk_strings(child, path + (str(index),)))
    elif isinstance(value, str) and "namse-mottomite" in value:
        found.append((path, value))
    return found


def key_identity(row, old_id):
    for key_name in ("pk", "sk"):
        key = row.get(key_name, "")
        if key == old_id or f"project_id={old_id}" in key.split("/"):
            return True
    return False


def identity_paths(doc_type, data, old_id):
    paths = []
    if doc_type in IDENTITY_FIELD_DOCS and data.get("project_id") == old_id:
        paths.append(("project_id",))
    if doc_type == "UserDoc":
        for index, project in enumerate(data.get("projects", [])):
            if isinstance(project, dict) and project.get("project_id") == old_id:
                paths.append(("projects", str(index), "project_id"))
    if doc_type == "WorkerManifestDoc" and old_id in data.get("project_manifests", {}):
        paths.append(("project_manifests", "<map-key>", old_id))
    if doc_type == "WorkerCertManifestDoc":
        for hostname, certificate in data.get("certs", {}).items():
            if isinstance(certificate, dict) and certificate.get("project_id") == old_id:
                paths.append(("certs", hostname, "project_id"))
    if doc_type == "WebSocketSingletonReconcileCursorDoc" and data.get("after_project_id") == old_id:
        paths.append(("after_project_id",))
    return paths


def external_paths(doc_type, data, old_id):
    paths = []
    allowed = EXTERNAL_FIELDS.get(doc_type, set())
    for path, value in walk_strings(data):
        if value != old_id and old_id in value and path and path[-1] in allowed:
            paths.append(path)
        elif value == old_id and path and path[-1] in allowed:
            paths.append(path)
        elif doc_type == "WorkerCertManifestDoc" and len(path) == 3 and path[:2] == ("certs", "<map-key>"):
            paths.append(path)
        elif path and path[-1] in PRESERVED_VALUE_FIELDS.get(doc_type, set()):
            paths.append(path)
        elif doc_type == "CronConfigDoc" and len(path) == 3 and path[0] == "jobs" and path[-1] == "function":
            paths.append(path)
    return paths


def classify_row(row, old_id):
    pk = row.get("pk", "")
    doc_type = pk.split("/", 1)[0]
    try:
        data = data_json(row.get("data", b"{}"))
    except (UnicodeDecodeError, json.JSONDecodeError, TypeError):
        data = None
    key_ref = key_identity(row, old_id)
    identities = identity_paths(doc_type, data, old_id) if isinstance(data, dict) else []
    external = external_paths(doc_type, data, old_id) if isinstance(data, dict) else []
    occurrences = []
    for key_name in ("pk", "sk"):
        key = row.get(key_name, "")
        if old_id in key:
            occurrences.append({"field": key_name, "kind": "exact_identity" if key_ref else "unrecognized"})
    if data is not None:
        identity_set = set(identities)
        external_set = set(external)
        for path, value in walk_strings(data):
            rendered = ".".join(path)
            if path in identity_set:
                kind = "exact_identity"
            elif path in external_set:
                kind = "substring_or_resource"
            elif value == old_id:
                kind = "preserved_exact_value" if doc_type in KNOWN_DOCS else "unrecognized"
            else:
                kind = "unrecognized"
            occurrences.append({"field": rendered, "kind": kind})
    elif key_ref:
        return {"classification": "UNRECOGNIZED", "occurrences": occurrences}
    has_occurrence = bool(occurrences)
    if not has_occurrence:
        return {"classification": None, "occurrences": []}
    if doc_type not in KNOWN_DOCS:
        classification = "UNRECOGNIZED"
    elif any(item["kind"] == "unrecognized" for item in occurrences):
        classification = "UNRECOGNIZED"
    elif doc_type in TRANSIENT_DOCS and (key_ref or identities):
        classification = "DELETE_TRANSIENT"
    elif key_ref:
        classification = "REKEY"
    elif identities:
        classification = "PATCH_IDENTITY"
    elif any(item["kind"] == "unrecognized" for item in occurrences):
        classification = "UNRECOGNIZED"
    elif external and len(external) == len(walk_strings(data)):
        classification = "KEEP_EXTERNAL_RESOURCE_NAME"
    elif doc_type in KNOWN_DOCS and all(item["kind"] in {"substring_or_resource", "preserved_exact_value"} for item in occurrences):
        classification = "KEEP_EXTERNAL_RESOURCE_NAME"
    else:
        classification = "UNRECOGNIZED"
    return {"classification": classification, "occurrences": occurrences}


def replace_identity(data, doc_type, old_id, new_id):
    updated = copy.deepcopy(data)
    if doc_type in IDENTITY_FIELD_DOCS and updated.get("project_id") == old_id:
        updated["project_id"] = new_id
    if doc_type == "UserDoc":
        for project in updated.get("projects", []):
            if isinstance(project, dict) and project.get("project_id") == old_id:
                project["project_id"] = new_id
    if doc_type == "WorkerManifestDoc":
        manifests = updated.get("project_manifests", {})
        if old_id in manifests:
            if new_id in manifests:
                raise MigrationError("destination manifest key already exists")
            manifests[new_id] = manifests.pop(old_id)
    if doc_type == "WorkerCertManifestDoc":
        for certificate in updated.get("certs", {}).values():
            if isinstance(certificate, dict) and certificate.get("project_id") == old_id:
                certificate["project_id"] = new_id
    return updated


def transformed_row(row, old_id, new_id, classification):
    doc_type = row["pk"].split("/", 1)[0]
    transformed = dict(row)
    if classification == "DELETE_TRANSIENT":
        return None
    for key_name in ("pk", "sk"):
        key = transformed.get(key_name, "")
        components = key.split("/")
        transformed[key_name] = "/".join(
            f"project_id={new_id}" if component == f"project_id={old_id}" else new_id if component == old_id else component
            for component in components
        )
    if classification in {"REKEY", "PATCH_IDENTITY"}:
        old_data = data_json(row["data"])
        new_data = replace_identity(old_data, doc_type, old_id, new_id)
        transformed["data"] = json_bytes(new_data)
        if (transformed["pk"], transformed["sk"]) == (row["pk"], row["sk"]):
            transformed["version"] = int(row.get("version", 0)) + 1
    return transformed


def row_stats(rows):
    return {"rows": len(rows), "data_bytes": sum(len(row["data"]) for row in rows)}


def compare_rows(source, destination):
    source_map = {(row["pk"], row["sk"]): row for row in source}
    destination_map = {(row["pk"], row["sk"]): row for row in destination}
    missing = [key for key in source_map if key not in destination_map]
    extra = [key for key in destination_map if key not in source_map]
    different_data = [key for key in source_map.keys() & destination_map.keys() if source_map[key]["data"] != destination_map[key]["data"]]
    different_version = [key for key in source_map.keys() & destination_map.keys() if source_map[key]["version"] != destination_map[key]["version"]]
    return {
        "source_rows": len(source),
        "destination_rows": len(destination),
        "source_bytes": row_stats(source)["data_bytes"],
        "destination_bytes": row_stats(destination)["data_bytes"],
        "missing": len(missing),
        "extra": len(extra),
        "different_data": len(different_data),
        "different_version": len(different_version),
    }


def inspect_control(turso):
    rows = turso.fetch_all(turso.control_url)
    return rows, project_ids(rows)


def project_ids(rows):
    results = []
    for row in rows:
        if not row.get("pk", "").startswith("ProjectDoc/"):
            continue
        data = data_json(row["data"])
        project_id = data.get("project_id")
        if not isinstance(project_id, str):
            raise MigrationError(f"ProjectDoc at {row['pk']} has no string project_id")
        results.append({"project_id": project_id, "canonical": bool(CANONICAL_ID.fullmatch(project_id)), "system": project_id == "fn0-control"})
    return results


def list_database_names(turso):
    return {
        item.get("Name", item.get("name"))
        for item in turso.list_databases()
        if isinstance(item, dict)
    }


def control_plan(rows, old_id, new_id):
    plan = []
    for row in rows:
        result = classify_row(row, old_id)
        if not result["classification"]:
            continue
        actions = {
            "REKEY": "insert transformed row at the new key, preserve version, delete old key",
            "PATCH_IDENTITY": "patch approved identity fields and increment version",
            "DELETE_TRANSIENT": "delete rebuildable runtime state",
            "KEEP_EXTERNAL_RESOURCE_NAME": "preserve literal as an external or user supplied value",
            "UNRECOGNIZED": "stop before mutation",
        }
        plan.append({
            "pk": row["pk"],
            "sk": row["sk"],
            "document_type": row["pk"].split("/", 1)[0],
            "classification": result["classification"],
            "planned_action": actions[result["classification"]],
            "occurrences": result["occurrences"],
        })
    if any(row["classification"] == "UNRECOGNIZED" for row in plan):
        unknown = [row for row in plan if row["classification"] == "UNRECOGNIZED"]
        raise MigrationError(f"control reference plan contains UNRECOGNIZED rows: {json.dumps(unknown, ensure_ascii=False, sort_keys=True)}")
    return plan


def validate_ids(old_id, new_id):
    if old_id != "namse-mottomite":
        raise MigrationError("this migration tool only supports the planned old project ID")
    if not CANONICAL_ID.fullmatch(new_id):
        raise MigrationError("new project ID must match ^[0-9a-z]{8}$")
    if new_id in {"fn0-control", "local"}:
        raise MigrationError("new project ID is reserved")


def signy_policy(secrets, project_id, client=None):
    outputs = secrets["outputs"]
    signy_url = outputs.get("signyUrl", "").rstrip("/")
    client_id = outputs.get("signyAccessClientId")
    client_secret = outputs.get("signyAccessClientSecret")
    if not all((signy_url, client_id, client_secret)):
        raise MigrationError("Signy credentials are unavailable")
    http = client or HttpClient()
    path = urllib.parse.quote(project_id, safe="")
    status, raw = http.request(
        f"{signy_url}/signy/api/v1/admin/tenants/{path}/retention",
        headers={"CF-Access-Client-Id": client_id, "CF-Access-Client-Secret": client_secret},
    )
    if status != 200:
        raise MigrationError(f"Signy retention read returned HTTP {status} for {project_id}")
    try:
        value = json.loads(raw)
    except json.JSONDecodeError:
        raise MigrationError("Signy returned invalid retention JSON") from None
    return {
        "revision": max(int(value.get("revision", 0)), 1),
        "base_retention": value.get("retention"),
        "log_retention_override": value.get("log_retention"),
        "trace_retention_override": value.get("trace_retention"),
        "metric_retention_override": value.get("metric_retention"),
        "max_stored_bytes": value.get("max_stored_bytes", "unlimited"),
    }


def write_signy_policy(secrets, project_id, policy, client=None):
    outputs = secrets["outputs"]
    signy_url = outputs.get("signyUrl", "").rstrip("/")
    client_id = outputs.get("signyAccessClientId")
    client_secret = outputs.get("signyAccessClientSecret")
    if not all((signy_url, client_id, client_secret)):
        raise MigrationError("Signy credentials are unavailable")
    http = client or HttpClient()
    path = urllib.parse.quote(project_id, safe="")
    body = {
        "revision": policy["revision"],
        "retention": policy["base_retention"],
        "log_retention": policy["log_retention_override"],
        "trace_retention": policy["trace_retention_override"],
        "metric_retention": policy["metric_retention_override"],
        "max_stored_bytes": policy["max_stored_bytes"],
    }
    status, raw = http.request(
        f"{signy_url}/signy/api/v1/admin/project-tenants/{path}/retention",
        method="PUT",
        headers={
            "CF-Access-Client-Id": client_id,
            "CF-Access-Client-Secret": client_secret,
            "Content-Type": "application/json",
        },
        body=json.dumps(body, separators=(",", ":")).encode(),
    )
    if status != 200:
        raise MigrationError(f"Signy policy write returned HTTP {status} for {project_id}")
    observed = signy_policy(secrets, project_id, http)
    if observed != policy:
        raise MigrationError(f"Signy policy read-back differed for {project_id}")
    return observed


def read_project(rows, project_id):
    matches = []
    for row in rows:
        if row["pk"].startswith("ProjectDoc/"):
            data = data_json(row["data"])
            if data.get("project_id") == project_id:
                matches.append(data)
    if len(matches) != 1:
        raise MigrationError(f"expected one ProjectDoc for {project_id}, found {len(matches)}")
    return matches[0]


def build_plan(turso, old_id, new_id):
    validate_ids(old_id, new_id)
    rows, projects = inspect_control(turso)
    active = [item for item in projects if not item["system"]]
    noncanonical = [item["project_id"] for item in active if not item["canonical"]]
    if noncanonical != [old_id]:
        raise MigrationError(f"noncanonical active project IDs differ from the expected singleton: {noncanonical}")
    if any(item["project_id"] == new_id for item in projects):
        raise MigrationError("new project ID already has a ProjectDoc")
    if new_id in list_database_names(turso):
        raise MigrationError("new project ID already has a Turso database")
    references = control_plan(rows, old_id, new_id)
    old_rows = turso.fetch_all(turso.db_url(old_id))
    old_project = read_project(rows, old_id)
    source_policy = old_project.get("telemetry_policy")
    external_policy = signy_policy(turso.secrets, old_id, turso.client)
    if source_policy != external_policy:
        raise MigrationError("ProjectDoc telemetry_policy differs from Signy retention policy")
    return {
        "old_project_id": old_id,
        "new_project_id": new_id,
        "projects": projects,
        "active_project_ids": [item["project_id"] for item in active],
        "noncanonical_active_project_ids": noncanonical,
        "control_references": references,
        "project_database": row_stats(old_rows),
        "signy_old_policy": external_policy,
        "control_rows": len(rows),
    }


def build_apply_plan(turso, old_id, new_id):
    validate_ids(old_id, new_id)
    rows, projects = inspect_control(turso)
    active = [item for item in projects if not item["system"]]
    noncanonical = [item["project_id"] for item in active if not item["canonical"]]
    if noncanonical != [old_id]:
        raise MigrationError(f"noncanonical active project IDs differ from the expected singleton: {noncanonical}")
    if any(item["project_id"] == new_id for item in projects):
        raise MigrationError("new project ID already has a ProjectDoc")
    references = control_plan(rows, old_id, new_id)
    old_rows = turso.fetch_all(turso.db_url(old_id))
    old_project = read_project(rows, old_id)
    policy = signy_policy(turso.secrets, old_id, turso.client)
    if old_project.get("telemetry_policy") != policy:
        raise MigrationError("ProjectDoc telemetry_policy differs from Signy retention policy")
    return {"control_rows": len(rows), "project_rows": row_stats(old_rows), "references": references, "policy": policy}


def copy_project_database(turso, old_id, new_id):
    source = turso.fetch_all(turso.db_url(old_id))
    destination_url = turso.db_url(new_id)
    turso.pipeline(destination_url, [("CREATE TABLE IF NOT EXISTS docs (pk TEXT, sk TEXT, data BLOB, version INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (pk, sk))", [])])
    destination = turso.fetch_all(destination_url)
    destination_map = {(row["pk"], row["sk"]): row for row in destination}
    inserted = 0
    skipped = 0
    for row in source:
        key = (row["pk"], row["sk"])
        existing = destination_map.get(key)
        if existing is not None:
            if existing["data"] != row["data"] or existing["version"] != row["version"]:
                raise MigrationError(f"destination row differs for {row['pk']} / {row['sk']}")
            skipped += 1
            continue
        args = [Turso.sql_arg(row["pk"]), Turso.sql_arg(row["sk"]), Turso.sql_arg(row["data"]), Turso.sql_arg(row["version"])]
        turso.pipeline(destination_url, [("INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, ?)", args)])
        inserted += 1
        destination_map[key] = dict(row)
    return {"copied": inserted, "skipped": skipped, "verify": verify_project_database(turso, old_id, new_id)}


def verify_project_database(turso, old_id, new_id):
    source = turso.fetch_all(turso.db_url(old_id))
    destination = turso.fetch_all(turso.db_url(new_id))
    result = compare_rows(source, destination)
    if any(result[key] for key in ("missing", "extra", "different_data", "different_version")) or result["source_rows"] != result["destination_rows"] or result["source_bytes"] != result["destination_bytes"]:
        raise MigrationError(f"project database exact verification failed: {json.dumps(result, sort_keys=True)}")
    return result


def apply_control_plan(turso, old_id, new_id):
    rows, _ = inspect_control(turso)
    plan = control_plan(rows, old_id, new_id)
    operations = [("BEGIN IMMEDIATE", [])]
    for row in rows:
        result = classify_row(row, old_id)
        classification = result["classification"]
        if not classification:
            continue
        transformed = transformed_row(row, old_id, new_id, classification)
        old_args = [Turso.sql_arg(row["pk"]), Turso.sql_arg(row["sk"]), Turso.sql_arg(row["version"]), Turso.sql_arg(row["data"])]
        if transformed is None:
            operations.append(("DELETE FROM docs WHERE pk = ? AND sk = ? AND version = ? AND data = ?", old_args))
            continue
        if (transformed["pk"], transformed["sk"]) != (row["pk"], row["sk"]):
            new_args = [Turso.sql_arg(transformed["pk"]), Turso.sql_arg(transformed["sk"]), Turso.sql_arg(transformed["data"]), Turso.sql_arg(transformed["version"])]
            operations.append(("INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, ?)", new_args))
            operations.append(("DELETE FROM docs WHERE pk = ? AND sk = ? AND version = ? AND data = ?", old_args))
        else:
            patch_args = [Turso.sql_arg(transformed["data"]), Turso.sql_arg(row["pk"]), Turso.sql_arg(row["sk"]), Turso.sql_arg(row["version"]), Turso.sql_arg(row["data"])]
            operations.append(("UPDATE docs SET data = ?, version = version + 1 WHERE pk = ? AND sk = ? AND version = ? AND data = ?", patch_args))
    operations.append(("COMMIT", []))
    turso.pipeline(turso.control_url, operations)
    return {"classified_rows": len(plan), "actions": {name: sum(1 for item in plan if item["classification"] == name) for name in sorted({item["classification"] for item in plan})}}


def migration_already_applied(turso, old_id, new_id):
    rows, projects = inspect_control(turso)
    old_present = any(item["project_id"] == old_id for item in projects)
    new_present = any(item["project_id"] == new_id for item in projects)
    return not old_present and new_present, rows


def verify_control(turso, old_id, new_id):
    rows, projects = inspect_control(turso)
    active = [item for item in projects if not item["system"]]
    if any(not item["canonical"] for item in active):
        raise MigrationError("active ProjectDoc inventory contains noncanonical IDs")
    old_refs = control_plan(rows, old_id, new_id)
    forbidden = [row for row in old_refs if row["classification"] != "KEEP_EXTERNAL_RESOURCE_NAME"]
    if forbidden:
        raise MigrationError("control database still contains project identity references to the old ID")
    read_project(rows, new_id)
    return {"active_project_ids": [item["project_id"] for item in active], "residual_literal_rows": old_refs}


def verify_signy(turso, old_id, new_id):
    old_policy = signy_policy(turso.secrets, old_id, turso.client)
    new_policy = signy_policy(turso.secrets, new_id, turso.client)
    if old_policy != new_policy:
        raise MigrationError("old and new Signy retention policies differ")
    return {"old": old_policy, "new": new_policy, "equal": True}


def parser():
    result = argparse.ArgumentParser()
    subparsers = result.add_subparsers(dest="command", required=True)
    for command in ("plan", "apply", "verify"):
        command_parser = subparsers.add_parser(command)
        command_parser.add_argument("--old-id", required=True)
        command_parser.add_argument("--new-id", required=True)
        if command == "apply":
            command_parser.add_argument("--apply", action="store_true")
    return result


def main(argv=None):
    args = parser().parse_args(argv)
    validate_ids(args.old_id, args.new_id)
    if args.command == "apply" and not args.apply:
        raise MigrationError("apply requires the explicit --apply flag")
    secrets = load_secrets()
    turso = Turso(secrets)
    if args.command == "plan":
        result = build_plan(turso, args.old_id, args.new_id)
    elif args.command == "apply":
        already_applied, _ = migration_already_applied(turso, args.old_id, args.new_id)
        if already_applied:
            result = {"already_applied": True, "project_database": verify_project_database(turso, args.old_id, args.new_id), "control": verify_control(turso, args.old_id, args.new_id), "signy": verify_signy(turso, args.old_id, args.new_id)}
        else:
            initial = build_apply_plan(turso, args.old_id, args.new_id)
            created = turso.create_database(args.new_id)
            copy_result = copy_project_database(turso, args.old_id, args.new_id)
            write_signy_policy(turso.secrets, args.new_id, initial["policy"], turso.client)
            control_result = apply_control_plan(turso, args.old_id, args.new_id)
            result = {"already_applied": False, "database_created": created, "database_copy": copy_result, "signy_policy": initial["policy"], "control_migration": control_result, "initial_plan": initial}
    else:
        result = {"project_database": verify_project_database(turso, args.old_id, args.new_id), "control": verify_control(turso, args.old_id, args.new_id), "signy": verify_signy(turso, args.old_id, args.new_id)}
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except MigrationError as error:
        print(f"project ID migration stopped: {error}", file=sys.stderr)
        raise SystemExit(1)
