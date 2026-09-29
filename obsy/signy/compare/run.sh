#!/usr/bin/env bash
#
# The one-command reproduction of docs/COMPARISON.md.
#
#     compare/run.sh
#
# Everything it does is in this file, in order, and every knob is a default
# rather than an assignment so a caller can vary one without editing anything
# (the rule scripts/run_load_local.sh was fixed to follow in M8).
#
# Three targets, one script. The first three-way numbers were produced by an
# ad-hoc shell loop that bypassed this file — and with it the agreement check,
# which is how a timing table went out over answers nobody had compared. The
# fix is structural: there is no per-target code path here to fork, only the
# TARGETS list, so a run that includes a system includes its checks.
#
# The shape of the run, and why it is this shape:
#
#   1. Fresh volumes, all containers up at the same memory limit.
#   2. **Ingest**, one target at a time. Sequential rather than concurrent, so
#      no system's throughput is a function of what the others were doing on
#      the same twelve cores. All are stopped at the same *event target*, so
#      all end up holding the same number of entries — a run capped on time
#      would let a faster system hold more data and then be measured querying
#      it.
#   3. **Seed**, the fixed verification dataset, identical bytes at identical
#      timestamps on every side. This is what the query numbers and the
#      row-equality check are taken over; see src/bin/load/matrix.rs.
#   4. **Settle**, equally: Loki is told to flush (its chunks otherwise sit in
#      the ingester for up to `max_chunk_age`, two hours, and a disk number
#      taken before that is not a disk number), VictoriaLogs is told to flush
#      its in-memory parts (rows are not searchable before that flush — three
#      whole runs once returned zero VictoriaLogs rows because a loop queried
#      immediately after seeding), then everything is left alone for
#      SETTLE_SECONDS so flush, merge and compaction can run. Disk and the
#      ingest memory peak are read here.
#   5. **Restart all**, so "cold" in the query matrix means a process that has
#      just started rather than one whose caches the seeding filled.
#   6. **Matrix**, one target at a time, six shapes, cold then warm.
#   7. Read the query-phase memory peak and disk again, then generate the
#      document from the JSON.
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT/compare"

OUT="${COMPARE_OUT:-$ROOT/target/compare}"
DOC="${COMPARE_DOC:-$ROOT/docs/COMPARISON.md}"
mkdir -p "$OUT"

export COMPARE_MEMORY="${COMPARE_MEMORY:-2g}"
export SIGNY_PORT="${SIGNY_PORT:-3110}"
export LOKI_PORT="${LOKI_PORT:-3120}"
export VICTORIALOGS_PORT="${VICTORIALOGS_PORT:-3130}"
export SIGNY_BUILD_REVISION="${SIGNY_BUILD_REVISION:-$(git -C "$ROOT" rev-parse --short HEAD)}"
export SIGNY_BUILD_BRANCH="${SIGNY_BUILD_BRANCH:-$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)}"

TARGETS="signy loki victorialogs"
LOKI_IMAGE="grafana/loki:3.3.2"
VICTORIALOGS_IMAGE="victoriametrics/victoria-logs:v1.52.0"

port_of() {
  case "$1" in
    signy) echo "$SIGNY_PORT" ;;
    loki) echo "$LOKI_PORT" ;;
    victorialogs) echo "$VICTORIALOGS_PORT" ;;
  esac
}
container_of() { echo "signy-compare-$1-1"; }
# VictoriaLogs has no /ready; /health is its readiness endpoint.
ready_path_of() {
  case "$1" in
    victorialogs) echo /health ;;
    *) echo /ready ;;
  esac
}
volume_of() { echo "signy-compare_$1-data"; }

