#!/usr/bin/env bash
#
# Ingest-only active-series capacity sweep for Signy, VictoriaMetrics and
# Grafana Mimir. Every trial starts one service with a fresh volume.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT/compare"

env_or() {
  local value
  value=$(printenv "$1" 2>/dev/null || true)
  if [[ -n "$value" ]]; then printf '%s' "$value"; else printf '%s' "$2"; fi
}

OUT=$(env_or CAPACITY_SWEEP_OUT "$ROOT/target/metric-capacity")
MEMORY=$(env_or COMPARE_MEMORY 2g)
TARGETS=$(env_or CAPACITY_SWEEP_TARGETS "signy victoriametrics mimir")
LOWER=$(env_or CAPACITY_SWEEP_LOWER 10000)
UPPER=$(env_or CAPACITY_SWEEP_UPPER 2000000)
TOLERANCE=$(env_or CAPACITY_SWEEP_TOLERANCE 10000)
RAMP_FACTOR=$(env_or CAPACITY_SWEEP_RAMP_FACTOR 2)
HOLD_SECONDS=$(env_or CAPACITY_SWEEP_HOLD_SECONDS 1)
ANCHOR_NS=$(env_or CAPACITY_SWEEP_ANCHOR_NS "$(( $(date +%s) * 1000000000 - 3600000000000 ))")
PROBE_VALUE=$(env_or CAPACITY_SWEEP_PROBE 0)
DRY_RUN=false
PROBE=false

usage() {
  cat <<'EOF'
Usage: compare/run_metric_capacity.sh [options]

  --targets LIST       space-separated signy victoriametrics mimir
  --memory VALUE       Docker memory and memswap limit (default: 2g)
  --lower N            first exponential-ramp candidate (default: 10000)
  --upper N            largest candidate to try (default: 2000000)
  --tolerance N        binary-search uncertainty in series (default: 10000)
  --ramp-factor N      exponential-ramp multiplier (default: 2)
  --hold-seconds N     seconds to keep the one-shot population observable
  --out DIR            artifact directory (default: target/metric-capacity)
  --probe              enable the disposable Signy-only capacity probe
  --dry-run            validate and print the plan; do not use Docker
  -h, --help           show this help

CAPACITY_SWEEP_* environment variables mirror these options. Existing
trials.jsonl entries are reused, so rerunning resumes at missing points.
EOF
}

