#!/usr/bin/env bash
#
# The feature probe: every read route, on a schedule, for as long as a soak
# runs.
#
#   scripts/soak_probe.sh <out-dir>
#
# A soak already says whether the engine stays inside its memory, whether parts
# and the WAL reach a steady state, and whether flush, merge and retention keep
# turning. It says nothing about whether the product still works. The load
# harness drives six log query shapes and five metric ones; the other eleven
# routes -- the autocompletes, the tail, the deletion surface, the trace
# timeline, the admin lifecycle -- had never been asked anything by a long run,
# so a route that started answering 500 on hour nine would have finished the
# soak with a clean verdict.
#
# So this walks the whole surface in docs/QUERY_API.md plus the admin routes,
# once every PROBE_INTERVAL, and writes one CSV row per probe per round. The
# artifact is a table of per-route success rates over 24 hours, which is what
# "everything still works" has to mean if it is to mean anything.
#
# What it does not do is judge. Rows go in the file; the verdict is computed at
# the end, over the whole run, where a single 429 in hour three can be seen for
# what it is instead of failing a round.
#
#   PROBE_INTERVAL=60 scripts/soak_probe.sh target/soak/run
#   PROBE_ROUNDS=1    scripts/soak_probe.sh target/soak/run   # one pass, for a smoke
set -uo pipefail

OUT="${1:?usage: soak_probe.sh <out-dir>}"
mkdir -p "$OUT"

ADDR="${PROBE_ADDR:-127.0.0.1:3190}"
BASE="http://$ADDR"
API="$BASE/signy/api/v1"
# The tenant the load harness writes logs, traces and metrics under: its corpus
# names them `<prefix>-<index>` and every leg uses the first.
TENANT="${PROBE_TENANT:-load-tenant-000}"
# A tenant of this probe's own, for the parts of the admin surface that create
# and destroy. Never one the load is writing to: a retention policy deleted out
# from under the harness would un-serve its tenant and drop every push after it.
ADMIN_TENANT="${PROBE_ADMIN_TENANT:-probe-tenant}"
INTERVAL="${PROBE_INTERVAL:-300}"
ROUNDS="${PROBE_ROUNDS:-0}"
# Instruments the metric leg publishes (src/bin/load/metric_workload.rs). A
# name this instance never had would make every metric probe read as a failure
# of the engine rather than of this script's guess.
METRIC="${PROBE_METRIC:-http_requests_total}"
HISTOGRAM="${PROBE_HISTOGRAM:-http_request_duration_seconds}"
# How long the tail is held open. Past the ~15 s keep-alive on purpose: on a
# tenant with nothing arriving, the heartbeat is the only thing that proves the
# stream is alive, and a window shorter than it would fail every round after the
# load stops.
TAIL_SECONDS="${PROBE_TAIL_SECONDS:-18}"
# Rounds between deletion-surface probes. Not every round: a request the engine
# promotes to `processed` before the probe withdraws it can no longer be
# withdrawn and stays in the tenant's listing for good, and a tenant may hold
# only MAX_DELETE_REQUESTS_PER_TENANT of them. Every sixth round is 48 probes
# across a day, which is far under that and still says the surface works.
DELETE_EVERY="${PROBE_DELETE_EVERY:-6}"

CSV="$OUT/probe.csv"
BODY=$(mktemp); HEAD=$(mktemp)
trap 'rm -f "$BODY" "$HEAD"' EXIT
[ -f "$CSV" ] || echo "round,t,probe,status,rows,ok,detail" >"$CSV"

# A tenant is served when a retention policy has been pushed for it, and the
# load harness pushes those for its own tenants as it starts. This probe is
# started first, so without this wait its first round asks every route about a
# tenant that does not exist yet and records the whole surface as broken.
wait_for_tenant() {
  local waited=0
  while [ "$waited" -lt "${PROBE_TENANT_WAIT:-600}" ]; do
    if [ "$(req GET "$API/admin/tenants/$TENANT/retention")" = "200" ]; then
      return 0
    fi
    sleep 5
    waited=$((waited + 5))
  done
  echo "tenant $TENANT was never onboarded; probing anyway"
}

T0=$(date +%s)
ROUND=0
PASS=0
FAIL=0
SKIP=0

