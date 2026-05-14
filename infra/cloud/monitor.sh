#!/bin/bash
# fn0 platform health monitor.
# Combines client-side latency probes against a deployed app with Grafana
# data-plane queries (Prometheus + Loki), then flags anomalies.
#
# Run from infra/cloud/ — needs `pulumi config get` for the Grafana token.
#
#   FN0_MONITOR_HOST     app host to probe        (default namse-mottomite.fn0.dev)
#   FN0_MONITOR_WINDOW   metric/log window, secs  (default 3600)
#   FN0_MONITOR_SAMPLES  curl samples per path    (default 5)
#
# The /debug2 and /debug routes are a differential probe pair:
#   /debug2 — no backend deps; slow here => fn0 platform core
#   /debug  — turso x4 sequential; slow only here => turso hijack / doc-db
set -uo pipefail

HOST="${FN0_MONITOR_HOST:-namse-mottomite.fn0.dev}"
WINDOW="${FN0_MONITOR_WINDOW:-3600}"
SAMPLES="${FN0_MONITOR_SAMPLES:-5}"

TOKEN=$(pulumi config get grafana:cloudAccessPolicyToken)
STACK_JSON=$(curl -s -H "Authorization: Bearer $TOKEN" https://grafana.com/api/instances/fn0)
read -r PROM PROMID LOKI LOKIID <<EOF
$(python3 - "$STACK_JSON" <<'PY'
import sys, json
d = json.loads(sys.argv[1])
print(d["hmInstancePromUrl"], d["hmInstancePromId"], d["hlInstanceUrl"], d["hlInstanceId"])
PY
)
EOF

NOW=$(python3 -c 'import time;print(int(time.time()))')
START_NS=$(( (NOW - WINDOW) * 1000000000 ))
END_NS=$(( NOW * 1000000000 ))

FLAGS=()

echo "=================================================================="
echo " fn0 monitor — host=$HOST  window=${WINDOW}s  $(date -u +%FT%TZ)"
echo "=================================================================="

# --- Section A: client-side latency probes -------------------------------
echo
echo "## A. HTTP latency probes ($SAMPLES samples each)"
probe() {
  local path="$1" want="$2"
  local codes="" ttfbs=""
  for _ in $(seq "$SAMPLES"); do
    local out
    out=$(curl -s -o /tmp/fn0_monitor_body -w '%{http_code} %{time_starttransfer}' -m 30 "https://$HOST$path" 2>/dev/null || echo "000 0")
    codes="$codes ${out%% *}"
    ttfbs="$ttfbs ${out##* }"
  done
  local body_ok="no"
  grep -q "$want" /tmp/fn0_monitor_body 2>/dev/null && body_ok="yes"
  python3 - "$path" "$body_ok" "$codes" "$ttfbs" <<'PY'
import sys
path, body_ok = sys.argv[1], sys.argv[2]
codes = sys.argv[3].split()
ttfbs = sorted(float(x) for x in sys.argv[4].split())
bad = [c for c in codes if c != "200"]
p50 = ttfbs[len(ttfbs)//2]
print(f"  {path:9s} codes={','.join(codes)}  ttfb min={ttfbs[0]:.3f} p50={p50:.3f} max={ttfbs[-1]:.3f}  body_ok={body_ok}")
PY
  echo "$codes" | grep -qv 200 && FLAGS+=("non-200 from $path:$codes")
  [ "$body_ok" = "no" ] && FLAGS+=("$path body content unexpected")
}
probe /debug2 "no turso"
probe /debug  "fn0 debug"

# --- Section B: Prometheus metrics ---------------------------------------
echo
echo "## B. Grafana metrics (last ${WINDOW}s)"
promq() {
  curl -s -u "$PROMID:$TOKEN" --data-urlencode "query=$1" "$PROM/api/prom/api/v1/query" 2>/dev/null
}
echo "  stage_duration_seconds avg per stage:"
promq "sum by (stage,code_id)(rate(stage_duration_seconds_sum[${WINDOW}s])) / sum by (stage,code_id)(rate(stage_duration_seconds_count[${WINDOW}s]))" \
 | python3 -c "import sys,json;
try:
  r=json.load(sys.stdin)['data']['result']
  [print(f\"    {x['metric'].get('stage','?'):22s} {x['metric'].get('code_id','?'):20s} {float(x['value'][1])*1000:8.1f} ms\") for x in sorted(r,key=lambda x:x['metric'].get('stage',''))]
  if not r: print('    (no stage_duration metrics yet — worker not redeployed?)')
except Exception as e: print('    query failed:',e)"

echo "  error counters (increase over window):"
for m in panicked request_deadline_exceeded oneshot_drop_before_response cpu_timeout wasmtime_error trapped canceled_unexpectedly proxy_returns_error_code create_instance; do
  v=$(promq "sum(increase(${m}_total[${WINDOW}s]))" | python3 -c "import sys,json;
try:
  r=json.load(sys.stdin)['data']['result']
  print(f\"{float(r[0]['value'][1]):.0f}\" if r else '0')
except: print('0')")
  printf "    %-32s %s\n" "$m" "$v"
  case "$m" in
    panicked|request_deadline_exceeded|oneshot_drop_before_response|cpu_timeout)
      [ "${v%.*}" -gt 0 ] 2>/dev/null && FLAGS+=("$m=$v over window") ;;
    proxy_returns_error_code)
      [ "${v%.*}" -gt 5 ] 2>/dev/null && FLAGS+=("$m=$v over window") ;;
  esac
done

# --- Section C: Loki recent panics / errors ------------------------------
echo
echo "## C. Loki — worker restarts & errors (last ${WINDOW}s)"
lokiq() {
  curl -s -u "$LOKIID:$TOKEN" \
    --data-urlencode "query=$1" \
    --data-urlencode "start=$START_NS" --data-urlencode "end=$END_NS" \
    --data-urlencode "limit=${2:-20}" --data-urlencode "direction=forward" \
    "$LOKI/loki/api/v1/query_range" 2>/dev/null
}
lokiq '{service_name="fn0-worker"} |~ "worker threads started|executor panicked|panic captured|wasm instance dead|request exceeded deadline|response body collect failed|wasm instance loop panicked"' 30 \
 | python3 -c "import sys,json,datetime,collections
try:
  rs=json.load(sys.stdin)['data']['result']
  lines=[(v[0],v[1]) for s in rs for v in s.get('values',[])]
  lines.sort()
  counts=collections.Counter()
  for _,m in lines:
    for key in ['worker threads started','executor panicked','panic captured','wasm instance dead','request exceeded deadline','response body collect failed','wasm instance loop panicked']:
      if key in m: counts[key]+=1
  for k,c in counts.items(): print(f'    {c:4d}  {k}')
  if not counts: print('    (clean)')
except Exception as e: print('    query failed:',e)"

loki_count() {
  lokiq "$1" 500 | python3 -c "import sys,json;
try: print(sum(len(s.get('values',[])) for s in json.load(sys.stdin)['data']['result']))
except: print(0)"
}
restarts=$(loki_count '{service_name="fn0-worker"} |= "worker threads started"')
panics=$(loki_count '{service_name="fn0-worker"} |~ "executor panicked|wasm instance loop panicked"')
deadlines=$(loki_count '{service_name="fn0-worker"} |= "request exceeded deadline"')
[ "${restarts:-0}" -gt 2 ] 2>/dev/null && FLAGS+=("$restarts worker restarts over window (instability)")
[ "${panics:-0}" -gt 0 ] 2>/dev/null && FLAGS+=("$panics worker panics over window (Loki)")
[ "${deadlines:-0}" -gt 0 ] 2>/dev/null && FLAGS+=("$deadlines requests hit wall-clock deadline")

# --- Section D: anomaly summary ------------------------------------------
echo
echo "## D. Anomaly flags"
if [ "${#FLAGS[@]}" -eq 0 ]; then
  echo "  none — platform looks healthy"
else
  for f in "${FLAGS[@]}"; do echo "  ⚠ $f"; done
fi
echo
