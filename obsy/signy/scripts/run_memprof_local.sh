#!/usr/bin/env bash
#
# The reproduction for docs/MEMORY_ATTRIBUTION.md.
#
#   scripts/run_memprof_local.sh <run-name>
#
# Runs the server under a cgroup v2 memory limit on this machine — no Docker —
# and drives it with the same offered load the comparison bed uses, sampling
# the cgroup's own accounting beside the engine's gauges and, when the binary
# was built with `--features memprof`, the arena attribution.
#
# Why a native cgroup rather than `compare/run.sh`: the OOM this exists to
# explain happens in the first minute of the ingest phase, and the comparison
# bed builds two images, starts two containers and sweeps two limits to get
# there. `systemd-run --user --scope` gives the same accounting — the same
# `memory.max`, the same `memory.stat`, the same `anon` an OOM kill is decided
# on — in about a minute per run. `MemorySwapMax=0` matches the bed's
# `memswap_limit == mem_limit`, which is zero swap on cgroup v2.
#
# Every knob is a default so a caller can vary exactly one:
#
#   M10_LIMIT=8G       ./scripts/run_memprof_local.sh headroom
#   M10_QUERY_EPS=0    ./scripts/run_memprof_local.sh ingest-only
#   M10_SERVER_ENV="SIGNY_MERGE_INTERVAL=3600s" ./scripts/run_memprof_local.sh no-merge
#   M10_SERVER_ENV="MALLOC_ARENA_MAX=1 MALLOC_TRIM_THRESHOLD_=131072" ./scripts/run_memprof_local.sh trimmed
#
# Query-shape weights are the harness's own `SIGNY_LOAD_QUERY_WEIGHT_*`
# and are inherited from the caller's environment.
set -uo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
NAME="${1:?usage: run_memprof_local.sh <run-name>}"
OUT="${M10_OUT:-$ROOT/target/memprof}/$NAME"
rm -rf "$OUT"; mkdir -p "$OUT"

LIMIT="${M10_LIMIT:-2G}"
SECONDS_CAP="${M10_SECONDS:-240}"
EVENTS="${M10_EVENTS:-1200000}"
EPS="${M10_EPS:-20000}"
CONNS="${M10_CONNECTIONS:-8}"
QUERY_EPS="${M10_QUERY_EPS:-5}"
# The comparison bed's seed, so this run's corpus is the bed's corpus.
SEED="${M10_SEED:-1592598566}"
# `load` drives the log workload; `metric-load` drives M14's series-churn one.
PHASE="${M10_PHASE:-load}"
BIN="${M10_BIN:-$ROOT/target/release/signy}"
PORT="${M10_PORT:-3151}"
UNIT="m10-$NAME"

if [ "${M10_SKIP_BUILD:-0}" != "1" ]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --release --bin load
  [ -n "${M10_BIN:-}" ] || cargo build --manifest-path "$ROOT/Cargo.toml" --release \
    --bin signy ${M10_FEATURES:+--features "$M10_FEATURES"}
fi

DATA="${M10_DATA:-$(mktemp -d /tmp/m10-data-XXXXXX)}"
SERVER_LOG="$OUT/server.log"

systemctl --user reset-failed "$UNIT.scope" 2>/dev/null

cleanup() {
  for p in ${SAMPLER_PID:-} ${WATCH_PID:-} ${SERVER_PID:-}; do kill "$p" 2>/dev/null; done
  sleep 1
  [ -n "${SERVER_PID:-}" ] && kill -KILL "$SERVER_PID" 2>/dev/null
  [ "${M10_KEEP_DATA:-0}" = "1" ] || rm -rf "$DATA"
  systemctl --user reset-failed "$UNIT.scope" 2>/dev/null
}
trap cleanup EXIT

# shellcheck disable=SC2086
env SIGNY_LISTEN_ADDR="127.0.0.1:$PORT" \
    SIGNY_DATA_DIR="$DATA" \
    ${M10_SERVER_ENV:-} \
    systemd-run --user --scope --quiet --unit="$UNIT" \
      -p MemoryMax="$LIMIT" -p MemorySwapMax=0 \
      -- "$BIN" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

