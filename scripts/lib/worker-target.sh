# shellcheck shell=bash
# Worker target doc write + host-status convergence polling. Source-only.

if [[ -n "${__FN0_WORKER_TARGET_LOADED:-}" ]]; then
  return 0
fi
__FN0_WORKER_TARGET_LOADED=1

WORKER_CONVERGE_TIMEOUT="${WORKER_CONVERGE_TIMEOUT:-1800}"      # 30 min
WORKER_CONVERGE_POLL_INTERVAL="${WORKER_CONVERGE_POLL_INTERVAL:-10}"
WORKER_CONVERGE_HOST_LIVE_WINDOW="${WORKER_CONVERGE_HOST_LIVE_WINDOW:-90}"  # seconds
# Abort if `live=0` for this many consecutive polls. Default 18 ≈ 3 min,
# which fits a worker docker pull + restart cycle without flagging it.
WORKER_CONVERGE_ZERO_LIVE_LIMIT="${WORKER_CONVERGE_ZERO_LIVE_LIMIT:-18}"

__control_db_https_url() {
  local url
  url="$(pulumi_pick controlDbUrl)"
  url="${url%/}"
  echo "$url"
}

__control_db_token() {
  pulumi_pick forteDbGroupToken
}

__write_target_image_doc() {
  local pk="$1" image_ref="$2"
  if [[ -z "$pk" || -z "$image_ref" ]]; then
    echo "__write_target_image_doc: missing pk or image_ref" >&2
    return 2
  fi
  local https_url token data_b64 sql req resp_file http_code
  https_url="$(__control_db_https_url)"
  token="$(__control_db_token)"
  data_b64="$(jq -nc --arg r "$image_ref" '{image_ref: $r}' | base64 | tr -d '\n')"
  sql="INSERT INTO docs (pk, sk, data, version) VALUES (?, '', ?, 0) ON CONFLICT(pk, sk) DO UPDATE SET data = excluded.data, version = docs.version + 1"
  req="$(jq -nc \
    --arg sql "$sql" \
    --arg pk "$pk" \
    --arg b64 "$data_b64" \
    '{requests: [
      {type: "execute", stmt: {sql: $sql, args: [
        {type: "text", value: $pk},
        {type: "blob", base64: $b64}
      ]}},
      {type: "close"}
    ]}')"
  resp_file="$(mktemp)"
  http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "${https_url}/v2/pipeline" \
    -H "Authorization: Bearer ${token}" \
    -H "Content-Type: application/json" \
    --data-raw "$req")"
  if [[ "$http_code" != "200" ]] || ! jq -e '.results[0].type == "ok"' <"$resp_file" >/dev/null 2>&1; then
    cat "$resp_file" >&2
    rm -f "$resp_file"
    echo "control DB write failed (HTTP ${http_code})" >&2
    return 1
  fi
  rm -f "$resp_file"
  echo ">> ${pk}.image_ref = ${image_ref}"
}

# Writes TargetFn0WorkerConfigDoc.image_ref directly to the control DB. Idempotent
# (same image_ref re-written has no harm; version increments).
write_worker_target_image() {
  local image_ref="$1"
  if [[ -z "$image_ref" ]]; then
    echo "write_worker_target_image: missing image_ref" >&2
    return 2
  fi
  __write_target_image_doc "TargetFn0WorkerConfigDoc" "$image_ref"
}

# Polls WorkerHostStatusDoc rows until every live host reports
# active_image_ref == <target_image_ref>. "Live" = reported_at within
# WORKER_CONVERGE_HOST_LIVE_WINDOW seconds. Stale rows are ignored.
# Fails if no live hosts found, or if timeout elapses.
wait_worker_target_converged() {
  local target_image_ref="$1"
  if [[ -z "$target_image_ref" ]]; then
    echo "wait_worker_target_converged: missing target_image_ref" >&2
    return 2
  fi
  local https_url token sql req resp_file
  https_url="$(__control_db_https_url)"
  token="$(__control_db_token)"
  sql="SELECT data FROM docs WHERE pk = 'WorkerHostStatusDoc' ORDER BY sk"
  req="$(jq -nc --arg sql "$sql" \
    '{requests: [{type: "execute", stmt: {sql: $sql, args: []}}, {type: "close"}]}')"

  echo ">> waiting for hosts to converge to ${target_image_ref} (timeout ${WORKER_CONVERGE_TIMEOUT}s, abort after ${WORKER_CONVERGE_ZERO_LIVE_LIMIT} live=0 polls)"
  local start_ts deadline now elapsed http_code rows now_epoch zero_live_count
  start_ts="$(date +%s)"
  deadline=$((start_ts + WORKER_CONVERGE_TIMEOUT))
  zero_live_count=0
  resp_file="$(mktemp)"
  trap 'rm -f "$resp_file"' RETURN

  while :; do
    http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
      -X POST "${https_url}/v2/pipeline" \
      -H "Authorization: Bearer ${token}" \
      -H "Content-Type: application/json" \
      --data-raw "$req" || echo 000)"

    if [[ "$http_code" != "200" ]] || ! jq -e '.results[0].type == "ok"' <"$resp_file" >/dev/null 2>&1; then
      echo "  control DB query failed (HTTP ${http_code}); retrying" >&2
    else
      now_epoch="$(date +%s)"
      rows="$(jq -c '.results[0].response.result.rows // []' <"$resp_file")"
      local report
      report="$(jq -c \
        --argjson now "$now_epoch" \
        --argjson live_window "$WORKER_CONVERGE_HOST_LIVE_WINDOW" \
        --arg target "$target_image_ref" '
        def parse_cell:
          if . == null then null
          elif .type == "blob" then (.base64 | @base64d | fromjson)
          else (.value // null | if . == null then null else fromjson end)
          end;
        [.[] | .[0] | parse_cell | select(. != null)]
        | map(select((.reported_at // 0) >= ($now - $live_window)))
        | { live: length,
            on_target: (map(select(.active_image_ref == $target)) | length),
            others: (map(select(.active_image_ref != $target)) | map(.active_image_ref // "<none>") | unique) }
      ' <<<"$rows")"

      local live on_target others_csv
      live="$(jq -r '.live' <<<"$report")"
      on_target="$(jq -r '.on_target' <<<"$report")"
      others_csv="$(jq -r '.others | join(", ")' <<<"$report")"

      if [[ "$live" -gt 0 && "$live" == "$on_target" ]]; then
        elapsed=$(( $(date +%s) - start_ts ))
        echo ">> converged (${live} live hosts on target) in ${elapsed}s"
        return 0
      fi

      echo "  live=${live} on_target=${on_target} others=[${others_csv}]"

      if [[ "$live" -eq 0 ]]; then
        zero_live_count=$((zero_live_count + 1))
        if (( zero_live_count >= WORKER_CONVERGE_ZERO_LIVE_LIMIT )); then
          elapsed=$(( $(date +%s) - start_ts ))
          echo "no live hosts for ${zero_live_count} consecutive polls (${elapsed}s); aborting" >&2
          echo "  hint: workers may be down, terminated, or unable to fetch the new image; check WorkerHostStatusDoc.reported_at and host system logs" >&2
          return 1
        fi
      else
        zero_live_count=0
      fi
    fi

    now="$(date +%s)"
    if (( now >= deadline )); then
      elapsed=$(( now - start_ts ))
      echo "timeout after ${elapsed}s; some hosts not converged" >&2
      return 1
    fi
    sleep "$WORKER_CONVERGE_POLL_INTERVAL"
  done
}
