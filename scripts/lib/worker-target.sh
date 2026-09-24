# shellcheck shell=bash

if [[ -n "${__FN0_WORKER_TARGET_LOADED:-}" ]]; then
  return 0
fi
__FN0_WORKER_TARGET_LOADED=1

WORKER_CONVERGE_TIMEOUT="${WORKER_CONVERGE_TIMEOUT:-1800}"
WORKER_CONVERGE_POLL_INTERVAL="${WORKER_CONVERGE_POLL_INTERVAL:-10}"
WORKER_CONVERGE_HOST_LIVE_WINDOW="${WORKER_CONVERGE_HOST_LIVE_WINDOW:-90}"
WORKER_CONVERGE_ZERO_LIVE_LIMIT="${WORKER_CONVERGE_ZERO_LIVE_LIMIT:-18}"

__write_target_image_doc() {
  local pk="$1" image_ref="$2" data
  if [[ -z "$pk" || -z "$image_ref" ]]; then
    echo "__write_target_image_doc: missing pk or image_ref" >&2
    return 2
  fi
  data="$(jq -nc --arg r "$image_ref" '{image_ref:$r}')"
  control_db_put "$pk" "" "$data" >/dev/null
  echo ">> ${pk}.image_ref = ${image_ref}"
}

write_worker_target_image() {
  local image_ref="$1"
  if [[ -z "$image_ref" ]]; then
    echo "write_worker_target_image: missing image_ref" >&2
    return 2
  fi
  __write_target_image_doc "TargetFn0WorkerConfigDoc" "$image_ref"
}

__worker_status_query_all() {
  local after_sk="" response page documents='[]' count
  while :; do
    response="$(control_db_query "WorkerHostStatusDoc" "$after_sk" 500)" || return 1
    page="$(jq -c '.documents' <<<"$response")"
    count="$(jq 'length' <<<"$page")"
    documents="$(jq -cn --argjson current "$documents" --argjson page "$page" '$current + $page')"
    if [[ "$count" -lt 500 ]]; then break; fi
    after_sk="$(jq -r '.[-1].sk' <<<"$page")"
  done
  jq -nc --argjson documents "$documents" '[ $documents[] | .data_base64 | @base64d | fromjson ]'
}

wait_worker_target_converged() {
  local target_image_ref="$1"
  if [[ -z "$target_image_ref" ]]; then
    echo "wait_worker_target_converged: missing target_image_ref" >&2
    return 2
  fi
  echo ">> waiting for hosts to converge to ${target_image_ref} (timeout ${WORKER_CONVERGE_TIMEOUT}s, abort after ${WORKER_CONVERGE_ZERO_LIVE_LIMIT} live=0 polls)"
  local start_ts deadline now elapsed rows now_epoch zero_live_count
  start_ts="$(date +%s)"
  deadline=$((start_ts + WORKER_CONVERGE_TIMEOUT))
  zero_live_count=0

  while :; do
    if ! rows="$(__worker_status_query_all)"; then
      echo "  control DB query failed; retrying" >&2
    else
      now_epoch="$(date +%s)"
      local report
      report="$(jq -c \
        --argjson now "$now_epoch" \
        --argjson live_window "$WORKER_CONVERGE_HOST_LIVE_WINDOW" \
        --arg target "$target_image_ref" '
        map(select(. != null and (.reported_at // 0) >= ($now - $live_window)))
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
      elapsed=$((now - start_ts))
      echo "timeout after ${elapsed}s; some hosts not converged" >&2
      return 1
    fi
    sleep "$WORKER_CONVERGE_POLL_INTERVAL"
  done
}
