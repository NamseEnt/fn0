#!/usr/bin/env bash
#
# The soak leg: hours-to-a-day of ingest + query + merge + retention on the
# 2 GiB rig (todo.md, "Next: production polish", item 1).
#
#   scripts/run_soak_local.sh <run-name>
#
# run_memprof_local.sh's rig — a native cgroup v2 scope, the bed's corpus and
# offered rate — pointed at what only hours can show: part-count growth against
# the unevictable sidecars, WAL size under the local-compaction policy, merge
# pacing, row-group-cache gauge drift over thousands of evictions, and
# allocator retention (the 1.34-1.69 anon/live ratio was measured at 60 s).
#
# What is deliberately different from the memprof script:
#
#   * The data directory lives on disk, not tmpfs. tmpfs pages are charged to
#     the writing cgroup as shmem and never reclaimed with swap off, so hours
#     of parts would eat the 2 GiB limit and manufacture an OOM the disk
#     engine does not have. The 240 s memprof runs fit under that; a soak
#     cannot.
#   * Retention is on (30 m period, 60 s interval, 5 m grace by default) so
#     parts are deleted, sidecars retract, and the disk reaches a steady state
#     the run can be judged on.
#   * All three signals, not one. The log leg is the rate; the metric leg
#     scrapes a live population onto the *same tenant* and reads it back
#     through the first-party metric routes. Metrics are the signal no long run
#     had ever touched, and the one whose storage changed most recently.
#   * An object store is configured by default (`file://` under the run's own
#     output directory). Without one the engine never offloads, never retires a
#     part to the store, and never compacts the WAL -- three production paths a
#     local-only soak silently skips. What `file://` cannot exercise is the
#     compare-and-swap: the code declares it a single-process development
#     backend and takes the overwrite path, so CAS stays a question only a real
#     provider's startup preflight answers.
#   * A disk guard fails the run before the root filesystem does.
#   * The verdict is a set of trends — quarter-by-quarter means — rather than
#     one peak: a soak passes when the residents stop growing, not when a
#     threshold survives a minute.
#
# Every knob is a default so a caller can vary exactly one:
#
#   SOAK_SECONDS=900                 ./scripts/run_soak_local.sh smoke
#   SOAK_LIMIT=8G                    ./scripts/run_soak_local.sh headroom
#   SOAK_RETENTION=off               ./scripts/run_soak_local.sh no-retention
#   SOAK_SERVER_ENV="MALLOC_ARENA_MAX=1" ./scripts/run_soak_local.sh trimmed
#   SOAK_MEMORY_HIGH=1800M           ./scripts/run_soak_local.sh throttled
#   SOAK_OBJECT_STORE=off            ./scripts/run_soak_local.sh local-only
#   SOAK_METRIC_SCRAPE_SECONDS=0     ./scripts/run_soak_local.sh logs-only
#   SOAK_FEATURES=                   ./scripts/run_soak_local.sh production-allocator
set -uo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
NAME="${1:?usage: run_soak_local.sh <run-name>}"
OUT="${SOAK_OUT:-$ROOT/target/soak}/$NAME"
rm -rf "$OUT"; mkdir -p "$OUT"

LIMIT="${SOAK_LIMIT:-2G}"
# Unset by default, so the rig keeps one wall and hits it. Set it and reclaim
# starts throttled and gradual below the hard limit instead of as a cliff at
# it — the question of whether the freeze is the cliff or the reclaim itself.
MEMORY_HIGH="${SOAK_MEMORY_HIGH:-}"
SECONDS_CAP="${SOAK_SECONDS:-86400}"
EPS="${SOAK_EPS:-20000}"
CONNS="${SOAK_CONNECTIONS:-8}"
QUERY_EPS="${SOAK_QUERY_EPS:-5}"
RETENTION="${SOAK_RETENTION:-30m}"
# Retention is a tenant policy the harness pushes, not a server-wide setting:
# SIGNY_RETENTION_PERIOD stopped being read when it moved there, so the leg
# this knob names had been running against nothing. `off` is this script's
# documented spelling; the policy's is `infinite`.
case "$RETENTION" in
  off) TENANT_RETENTION="infinite" ;;
  *)   TENANT_RETENTION="$RETENTION" ;;
esac
RETENTION_INTERVAL="${SOAK_RETENTION_INTERVAL:-60s}"
RETENTION_GRACE="${SOAK_RETENTION_GRACE:-5m}"
# The metric leg. The scrape interval and the query rate are what turn it on;
# `0` on either turns that half off. The population is 8 services x this many
# instances x (4 gauges + 4 counters + 1 histogram), so the default offers
# about 1,150 series -- a real instrument set beside the log rate rather than a
# cardinality test, which the bed measures separately.
METRIC_SCRAPE="${SOAK_METRIC_SCRAPE_SECONDS:-10}"
METRIC_QUERY_EPS="${SOAK_METRIC_QUERY_EPS:-1}"
METRIC_INSTANCES="${SOAK_METRIC_INSTANCES:-16}"
METRIC_CHURN="${SOAK_METRIC_CHURN_PER_SCRAPE:-0}"
# Shorter than the retention period by construction: a window past what the
# tenant keeps reads empty, and the leg's empty-answer gate would then be
# firing on this script's configuration rather than on the engine.
METRIC_WINDOW="${SOAK_METRIC_QUERY_WINDOW_SECONDS:-900}"
# The object store. `off` reverts to the local-only engine the published soaks
# ran on, which is the comparison, not the production shape.
STORE="${SOAK_OBJECT_STORE:-file://$OUT/store}"
# The comparison bed's seed, so this run's corpus is the bed's corpus.
SEED="${SOAK_SEED:-1592598566}"
BIN="${SOAK_BIN:-$ROOT/target/release/signy}"
PORT="${SOAK_PORT:-3153}"
# memprof is on by default: allocator retention over hours is half of what
# this run exists to observe, and anon/live needs the arena live bytes.
#
# `SOAK_FEATURES=` (explicitly empty) builds the *production* binary instead,
# which is a different allocator and not a detail: the memprof build's
# instrumented wrapper allocates through glibc, and the shipped binary uses
# mimalloc (`src/main.rs`). An absolute footprint and an arena split cannot be
# read off the same run, so a question about whether production survives has to
# be asked of this build. The `-` rather than `:-` is what makes the empty
# value mean "no features" instead of falling back to the default.
FEATURES="${SOAK_FEATURES-memprof}"
# Fail the run while the machine still has room, well before the filesystem
# is full: retention churn needs headroom to delete into.
DISK_MIN_AVAIL_KB="${SOAK_DISK_MIN_AVAIL_KB:-$((4 * 1024 * 1024))}"
# Seconds to keep sampling after the load stops, with the server still up.
#
# This answers the one question every run so far has left open: when nothing is
# being asked of it any more, does the process give its memory back? If
# resident falls toward live, what it was holding was freed memory the decay
# policy had not returned, which is a setting. If it does not fall, the memory
# is either still live or pinned by fragmentation, which is a defect in what
# the engine allocates rather than in what the allocator does with it. The runs
# before this killed the server the moment the harness stopped, so the question
# had never been put.
SETTLE="${SOAK_SETTLE_SECONDS:-120}"
UNIT="soak-$NAME"