# One request. Leaves the body in $BODY and prints the status code, or `000`
# when nothing answered -- which is a failure the same as a 500 and has to be
# distinguishable from one in the file.
req() {
  local method="$1" url="$2" tenant="${3:-$TENANT}" code
  # Emptied first: a curl that cannot connect leaves the previous probe's body
  # in place, and the failure would then be recorded with another route's
  # answer as its evidence.
  : >"$BODY"
  # No `|| echo`: curl prints `000` for a connection that never happened and
  # exits non-zero as well, so a fallback echo would put two codes in the
  # substitution and a newline in the middle of a CSV row.
  code=$(curl -sS -o "$BODY" -D "$HEAD" -w '%{http_code}' \
    --max-time 30 -X "$method" -H "X-Tenant-Id: $tenant" "$url" 2>/dev/null)
  echo "${code:-000}"
}

# Non-empty lines in the last body. awk rather than `grep -c`, which exits 1 on
# a count of zero and would put a second line in the substitution — a newline in
# the middle of a CSV row, which reads back as a row of its own.
rows() {
  local n
  n=$(awk 'NF { n++ } END { print n + 0 }' "$BODY" 2>/dev/null)
  echo "${n:-0}"
}

# Record one probe. `detail` is whatever makes a failure diagnosable a day
# later without the server still being up.
#
# `ok` is 1, 0, or `skip`. Skipped is its own state and not a quiet failure:
# a probe this round deliberately did not run says nothing about the route, and
# folding it into either column would make the end-of-run table a lie.
record() {
  local name="$1" status="$2" row_count="$3" ok="$4" detail="${5:-}"
  printf '%d,%d,%s,%s,%s,%s,"%s"\n' \
    "$ROUND" "$(( $(date +%s) - T0 ))" "$name" "$status" "$row_count" "$ok" \
    "${detail//\"/\'}" >>"$CSV"
  case "$ok" in
    1) PASS=$((PASS + 1)) ;;
    skip) SKIP=$((SKIP + 1)) ;;
    *) FAIL=$((FAIL + 1)) ;;
  esac
}

# Every probe asks twice before it calls a route broken.
#
# A restart lands in the middle of a round often enough to matter -- the engine
# is back in about a second -- and recording fourteen route failures for one
# scheduled restart would drown the thing this file exists to show. A route
# that is actually broken fails both asks.
#
# probe_get: a 200 whose body has at least $3 non-empty lines, and optionally
# contains $4.
probe_get() {
  local name="$1" url="$2" want="${3:-1}" want_text="${4:-}"
  local status got attempt
  for attempt in 1 2; do
    status=$(req GET "$url")
    got=$(rows)
    if [ "$status" = "200" ] && [ "$got" -ge "$want" ] \
       && { [ -z "$want_text" ] || grep -q "$want_text" "$BODY"; }; then
      record "$name" "$status" "$got" 1 \
        "$([ "$attempt" = 2 ] && echo 'answered on the second ask')"
      return
    fi
    [ "$attempt" = 1 ] && sleep 2
  done
  local why="$(head -c 160 "$BODY" | tr -d '\n')"
  [ -n "$want_text" ] && [ "$status" = "200" ] && why="does not contain $want_text"
  record "$name" "$status" "$got" 0 "$why"
}

# probe_status: the status code is the whole of the check. For routes whose
# answer may legitimately be empty, and for the fallback's 404.
probe_status() {
  local name="$1" url="$2" want="${3:-200}"
  local status attempt
  for attempt in 1 2; do
    status=$(req GET "$url")
    if [ "$status" = "$want" ]; then
      record "$name" "$status" "$(rows)" 1 \
        "$([ "$attempt" = 2 ] && echo 'answered on the second ask')"
      return
    fi
    [ "$attempt" = 1 ] && sleep 2
  done
  record "$name" "$status" "$(rows)" 0 "expected $want: $(head -c 140 "$BODY" | tr -d '\n')"
}

# The first value of a JSON string field, off the first NDJSON line. The bodies
# here are one flat object per line, so this is enough and pulling in jq for it
# would put a dependency in the soak rig.
field() {
  head -1 "$BODY" | grep -o "\"$1\":\"[^\"]*\"" | head -1 | cut -d'"' -f4
}

