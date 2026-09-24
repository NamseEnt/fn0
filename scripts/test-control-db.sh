#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT
source "${REPO_ROOT}/scripts/lib/control-db.sh"

pulumi() {
  [[ "$1" == config && "$2" == get && "$3" == fn0Cloud:dbBackend ]]
  printf '%s\n' "$TEST_DB_BACKEND"
}

pulumi_pick() {
  [[ "${TEST_DB_BACKEND:-}" != dodb ]] || return 99
  case "$1" in
    controlDbUrl) printf '%s\n' 'libsql://control.example' ;;
    forteDbGroupToken) printf '%s\n' 'fake-token' ;;
    *) return 1 ;;
  esac
}

test_turso_dispatch() {
  unset CONTROL_DB_BACKEND __FN0_CONTROL_DB_INITIALIZED
  TEST_DB_BACKEND=turso
  dodb_calls=0
  turso_calls=0
  dodb_control_db_call() { dodb_calls=$((dodb_calls + 1)); }
  __control_db_turso_request() {
    turso_calls=$((turso_calls + 1))
    printf '%s\n' '{"results":[{"type":"ok","response":{"result":{"affected_row_count":1}}}]}'
  }
  control_db_put test "" '{}' >/dev/null
  [[ "$dodb_calls" == 0 && "$turso_calls" == 1 ]]
  [[ "$CONTROL_DB_URL" == https://control.example ]]
}

test_dodb_dispatch() {
  unset CONTROL_DB_BACKEND __FN0_CONTROL_DB_INITIALIZED
  TEST_DB_BACKEND=dodb
  dodb_call_file="$(mktemp)"
  printf '0' >"$dodb_call_file"
  dodb_control_db_call() {
    local call_count
    call_count="$(cat "$dodb_call_file")"
    printf '%s' "$((call_count + 1))" >"$dodb_call_file"
    [[ "$1" == get-observed && "$2" == --pk && "$3" == pk && "$4" == --sk && "$5" == sk ]]
    printf '%s\n' '{"found":false,"revision":0,"data_base64":null}'
  }
  local observed
  observed="$(control_db_get_observed pk sk)"
  [[ "$(cat "$dodb_call_file")" == 1 ]]
  [[ "$(jq -r '.found' <<<"$observed")" == false ]]
  rm -f "$dodb_call_file"
}

test_dodb_scan_dispatch() {
  unset CONTROL_DB_BACKEND __FN0_CONTROL_DB_INITIALIZED
  TEST_DB_BACKEND=dodb
  dodb_control_db_call() {
    [[ "$1" == scan && "$2" == --limit && "$3" == 17 ]]
    [[ "$4" == --after-pk && "$5" == ProjectDoc/a && "$6" == --after-sk && "$7" == "" ]]
    printf '%s\n' '{"documents":[]}'
  }
  local result
  result="$(control_db_scan ProjectDoc/a "" 17 true)"
  [[ "$(jq -r '.documents | length' <<<"$result")" == 0 ]]
}

test_turso_scan_dispatch() {
  unset CONTROL_DB_BACKEND __FN0_CONTROL_DB_INITIALIZED
  TEST_DB_BACKEND=turso
  turso_scan_sql=""
  __control_db_turso_request() {
    turso_scan_sql="$(jq -r '.requests[0].stmt.sql' <<<"$1")"
    printf '%s\n' '{"results":[{"type":"ok","response":{"result":{"rows":[]}}}]}'
  }
  local result
  result="$(control_db_scan ProjectDoc/a "" 17 true)"
  [[ "$turso_scan_sql" == *"ORDER BY pk, sk LIMIT ?"* ]]
  [[ "$turso_scan_sql" == *"pk > ? OR (pk = ? AND sk > ?)"* ]]
  [[ "$(jq -r '.documents | length' <<<"$result")" == 0 ]]
}

test_invalid_backend() {
  unset CONTROL_DB_BACKEND __FN0_CONTROL_DB_INITIALIZED
  TEST_DB_BACKEND=""
  if control_db_init 2>/dev/null; then return 1; else [[ "$?" == 2 ]]; fi
}

source "${REPO_ROOT}/scripts/lib/control-seed.sh"

test_manifest_cas_retry() {
  manifest_get_file="$(mktemp)"
  printf '0' >"$manifest_get_file"
  manifest_write_count=0
  control_db_get_observed() {
    local get_count
    get_count="$(cat "$manifest_get_file")"
    get_count=$((get_count + 1))
    printf '%s' "$get_count" >"$manifest_get_file"
    if [[ "$get_count" == 1 ]]; then
      local initial='{"manifest_version":3,"project_manifests":{"sibling":{"code_version":9}}}'
      jq -nc --arg encoded "$(printf '%s' "$initial" | base64 | tr -d '\n')" '{found:true,revision:7,data_base64:$encoded}'
    else
      local latest='{"manifest_version":4,"project_manifests":{"sibling":{"code_version":10}}}'
      jq -nc --arg encoded "$(printf '%s' "$latest" | base64 | tr -d '\n')" '{found:true,revision:8,data_base64:$encoded}'
    fi
  }
  control_db_put_if_revision() {
    manifest_write_count=$((manifest_write_count + 1))
    if [[ "$manifest_write_count" == 1 ]]; then return 3; fi
    [[ "$3" == 8 ]]
    jq -e '.project_manifests.sibling.code_version == 10 and .project_manifests.target.code_version == 11' <<<"$4" >/dev/null
  }
  seed_worker_manifest target 11 target.example >/dev/null
  [[ "$(cat "$manifest_get_file")" == 2 && "$manifest_write_count" == 2 ]]
  rm -f "$manifest_get_file"
}

test_manifest_storage_cas_retry() {
  manifest_get_file="$(mktemp)"
  printf '0' >"$manifest_get_file"
  manifest_write_count=0
  control_db_get_observed() {
    if [[ "$1" == ProjectCloudflareConfigDoc/project_id=target ]]; then
      local config='{"account_id":"acct","worker_access_key_id":"key","worker_secret_ciphertext":"secret","private_object_storage_bucket":"private","public_object_storage_bucket":"public","public_object_storage_hostname":"public.example","config_version":1}'
      jq -nc --arg encoded "$(printf '%s' "$config" | base64 | tr -d '\n')" '{found:true,revision:1,data_base64:$encoded}'
      return
    fi
    local get_count
    get_count="$(cat "$manifest_get_file")"
    get_count=$((get_count + 1))
    printf '%s' "$get_count" >"$manifest_get_file"
    if [[ "$get_count" == 1 ]]; then
      local initial='{"manifest_version":3,"project_manifests":{"target":{},"sibling":{"code_version":9}}}'
      jq -nc --arg encoded "$(printf '%s' "$initial" | base64 | tr -d '\n')" '{found:true,revision:7,data_base64:$encoded}'
    else
      local latest='{"manifest_version":4,"project_manifests":{"target":{},"sibling":{"code_version":10}}}'
      jq -nc --arg encoded "$(printf '%s' "$latest" | base64 | tr -d '\n')" '{found:true,revision:8,data_base64:$encoded}'
    fi
  }
  control_db_put_if_revision() {
    manifest_write_count=$((manifest_write_count + 1))
    if [[ "$manifest_write_count" == 1 ]]; then return 3; fi
    [[ "$3" == 8 ]]
    jq -e '.project_manifests.sibling.code_version == 10 and .manifest_version == 5 and .project_manifests.target.storage.region == "auto"' <<<"$4" >/dev/null
  }
  seed_manifest_storage target >/dev/null
  [[ "$(cat "$manifest_get_file")" == 2 && "$manifest_write_count" == 2 ]]
  rm -f "$manifest_get_file"
}

test_insert_only_seed_is_idempotent() {
  control_db_put_if_missing() { return 3; }
  __seed_insert_only ProjectDoc/project_id=target '{"project_id":"target"}' >/dev/null
}

test_manifest_cas_bounded() {
  manifest_get_file="$(mktemp)"
  printf '0' >"$manifest_get_file"
  manifest_write_count=0
  control_db_get_observed() {
    local get_count
    get_count="$(cat "$manifest_get_file")"
    get_count=$((get_count + 1))
    printf '%s' "$get_count" >"$manifest_get_file"
    local value='{"manifest_version":1,"project_manifests":{}}'
    jq -nc --arg encoded "$(printf '%s' "$value" | base64 | tr -d '\n')" '{found:true,revision:2,data_base64:$encoded}'
  }
  control_db_put_if_revision() {
    manifest_write_count=$((manifest_write_count + 1))
    return 3
  }
  if seed_worker_manifest target 11 target.example >/dev/null 2>&1; then return 1; fi
  [[ "$(cat "$manifest_get_file")" == 12 && "$manifest_write_count" == 12 ]]
  rm -f "$manifest_get_file"
}

source "${REPO_ROOT}/scripts/lib/worker-target.sh"

test_worker_status_pagination() {
  status_query_file="$(mktemp)"
  printf '0' >"$status_query_file"
  control_db_query() {
    local pk="$1" cursor="$2" limit="$3"
    [[ "$pk" == WorkerHostStatusDoc && "$limit" == 500 ]]
    local query_count
    query_count="$(cat "$status_query_file")"
    query_count=$((query_count + 1))
    printf '%s' "$query_count" >"$status_query_file"
    if [[ "$query_count" == 1 ]]; then
      jq -nc '[range(0;500) | {pk:"WorkerHostStatusDoc",sk:("sk" + (1000 + . | tostring)),data_base64:"eyJyZXBvcnRlZF9hdCI6MH0="}] | {documents:.}'
    else
      [[ "$cursor" == sk1499 ]]
      jq -nc '[{pk:"WorkerHostStatusDoc",sk:"sk1500",data_base64:"eyJyZXBvcnRlZF9hdCI6MH0="}] | {documents:.}'
    fi
  }
  local result
  result="$( __worker_status_query_all)"
  [[ "$(cat "$status_query_file")" == 2 && "$(jq 'length' <<<"$result")" == 501 ]]
  rm -f "$status_query_file"
}

test_turso_dispatch
test_dodb_dispatch
test_dodb_scan_dispatch
test_turso_scan_dispatch
test_invalid_backend
test_manifest_cas_retry
test_manifest_cas_bounded
test_manifest_storage_cas_retry
test_insert_only_seed_is_idempotent
test_worker_status_pagination
printf '%s\n' 'control DB shell tests passed'