while (($# > 0)); do
  case "$1" in
    --targets) (($# >= 2)) || { echo "--targets needs a value" >&2; exit 2; }; TARGETS=$2; shift 2 ;;
    --memory) (($# >= 2)) || { echo "--memory needs a value" >&2; exit 2; }; MEMORY=$2; shift 2 ;;
    --lower) (($# >= 2)) || { echo "--lower needs a value" >&2; exit 2; }; LOWER=$2; shift 2 ;;
    --upper) (($# >= 2)) || { echo "--upper needs a value" >&2; exit 2; }; UPPER=$2; shift 2 ;;
    --tolerance) (($# >= 2)) || { echo "--tolerance needs a value" >&2; exit 2; }; TOLERANCE=$2; shift 2 ;;
    --ramp-factor) (($# >= 2)) || { echo "--ramp-factor needs a value" >&2; exit 2; }; RAMP_FACTOR=$2; shift 2 ;;
    --hold-seconds) (($# >= 2)) || { echo "--hold-seconds needs a value" >&2; exit 2; }; HOLD_SECONDS=$2; shift 2 ;;
    --out) (($# >= 2)) || { echo "--out needs a value" >&2; exit 2; }; OUT=$2; shift 2 ;;
    --probe) PROBE=true; shift ;;
    --dry-run) DRY_RUN=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ "$PROBE" == false ]]; then
  case "${PROBE_VALUE,,}" in
    1|true|yes) PROBE=true ;;
    0|false|no) PROBE=false ;;
    *) echo "CAPACITY_SWEEP_PROBE must be 0 or 1" >&2; exit 2 ;;
  esac
fi

is_uint() { [[ "$1" =~ ^[0-9]+$ ]]; }
for pair in "lower:$LOWER" "upper:$UPPER" "tolerance:$TOLERANCE" \
  "ramp-factor:$RAMP_FACTOR" "hold-seconds:$HOLD_SECONDS"; do
  name=$(cut -d: -f1 <<<"$pair")
  value=$(cut -d: -f2- <<<"$pair")
  is_uint "$value" || { echo "$name must be a non-negative integer: $value" >&2; exit 2; }
done
(( LOWER >= 1 )) || { echo "lower must be at least 1" >&2; exit 2; }
(( UPPER >= LOWER )) || { echo "upper must be >= lower" >&2; exit 2; }
(( TOLERANCE >= 1 )) || { echo "tolerance must be at least 1" >&2; exit 2; }
(( RAMP_FACTOR >= 2 )) || { echo "ramp-factor must be at least 2" >&2; exit 2; }
for target in $TARGETS; do
  case "$target" in
    signy|victoriametrics|mimir) ;;
    *) echo "unsupported capacity target: $target" >&2; exit 2 ;;
  esac
done
if [[ "$PROBE" == true ]]; then
  for target in $TARGETS; do
    [[ "$target" == signy ]] || {
      echo "--probe is only supported with the signy target (got $target)" >&2
      exit 2
    }
  done
fi

if [[ "$DRY_RUN" == true ]]; then
  printf 'targets=%s\nmemory=%s\nlower=%s\nupper=%s\ntolerance=%s\nramp_factor=%s\nhold_seconds=%s\nprobe=%s\nout=%s\n' \
    "$TARGETS" "$MEMORY" "$LOWER" "$UPPER" "$TOLERANCE" "$RAMP_FACTOR" "$HOLD_SECONDS" "$PROBE" "$OUT"
  exit 0
fi

for command in docker curl jq cargo; do
  command -v "$command" >/dev/null 2>&1 || { echo "required command not found: $command" >&2; exit 2; }
done
docker compose version >/dev/null 2>&1 || { echo "Docker Compose v2 is required" >&2; exit 2; }

mkdir -p "$OUT"
JSONL="$OUT/trials.jsonl"
CSV="$OUT/trials.csv"
MANIFEST="$OUT/manifest.json"
touch "$JSONL"
CSV_HEADER='recorded_at,target,probe,requested_series,offered_series,accepted_series,refused_series,offered_datapoints,accepted_datapoints,refused_datapoints,partial_rejected_datapoints,status_200,status_429,anon_peak_bytes,cgroup_memory_peak_bytes,cgroup_limit_bytes,alive,oom_killed,harness_exit,elapsed_seconds,latency_max_ms,pass,safe_saturation,result_path,stderr_path'
if [[ ! -s "$CSV" ]]; then
  printf '%s\n' "$CSV_HEADER" >"$CSV"
elif [[ "$(head -n 1 "$CSV")" != "$CSV_HEADER" ]]; then
  echo "existing CSV has an incompatible schema; use a new --out directory" >&2
  exit 2
fi
if [[ ! -e "$MANIFEST" ]]; then
  jq -n \
    --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg memory "$MEMORY" --arg targets "$TARGETS" \
    --argjson lower "$LOWER" --argjson upper "$UPPER" \
    --argjson tolerance "$TOLERANCE" --argjson ramp_factor "$RAMP_FACTOR" \
    --argjson hold_seconds "$HOLD_SECONDS" --argjson anchor_ns "$ANCHOR_NS" \
    --argjson probe "$PROBE" \
    '{schema: 1, generated_at: $generated_at, memory: $memory,
      targets: ($targets | split(" ") | map(select(length > 0))),
      lower: $lower, upper: $upper, tolerance: $tolerance,
      ramp_factor: $ramp_factor, hold_seconds: $hold_seconds,
      metric_anchor_ns: $anchor_ns, probe: $probe,
      definition: "largest trial with alive=true, oom_killed=false, 100% series/request/datapoint acceptance, and anon_peak_bytes < cgroup_limit_bytes"}' \
    >"$MANIFEST"