round() {
  ROUND=$((ROUND + 1))
  PASS=0; FAIL=0; SKIP=0

  # --- liveness -----------------------------------------------------------
  status=$(req GET "$BASE/ready")
  if [ "$status" != "200" ]; then
    # The engine is not answering. Every route would fail, and recording
    # twenty-eight failures would say the surface broke when what happened is
    # that the process was down -- which `ready` above already records, and
    # which the run's fault log explains. So the rest of the round is skipped.
    record ready "$status" 0 0 "the engine did not answer /ready"
    for route in metrics logs logs_filtered logs_histogram logs_attributes \
                 logs_attribute_values logs_tail delete_submit delete_list \
                 delete_cancel traces_search traces_by_id traces_attributes \
                 traces_attribute_values metrics_names metrics_labels \
                 metrics_label_values metrics_series metrics_query \
                 metrics_instant metrics_quantile admin_list \
                 admin_retention_get admin_usage admin_retention_put \
                 admin_retention_delete unknown_route; do
      record "$route" skipped 0 skip "the engine was not ready"
    done
    echo "round $ROUND at +$(( $(date +%s) - T0 ))s: engine not ready, round skipped"
    return
  fi
  record ready "$status" 0 1
  probe_get metrics "$BASE/metrics" 1 '^signy_ingest_requests_total'

  # --- logs ---------------------------------------------------------------
  probe_get logs "$API/logs?start=-5m&limit=5" 1

  # The filtering grammar rather than the route: a line filter, a parser and a
  # direction in one request. An empty answer is legitimate here -- the filters
  # may match nothing in the window -- so this one is gated on the status.
  probe_status logs_filtered "$API/logs?start=-5m&limit=5&parse=logfmt&direction=forward&contains=e"

  probe_get logs_histogram "$API/logs/histogram?start=-15m&bucket=1m" 1
  probe_get logs_attributes "$API/logs/attributes?start=-5m" 1
  key=$(field key)
  if [ -n "$key" ]; then
    probe_get logs_attribute_values "$API/logs/attributes/$key/values?start=-5m" 1
  else
    record logs_attribute_values skipped 0 skip "the keys probe offered none"
  fi

  # The tail is a stream, so it is timed out on purpose and whatever arrived is
  # the answer. A heartbeat counts: it is the route proving it is streaming on a
  # tenant that happens to be quiet.
  tailed=0
  for attempt in 1 2; do
    : >"$BODY"
    curl -sN --max-time "$TAIL_SECONDS" -H "X-Tenant-Id: $TENANT" \
      "$API/logs/tail?limit=10" >"$BODY" 2>/dev/null
    tailed=$(rows)
    [ "$tailed" -ge 1 ] && break
  done
  [ "$tailed" -ge 1 ] && record logs_tail stream "$tailed" 1 \
    || record logs_tail stream "$tailed" 0 "nothing in ${TAIL_SECONDS}s, twice"

  # --- deletion surface ---------------------------------------------------
  # Deliberately a selector nothing matches. Submitting, listing and cancelling
  # is the whole of what this can check without hiding rows the rest of the
  # soak is about to query.
  if [ $(( ROUND % DELETE_EVERY )) -ne 1 ]; then
    record delete_submit skipped 0 skip "runs every ${DELETE_EVERY} rounds"
    record delete_list skipped 0 skip "runs every ${DELETE_EVERY} rounds"
    record delete_cancel skipped 0 skip "runs every ${DELETE_EVERY} rounds"
    status=skipped
  else
  status=$(req POST "$API/logs/delete?attr=service_name=__probe_no_such_service__&start=-2m")
  if [ "$status" = "204" ]; then
    record delete_submit "$status" 0 1
    probe_get delete_list "$API/logs/delete" 1
    id=$(field request_id)
    [ -n "$id" ] || id=$(head -1 "$BODY" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    if [ -n "$id" ]; then
      status=$(req DELETE "$API/logs/delete?request_id=$id")
      if [ "$status" = "204" ]; then
        record delete_cancel "$status" 0 1
      elif grep -q "already been applied" "$BODY"; then
        # The request finished its life before the probe could withdraw it.
        # That is the deletion path working, not the cancel route failing.
        record delete_cancel "$status" 0 1 "applied before it could be withdrawn"
      else
        record delete_cancel "$status" 0 0 "$(head -c 200 "$BODY" | tr -d '\n')"
      fi
    else
      record delete_cancel skipped 0 0 "the listing named no request id"
    fi
  else
    record delete_submit "$status" 0 0 "$(head -c 200 "$BODY" | tr -d '\n')"
    record delete_list skipped 0 skip "no request was accepted"
    record delete_cancel skipped 0 skip "no request was accepted"
  fi
  fi

  # --- traces -------------------------------------------------------------
  probe_get traces_search "$API/traces?start=-5m&limit=5" 1
  trace=$(field trace_id)
  if [ -n "$trace" ]; then
    probe_get traces_by_id "$API/traces/$trace" 1
  else
    record traces_by_id skipped 0 skip "the search returned no trace to fetch"
  fi
  probe_get traces_attributes "$API/traces/attributes?start=-5m" 1
  key=$(field key)
  if [ -n "$key" ]; then
    probe_get traces_attribute_values "$API/traces/attributes/$key/values?start=-5m" 1
  else
    record traces_attribute_values skipped 0 skip "the keys probe offered none"
  fi

  # --- metrics ------------------------------------------------------------
  probe_get metrics_names "$API/metrics/names?start=-5m" 1
  probe_get metrics_labels "$API/metrics/labels?start=-5m&metric=$METRIC" 1
  key=$(field key)
  if [ -n "$key" ]; then
    probe_get metrics_label_values "$API/metrics/labels/$key/values?start=-5m&metric=$METRIC" 1
  else
    record metrics_label_values skipped 0 skip "the keys probe offered none"
  fi
  probe_get metrics_series "$API/metrics/series?metric=$METRIC&start=-5m&limit=5" 1
  probe_get metrics_query \
    "$API/metrics/query?metric=$METRIC&start=-5m&step=30s&func=rate&range=60s&agg=sum&by=service" 1
  probe_get metrics_instant "$API/metrics/instant?metric=$METRIC&func=rate&range=60s&agg=max" 1
  probe_get metrics_quantile \
    "$API/metrics/quantile?metric=$HISTOGRAM&q=0.99&start=-5m&step=30s&range=60s" 1

  # --- admin --------------------------------------------------------------
  probe_get admin_list "$API/admin/tenants" 1 "$TENANT"
  probe_get admin_retention_get "$API/admin/tenants/$TENANT/retention" 1
  probe_get admin_usage "$API/admin/tenants/$TENANT/usage" 1

  # The half of the lifecycle that writes, on a tenant of this probe's own.
  : >"$BODY"
  status=$(curl -sS -o "$BODY" -w '%{http_code}' --max-time 30 \
    -X PUT -H 'Content-Type: application/json' \
    -d '{"retention":"1h"}' "$API/admin/tenants/$ADMIN_TENANT/retention" 2>/dev/null)
  status="${status:-000}"
  if [ "$status" = "200" ] || [ "$status" = "204" ]; then
    record admin_retention_put "$status" 0 1
    status=$(req DELETE "$API/admin/tenants/$ADMIN_TENANT/retention" "$ADMIN_TENANT")
    { [ "$status" = "200" ] || [ "$status" = "204" ]; } \
      && record admin_retention_delete "$status" 0 1 \
      || record admin_retention_delete "$status" 0 0 "$(head -c 200 "$BODY" | tr -d '\n')"
  else
    record admin_retention_put "$status" 0 0 "$(head -c 200 "$BODY" | tr -d '\n')"
    record admin_retention_delete skipped 0 skip "no policy was written to withdraw"
  fi

  # --- the fallback -------------------------------------------------------
  # A 404 that lists the real routes is a feature of the API, and a router that
  # started answering something else would be a regression nothing else here
  # would catch.
  probe_status unknown_route "$API/no-such-route" 404

  echo "round $ROUND at +$(( $(date +%s) - T0 ))s: $PASS ok, $FAIL failed, $SKIP skipped"
}

wait_for_tenant

while true; do
  round
  [ "$ROUNDS" -gt 0 ] && [ "$ROUND" -ge "$ROUNDS" ] && break
  sleep "$INTERVAL"
done