# Ingest phase. The event target is the stop condition; the duration is only a
# cap, and a run that hits the cap instead is reported as such.
LOAD_SECONDS="${COMPARE_LOAD_SECONDS:-240}"
LOAD_EVENTS="${COMPARE_LOAD_EVENTS:-1200000}"
LOAD_EPS="${COMPARE_LOAD_EPS:-20000}"
LOAD_CONNECTIONS="${COMPARE_LOAD_CONNECTIONS:-8}"
LOAD_QUERY_EPS="${COMPARE_LOAD_QUERY_EPS:-5}"
# Verification dataset.
VERIFY_ROWS="${COMPARE_VERIFY_ROWS:-150000}"
VERIFY_STREAMS="${COMPARE_VERIFY_STREAMS:-32}"
MATRIX_REPEATS="${COMPARE_MATRIX_REPEATS:-5}"
MATRIX_WINDOWS="${COMPARE_MATRIX_WINDOWS:-3}"
SETTLE_SECONDS="${COMPARE_SETTLE_SECONDS:-150}"
SEED="${COMPARE_SEED:-1592598566}"
# The query matrix runs once per limit in this list, and each one is given its
# own restart so every limit gets a cold pass.
#
# The first published run used 20000 alone, and that limit never binds: a window
# holds about 1250 matching rows, so no bound reaches the scan and a bounded
# executor cannot show in the number. That made the comparison blind to the one
# thing the read path was changed to do. 100 is Loki's own default and what
# Grafana sends unless told otherwise. Both are reported: dropping 20000 would
# break continuity with the published table, and reporting only 100 would be
# choosing the condition that flatters this engine.
MATRIX_LIMITS="${COMPARE_MATRIX_LIMITS:-20000 100}"

# The verification dataset's log timestamps. Computed once and given to every
# run, because runs deriving it from their own clocks would seed different
# datasets and the row-equality check would fail for a reason that has nothing
# to do with any engine.
VERIFY_STEP_NS=1000000
MATRIX_STEP_SECONDS="${COMPARE_MATRIX_STEP_SECONDS:-10}"
VERIFY_SPAN_S=$(( VERIFY_ROWS * VERIFY_STEP_NS / 1000000000 ))
# Rounded down to a whole query step, so that the matrix's window boundaries
# land on the grid Loki aligns metric samples to — which is also the grid
# LogsQL aligns `_time` buckets to — and the last window is not truncated by
# the alignment. See `align_to_step` in src/bin/load/matrix.rs.
ANCHOR_S=$(( ( $(date +%s) - VERIFY_SPAN_S - 300 ) / MATRIX_STEP_SECONDS * MATRIX_STEP_SECONDS ))
ANCHOR_NS=$(( ANCHOR_S * 1000000000 ))

say() { printf '\n=== %s ===\n' "$*" >&2; }

cgroup_of() {
  local pid
  pid=$(docker inspect -f '{{.State.Pid}}' "$1")
  echo "/sys/fs/cgroup$(cut -d: -f3 "/proc/$pid/cgroup")"
}

wait_ready() {
  local target=$1 tries=0
  local port path
  port=$(port_of "$target")
  path=$(ready_path_of "$target")
  until [ "$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$port$path")" = "200" ]; do
    tries=$((tries + 1))
    if [ "$tries" -gt 120 ]; then
      echo "$target never became ready" >&2
      docker compose logs --tail 40 "$target" >&2
      exit 1
    fi
    sleep 2
  done
}

declare -A CGROUP

read_cgroups() {
  local target
  for target in $TARGETS; do
    CGROUP[$target]=$(cgroup_of "$(container_of "$target")")
  done
}

bring_up() {
  say "starting the bed at ${COMPARE_MEMORY} per container"
  docker compose down -v --remove-orphans >/dev/null 2>&1 || true
  docker compose up -d
  local target
  for target in $TARGETS; do
    wait_ready "$target"
  done
  read_cgroups
}

alive() {
  [ "$(docker inspect -f '{{.State.Running}}' "$1")" = "true" ]
}

say "building the images"
docker compose build
docker compose pull --ignore-buildable -q || true
bring_up