else
  manifest_probe=$(jq -r '.probe // false' "$MANIFEST" 2>/dev/null) || {
    echo "existing manifest is invalid or predates probe recording; use a new --out directory" >&2
    exit 2
  }
  [[ "$manifest_probe" == "$PROBE" ]] || {
    echo "existing manifest probe=$manifest_probe does not match requested probe=$PROBE; use a new --out directory" >&2
    exit 2
  }
fi

compose() {
  if [[ "$PROBE" == true ]]; then
    SIGNY_CAPACITY_PROBE=1 docker compose --profile metrics "$@"
  else
    SIGNY_CAPACITY_PROBE= docker compose --profile metrics "$@"
  fi
}
port_of() {
  case "$1" in
    signy) env_or SIGNY_PORT 3110 ;;
    victoriametrics) env_or VICTORIAMETRICS_PORT 3140 ;;
    mimir) env_or MIMIR_PORT 3150 ;;
  esac
}
ready_path_of() {
  case "$1" in
    victoriametrics) echo /health ;;
    *) echo /ready ;;
  esac
}
container_of() {
  # Resolve through Compose so COMPOSE_PROJECT_NAME can isolate a sweep from
  # another comparison stack and the script never guesses a container name.
  compose ps -q "$1" | tail -n 1
}