for _ in $(seq 1 180); do
  curl -fsS "http://127.0.0.1:$PORT/ready" >/dev/null 2>&1 && break
  kill -0 "$SERVER_PID" 2>/dev/null || { echo "server died before ready"; cat "$SERVER_LOG"; exit 1; }
  sleep 0.5
done
# Read from the server's own process rather than derived from this shell's:
# where systemd places a user scope depends on the manager, and a wrong guess
# here would silently sample an empty cgroup.
CG="/sys/fs/cgroup$(cut -d: -f3 "/proc/$SERVER_PID/cgroup")"
[ -d "$CG" ] || { echo "no cgroup at $CG"; exit 1; }
echo "cgroup=$CG memory.max=$(cat "$CG/memory.max") swap.max=$(cat "$CG/memory.swap.max")"

# `anon` is what an OOM kill is decided on and `file` is the page cache the
# cgroup peak also includes, so both are sampled beside the engine's own
# gauges: a gap between them is then visible in one row rather than inferred
# across two files.
(
  echo "t,current,peak,anon,file,slab,sock,kstack,pgtables,memtable,memtable_buffered,pending_flush,wal_backlog,parts,sidecar,part_meta,rg_cache,query_success,query_errors,threads,mp_other,mp_ingest,mp_wal,mp_flush,mp_merge,mp_query,mp_sidecar,mp_part_meta,mp_rg_cache,mp_header,mi_arena,mi_mmap,mi_inuse,mi_free,mp_series_memtable,mp_series_catalog,active_series,metric_parts,series_memtable_gauge"
  T0=$(date +%s.%N)
  while [ -d "$CG" ]; do
    now=$(date +%s.%N)
    cur=$(cat "$CG/memory.current" 2>/dev/null) || break
    peak=$(cat "$CG/memory.peak" 2>/dev/null)
    st=$(cat "$CG/memory.stat" 2>/dev/null)
    m=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/metrics" 2>/dev/null)
    cg=$(echo "$st" | awk '
      $1=="anon"{a=$2} $1=="file"{f=$2} $1=="slab"{s=$2} $1=="sock"{k=$2}
      $1=="kernel_stack"{ks=$2} $1=="pgtables"{p=$2}
      END{printf "%d,%d,%d,%d,%d,%d", a, f, s, k, ks, p}')
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
      /^signy_active_series /{as=$2}
      /^signy_metric_part_count /{mpc=$2}
      /^signy_series_memtable_bytes /{smb=$2}
      END{printf "%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d",
          v["other"], v["ingest"], v["wal"], v["flush"], v["merge"],
          v["query"], v["sidecar"], v["part_meta"], v["row_group_cache"], h,
          i["arena"], i["mmapped"], i["in_use"], i["free"],
          v["series_memtable"], v["series_catalog"], as, mpc, smb}')
    printf '%s,%s,%s,%s,%s,%s,%s\n' \
      "$(awk -v a="$now" -v b="$T0" 'BEGIN{printf "%.2f", a-b}')" \
      "$cur" "$peak" "$cg" "$gg" "${threads:-0}" "$mp"
    sleep 0.25
  done
) >"$OUT/mem.csv" 2>/dev/null &
SAMPLER_PID=$!

# The cumulative counters, kept whole: `mem.csv` carries live bytes, and the
# allocation traffic that explains them is only in the totals.
(
  while true; do
    body=$(curl -fsS --max-time 2 "http://127.0.0.1:$PORT/metrics" 2>/dev/null | grep memprof)
    [ -n "$body" ] && { echo "=== $(date +%s.%N)"; echo "$body"; }
    sleep 1
  done
) >"$OUT/memprof.txt" 2>/dev/null &
PROF_PID=$!

# The harness has a 60 s request timeout, so without this a run whose server
# was killed at t=40 s would sit here for minutes producing nothing.
( while kill -0 "$SERVER_PID" 2>/dev/null; do sleep 1; done; sleep 3; pkill -f 'target/release/load' ) &
WATCH_PID=$!

SIGNY_LOAD_TARGET=signy \
SIGNY_LOAD_PHASE="$PHASE" \
SIGNY_LOAD_ADDR="127.0.0.1:$PORT" \
SIGNY_LOAD_CGROUP="$CG" \
SIGNY_LOAD_TIER=memprof \
SIGNY_LOAD_SEED="$SEED" \
SIGNY_LOAD_SECONDS="$SECONDS_CAP" \
SIGNY_LOAD_EVENTS="$EVENTS" \
SIGNY_LOAD_TARGET_EPS="$EPS" \
SIGNY_LOAD_CONNECTIONS="$CONNS" \
SIGNY_LOAD_QUERY_EPS="$QUERY_EPS" \
SIGNY_LOAD_OTLP_EPS=0 \
SIGNY_LOAD_RESULT_PATH="$OUT/load.json" \
  "$ROOT/target/release/load" >"$OUT/harness.log" 2>&1
HARNESS_STATUS=$?

kill "$WATCH_PID" 2>/dev/null; WATCH_PID=
kill "$PROF_PID" 2>/dev/null
sleep 0.5
kill "$SAMPLER_PID" 2>/dev/null; SAMPLER_PID=

ALIVE=false
kill -0 "$SERVER_PID" 2>/dev/null && ALIVE=true

# The row at the peak of `anon`, restricted to samples whose scrape succeeded:
# the last sample of a killed run has a live cgroup reading and a dead server,
# and reporting it would attribute the whole peak to nothing.
{
  echo "run=$NAME limit=$LIMIT alive=$ALIVE harness_status=$HARNESS_STATUS"
  # Exit 3 is the harness reporting that nothing it pushed reached storage. A
  # peak measured through that is the resident set of an idle server, and it is
  # low, so it reads as the best result this script can produce.
  if [ "$HARNESS_STATUS" = 3 ]; then
    echo "NO LOAD DELIVERED: harness exit 3, so the peak below is an idle server"
  fi
  echo "data_dir_bytes=$(du -sb "$DATA" 2>/dev/null | cut -f1)"
  awk -F, 'NR>1 && $4+0>best { best=$4+0; for (i=1;i<=NF;i++) r[i]=$i }
           NR>1 && $30+0>0 && $4+0>bestm { bestm=$4+0; for (i=1;i<=NF;i++) s[i]=$i }
    END {
      m = 1048576.0
      printf "duration_s=%s\nanon_peak_mib=%.1f\nfile_at_peak_mib=%.1f\n", r[1], r[4]/m, r[5]/m
      if (bestm > 0) {
        live = (s[21]+s[22]+s[23]+s[24]+s[25]+s[26]+s[27]+s[28]+s[29]+s[35]+s[36])/m
        printf "at_last_scrape_t=%s\n", s[1]
        printf "  cgroup   anon=%.1f file=%.1f threads=%s\n", s[4]/m, s[5]/m, s[20]
        printf "  glibc    arena=%.1f mmapped=%.1f in_use=%.1f free=%.1f\n", s[31]/m, s[32]/m, s[33]/m, s[34]/m
        printf "  arenas   other=%.1f ingest=%.1f wal=%.1f flush=%.1f merge=%.1f query=%.1f sidecar=%.1f part_meta=%.1f rg_cache=%.1f\n", \
               s[21]/m, s[22]/m, s[23]/m, s[24]/m, s[25]/m, s[26]/m, s[27]/m, s[28]/m, s[29]/m
        printf "  live=%.1f instrument_header=%.1f anon/live=%.2f free/anon=%.2f\n", live, s[30]/m, s[4]/m/live, s[34]/s[4]
        printf "  gauges   memtable=%.1f wal_backlog=%.1f parts=%s rg_cache_gauge=%.1f ingest_live/memtable=%.2f\n", \
               s[10]/m, s[13]/m, s[14], s[17]/m, s[22]/(s[10]+1)
        printf "  metrics  series_memtable=%.1f series_catalog=%.1f active_series=%s metric_parts=%s accounted=%.1f\n", \
               s[35]/m, s[36]/m, s[37], s[38], s[39]/m
        if (s[37]+0 > 0) {
          printf "  per_series memtable=%.0fB catalog=%.0fB catalog_per_part=%.0fB anon=%.0fB\n", \
                 s[35]/s[37], s[36]/s[37], (s[38]+0>0 ? s[36]/s[37]/s[38] : 0), s[4]/s[37]
        }
      }
    }' "$OUT/mem.csv"
  echo "server_log_tail:"; sed 's/\x1b\[[0-9;]*m//g' "$SERVER_LOG" | tail -3
} | tee "$OUT/verdict.txt"