# Each system's own report of what it is running with, captured from the
# process rather than from a file this repository wrote. Loki: `?mode=diff` is
# broken in 3.3.2 ("unsupported type <nil>"), so the full config and Loki's
# own defaults are both captured and the document diffs them. VictoriaLogs:
# /flags lists exactly the flags set to non-default values.
curl -s "http://127.0.0.1:$LOKI_PORT/config" -o "$OUT/loki_config.yaml"
curl -s "http://127.0.0.1:$LOKI_PORT/config?mode=defaults" -o "$OUT/loki_config_defaults.yaml"
diff -U0 \
  <(sed 's/[[:space:]]*$//' "$OUT/loki_config_defaults.yaml") \
  <(sed 's/[[:space:]]*$//' "$OUT/loki_config.yaml") \
  >"$OUT/loki_config.diff" || true
curl -s "http://127.0.0.1:$LOKI_PORT/loki/api/v1/status/buildinfo" -o "$OUT/loki_buildinfo.json"
# signy has no buildinfo endpoint since the first-party API replaced the
# Loki surface; the image digest recorded below is its build identity.
curl -s "http://127.0.0.1:$SIGNY_PORT/ready" -o "$OUT/signy_ready.txt"
curl -s "http://127.0.0.1:$VICTORIALOGS_PORT/flags" -o "$OUT/victorialogs_flags.txt"
# signy's side of the same disclosure: the container's environment is its
# whole configuration surface, and it logs the derived numbers at startup.
docker inspect -f '{{range .Config.Env}}{{println .}}{{end}}' \
  "$(container_of signy)" >"$OUT/signy_env.txt"
docker compose logs --no-color --no-log-prefix signy >"$OUT/signy_startup.log" 2>&1 || true

run_load() {
  local target=$1
  say "ingest: $target"
  SIGNY_LOAD_TARGET="$target" \
  SIGNY_LOAD_PHASE=load \
  SIGNY_LOAD_ADDR="127.0.0.1:$(port_of "$target")" \
  SIGNY_LOAD_CGROUP="${CGROUP[$target]}" \
  SIGNY_LOAD_TIER=compare \
  SIGNY_LOAD_SEED="$SEED" \
  SIGNY_LOAD_SECONDS="$LOAD_SECONDS" \
  SIGNY_LOAD_EVENTS="$LOAD_EVENTS" \
  SIGNY_LOAD_TARGET_EPS="$LOAD_EPS" \
  SIGNY_LOAD_CONNECTIONS="$LOAD_CONNECTIONS" \
  SIGNY_LOAD_QUERY_EPS="$LOAD_QUERY_EPS" \
  SIGNY_LOAD_OTLP_EPS=0 \
  SIGNY_LOAD_RESULT_PATH="$OUT/load_$target.json" \
  SIGNY_BUILD_REVISION="$SIGNY_BUILD_REVISION" \
    "$LOAD_BIN" >/dev/null || echo "  (load verdict was not PASS for $target; the result file says why)" >&2
}

# The verdict the surviving limit's run recorded for a target, straight from
# its result file — `run_load` deliberately keeps the bed running on a FAIL so
# every artifact still gets produced; the gate at the end reads this instead.
load_verdict_of() {
  grep -o '"verdict": *"[A-Z_]*"' "$OUT/load_$1.json" 2>/dev/null \
    | head -1 | grep -o '[A-Z_]*' | tail -1
}