# The collector in front. `off` reverts to the shape every published soak ran
# in — the harness framing its own collecty batch and posting it to the collect
# route — which measures the engine and skips the only ingest path production
# has. On, the harness stands where an application's exporter stands and
# collecty's intake, its disk queue, its retries and its resume are under load
# for the first time.
COLLECTY="${SOAK_COLLECTY:-on}"
COLLECTY_PORT="${SOAK_COLLECTY_PORT:-4397}"
COLLECTY_BIN="${SOAK_COLLECTY_BIN:-$ROOT/../collecty/target/release/collecty}"
# collecty's own cage. Its resident is supposed to be a function of its
# configured ceilings and nothing else — not of how far behind signy is, which
# lives on disk — and that claim has never been measured over hours. Small on
# purpose: a cage at the claim is what turns the claim into a test.
COLLECTY_LIMIT="${SOAK_COLLECTY_LIMIT:-256M}"
# And the threshold below it, which is the recommended deployment pair --
# collecty/docs/CONFIGURATION.md. `memory.high` is where the kernel starts
# reclaiming this cgroup rather than killing anything, and collecty's cage is
# mostly its own queue's page cache: clean, on the inactive list, and cheap to
# take back. Measured there at 2 GiB of backlog against a 96 MiB threshold,
# resident page cache held at 82 MiB for 0.48 s of stall in 23 minutes. Without
# it `memory.current` simply follows the backlog, and a soak that never asks
# the kernel to reclaim never finds out whether it can.
COLLECTY_HIGH="${SOAK_COLLECTY_HIGH:-96M}"
# The three ceilings that decide collecty's footprint, left at their defaults
# so the run measures the shipped configuration rather than a tuned one.
COLLECTY_INFLIGHT="${SOAK_COLLECTY_INFLIGHT:-64MiB}"
COLLECTY_QUEUE_MAX="${SOAK_COLLECTY_QUEUE_MAX:-1GiB}"
COLLECTY_SEGMENT="${SOAK_COLLECTY_SEGMENT:-8MiB}"
COLLECTY_UNIT="soak-collecty-$NAME"
# The tenant collecty-generated telemetry is filed under. The load harness's
# first corpus tenant, so the collector's counters land where this run's queries can
# read them — an export naming a tenant the instance does not serve is dropped
# on arrival and said nothing about, so an unserved tenant here would silently
# delete the collector's own evidence.
COLLECTY_GENERATED_TELEMETRY_TENANT_ID="${SOAK_COLLECTY_GENERATED_TELEMETRY_TENANT_ID:-load-tenant-000}"
# The trace leg. Off in every soak so far, and its read routes have never been
# crossed by a long run at all — see the harness's trace read-back, which this
# turns on with it.
TRACE_EPS="${SOAK_TRACE_EPS:-5}"
# The feature probe: every read route, every this many seconds, for the whole
# run. `0` turns it off.
PROBE_INTERVAL="${SOAK_PROBE_INTERVAL:-300}"
# The fault schedule: restarts and an outage at fixed fractions of the run.
#
# A day of steady state answers half the question. The other half is what
# happens when a process comes back — whether the WAL replays what it owed,
# whether the collector's queue drains into the engine that returns, whether a
# segment already stored is skipped rather than stored twice, and whether the
# read surface is whole afterwards. None of that is reachable from a run
# nothing interrupts.
FAULTS="${SOAK_FAULTS:-on}"
# How long the engine is left down at the outage step. Long enough for the
# collector's queue to grow visibly and its sender to back off, short enough
# not to fill it.
DOWNTIME="${SOAK_DOWNTIME_SECONDS:-180}"
# How many of those outages there are. One says a drain works; several say
# whether the cycle -- outage, backlog, recovery, drain -- leaves anything
# behind, and that is the question a day-long run exists to answer. The extras
# are spread through the last third, after the named faults have had their
# turn, and stop short of the end so the last one has room to drain.
OUTAGES="${SOAK_OUTAGES:-3}"
# Queries fail while the engine is deliberately down, and the harness's error
# gate is zero by default -- so a scheduled outage would fail the run on the
# errors the schedule caused. This is the budget those windows are allowed,
# applied only when faults are on. The failures stay in the report and the
# fault log says exactly when each window was.
FAULT_ERROR_BUDGET="${SOAK_FAULT_ERROR_BUDGET:-0.02}"

if [ "${SOAK_SKIP_BUILD:-0}" != "1" ]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --release --bin load
  [ -n "${SOAK_BIN:-}" ] || cargo build --manifest-path "$ROOT/Cargo.toml" --release \
    --bin signy ${FEATURES:+--features "$FEATURES"}
  if [ "$COLLECTY" != "off" ] && [ -z "${SOAK_COLLECTY_BIN:-}" ]; then
    # Its own crate, its own lockfile and its own toolchain: there is no
    # workspace at the root, so this is a second build and not another target.
    cargo build --manifest-path "$ROOT/../collecty/Cargo.toml" --release --bin collecty
  fi
fi

DATA="${SOAK_DATA:-$OUT/data}"
mkdir -p "$DATA"
SERVER_LOG="$OUT/server.log"

# The store's own directory, so its size can be sampled beside the data dir:
# with an object store configured the data dir is a cache and a WAL, and the
# bytes that outlive a part are over here.
STORE_DIR=""
STORE_ENV=""
if [ "$STORE" != "off" ]; then
  case "$STORE" in
    file://*) STORE_DIR="${STORE#file://}"; mkdir -p "$STORE_DIR" ;;
  esac
  STORE_ENV="SIGNY_OBJECT_STORE_URL=$STORE"
fi

# collecty's queue. Its own directory, sampled beside signy's: what is here is
# what signy has not taken yet, and it is the only copy of an acknowledged
# export until signy does.
QUEUE_DIR="$OUT/collecty"
COLLECTY_LOG="$OUT/collecty.log"
COLLECTY_ADDR="127.0.0.1:$COLLECTY_PORT"
# What sends the harness's writes through the collector. Unset, the harness
# frames its own batch and posts it to the collect route, which is what every
# published soak measured. Exported rather than put in the invocation's
# assignment prefix: an expansion there is a command name, not an assignment.
if [ "$COLLECTY" != "off" ]; then
  export SIGNY_LOAD_PUSH_ADDR="$COLLECTY_ADDR"
fi

systemctl --user reset-failed "$UNIT.scope" 2>/dev/null
systemctl --user reset-failed "$COLLECTY_UNIT.scope" 2>/dev/null

cleanup() {
  rm -f "$OUT/RUNNING"
  for p in ${PROBE_PID:-} ${FAULT_PID:-} ${SAMPLER_PID:-} ${QUEUE_SAMPLER_PID:-} \
           ${PROF_PID:-} ${WATCH_PID:-} $(collecty_pid) $(signy_pid); do
    kill "$p" 2>/dev/null
  done
  sleep 1
  # The pid files, not the variables: a restart happened in a background shell
  # and this one's copies are a generation behind.
  kill -KILL "$(collecty_pid)" 2>/dev/null
  kill -KILL "$(signy_pid)" 2>/dev/null
  [ "${SOAK_KEEP_DATA:-0}" = "1" ] || rm -rf "$DATA" "$QUEUE_DIR" ${STORE_DIR:+"$STORE_DIR"}
  systemctl --user reset-failed "$UNIT.scope" 2>/dev/null
  systemctl --user reset-failed "$COLLECTY_UNIT.scope" 2>/dev/null
}
trap cleanup EXIT

signy_pid() { cat "$OUT/signy.pid" 2>/dev/null; }
collecty_pid() { cat "$OUT/collecty.pid" 2>/dev/null; }
fault_log() { echo "$(date +%s) +$(( $(date +%s) - ${RUN_STARTED:-$(date +%s)} ))s $*" >>"$OUT/faults.log"; }

# Signal a process and wait for it to actually be gone. A restart that reuses
# the unit name cannot start until the old scope is empty, so returning before
# the process has exited would race the next start against its own predecessor.
stop_process() {
  local pid="$1" signal="$2"
  [ -n "$pid" ] || return 0
  kill "-$signal" "$pid" 2>/dev/null
  for _ in $(seq 1 1200); do
    kill -0 "$pid" 2>/dev/null || return 0
    sleep 0.5
  done
  kill -KILL "$pid" 2>/dev/null
  sleep 1
}

# Both processes start from a function rather than inline, because a soak that
# never restarts either of them has not been told anything about what happens
# when one of them comes back — which is most of what a day is for.
start_signy() {
  systemctl --user reset-failed "$UNIT.scope" 2>/dev/null
  # shellcheck disable=SC2086
  env SIGNY_LISTEN_ADDR="127.0.0.1:$PORT" \
      SIGNY_DATA_DIR="$DATA" \
      SIGNY_RETENTION_INTERVAL="$RETENTION_INTERVAL" \
      SIGNY_RETENTION_GRACE_PERIOD="$RETENTION_GRACE" \
      ${STORE_ENV:-} \
      ${SOAK_SERVER_ENV:-} \
      systemd-run --user --scope --quiet --unit="$UNIT" \
        -p MemoryMax="$LIMIT" -p MemorySwapMax=0 \
        ${MEMORY_HIGH:+-p MemoryHigh="$MEMORY_HIGH"} \
        -- "$BIN" >>"$SERVER_LOG" 2>&1 &
  SERVER_PID=$!
  # Written down as well as held, because the restarts happen in a background
  # shell whose variables this one never sees.
  echo "$SERVER_PID" >"$OUT/signy.pid"
  for _ in $(seq 1 600); do
    curl -fsS "http://127.0.0.1:$PORT/ready" >/dev/null 2>&1 && return 0
    kill -0 "$SERVER_PID" 2>/dev/null || { echo "server died before ready"; tail -40 "$SERVER_LOG"; return 1; }
    sleep 0.5
  done
  echo "server did not become ready"
  return 1
}