wait_ready() {
  local target=$1 port path tries=0
  port=$(port_of "$target")
  path=$(ready_path_of "$target")
  until [[ "$(curl -sS -o /dev/null -w '%{http_code}' --max-time 2 \
    "http://127.0.0.1:$port$path" 2>/dev/null || true)" == 200 ]]; do
    tries=$((tries + 1))
    if (( tries > 120 )); then
      echo "$target never became ready" >&2
      compose logs --tail 40 "$target" >&2 || true
      return 1
    fi
    sleep 1
  done
}

cgroup_of() {
  local container=$1 pid
  pid=$(docker inspect -f '{{.State.Pid}}' "$container")
  [[ "$pid" =~ ^[1-9][0-9]*$ ]] || return 1
  echo "/sys/fs/cgroup$(cut -d: -f3 "/proc/$pid/cgroup")"
}
num_or_zero() {
  [[ "$1" =~ ^[0-9]+([.][0-9]+)?$ ]] && echo "$1" || echo 0
}
csv_quote() {
  printf '"%s"' "$1"
}
trial_exists() {
  local target=$1 candidate=$2
  [[ -s "$JSONL" ]] && jq -e --arg target "$target" --argjson candidate "$candidate" \
    --argjson probe "$PROBE" \
    'select(.target == $target and .requested_series == $candidate and (.probe // false) == $probe)' "$JSONL" >/dev/null 2>&1
}
LAST_PASS=false

run_trial() {
  local target=$1 candidate=$2 container cg limit result stderr stdout
  local harness_exit=0 alive=false oom=true cgroup_peak=0
  local offered_series accepted_series refused_series offered_dp accepted_dp refused_dp partial_dp
  local status_200 status_429 anon_peak elapsed latency statuses_json
  local pass=false safe=false record

  if trial_exists "$target" "$candidate"; then
    LAST_PASS=$(jq -r --arg target "$target" --argjson candidate "$candidate" \
      --argjson probe "$PROBE" \
      'select(.target == $target and .requested_series == $candidate and (.probe // false) == $probe) | .pass' "$JSONL" | tail -1)
    echo "reuse $target candidate=$candidate pass=$LAST_PASS" >&2
    return 0
  fi

  result="$OUT/$target""_$candidate.json"
  stderr="$OUT/$target""_$candidate.stderr.log"
  stdout="$OUT/$target""_$candidate.stdout.log"
  echo "trial $target candidate=$candidate" >&2
  compose down -v --remove-orphans >/dev/null 2>&1 || true
  compose up -d "$target" >/dev/null
  wait_ready "$target"
  container=$(container_of "$target")
  cg=$(cgroup_of "$container") || { echo "could not locate cgroup for $container" >&2; return 1; }
  limit=$(cat "$cg/memory.max" 2>/dev/null || echo 0)
  [[ "$limit" =~ ^[0-9]+$ ]] || limit=0

  local -a load_env=(
    "SIGNY_LOAD_TARGET=$target"
    "SIGNY_LOAD_PHASE=metric-load"
    "SIGNY_LOAD_ADDR=127.0.0.1:$(port_of "$target")"
    "SIGNY_LOAD_CGROUP=$cg"
    "SIGNY_LOAD_SEED=1592598566"
    "SIGNY_LOAD_METRIC_ANCHOR_NS=$ANCHOR_NS"
    "SIGNY_LOAD_METRIC_SCRAPES=4"
    "SIGNY_LOAD_METRIC_SCRAPE_SECONDS=1"
    "SIGNY_LOAD_METRIC_SERVICES=1"
    "SIGNY_LOAD_METRIC_INSTANCES=1"
    "SIGNY_LOAD_METRIC_GAUGES=1"
    "SIGNY_LOAD_METRIC_COUNTERS=1"
    "SIGNY_LOAD_METRIC_CONNECTIONS=1"
    "SIGNY_LOAD_METRIC_STEADY_SECONDS=0"
    "SIGNY_LOAD_METRIC_CHURN_SECONDS=0"
    "SIGNY_LOAD_METRIC_CHURN_REPLACE=0"
    "SIGNY_LOAD_METRIC_EXPLOSION_SECONDS=$HOLD_SECONDS"
    "SIGNY_LOAD_METRIC_EXPLOSION_SERIES=$candidate"
    "SIGNY_LOAD_RESULT_PATH=$result"
  )
  if [[ "$PROBE" == true ]]; then
    load_env+=(SIGNY_CAPACITY_PROBE=1)
  fi
  env -u SIGNY_CAPACITY_PROBE "${load_env[@]}" "$LOAD_BIN" >"$stdout" 2>"$stderr" || harness_exit=$?

  alive=$(docker inspect -f '{{.State.Running}}' "$container" 2>/dev/null || echo false)
  oom=$(docker inspect -f '{{.State.OOMKilled}}' "$container" 2>/dev/null || echo true)
  cgroup_peak=$(cat "$cg/memory.peak" 2>/dev/null || echo 0)
  [[ "$cgroup_peak" =~ ^[0-9]+$ ]] || cgroup_peak=0

  if [[ -s "$result" ]] && jq -e . "$result" >/dev/null 2>&1; then
    offered_series=$(jq -r '([.load.phases[]?.series_offered // 0] | add) // .load.series_offered // 0' "$result")
    accepted_series=$(jq -r '([.load.phases[]?.series_accepted // 0] | add) // 0' "$result")
    refused_series=$(jq -r '([.load.phases[]?.series_rejected // 0] | add) // 0' "$result")
    offered_dp=$(jq -r '([.load.phases[]?.datapoints_offered // 0] | add) // 0' "$result")
    accepted_dp=$(jq -r '([.load.phases[]?.datapoints_accepted // 0] | add) // 0' "$result")
    partial_dp=$(jq -r '([.load.phases[]?.datapoints_rejected // 0] | add) // 0' "$result")
    refused_dp=$(jq -r '([.load.phases[]? | (.datapoints_offered - .datapoints_accepted)] | add) // 0' "$result")
    status_200=$(jq -r '([.load.phases[]?.statuses["200"] // 0] | add) // 0' "$result")
    status_429=$(jq -r '([.load.phases[]?.statuses["429"] // 0] | add) // 0' "$result")
    elapsed=$(jq -r '.load.elapsed_seconds // 0' "$result")
    latency=$(jq -r '([.load.phases[]?.latency_ms.max_ms // 0] | max) // 0' "$result")
    statuses_json=$(jq -c '([.load.phases[]?.statuses // {}] | add) // {}' "$result")
    anon_peak=$(jq -r '.memory.anon_peak_bytes // null' "$result")
  else
    offered_series=$((candidate + 13)); accepted_series=0; refused_series=$offered_series
    offered_dp=0; accepted_dp=0; partial_dp=0; refused_dp=0
    status_200=0; status_429=0; elapsed=0; latency=0; statuses_json='{}'; anon_peak=null
  fi
  offered_series=$(num_or_zero "$offered_series")
  accepted_series=$(num_or_zero "$accepted_series")
  refused_series=$(num_or_zero "$refused_series")
  offered_dp=$(num_or_zero "$offered_dp")
  accepted_dp=$(num_or_zero "$accepted_dp")
  partial_dp=$(num_or_zero "$partial_dp")
  refused_dp=$(num_or_zero "$refused_dp")
  status_200=$(num_or_zero "$status_200")
  status_429=$(num_or_zero "$status_429")
  elapsed=$(num_or_zero "$elapsed")
  latency=$(num_or_zero "$latency")
  [[ "$anon_peak" =~ ^[0-9]+$ ]] || anon_peak=null

  if [[ "$alive" == true && "$oom" == false && "$offered_series" -gt 0 \
    && "$accepted_series" -eq "$offered_series" && "$offered_dp" -gt 0 \
    && "$accepted_dp" -eq "$offered_dp" && "$status_429" -eq 0 \
    && "$anon_peak" != null && "$limit" -gt 0 && "$anon_peak" -lt "$limit" ]]; then
    pass=true
  fi
  if [[ "$alive" == true && "$oom" == false && "$status_429" -gt 0 ]]; then
    safe=true
  fi
  record=$(jq -cn \
    --arg recorded_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --arg target "$target" \
    --arg result_path "$result" --arg stderr_path "$stderr" \
    --argjson probe "$PROBE" \
    --argjson requested_series "$candidate" --argjson offered_series "$offered_series" \
    --argjson accepted_series "$accepted_series" --argjson refused_series "$refused_series" \
    --argjson offered_datapoints "$offered_dp" --argjson accepted_datapoints "$accepted_dp" \
    --argjson refused_datapoints "$refused_dp" --argjson partial_rejected_datapoints "$partial_dp" \
    --argjson status_200 "$status_200" --argjson status_429 "$status_429" \
    --argjson anon_peak_bytes "$anon_peak" --argjson cgroup_memory_peak_bytes "$(num_or_zero "$cgroup_peak")" \
    --argjson cgroup_limit_bytes "$(num_or_zero "$limit")" --argjson alive "$alive" \
    --argjson oom_killed "$oom" --argjson harness_exit "$harness_exit" \
    --argjson elapsed_seconds "$elapsed" --argjson latency_max_ms "$latency" \
    --argjson statuses "$statuses_json" --argjson pass "$pass" --argjson safe_saturation "$safe" \
    '{recorded_at: $recorded_at, target: $target, probe: $probe, requested_series: $requested_series,
      offered_series: $offered_series, accepted_series: $accepted_series,
      refused_series: $refused_series, offered_datapoints: $offered_datapoints,
      accepted_datapoints: $accepted_datapoints, refused_datapoints: $refused_datapoints,
      partial_rejected_datapoints: $partial_rejected_datapoints, statuses: $statuses,
      status_200: $status_200, status_429: $status_429, anon_peak_bytes: $anon_peak_bytes,
      cgroup_memory_peak_bytes: $cgroup_memory_peak_bytes,
      cgroup_limit_bytes: $cgroup_limit_bytes, alive: $alive, oom_killed: $oom_killed,
      harness_exit: $harness_exit, elapsed_seconds: $elapsed_seconds,
      latency_max_ms: $latency_max_ms, pass: $pass, safe_saturation: $safe_saturation,
      result_path: $result_path, stderr_path: $stderr_path}')
  printf '%s\n' "$record" >>"$JSONL"
  {
    csv_quote "$(jq -r .recorded_at <<<"$record")"; printf ','
    csv_quote "$target"; printf ','
    csv_quote "$PROBE"; printf ','
    for value in "$candidate" "$offered_series" "$accepted_series" "$refused_series" \
      "$offered_dp" "$accepted_dp" "$refused_dp" "$partial_dp" "$status_200" "$status_429" \
      "$anon_peak" "$cgroup_peak" "$limit" "$alive" "$oom" "$harness_exit" "$elapsed" "$latency" \
      "$pass" "$safe" "$result" "$stderr"; do
      csv_quote "$value"
      [[ "$value" == "$stderr" ]] || printf ','
    done
    printf '\n'
  } >>"$CSV"
  LAST_PASS=$pass
  echo "  pass=$pass alive=$alive oom=$oom offered_dp=$offered_dp accepted_dp=$accepted_dp anon_peak=$anon_peak limit=$limit" >&2
}

write_state() {
  local target=$1 best=$2 fail=$3 phase=$4
  jq -n --arg target "$target" --arg phase "$phase" --argjson best "$best" --argjson fail "$fail" \
    '{target: $target, phase: $phase,
      best_passing_series: (if $best > 0 then $best else null end),
      failing_upper_bound: (if $fail > 0 then $fail else null end),
      updated_at: now | todateiso8601}' >"$OUT/state-$target.json"
}

echo "building load harness" >&2
cargo build --manifest-path "$ROOT/Cargo.toml" --release --bin load >/dev/null
LOAD_BIN="$ROOT/target/release/load"
if [[ "$TARGETS" == *signy* ]]; then compose build signy >/dev/null; fi
compose pull --ignore-buildable -q || true
trap 'compose down -v --remove-orphans >/dev/null 2>&1 || true' EXIT

for target in $TARGETS; do
  best=0
  fail=0
  candidate=$LOWER
  write_state "$target" "$best" "$fail" ramp
  while (( candidate <= UPPER )); do
    run_trial "$target" "$candidate"
    if [[ "$LAST_PASS" == true ]]; then
      best=$candidate
      if (( candidate == UPPER )); then
        break
      fi
      candidate=$((candidate * RAMP_FACTOR))
      # Always measure the configured upper bound. Without this step a
      # non-power-of-two upper bound (for example 150) would be reported as a
      # pass after only testing 100.
      if (( candidate > UPPER )); then candidate=$UPPER; fi
    else
      fail=$candidate
      break
    fi
    write_state "$target" "$best" "$fail" ramp
  done
  if (( fail > 0 && best > 0 )); then
    low=$best
    high=$fail
    while (( high - low > TOLERANCE )); do
      mid=$((low + (high - low + 1) / 2))
      run_trial "$target" "$mid"
      if [[ "$LAST_PASS" == true ]]; then low=$mid; else high=$mid; fi
      write_state "$target" "$low" "$high" binary
    done
    best=$low
    # high itself was tested and is the final known failing bound. Keeping it
    # (rather than high-1) makes the persisted interval truthful on resume.
    fail=$high
  elif (( fail == 0 )); then
    best=$UPPER
  fi
  write_state "$target" "$best" "$fail" complete
  echo "capacity $target: $best series (uncertainty <= $TOLERANCE)" >&2
done

echo "JSONL: $JSONL" >&2
echo "CSV: $CSV" >&2