run_verify() {
  local phase=$1 target=$2 limit=${3:-20000} suffix=${4:-}
  say "$phase: $target${suffix:+ (limit $limit)}"
  SIGNY_LOAD_TARGET="$target" \
  SIGNY_LOAD_PHASE="$phase" \
  SIGNY_LOAD_ADDR="127.0.0.1:$(port_of "$target")" \
  SIGNY_LOAD_CGROUP="${CGROUP[$target]}" \
  SIGNY_LOAD_SEED="$SEED" \
  SIGNY_LOAD_VERIFY_ANCHOR_NS="$ANCHOR_NS" \
  SIGNY_LOAD_VERIFY_ROWS="$VERIFY_ROWS" \
  SIGNY_LOAD_VERIFY_STREAMS="$VERIFY_STREAMS" \
  SIGNY_LOAD_VERIFY_STEP_NS="$VERIFY_STEP_NS" \
  SIGNY_LOAD_MATRIX_REPEATS="$MATRIX_REPEATS" \
  SIGNY_LOAD_MATRIX_WINDOWS="$MATRIX_WINDOWS" \
  SIGNY_LOAD_MATRIX_STEP_SECONDS="$MATRIX_STEP_SECONDS" \
  SIGNY_LOAD_MATRIX_LIMIT="$limit" \
  SIGNY_LOAD_RESULT_PATH="$OUT/${phase}_$target$suffix.json" \
  SIGNY_BUILD_REVISION="$SIGNY_BUILD_REVISION" \
    "$LOAD_BIN" >/dev/null || echo "  ($phase verdict was not PASS for $target; the result file says why)" >&2
}

say "building the harness"
cargo build --manifest-path "$ROOT/Cargo.toml" --release --bin load
LOAD_BIN="$ROOT/target/release/load"

# The ingest phase is run at each limit in COMPARE_MEMORY_LIMITS, in order, and
# the first limit at which **every** system survives carries the rest of the
# pipeline. A limit at which one of them is OOM-killed is not an error to
# retry past silently: it is an ingest result at that limit, it is recorded in
# `attempts` below, and the published document reports it. The list is a
# default like everything else here, and a single-entry list turns the sweep
# off.
declare -A ALIVE OOM_AT
ATTEMPTS=""
SURVIVED=""
for LIMIT in ${COMPARE_MEMORY_LIMITS:-2g 8g}; do
  export COMPARE_MEMORY="$LIMIT"
  bring_up
  ALL_PRIOR_ALIVE=true
  for TARGET in $TARGETS; do
    if [ "$ALL_PRIOR_ALIVE" = "true" ]; then
      run_load "$TARGET"
    fi
    if alive "$(container_of "$TARGET")"; then
      ALIVE[$TARGET]=true
    else
      ALIVE[$TARGET]=false
      ALL_PRIOR_ALIVE=false
    fi
    OOM_AT[$TARGET]=$(docker inspect -f '{{.State.OOMKilled}}' "$(container_of "$TARGET")")
  done

  ATTEMPT="{\"limit\":\"$LIMIT\""
  ALL_ALIVE=true
  for TARGET in $TARGETS; do
    ATTEMPT="$ATTEMPT,\"${TARGET}_survived\":${ALIVE[$TARGET]},\"${TARGET}_oom_killed\":${OOM_AT[$TARGET]}"
    [ "${ALIVE[$TARGET]}" = "true" ] || ALL_ALIVE=false
    cp "$OUT/load_$TARGET.json" "$OUT/load_${TARGET}_$LIMIT.json" 2>/dev/null || true
  done
  ATTEMPTS="$ATTEMPTS${ATTEMPTS:+,}$ATTEMPT}"

  if [ "$ALL_ALIVE" = "true" ]; then
    SURVIVED="$LIMIT"
    break
  fi
  say "a container did not survive ${LIMIT}; keeping the result and trying the next limit"
done

if [ -z "$SURVIVED" ]; then
  echo "no limit in '${COMPARE_MEMORY_LIMITS:-2g 8g}' let every system finish the ingest phase;" >&2
  echo "the per-limit results are in $OUT and no query comparison was run." >&2
  exit 1
fi

for TARGET in $TARGETS; do
  run_verify seed "$TARGET"
done

say "settling for ${SETTLE_SECONDS}s"
# Loki holds a chunk in the ingester until it is idle for `chunk_idle_period`
# (30m) or reaches `max_chunk_age` (2h). Neither happens inside a run that
# takes minutes, so without this its chunks would never reach the filesystem
# and its bytes-on-disk would be a number about the run length. VictoriaLogs
# makes rows searchable only after its in-memory flush, so it is flushed too —
# the settle is load-bearing for it, not a courtesy. signy flushes on its
# own every five seconds at its default, so this is what makes the three
# comparable rather than what gives anyone an advantage.
curl -s -X POST "http://127.0.0.1:$LOKI_PORT/flush" >/dev/null || true
curl -s "http://127.0.0.1:$VICTORIALOGS_PORT/internal/force_flush" >/dev/null || true
sleep "$SETTLE_SECONDS"