# collecty answers no readiness route and publishes no metrics port — its own
# counters travel through its queue as ordinary OTLP — so "up" is a refusal
# from the process. `GET /v1/logs` is 405, and a 405 is proof.
start_collecty() {
  [ "$COLLECTY" = "off" ] && return 0
  systemctl --user reset-failed "$COLLECTY_UNIT.scope" 2>/dev/null
  mkdir -p "$QUEUE_DIR"
  env COLLECTY_LISTEN_ADDR="$COLLECTY_ADDR" \
      COLLECTY_DATA_DIR="$QUEUE_DIR" \
      COLLECTY_SIGNY_URL="http://127.0.0.1:$PORT" \
      COLLECTY_MAX_INFLIGHT_BYTES="$COLLECTY_INFLIGHT" \
      COLLECTY_QUEUE_MAX_BYTES="$COLLECTY_QUEUE_MAX" \
      COLLECTY_QUEUE_SEGMENT_BYTES="$COLLECTY_SEGMENT" \
      COLLECTY_GENERATED_TELEMETRY_TENANT_ID="$COLLECTY_GENERATED_TELEMETRY_TENANT_ID" \
      COLLECTY_REPORT_INTERVAL="30s" \
      systemd-run --user --scope --quiet --unit="$COLLECTY_UNIT" \
        -p MemoryMax="$COLLECTY_LIMIT" -p MemorySwapMax=0 \
        ${COLLECTY_HIGH:+-p MemoryHigh="$COLLECTY_HIGH"} \
        -- "$COLLECTY_BIN" >>"$COLLECTY_LOG" 2>&1 &
  COLLECTY_PID=$!
  echo "$COLLECTY_PID" >"$OUT/collecty.pid"
  for _ in $(seq 1 120); do
    curl -s -o /dev/null "http://$COLLECTY_ADDR/v1/logs" && return 0
    kill -0 "$COLLECTY_PID" 2>/dev/null || { echo "collecty died before it listened"; tail -40 "$COLLECTY_LOG"; return 1; }
    sleep 0.5
  done
  echo "collecty never answered"
  return 1
}

touch "$OUT/RUNNING"
start_signy || exit 1
# The generated-telemetry tenant, onboarded before the collector exists.
#
# collecty exports its generated telemetry the moment it is up, and the harness that
# pushes tenant policies does not start until after it. That left a window of
# about a second and a half in which signy was serving no tenant by that name
# and refused the export, exactly as it is specified to -- one dropped resource
# per run, at +1.6 s, in every soak ever taken. The drop was the running order,
# not the engine, and the fix is to put the policy in first. The harness pushes
# it again for every tenant when it starts; the endpoint is idempotent.
if [ "$COLLECTY" != "off" ]; then
  curl -fsS --max-time 10 -X PUT \
    -H 'Content-Type: application/json' \
    -d "{\"retention\": \"$TENANT_RETENTION\"}" \
    "http://127.0.0.1:$PORT/signy/api/v1/admin/tenants/$COLLECTY_GENERATED_TELEMETRY_TENANT_ID/retention" \
    >/dev/null || echo "warning: could not onboard $COLLECTY_GENERATED_TELEMETRY_TENANT_ID before collecty"
fi
start_collecty || exit 1

# Read from the server's own process rather than derived from this shell's:
# where systemd places a user scope depends on the manager, and a wrong guess
# here would silently sample an empty cgroup.
CG="/sys/fs/cgroup$(cut -d: -f3 "/proc/$SERVER_PID/cgroup")"
[ -d "$CG" ] || { echo "no cgroup at $CG"; exit 1; }
COLLECTY_CG=""
if [ "$COLLECTY" != "off" ]; then
  COLLECTY_CG="/sys/fs/cgroup$(cut -d: -f3 "/proc/$COLLECTY_PID/cgroup")"
fi
echo "cgroup=$CG memory.max=$(cat "$CG/memory.max") memory.high=$(cat "$CG/memory.high") swap.max=$(cat "$CG/memory.swap.max")"
if [ -n "$COLLECTY_CG" ]; then
  echo "collecty cgroup=$COLLECTY_CG memory.max=$(cat "$COLLECTY_CG/memory.max") memory.high=$(cat "$COLLECTY_CG/memory.high")"
  WANT_HIGH=$(numfmt --from=iec "${COLLECTY_HIGH%B}" 2>/dev/null || echo 0)
  GOT_HIGH=$(cat "$COLLECTY_CG/memory.high" 2>/dev/null)
  [ -z "$COLLECTY_HIGH" ] || [ "$GOT_HIGH" = "$WANT_HIGH" ] || {
    echo "collecty memory.high is $GOT_HIGH, asked for $WANT_HIGH"; exit 1; }
fi
echo "out=$OUT data=$DATA seconds=$SECONDS_CAP eps=$EPS retention=$RETENTION"
echo "store=$STORE metric_scrape=${METRIC_SCRAPE}s metric_query_eps=$METRIC_QUERY_EPS"
echo "collecty=$COLLECTY addr=$COLLECTY_ADDR limit=$COLLECTY_LIMIT trace_eps=$TRACE_EPS probe=${PROBE_INTERVAL}s"