# Disk is measured from each service's *volume*, mounted read-only into a
# busybox helper, not with `docker exec du` inside the service container —
# VictoriaLogs' image is built from scratch and carries neither a shell nor
# `du`, and a measurement path that only works for some targets is how a
# target gets silently measured differently.
disk_of() {
  docker run --rm -v "$(volume_of "$1"):/data:ro" busybox du -sb /data | cut -f1
}
disk_breakdown() {
  docker run --rm -v "$(volume_of "$1"):/data:ro" busybox sh -c 'du -sb /data/* 2>/dev/null' \
    | awk '{printf "%s%s:%s", (NR>1?",":""), $2, $1}'
}

declare -A DISK DISK_PARTS PEAK_INGEST PEAK_QUERY DISK_END OOM
for TARGET in $TARGETS; do
  DISK[$TARGET]=$(disk_of "$TARGET")
  DISK_PARTS[$TARGET]=$(disk_breakdown "$TARGET")
  PEAK_INGEST[$TARGET]=$(cat "${CGROUP[$TARGET]}/memory.peak")
  PEAK_QUERY[$TARGET]=0
done

PRIMARY_LIMIT=""
MATRIX_RUNS=""
for LIMIT in $MATRIX_LIMITS; do
  say "restarting everything so the cold query pass at limit $LIMIT is cold"
  docker compose restart
  for TARGET in $TARGETS; do
    wait_ready "$TARGET"
  done
  read_cgroups

  for TARGET in $TARGETS; do
    run_verify matrix "$TARGET" "$LIMIT" "_l$LIMIT"
    # The query-phase memory peak is the worst any limit reached, not the last
    # one measured: a terminal sample meaning nothing is a lesson this
    # repository already paid for three times over.
    THIS=$(cat "${CGROUP[$TARGET]}/memory.peak")
    [ "$THIS" -gt "${PEAK_QUERY[$TARGET]}" ] && PEAK_QUERY[$TARGET]=$THIS
  done

  MATRIX_RUNS="$MATRIX_RUNS${MATRIX_RUNS:+,}{\"limit\":$LIMIT,\"suffix\":\"_l$LIMIT\"}"
  # The first limit in the list carries the report's headline tables, so the
  # document stays comparable with the published one.
  if [ -z "$PRIMARY_LIMIT" ]; then
    PRIMARY_LIMIT=$LIMIT
    for TARGET in $TARGETS; do
      cp "$OUT/matrix_${TARGET}_l$LIMIT.json" "$OUT/matrix_$TARGET.json"
    done
  fi
done

# Whether a container was killed for exceeding the limit. An OOM kill is a
# result, not an error, and a comparison that quietly restarted through one
# would be reporting the wrong thing.
for TARGET in $TARGETS; do
  DISK_END[$TARGET]=$(disk_of "$TARGET")
  OOM[$TARGET]=$(docker inspect -f '{{.State.OOMKilled}}' "$(container_of "$TARGET")")
done

PEAK_JSON=""
DISK_JSON=""
for TARGET in $TARGETS; do
  PEAK_JSON="$PEAK_JSON${PEAK_JSON:+,}
    \"$TARGET\": { \"ingest\": ${PEAK_INGEST[$TARGET]}, \"query\": ${PEAK_QUERY[$TARGET]}, \"oom_killed\": ${OOM[$TARGET]} }"
  DISK_JSON="$DISK_JSON${DISK_JSON:+,}
    \"$TARGET\": { \"settled\": ${DISK[$TARGET]}, \"after_queries\": ${DISK_END[$TARGET]}, \"breakdown\": \"${DISK_PARTS[$TARGET]}\" }"
done

cat >"$OUT/bed.json" <<JSON
{
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "revision": "$SIGNY_BUILD_REVISION",
  "branch": "$SIGNY_BUILD_BRANCH",
  "machine": "$(uname -sr); $(nproc) logical CPUs; $(awk '/MemTotal/ {printf "%.1f GiB RAM", $2/1048576}' /proc/meminfo)",
  "docker": "$(docker version --format '{{.Server.Version}}')",
  "compose": "$(docker compose version --short)",
  "loki_image": "$LOKI_IMAGE",
  "victorialogs_image": "$VICTORIALOGS_IMAGE",
  "memory_limit": "$COMPARE_MEMORY",
  "memory_limit_attempts": [$ATTEMPTS],
  "memory_limit_bytes": $(docker exec "$(container_of signy)" cat /sys/fs/cgroup/memory.max),
  "settle_seconds": $SETTLE_SECONDS,
  "verify_anchor_ns": $ANCHOR_NS,
  "matrix_primary_limit": $PRIMARY_LIMIT,
  "matrix_runs": [$MATRIX_RUNS],
  "object_store": "none: every system is on its local filesystem",
  "peak_bytes": {$PEAK_JSON
  },
  "disk_bytes": {$DISK_JSON
  }
}
JSON

# The artifacts are checked in beside the document, and copied by the same run
# that wrote the document, so the two cannot drift. The retired numbers this
# repository is recovering from had one cited artifact that did not exist and
# another that disagreed with the document citing it — and on 2026-08-12 it
# happened again from the other direction: a second corpus wrote its JSON into
# the directory the first corpus's document names. So the document is told
# which directory it is citing rather than assuming, and a run with its own
# COMPARE_ARTIFACTS says so on its own first page.
ARTIFACTS="${COMPARE_ARTIFACTS:-$ROOT/docs/artifacts/m9}"
mkdir -p "$ARTIFACTS"
COMPARE_ARTIFACTS_REL="$(realpath --relative-to="$(dirname "$DOC")" "$ARTIFACTS")"
export COMPARE_ARTIFACTS_REL

say "generating $DOC"
cargo build --manifest-path "$ROOT/Cargo.toml" --release --bin compare_report
"$ROOT/target/release/compare_report" "$OUT" "$DOC"

# Everything the run produced, not a list of it. The list this replaces named
# 18 of the 29 files a run writes, and the eleven it missed were not spares:
# the per-limit matrix JSON the document's own limit table is computed from,
# both buildinfo files behind its build table, and the startup log it prints
# verbatim. So `compare_report` aborted on the first missing file when pointed
# at the directory the document tells a reader to point it at, and the
# sentence "every number below comes from the JSON in artifacts/" was false
# for three of its sections. A hand-maintained copy list drifts from what the
# report reads every time the report reads something new; copying the whole
# directory cannot.
cp "$OUT"/* "$ARTIFACTS/" 2>/dev/null || true
echo "results: $OUT" >&2
echo "document: $DOC" >&2

# The load verdict is a gate, not a footnote. Every revision failed it until
# the flush was chunked and merge left the operation lock, so a regression
# here is exactly the kind that used to ship unnoticed for months. The gate
# runs last, after the document and artifacts exist, so a FAIL still leaves
# everything needed to diagnose it. Loki and VictoriaLogs are judged on their
# own behavioral gates and are not required here by default.
for TARGET in ${COMPARE_REQUIRE_PASS:-signy}; do
  [ "$TARGET" = "off" ] && break
  VERDICT="$(load_verdict_of "$TARGET")"
  if [ "$VERDICT" != "PASS" ]; then
    echo "load verdict for $TARGET at ${SURVIVED} is '${VERDICT:-missing}', not PASS; failing the bed" >&2
    echo "(set COMPARE_REQUIRE_PASS=off to record a known-failing run without gating)" >&2
    exit 1
  fi
  say "load verdict gate: $TARGET PASS at $SURVIVED"
done