# The memprof sampler's columns plus the disk: data_dir and journal.wal sizes
# (du is paid once a minute, the value carried between), and the filesystem's
# available space, which trips the guard. `anon` is what an OOM kill is
# decided on; `file` is the page cache the cgroup peak also includes.
#
# The last eight columns are the stall evidence, all cumulative. Memory: PSI's
# `some` and `full` stall microseconds (`full` = every thread in the cgroup
# stalled at once, which is what a whole-server freeze looks like from the
# kernel's side), direct reclaim's scanned and stolen pages (as against
# kswapd's, which costs the workload nothing), and file refaults, which say a
# reclaimed page was wanted again. Then the same `some`/`full` for I/O and
# `some` for CPU, because the first instrumented hour ruled memory out — a
# freeze with zero direct reclaim in it is not a reclaim stall, and the next
# question is which resource the threads were actually waiting on.
(
  echo "t,current,peak,anon,file,slab,sock,kstack,pgtables,memtable,memtable_buffered,pending_flush,wal_backlog,parts,sidecar,part_meta,rg_cache,query_success,query_errors,threads,mp_other,mp_ingest,mp_wal,mp_flush,mp_merge,mp_query,mp_sidecar,mp_part_meta,mp_rg_cache,mp_header,mi_arena,mi_mmap,mi_inuse,mi_free,data_dir,wal_file,disk_avail_kb,psi_some_us,psi_full_us,pgscan_direct,pgsteal_direct,refault_file,io_some_us,io_full_us,cpu_some_us,active_series,series_memtable,metric_parts,metric_dp_rejected,metric_mem_rejected,metrics_dir,store_dir,proc_rss,alloc_committed,alloc_live,alloc_retained,alloc_active,alloc_metadata,account_in_use,account_budget,account_exhausted,account_deferred,cpu_usec,sidecar_hits,sidecar_misses,sidecar_read_bytes,sc_installs,sc_inst_bytes,sc_evictions,sc_resident_ns,sc_redecodes,sc_gap_ns"
  T0=$(date +%s.%N)
  echo "$T0" >"$OUT/sampler_t0"
  i=0; du_bytes=0; wal_bytes=0; metrics_bytes=0; store_bytes=0
  # The run's own flag rather than the cgroup's existence: a restart takes the
  # scope down and brings it back at the same path, and a sampler that exited
  # on the gap would end the series at the first fault instead of measuring
  # what came after it.
  while [ -f "$OUT/RUNNING" ]; do
    now=$(date +%s.%N)
    cur=$(cat "$CG/memory.current" 2>/dev/null) || { sleep 1; continue; }
    peak=$(cat "$CG/memory.peak" 2>/dev/null)
    st=$(cat "$CG/memory.stat" 2>/dev/null)
    psi=$(cat "$CG/memory.pressure" 2>/dev/null)
    psi_io=$(cat "$CG/io.pressure" 2>/dev/null)
    psi_cpu=$(cat "$CG/cpu.pressure" 2>/dev/null)
    m=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/metrics" 2>/dev/null)
    if [ $((i % 60)) -eq 0 ]; then
      du_bytes=$(du -sb "$DATA" 2>/dev/null | cut -f1)
      metrics_bytes=$(du -sb "$DATA/metrics" 2>/dev/null | cut -f1)
      store_bytes=0
      [ -n "$STORE_DIR" ] && store_bytes=$(du -sb "$STORE_DIR" 2>/dev/null | cut -f1)
      wal_bytes=$(find "$DATA" -name 'journal.wal' -printf '%s\n' 2>/dev/null | awk '{s+=$1} END{print s+0}')
      avail_kb=$(df -k --output=avail "$DATA" 2>/dev/null | tail -1 | tr -d ' ')
      if [ "${avail_kb:-0}" -lt "$DISK_MIN_AVAIL_KB" ]; then
        echo "disk guard: ${avail_kb} KB available < ${DISK_MIN_AVAIL_KB} KB" >"$OUT/DISK_GUARD"
        pkill -f 'target/release/load'
        break
      fi
    fi
    cg=$(echo "$st" | awk '
      $1=="anon"{a=$2} $1=="file"{f=$2} $1=="slab"{s=$2} $1=="sock"{k=$2}
      $1=="kernel_stack"{ks=$2} $1=="pgtables"{p=$2}
      END{printf "%d,%d,%d,%d,%d,%d", a, f, s, k, ks, p}')
    rcl=$(echo "$psi" | awk -F'total=' '
      /^some /{s=$2+0} /^full /{f=$2+0} END{printf "%d,%d", s, f}')
    rcl="$rcl,$(echo "$st" | awk '
      $1=="pgscan_direct"{sc=$2} $1=="pgsteal_direct"{stl=$2}
      $1=="workingset_refault_file"{rf=$2}
      END{printf "%d,%d,%d", sc, stl, rf}')"
    rcl="$rcl,$(echo "$psi_io" | awk -F'total=' '
      /^some /{s=$2+0} /^full /{f=$2+0} END{printf "%d,%d", s, f}')"
    rcl="$rcl,$(echo "$psi_cpu" | awk -F'total=' '/^some /{printf "%d", $2+0}')"
    mx=$(echo "$m" | awk '
      $1=="signy_active_series"{as=$2}
      $1=="signy_series_memtable_bytes"{sm=$2}
      $1=="signy_metric_part_count"{mp=$2}
      $1=="signy_metric_datapoints_rejected_total"{dr=$2}
      $1=="signy_metric_memory_rejected_total"{mr=$2}
      END{printf "%d,%d,%d,%d,%d", as, sm, mp, dr, mr}')
    # Resident against committed, which is the production build's only answer
    # to "is this process big because it is using memory or because the
    # allocator kept it". The memprof build publishes the resident half only
    # and leaves the other at zero.
    # The shared memory account: what admitted work has declared it holds,
    # against the ceiling it is admitted on. Distinct from every column beside
    # it -- those measure the heap, this one measures what the engine *decided*
    # -- and the pair is the only way to read a 429 back to its cause.
    # `exhausted` is the client-facing refusal, `deferred` is the same event
    # for flush and compaction, which have nobody to refuse and postpone
    # instead.
    # What the sidecar cache costs, not just what it holds. A miss re-reads the
    # whole of index.bin into an owned buffer and drops it after decoding, so
    # the read-bytes rate is a rate of large allocate-and-free.
    sc=$(echo "$m" | awk '
      $1=="signy_part_sidecar_hits_total"{h=$2}
      $1=="signy_part_sidecar_misses_total"{ms=$2}
      $1=="signy_part_sidecar_read_bytes_total"{rb=$2}
      $1=="signy_part_sidecar_installs_total"{si=$2}
      $1=="signy_part_sidecar_installed_bytes_total"{sib=$2}
      $1=="signy_part_sidecar_evictions_total"{se=$2}
      $1=="signy_part_sidecar_resident_nanos_total"{srn=$2}
      $1=="signy_part_sidecar_redecodes_total"{sr=$2}
      $1=="signy_part_sidecar_redecode_gap_nanos_total"{sgn=$2}
      END{printf "%d,%d,%d,%d,%d,%d,%d,%d,%d", h, ms, rb, si, sib, se, srn, sr, sgn}')
    ac=$(echo "$m" | awk '
      $1=="signy_memory_account_in_use_bytes"{u=$2}
      $1=="signy_memory_account_budget_bytes"{b=$2}
      $1=="signy_query_memory_exhausted_total"{e=$2}
      $1=="signy_memory_account_deferred_total"{d=$2}
      END{printf "%d,%d,%d,%d", u, b, e, d}')
    al=$(echo "$m" | awk '
      $1=="signy_process_rss_bytes"{rs=$2}
      $1=="signy_allocator_committed_bytes"{cm=$2}
      $1=="signy_allocator_live_bytes"{lv=$2}
      $1=="signy_allocator_retained_bytes"{rt=$2}
      $1=="signy_allocator_active_bytes"{ac=$2}
      $1=="signy_allocator_metadata_bytes"{md=$2}
      END{printf "%d,%d,%d,%d,%d,%d", rs, cm, lv, rt, ac, md}')
    gg=$(echo "$m" | awk '
      $1=="signy_memtable_bytes"{mt=$2}
      $1=="signy_memtable_buffered_bytes"{mb=$2}
      $1=="signy_pending_flush_bytes"{pf=$2}
      $1=="signy_wal_backlog_bytes"{wb=$2}
      $1=="signy_part_count"{pc=$2}
      $1=="signy_part_sidecar_resident_bytes"{sc=$2}
      $1=="signy_part_meta_bytes"{pm=$2}
      $1=="signy_row_group_cache_bytes"{rc=$2}
      $1=="signy_query_success_total"{qs=$2}
      $1=="signy_query_errors_total"{qe=$2}
      END{printf "%d,%d,%d,%d,%d,%d,%d,%d,%d,%d", mt, mb, pf, wb, pc, sc, pm, rc, qs, qe}')
    threads=$(awk '/^Threads:/{print $2}' "/proc/$(head -1 "$CG/cgroup.procs" 2>/dev/null)/status" 2>/dev/null)
    mp=$(echo "$m" | awk -F'[{}" ]+' '
      /^signy_memprof_live_bytes\{/{v[$3]=$NF}
      /^signy_memprof_malloc_bytes\{/{i[$3]=$NF}
      /^signy_memprof_header_bytes /{h=$2}
      END{printf "%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d",
          v["other"], v["ingest"], v["wal"], v["flush"], v["merge"],
          v["query"], v["sidecar"], v["part_meta"], v["row_group_cache"], h,
          i["arena"], i["mmapped"], i["in_use"], i["free"]}')
    # CPU the cgroup has burned, cumulative. Without it a run that lowers
    # resident cannot be told from a run that simply did less work -- an
    # allocator knob that trades throughput for memory reads as a win on
    # every other column here.
    cpu_usec=$(awk '/^usage_usec /{print $2}' "$CG/cpu.stat" 2>/dev/null)
    printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
      "$(awk -v a="$now" -v b="$T0" 'BEGIN{printf "%.2f", a-b}')" \
      "$cur" "$peak" "$cg" "$gg" "${threads:-0}" "$mp" \
      "$du_bytes" "$wal_bytes" "${avail_kb:-0}" "$rcl" \
      "$mx" "${metrics_bytes:-0}" "${store_bytes:-0}" "$al" "$ac" "${cpu_usec:-0}" "$sc"
    i=$((i + 1))
    sleep 1
  done
) >"$OUT/mem.csv" 2>/dev/null &
SAMPLER_PID=$!

# The cumulative memprof counters, once a minute: mem.csv carries live bytes,
# and the allocation traffic that explains them is only in the totals.
(
  while true; do
    body=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/metrics" 2>/dev/null | grep memprof)
    [ -n "$body" ] && { echo "=== $(date +%s.%N)"; echo "$body"; }
    sleep 60
  done
) >"$OUT/memprof.txt" 2>/dev/null &
PROF_PID=$!

# The collector's own series, kept apart from mem.csv rather than added to its
# seventy columns: its verdict is a different question with a different shape.
#
# Three numbers answer it. `anon` against its cage is whether the documented
# footprint — the in-flight ceiling plus a batch buffer plus the runtime — is
# what it actually costs, which nothing has ever measured over hours. `queue`
# is how far behind signy is, and it is also the only copy of an acknowledged
# export until signy takes it, so a queue that grows and does not come back
# down is data at risk rather than a slow sender. `segments` never falls below
# three: each signal holds one open.
if [ "$COLLECTY" != "off" ]; then
  (
    echo "t,anon,current,peak,queue_bytes,segments,rss,file,file_dirty,file_writeback,threads,fds,ev_high,ev_max,ev_oom,oom_kills,psi_some_us,psi_full_us"
    T0=$(cat "$OUT/sampler_t0" 2>/dev/null || date +%s.%N)
    while [ -f "$OUT/RUNNING" ]; do
      now=$(date +%s.%N)
      mem=$(awk '$1=="anon"{a=$2} $1=="file"{f=$2} $1=="file_dirty"{d=$2}
                 $1=="file_writeback"{w=$2}
                 END{printf "%d,%d,%d,%d", a, f, d, w}' \
                "$COLLECTY_CG/memory.stat" 2>/dev/null)
      cur=$(cat "$COLLECTY_CG/memory.current" 2>/dev/null)
      peak=$(cat "$COLLECTY_CG/memory.peak" 2>/dev/null)
      queue=$(du -sb "$QUEUE_DIR" 2>/dev/null | cut -f1)
      segments=$(find "$QUEUE_DIR" -type f -name '*.seg' 2>/dev/null | wc -l)
      # Threads and open descriptors are on the pass list beside memory: a
      # collector that leaks either ratchets just as surely and a memory
      # column will not show it.
      pid=$(head -1 "$COLLECTY_CG/cgroup.procs" 2>/dev/null)
      rss=$(awk '/^VmRSS:/{print $2 * 1024}' "/proc/$pid/status" 2>/dev/null)
      threads=$(awk '/^Threads:/{print $2}' "/proc/$pid/status" 2>/dev/null)
      fds=$(ls "/proc/$pid/fd" 2>/dev/null | wc -l)
      # Whether the kernel had to push back, and what it cost. `high` counting
      # up with `max` and `oom` at zero is the throttle doing its job; PSI is
      # how long the collector was stopped while it did.
      events=$(awk '$1=="high"{h=$2} $1=="max"{m=$2} $1=="oom"{o=$2}
                    $1=="oom_kill"{k=$2} END{printf "%d,%d,%d,%d", h, m, o, k}' \
                   "$COLLECTY_CG/memory.events.local" 2>/dev/null)
      psi=$(awk '{for (n = 1; n <= NF; n++) if ($n ~ /^total=/) { sub("total=", "", $n)
                    if ($1 == "some") s = $n; else if ($1 == "full") u = $n }}
                 END{printf "%d,%d", s, u}' "$COLLECTY_CG/memory.pressure" 2>/dev/null)
      printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
        "$(awk -v a="$now" -v b="$T0" 'BEGIN{printf "%.2f", a-b}')" \
        "${mem%%,*}" "${cur:-0}" "${peak:-0}" "${queue:-0}" "${segments:-0}" "${rss:-0}" \
        "${mem#*,}" "${threads:-0},${fds:-0}" "${events:-0,0,0,0},${psi:-0,0}"
      sleep 5
    done
  ) >"$OUT/collecty.csv" 2>/dev/null &
  QUEUE_SAMPLER_PID=$!
fi

# The feature probe. Every read route in docs/QUERY_API.md, on its own clock,
# for the whole run — see scripts/soak_probe.sh for why the memory trends below
# are not an answer to "does it still work".
if [ "$PROBE_INTERVAL" -gt 0 ] 2>/dev/null; then
  PROBE_ADDR="127.0.0.1:$PORT" \
  PROBE_INTERVAL="$PROBE_INTERVAL" \
  PROBE_TENANT="$COLLECTY_GENERATED_TELEMETRY_TENANT_ID" \
    "$ROOT/scripts/soak_probe.sh" "$OUT" >"$OUT/probe.log" 2>&1 &
  PROBE_PID=$!
fi

# The harness has a 60 s request timeout, so without this a run whose server
# died mid-soak would sit for minutes producing nothing.
#
# It has to tell a death from a scheduled restart, or the first fault would end
# the run it exists to survive: a stop the schedule is in the middle of raises
# FAULT_ACTIVE, and a server that is gone for half a minute with no fault to
# explain it is the case this was written for.
(
  gone=0
  while [ -f "$OUT/RUNNING" ]; do
    if kill -0 "$(signy_pid)" 2>/dev/null || [ -f "$OUT/FAULT_ACTIVE" ]; then
      gone=0
    else
      gone=$((gone + 1))
    fi
    if [ "$gone" -ge 30 ]; then
      echo "server gone for 30 s with no fault in progress" >"$OUT/SERVER_DIED"
      pkill -f 'target/release/load'
      break
    fi
    sleep 1
  done
) &
WATCH_PID=$!

RUN_STARTED=$(date +%s)

# The schedule. Fractions of the run rather than wall-clock offsets, so a
# thirty-minute smoke exercises the same sequence a day does.
#
#   10%  the collector restarts. Its queue is the only copy of an acked export
#        until signy takes it, so this asks whether that survives a stop.
#   25%  the engine restarts gracefully. The WAL replays, the collector's
#        backlog drains into it, and a segment it already stored is skipped
#        rather than stored twice.
#   40%  the engine is down for DOWNTIME. The collector queues and backs off;
#        nothing acked may be lost.
#   55%  the engine is killed outright. This is the crash path -- the WAL
#        replay that a clean shutdown never exercises.
#   70%  the engine restarts again, now with hours of parts on disk. Startup is
#        linear in part count, and this is the end of the line that the earlier
#        restart is the other end of.
#
# and then SOAK_OUTAGES-1 more outages spread through what is left, because a
# single drain cannot say whether repeating the cycle leaves a floor behind.
#
# The collector is never killed outright. An open segment has not been fsynced,
# and losing it would be collecty behaving as documented -- a loss window this
# run cannot then tell apart from a defect.
if [ "$FAULTS" != "off" ]; then
  (
    pct() { echo $(( SECONDS_CAP * $1 / 100 )); }
    wait_until() {
      while [ -f "$OUT/RUNNING" ]; do
        [ $(( $(date +%s) - RUN_STARTED )) -ge "$1" ] && return 0
        sleep 5
      done
      return 1
    }
    fault() { touch "$OUT/FAULT_ACTIVE"; fault_log "$@"; }
    done_fault() { rm -f "$OUT/FAULT_ACTIVE"; fault_log "$@"; }

    if [ "$COLLECTY" != "off" ]; then
      wait_until "$(pct 10)" || exit 0
      fault "collecty: SIGTERM"
      stop_process "$(collecty_pid)" TERM
      start_collecty && done_fault "collecty: back up" || done_fault "collecty: FAILED TO RESTART"
    fi

    wait_until "$(pct 25)" || exit 0
    fault "signy: SIGTERM"
    stop_process "$(signy_pid)" TERM
    start_signy && done_fault "signy: back up" || done_fault "signy: FAILED TO RESTART"

    wait_until "$(pct 40)" || exit 0
    fault "signy: down for ${DOWNTIME}s"
    stop_process "$(signy_pid)" TERM
    sleep "$DOWNTIME"
    start_signy && done_fault "signy: back up after the outage" || done_fault "signy: FAILED TO RESTART"

    wait_until "$(pct 55)" || exit 0
    fault "signy: SIGKILL"
    stop_process "$(signy_pid)" KILL
    start_signy && done_fault "signy: back up after the kill" || done_fault "signy: FAILED TO RESTART"

    wait_until "$(pct 70)" || exit 0
    fault "signy: SIGTERM with the run's parts on disk"
    stop_process "$(signy_pid)" TERM
    start_signy && done_fault "signy: back up" || done_fault "signy: FAILED TO RESTART"

    # The outage again, and again. Evenly through 70%-98%, so the last one
    # still has a fifth of the tail to drain into.
    extra=$(( OUTAGES > 1 ? OUTAGES - 1 : 0 ))
    round=1
    while [ "$round" -le "$extra" ]; do
      wait_until "$(pct $(( 70 + round * 28 / (extra + 1) )))" || exit 0
      fault "signy: down for ${DOWNTIME}s (outage $(( round + 1 )) of $OUTAGES)"
      stop_process "$(signy_pid)" TERM
      sleep "$DOWNTIME"
      start_signy \
        && done_fault "signy: back up after outage $(( round + 1 ))" \
        || done_fault "signy: FAILED TO RESTART"
      round=$(( round + 1 ))
    done
  ) &
  FAULT_PID=$!
fi

[ "$FAULTS" != "off" ] && export SIGNY_TARGET_MAX_ERROR_RATE="$FAULT_ERROR_BUDGET"

SIGNY_LOAD_TARGET=signy \
SIGNY_LOAD_PHASE=load \
SIGNY_LOAD_ADDR="127.0.0.1:$PORT" \
SIGNY_LOAD_CGROUP="$CG" \
SIGNY_LOAD_TIER=soak \
SIGNY_LOAD_SEED="$SEED" \
SIGNY_LOAD_SECONDS="$SECONDS_CAP" \
SIGNY_LOAD_EVENTS=0 \
SIGNY_LOAD_TARGET_EPS="$EPS" \
SIGNY_LOAD_CONNECTIONS="$CONNS" \
SIGNY_LOAD_QUERY_EPS="$QUERY_EPS" \
SIGNY_LOAD_OTLP_EPS="$TRACE_EPS" \
SIGNY_LOAD_METRIC_LEG_SCRAPE_SECONDS="$METRIC_SCRAPE" \
SIGNY_LOAD_METRIC_LEG_CHURN_PER_SCRAPE="$METRIC_CHURN" \
SIGNY_LOAD_METRIC_QUERY_EPS="$METRIC_QUERY_EPS" \
SIGNY_LOAD_METRIC_QUERY_WINDOW_SECONDS="$METRIC_WINDOW" \
SIGNY_LOAD_METRIC_INSTANCES="$METRIC_INSTANCES" \
SIGNY_LOAD_TENANT_RETENTION="$TENANT_RETENTION" \
SIGNY_LOAD_RESULT_PATH="$OUT/load.json" \
  "$ROOT/target/release/load" >"$OUT/harness.log" 2>&1
HARNESS_STATUS=$?
LOAD_STOPPED_AT=$(date +%s.%N)

# The allocator's own report at the two moments worth comparing: with the load
# on it, and after it has had the settle window to give memory back. Both are
# pulled only while the server is up, which is the only time they exist.
capture_allocator() {
  kill -0 "$(signy_pid)" 2>/dev/null || return 0
  curl -fsS --max-time 15 "http://127.0.0.1:$PORT/debug/allocator" \
    >"$OUT/allocator-$1.txt" 2>/dev/null || true
  curl -fsS --max-time 60 "http://127.0.0.1:$PORT/debug/allocator/profile" \
    >>"$OUT/allocator-$1.txt" 2>/dev/null || true
}
capture_allocator loaded

# The settle window: the load is gone, the server is not. Sampling continues,
# so the series carries a stretch with no work in it to compare against.
if [ "$SETTLE" -gt 0 ] 2>/dev/null && kill -0 "$(signy_pid)" 2>/dev/null; then
  echo "settling for ${SETTLE}s with the load stopped"
  sleep "$SETTLE"
  capture_allocator settled
fi
# Heap profiles live in the data directory, which cleanup removes.
cp -r "$DATA/heap-profiles" "$OUT/" 2>/dev/null || true

# Scraped while the server is still up, which is the only time it can be. The
# journal writer's phase histograms are cumulative over the run, so one scrape
# at the end is the whole distribution — and it is the only thing that says
# which phase a slow push was in. mem.csv cannot carry them: they are buckets,
# not a gauge.
curl -fsS --max-time 5 "http://127.0.0.1:$PORT/metrics" >"$OUT/metrics-final.txt" 2>/dev/null || true

kill "$WATCH_PID" 2>/dev/null; WATCH_PID=
kill "$PROF_PID" 2>/dev/null; PROF_PID=
kill "${PROBE_PID:-}" 2>/dev/null; PROBE_PID=
sleep 2
# The flag the two samplers loop on. Dropping it is how they stop, and it has
# to come down before they are signalled or a restarted scope would keep one
# of them alive past the run.
rm -f "$OUT/RUNNING"
sleep 2
kill "$SAMPLER_PID" 2>/dev/null; SAMPLER_PID=
kill "${QUEUE_SAMPLER_PID:-}" 2>/dev/null; QUEUE_SAMPLER_PID=

ALIVE=false
kill -0 "$(signy_pid)" 2>/dev/null && ALIVE=true
GUARD=ok
[ -f "$OUT/DISK_GUARD" ] && GUARD="tripped: $(cat "$OUT/DISK_GUARD")"

# The verdict is trends. Samples are split into quarters by time (restricted
# to rows whose metrics scrape succeeded), and each resident's quarter means
# are printed side by side: a healthy soak's Q2..Q4 are flat. GROWING flags
# Q4 > 1.10 * Q2 — past the warmup, still climbing at the end.
{
  echo "run=$NAME limit=$LIMIT seconds=$SECONDS_CAP eps=$EPS retention=$RETENTION"
  # Which allocator the run measured. An absolute footprint from the memprof
  # build and one from the shipped build are different numbers about different
  # heaps, and a verdict that does not say which is a trap for whoever reads it
  # next.
  case "$FEATURES" in
    "")         SOAK_ALLOCATOR=mimalloc ;;
    *memprof*)  SOAK_ALLOCATOR=glibc+memprof ;;
    *jemalloc*) SOAK_ALLOCATOR=jemalloc ;;
    *)          SOAK_ALLOCATOR="$FEATURES" ;;
  esac
  echo "build=${FEATURES:-production} allocator=$SOAK_ALLOCATOR store=$STORE"
  echo "alive=$ALIVE harness_status=$HARNESS_STATUS disk_guard=$GUARD"
  # Exit 3 is the harness reporting that nothing it pushed reached storage. The
  # verdict below is trends, and a server that was never given anything to hold
  # is flat in every quarter, which is what a healthy soak looks like.
  if [ "$HARNESS_STATUS" = 3 ]; then
    echo "NO LOAD DELIVERED: harness exit 3, so the trends below are of an idle server"
  fi
  grep -o '"verdict":"[A-Z_]*"' "$OUT/load.json" 2>/dev/null | head -1
  awk -F, '
    NR>1 && ($10+0>0 || $14+0>0) { n++; t[n]=$1+0
      for (c=2; c<=NF; c++) v[c,n]=$c+0
      if ($4+0 > anon_peak) anon_peak = $4+0
    }
    END {
      if (n < 8) { print "too few samples for trends: " n; exit }
      m = 1048576.0
      # column, name, unit. `count` prints as it is; everything else is MiB.
      nr = split("4 anon mib;13 wal_backlog mib;14 parts count;15 sidecar mib;\
16 part_meta mib;17 rg_cache mib;35 data_dir mib;36 wal_file mib;\
51 metrics_dir mib;52 store_dir mib;46 active_series count;\
47 series_memtable mib;48 metric_parts count;53 proc_rss mib;\
54 alloc_committed mib;55 alloc_live mib;56 alloc_retained mib;\
57 alloc_active mib;58 alloc_metadata mib;59 account_in_use mib;\
61 account_exhausted count;62 account_deferred count", rows, ";")
      printf "%-14s %12s %12s %12s %12s  %s\n", "mib/count", "Q1", "Q2", "Q3", "Q4", "trend"
      for (r = 1; r <= nr; r++) {
        split(rows[r], f, " "); c = f[1] + 0
        for (q = 0; q < 4; q++) {
          lo = int(n * q / 4) + 1; hi = int(n * (q + 1) / 4); s = 0
          for (k = lo; k <= hi; k++) s += v[c,k]
          mean[q] = s / (hi - lo + 1)
        }
        trend = (mean[3] > mean[1] * 1.10) ? "GROWING" : "flat"
        if (f[3] == "count")
          printf "%-14s %12.0f %12.0f %12.0f %12.0f  %s\n", f[2], mean[0], mean[1], mean[2], mean[3], trend
        else
          printf "%-14s %12.1f %12.1f %12.1f %12.1f  %s\n", f[2], mean[0]/m, mean[1]/m, mean[2]/m, mean[3]/m, trend
      }
      printf "anon_peak_mib=%.1f cgroup_peak_mib=%.1f duration_s=%s samples=%d\n", \
             anon_peak/m, v[3,n]/m, t[n], n
      # anon/live at the last scraped sample, when the build carries memprof.
      live = 0; for (c = 21; c <= 29; c++) live += v[c,n]
      if (live > 0)
        printf "final anon/live=%.2f (anon=%.1f live=%.1f free=%.1f)\n", \
               v[4,n]/live, v[4,n]/m, live/m, v[34,n]/m
      printf "final queries: success=%d errors=%d threads=%d\n", v[18,n], v[19,n], v[20,n]
    }' "$OUT/mem.csv"
  # The stall table: the run's freezes as the server itself saw them — a query
  # success counter that does not advance — each one carried alongside the
  # kernel's reclaim evidence for the same window. A freeze whose psi_full
  # covers most of its length is the cgroup at its limit, not a slow query.
  awk -F, '
    NR>1 && ($10+0>0 || $14+0>0) { n++
      t[n]=$1+0; qs[n]=$18+0; cur[n]=$2+0; file[n]=$5+0
      psf[n]=$39+0; stl[n]=$41+0; rf[n]=$42+0
      iof[n]=$44+0; cps[n]=$45+0
    }
    END {
      if (n < 8) { print "too few samples for stalls: " n; exit }
      m = 1048576.0; MIN = 3.0
      start = 0; ne = 0
      for (k = 2; k <= n; k++) {
        if (qs[k] == qs[k-1]) { if (!start) start = k - 1 }
        else if (start) { d = t[k] - t[start]
          if (d >= MIN) { ne++; es[ne] = start; ee[ne] = k; ed[ne] = d; tot += d }
          start = 0 }
      }
      if (start && t[n] - t[start] >= MIN) { ne++; es[ne]=start; ee[ne]=n; ed[ne]=t[n]-t[start]; tot += t[n]-t[start] }
      printf "stalls (query counter flat >= %.0f s): n=%d total=%.1fs", MIN, ne, tot
      if (ne) { mx = 0; for (e = 1; e <= ne; e++) if (ed[e] > mx) mx = ed[e]; printf " max=%.1fs", mx }
      printf "\n"
      # Longest first, at most five: a table is evidence, a list of forty is noise.
      for (e = 1; e <= ne; e++) ord[e] = e
      for (a = 1; a <= ne; a++) for (b = a+1; b <= ne; b++)
        if (ed[ord[b]] > ed[ord[a]]) { tmp = ord[a]; ord[a] = ord[b]; ord[b] = tmp }
      for (r = 1; r <= ne && r <= 5; r++) { e = ord[r]; s = es[e]; f = ee[e]
        printf "  t=%-7.1f %6.1fs  mem_full=+%.1fs io_full=+%.1fs cpu_some=+%.1fs pgsteal_direct=+%d refault=+%d cur=%.0f file=%.0f\n", \
               t[s], ed[e], (psf[f]-psf[s])/1e6, (iof[f]-iof[s])/1e6, (cps[f]-cps[s])/1e6, \
               stl[f]-stl[s], rf[f]-rf[s], cur[f]/m, file[f]/m
      }
      d = t[n] - t[1]
      printf "run totals: mem_full=%.1fs (%.2f%%) io_full=%.1fs (%.2f%%) cpu_some=%.1fs (%.2f%%) pgsteal_direct=%d refault=%d\n", \
             (psf[n]-psf[1])/1e6, 100.0*(psf[n]-psf[1])/1e6/d, \
             (iof[n]-iof[1])/1e6, 100.0*(iof[n]-iof[1])/1e6/d, \
             (cps[n]-cps[1])/1e6, 100.0*(cps[n]-cps[1])/1e6/d, stl[n]-stl[1], rf[n]-rf[1]
    }' "$OUT/mem.csv"
  # Where an accepted push's server-side time went. Every push in the process
  # is written by one task, so queue/write/fsync/insert are the whole of it.
  # The percentiles are bucket bounds, not interpolations: the histogram is
  # cumulative in `le`, so what this can honestly say is "p95 is at or below
  # this bound", and saying more would be inventing resolution the buckets do
  # not have.
  awk '
    /^signy_(journal|flush)_.*_bucket\{le="/ {
      split($1, a, "_bucket"); name = a[1]
      split($1, b, "\""); bound = b[2]
      if (bound == "+Inf") next
      if (!(name in seen)) { seen[name] = ++order; oname[order] = name }
      k = name SUBSEP (bound + 0); cum[k] = $2 + 0
      bounds[name SUBSEP (++nb[name])] = bound + 0
    }
    /^signy_(journal|flush)_.*_count / { split($1, a, "_count"); total[a[1]] = $2 + 0 }
    /^signy_(journal|flush)_.*_sum / { split($1, a, "_sum"); sum[a[1]] = $2 + 0 }
    /^signy_journal_batches_total /{ batches = $2 + 0 }
    /^signy_journal_batched_records_total /{ records = $2 + 0 }
    /^signy_flush_rows_total /{ frows = $2 + 0 }
    /^signy_flush_parts_total /{ fparts = $2 + 0 }
    END {
      if (!order) exit
      printf "push and flush phases (one writer task, one flush loop; ms, bucketed upper bounds):\n"
      printf "  %-26s %10s %8s %8s %8s\n", "phase", "n", "mean", "p50<=", "p95<="
      for (o = 1; o <= order; o++) {
        name = oname[o]; n = total[name]; if (!n) continue
        p50 = ""; p95 = ""
        for (i = 1; i <= nb[name]; i++) {
          bd = bounds[name SUBSEP i]; c = cum[name SUBSEP bd]
          if (p50 == "" && c >= 0.50 * n) p50 = bd
          if (p95 == "" && c >= 0.95 * n) p95 = bd
        }
        short = name; sub(/^signy_/, "", short); sub(/_ms$/, "", short)
        printf "  %-26s %10d %8.2f %8s %8s\n", short, n, sum[name] / n, \
               (p50 == "" ? ">30000" : p50), (p95 == "" ? ">30000" : p95)
      }
      if (batches) printf "  batches=%d records=%d records/batch=%.2f\n", \
                          batches, records, records / batches
      if (fparts) printf "  flush rows=%d parts=%d rows/part=%.0f\n", \
                          frows, fparts, frows / fparts
    }' "$OUT/metrics-final.txt" 2>/dev/null
  # Did stopping the work give the memory back? The window before the load
  # stopped against the tail of the settle window, on the numbers that decide
  # it. A resident that falls toward live was holding freed memory the decay
  # policy had not returned; one that does not fall is holding memory that is
  # either still live or pinned, and those are the engine's problem rather than
  # the allocator's.
  if [ -f "$OUT/sampler_t0" ] && [ "$SETTLE" -gt 0 ] 2>/dev/null; then
    awk -F, -v t0="$(cat "$OUT/sampler_t0")" -v stopped="$LOAD_STOPPED_AT" -v settle="$SETTLE" '
      BEGIN { at = stopped - t0; window = 30 }
      NR>1 {
        t = $1 + 0
        if (t > at - window && t <= at) { bn++; ba+=$4; br+=$53; bc+=$54; bl+=$55; bv+=$57 }
        if (t >= at + settle - window) { an++; aa+=$4; ar+=$53; ac+=$54; al+=$55; av+=$57 }
      }
      END {
        if (!bn || !an) { printf "settle: not enough samples (before=%d after=%d)\n", bn, an; exit }
        m = 1048576.0
        printf "settle (%.0fs with the load stopped), mean of 30 s before and 30 s after:\n", settle
        printf "  %-16s %10s %10s %10s\n", "mib", "loaded", "settled", "returned"
        printf "  %-16s %10.1f %10.1f %10.1f\n", "anon",      ba/bn/m, aa/an/m, (ba/bn-aa/an)/m
        printf "  %-16s %10.1f %10.1f %10.1f\n", "proc_rss",  br/bn/m, ar/an/m, (br/bn-ar/an)/m
        if (bc > 0) printf "  %-16s %10.1f %10.1f %10.1f\n", "alloc_committed", bc/bn/m, ac/an/m, (bc/bn-ac/an)/m
        if (bl > 0) printf "  %-16s %10.1f %10.1f %10.1f\n", "alloc_live", bl/bn/m, al/an/m, (bl/bn-al/an)/m
        if (bv > 0) {
          printf "  %-16s %10.1f %10.1f %10.1f\n", "alloc_active", bv/bn/m, av/an/m, (bv/bn-av/an)/m
          printf "  fragmentation (active-live) %.1f -> %.1f MiB; awaiting decay (committed-active) %.1f -> %.1f MiB\n", \
                 (bv/bn-bl/bn)/m, (av/an-al/an)/m, (bc/bn-bv/bn)/m, (ac/an-av/an)/m
        }
      }' "$OUT/mem.csv"
  fi
  # The metric leg, from the harness's own result rather than from the gauges:
  # what was offered, what was kept, and whether every read shape that must
  # return rows kept returning them. `jq` is already a dependency of
  # compare/run_metric_capacity.sh; without it the block is skipped rather than
  # the verdict failing.
  if command -v jq >/dev/null 2>&1 && [ -f "$OUT/load.json" ]; then
    jq -r '
      .behavioral.metric_leg as $leg |
      if ($leg == null) or ($leg.enabled != true) then "metric leg: off"
      else
        "metric leg: pass=\($leg.pass) scrapes=\($leg.ingest.scrapes) late=\($leg.ingest.scrapes_late) " +
        "datapoints offered=\($leg.ingest.datapoints_offered) accepted=\($leg.ingest.datapoints_accepted) " +
        "acceptance=\($leg.ingest.acceptance) refused_requests=\($leg.ingest.requests_refused) errors=\($leg.ingest.errors)",
        "  server: active_series=\($leg.server.active_series_end) metric_parts=\($leg.server.metric_part_count_end) " +
        "created=\($leg.server.series_created) retired_flushed=\($leg.server.series_retired_flushed) " +
        "evicted_idle=\($leg.server.series_evicted_idle) cardinality_rejected=\($leg.server.cardinality_rejected) " +
        "memory_rejected=\($leg.server.memory_rejected)",
        "  reads: answered=\($leg.queries.answered) errors=\($leg.queries.errors) throttled=\($leg.queries.throttled)",
        ($leg.queries.per_shape | to_entries[] |
          "    \(.key | .[0:22]): issued=\(.value.issued) judged=\(.value.judged) series=\(.value.series_returned) " +
          "empty=\(.value.empty_answers) p95=\(.value.latency_ms.response.p95_ms // "-")"),
        (if ($leg.shapes_that_answered_nothing | length) > 0
         then "  SHAPES THAT ANSWERED NOTHING: \($leg.shapes_that_answered_nothing | join(", "))"
         else empty end),
        (if $leg.ingest.first_error then "  first ingest error: \($leg.ingest.first_error)" else empty end),
        (if $leg.queries.first_error then "  first read error: \($leg.queries.first_error)" else empty end)
      end' "$OUT/load.json" 2>/dev/null

    # What a retained datapoint costs on disk. An estimate, and labelled one:
    # the metrics directory holds about `retention` seconds of accepted
    # datapoints, so this divides the steady directory size by that many. The
    # mix is printed with it because a gauge point and a histogram point are
    # not the same point, and this run's ratio is neither one alone.
    RETENTION_SECONDS=$(awk -v r="$RETENTION" 'BEGIN{
      if (r ~ /^[0-9]+s$/) { printf "%d", r + 0 }
      else if (r ~ /^[0-9]+m$/) { printf "%d", (r + 0) * 60 }
      else if (r ~ /^[0-9]+h$/) { printf "%d", (r + 0) * 3600 }
      else { print 0 } }')
    ACCEPTED=$(jq -r '.behavioral.metric_leg.ingest.datapoints_accepted // 0' "$OUT/load.json")
    ELAPSED=$(jq -r '.run.elapsed_seconds // 0' "$OUT/load.json")
    awk -F, -v accepted="$ACCEPTED" -v elapsed="$ELAPSED" -v retention="$RETENTION_SECONDS" '
      NR>1 && ($10+0>0 || $14+0>0) { n++; last = $51 + 0 }
      END {
        if (!n || retention <= 0 || elapsed <= 0 || accepted <= 0) exit
        held = accepted / elapsed * retention
        if (held <= 0) exit
        printf "metrics on disk: %.1f MiB holding ~%.0f datapoints, %.0f B/datapoint (mixed instruments)\n", \
               last / 1048576.0, held, last / held
      }' "$OUT/mem.csv"
  fi
  if [ -f "$OUT/faults.log" ]; then
    echo "faults (error budget for their windows: $FAULT_ERROR_BUDGET):"
    sed 's/^/  /' "$OUT/faults.log"
    grep -q "FAILED TO RESTART" "$OUT/faults.log" && echo "  A PROCESS DID NOT COME BACK"
  fi
  [ -f "$OUT/SERVER_DIED" ] && echo "watchdog: $(cat "$OUT/SERVER_DIED")"

  # The trace leg's read-back, and the wait for the collector's queue. Both are
  # booleans about whether this run can be believed: a trace that did not come
  # back is a wrong answer, and accounting taken before the backlog arrived is
  # not accounting at all.
  jq -r '
    if .traces then
      "traces: sent=\(.traces.sent) spans=\(.traces.spans_sent) errors=\(.traces.errors) " +
      "readback pass=\(.traces.readback.pass) verified=\(.traces.readback.verified) " +
      "missing=\(.traces.readback.missing) short=\(.traces.readback.short) " +
      "unreachable=\(.traces.readback.unreachable) retried=\(.traces.readback.retried) " +
      "search_empty=\(.traces.readback.search_empty)/\(.traces.readback.search_probes)",
      (if .traces.readback.first_error then "  first trace error: \(.traces.readback.first_error)" else empty end)
    else empty end,
    if .behavioral.collector_drain then
      "collector drain: path=\(.behavioral.ingest_path) waited=\(.behavioral.collector_drain.waited) " +
      "settled=\(.behavioral.collector_drain.settled) seconds=\(.behavioral.collector_drain.seconds | floor) " +
      "collect_dropped=\(.behavioral.collect_dropped_records) skipped=\(.behavioral.collect_skipped_records)"
    else empty end' "$OUT/load.json" 2>/dev/null

  # The feature probe's whole point: a route that answered for twenty-three
  # hours and stopped is a failure, and a mean would hide it. So this is per
  # route, with the round a failure first appeared in.
  if [ -f "$OUT/probe.csv" ]; then
    awk -F, '
      NR>1 {
        name = $3; ok = $6
        seen[name] = 1
        if (!(name in order)) { order[name] = ++n; byindex[n] = name }
        if (ok == 1) pass[name]++
        else if (ok == "skip") skip[name]++
        else {
          fail[name]++
          if (!(name in firstfail)) {
            firstfail[name] = $1
            detail = $7; for (f = 8; f <= NF; f++) detail = detail "," $f
            why[name] = $4 " " substr(detail, 1, 60)
          }
        }
        if ($1 + 0 > rounds) rounds = $1 + 0
      }
      END {
        if (!n) exit
        printf "feature probe: %d rounds over the run\n", rounds
        printf "  %-26s %6s %6s %6s  %s\n", "route", "ok", "fail", "skip", "first failure"
        for (i = 1; i <= n; i++) {
          name = byindex[i]
          printf "  %-26s %6d %6d %6d  %s\n", name, pass[name], fail[name], skip[name], \
                 (name in firstfail) ? "round " firstfail[name] ": " why[name] : "-"
          if (fail[name]) failed++
        }
        printf "probe verdict: %s (%d routes with a failure)\n", \
               failed ? "ROUTES FAILED" : "every route answered every round", failed + 0
      }' "$OUT/probe.csv"
    # Every failing row verbatim, because the fault log above is on the same
    # clock and most of what a soak's probe failures turn out to be is a
    # scheduled restart landing inside a round.
    if awk -F, '$6 == 0' "$OUT/probe.csv" | head -1 | grep -q .; then
      echo "  failing probes (t is seconds from the probe's start):"
      awk -F, '$6 == 0' "$OUT/probe.csv" | head -20 | sed 's/^/    /'
    fi
  fi

  # The collector's own accounting, read back out of signy — which is where it
  # lives: collecty has no metrics port, its counters travel through its own
  # queue as ordinary OTLP. Reading them here therefore also proves that path.
  #
  # `refused` and `dropped` are the two that must be zero. Any movement in
  # either is data thrown away: refused is signy declining a segment, dropped
  # is the queue full and unlinking the oldest.
  if [ "$COLLECTY" != "off" ] && [ "$ALIVE" = true ]; then
    collecty_metric() {
      curl -fsS --max-time 10 -H "X-Tenant-Id: $COLLECTY_GENERATED_TELEMETRY_TENANT_ID" \
        "http://127.0.0.1:$PORT/signy/api/v1/metrics/instant?metric=$1" 2>/dev/null \
        | head -1 | jq -r '.value // "-"' 2>/dev/null
      }
    echo "collecty (counters read back through signy, tenant $COLLECTY_GENERATED_TELEMETRY_TENANT_ID):"
    printf "  %-28s %s\n" "records_appended" "$(collecty_metric collecty_records_appended_total)"
    printf "  %-28s %s\n" "segments_sent" "$(collecty_metric collecty_segments_sent_total)"
    printf "  %-28s %s\n" "bytes_appended" "$(collecty_metric collecty_bytes_appended_total)"
    printf "  %-28s %s\n" "bytes_sent" "$(collecty_metric collecty_bytes_sent_total)"
    printf "  %-28s %s\n" "send_retries" "$(collecty_metric collecty_send_retries_total)"
    printf "  %-28s %s\n" "segments_refused (must be 0)" "$(collecty_metric collecty_segments_refused_total)"
    printf "  %-28s %s\n" "queue_dropped_segments (0)" "$(collecty_metric collecty_queue_dropped_segments_total)"
    printf "  %-28s %s\n" "queue_bytes" "$(collecty_metric collecty_queue_bytes)"
    # What the harness got a 2xx for, against what the collector recorded.
    #
    # These are not expected to be equal and a difference is not a loss. The
    # counters above are read out of the last self-report collecty managed to
    # ship, so they trail the run by up to one report interval; and its own
    # metric exports are records it appended too, so they push the other way.
    # What must be exactly zero is `refused` and `dropped` -- those are the
    # counters that mean bytes were thrown away.
    EXPORTS=$(jq -r '.exports_accepted // 0' "$OUT/load.json" 2>/dev/null)
    printf "  %-28s %s\n" "harness exports (2xx)" "${EXPORTS:-0}"
    echo "  (the counters above trail by up to one 30s report interval)"
  fi

  # The collector's memory and backlog over the run. Quarters, like the engine's
  # trends above and for the same reason: what matters is whether either is
  # still climbing at the end.
  if [ -f "$OUT/collecty.csv" ]; then
    awk -F, '
      NR>1 && $2+0 > 0 { n++; for (c = 2; c <= 18; c++) v[c,n] = $c + 0 }
      END {
        if (n < 8) { print "collecty: too few samples for a trend: " n; exit }
        m = 1048576.0
        nr = split("2 anon mib;3 current mib;8 file mib;5 queue mib;6 segments count;\
7 rss mib;11 threads count;12 fds count", rows, ";")
        printf "collecty trends:\n"
        printf "  %-12s %12s %12s %12s %12s  %s\n", "mib/count", "Q1", "Q2", "Q3", "Q4", "trend"
        for (r = 1; r <= nr; r++) {
          split(rows[r], f, " "); c = f[1] + 0
          for (q = 0; q < 4; q++) {
            lo = int(n * q / 4) + 1; hi = int(n * (q + 1) / 4); s = 0
            for (k = lo; k <= hi; k++) s += v[c,k]
            mean[q] = s / (hi - lo + 1)
          }
          trend = (mean[3] > mean[1] * 1.10) ? "GROWING" : "flat"
          if (f[3] == "count")
            printf "  %-12s %12.0f %12.0f %12.0f %12.0f  %s\n", f[2], mean[0], mean[1], mean[2], mean[3], trend
          else
            printf "  %-12s %12.1f %12.1f %12.1f %12.1f  %s\n", f[2], mean[0]/m, mean[1]/m, mean[2]/m, mean[3]/m, trend
        }
        peak = 0; for (k = 1; k <= n; k++) if (v[4,k] > peak) peak = v[4,k]
        printf "  peak=%.1f MiB against its cage; samples=%d\n", peak/m, n
        # What the kernel did about the cage and what it cost. `high` counting
        # up is the throttle working. `max`, `oom` and `oom_kill` must be zero,
        # and PSI is the total time the collector was stopped for reclaim --
        # what matters is that it does not climb with every outage.
        printf "  reclaim: high=%d max=%d oom=%d oom_kill=%d psi_some=%.2fs psi_full=%.2fs\n", \
          v[13,n], v[14,n], v[15,n], v[16,n], (v[17,n] - v[17,1]) / 1e6, (v[18,n] - v[18,1]) / 1e6
        dirty = 0; wb = 0
        for (k = 1; k <= n; k++) {
          if (v[9,k] > dirty) dirty = v[9,k]
          if (v[10,k] > wb) wb = v[10,k]
        }
        printf "  cache: peak dirty=%.1f MiB writeback=%.1f MiB (reclaim is cheap while these are small)\n", \
          dirty/m, wb/m
      }' "$OUT/collecty.csv"
  fi

  echo "slow batches (>=250 ms), longest first:"
  sed 's/\x1b\[[0-9;]*m//g' "$SERVER_LOG" | grep "journal batch slow" \
    | sed 's/.*records=/records=/' | sort -t= -k4 -rn | head -5
  echo "server_log_tail:"; sed 's/\x1b\[[0-9;]*m//g' "$SERVER_LOG" | tail -3
} | tee "$OUT/verdict.txt"
