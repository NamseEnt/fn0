#!/usr/bin/env bash
# One memory experiment, cages included.
#
# docs/MEMORY.md lists the experiments this runs. Each one is this script with
# exactly one thing changed, so that two runs are comparable: the load, the
# sink, the corpus, the rate and the connection count are the same every time
# unless the experiment is about one of them.
#
#   scripts/run_mem_local.sh baseline
#   MEM_ENV="MALLOC_ARENA_MAX=1" scripts/run_mem_local.sh arena1
#   MEM_FEATURES="memprof,mimalloc" scripts/run_mem_local.sh mimalloc
#   MEM_OUTAGE_AT=120 scripts/run_mem_local.sh drain
#   MEM_CONNECTIONS=256 scripts/run_mem_local.sh conn-256
#
# The verdict is peak anon out of the cgroup's own memory.stat, because that is
# what an OOM kill is decided on. The collector's own view is beside it and is
# never the verdict -- it is the thing being audited.
set -uo pipefail

NAME="${1:?usage: run_mem_local.sh <name>}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Overridable so a build from a worktree can still put its queue on a real
# filesystem: see the tmpfs check below for why that is not a detail.
OUT="${MEM_OUT:-$ROOT/target/mem}/$NAME"

SECONDS_CAP="${MEM_SECONDS:-300}"
SETTLE="${MEM_SETTLE:-60}"
# Calibrated against what the 24-hour soak's collector actually received,
# read out of its own queue reports: 205 exports/s of a 41,785 byte mean, so
# 8.57 MB/s. These give 204 exports/s of 41,656 B. The one thing that does not
# match is the compression ratio -- 3.63x here against the soak's 4.87x -- so
# a segment here is larger than production's, which errs towards more memory
# pressure rather than less.
EPS="${MEM_EPS:-33000}"
CONNECTIONS="${MEM_CONNECTIONS:-8}"
RECORDS="${MEM_RECORDS:-161}"
TRACE_EPS="${MEM_TRACE_EPS:-5}"
METRIC_EVERY="${MEM_METRIC_EVERY:-10}"
LIMIT="${MEM_LIMIT:-256M}"
# The throttle below the limit. Crossing `memory.high` makes the kernel
# reclaim this cgroup and hold the writer up while it does; crossing
# `memory.max` is where the OOM killer comes in. Unset by default, because
# every run before this one had it unset.
HIGH="${MEM_HIGH:-}"
# `-` and not `:-`: MEM_FEATURES="" means the shipped build, which is a
# different experiment from not saying which build to use.
FEATURES="${MEM_FEATURES-memprof}"
COLLECTY_PORT="${MEM_COLLECTY_PORT:-4318}"
SINK_PORT="${MEM_SINK_PORT:-4319}"
OUTAGE_AT="${MEM_OUTAGE_AT:-}"
OUTAGE_FOR="${MEM_OUTAGE_FOR:-180}"
# The shipped ceilings. Left alone unless the experiment is about one of them,
# so a run measures the configuration a deployment gets.
INFLIGHT="${MEM_INFLIGHT:-64MiB}"
QUEUE_MAX="${MEM_QUEUE_MAX:-1GiB}"
SEGMENT="${MEM_SEGMENT:-8MiB}"
UNIT="collecty-mem-$NAME"

# A scope that failed -- an OOM kill, say -- stays registered under its name
# and systemd-run then refuses to reuse it, which surfaces as a cgroup that
# never appears rather than as anything about the last run.
systemctl --user reset-failed "$UNIT.scope" 2>/dev/null

rm -rf "$OUT"; mkdir -p "$OUT"
QUEUE_DIR="$OUT/queue"

cleanup() {
  rm -f "$OUT/RUNNING"
  [ -n "${SAMPLER_PID:-}" ] && kill "$SAMPLER_PID" 2>/dev/null
  systemctl --user stop "$UNIT.scope" 2>/dev/null
  [ -n "${COLLECTY_PID:-}" ] && kill "$COLLECTY_PID" 2>/dev/null
  sleep 0.5
  [ -n "${COLLECTY_PID:-}" ] && kill -KILL "$COLLECTY_PID" 2>/dev/null
  [ "${MEM_KEEP_QUEUE:-0}" = "1" ] || rm -rf "$QUEUE_DIR"
  return 0
}
trap cleanup EXIT

require() { command -v "$1" >/dev/null || { echo "$1 is needed and is not here"; exit 4; }; }
require systemd-run
require systemctl
[ "$(stat -fc %T /sys/fs/cgroup 2>/dev/null)" = "cgroup2fs" ] || {
  echo "NOT_MEASURED: cgroup v2 is not mounted, so nothing would be limited"; exit 4; }

# The queue has to be on a real filesystem. tmpfs pages are shmem, they are
# charged to the writing cgroup, and with swap off they cannot be reclaimed --
# so a backlog the collector is designed to put on disk instead fills the cage
# and the kernel kills the process. That is not a memory result, it is the
# measurement destroying itself, and it took one wasted run to find.
FSTYPE=$(stat -fc %T "$(dirname "$OUT")" 2>/dev/null || stat -fc %T "$ROOT")
[ "$FSTYPE" = "tmpfs" ] && {
  echo "NOT_MEASURED: $OUT is on tmpfs; its queue would be charged to the cage as shmem"
  echo "              set MEM_OUT to a directory on a real filesystem"
  exit 4; }

echo "building (features: ${FEATURES:-none})"
if [ -n "$FEATURES" ]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --release --bin collecty --features "$FEATURES" || exit 4
else
  cargo build --manifest-path "$ROOT/Cargo.toml" --release --bin collecty || exit 4
fi
cargo build --manifest-path "$ROOT/Cargo.toml" --release --example memrig || exit 4

# The collector in its own scope at the declared limit, swap off, exactly as
# signy's gate cages its engine.
env COLLECTY_DATA_DIR="$OUT" \
    COLLECTY_LISTEN_ADDR="127.0.0.1:$COLLECTY_PORT" \
    COLLECTY_SIGNY_URL="http://127.0.0.1:$SINK_PORT" \
    COLLECTY_MAX_INFLIGHT_BYTES="$INFLIGHT" \
    COLLECTY_QUEUE_MAX_BYTES="$QUEUE_MAX" \
    COLLECTY_QUEUE_SEGMENT_BYTES="$SEGMENT" \
    COLLECTY_REPORT_INTERVAL="10s" \
    COLLECTY_MEMPROF_CSV="$OUT/memprof.csv" \
    COLLECTY_MEMPROF_INTERVAL_MS=250 \
    ${MEM_ENV:-} \
    systemd-run --user --scope --quiet --unit="$UNIT" \
      -p MemoryMax="$LIMIT" -p MemorySwapMax=0 ${HIGH:+-p MemoryHigh="$HIGH"} \
      "$ROOT/target/release/collecty" >"$OUT/collecty.log" 2>&1 &
COLLECTY_PID=$!

CGROUP=""
for _ in $(seq 1 50); do
  CANDIDATE="/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$UNIT.scope"
  [ -f "$CANDIDATE/memory.max" ] && { CGROUP="$CANDIDATE"; break; }
  CANDIDATE="/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/app.slice/$UNIT.scope"
  [ -f "$CANDIDATE/memory.max" ] && { CGROUP="$CANDIDATE"; break; }
  sleep 0.2
done
[ -n "$CGROUP" ] || { echo "NOT_MEASURED: the scope's cgroup never appeared"; exit 4; }

# The limit has to be readable and the one that was asked for, or the number
# this run reports means nothing.
WANT=$(numfmt --from=iec "${LIMIT%B}" 2>/dev/null || echo 0)
GOT=$(cat "$CGROUP/memory.max")
[ "$GOT" = "$WANT" ] || { echo "NOT_MEASURED: memory.max is $GOT, asked for $WANT"; exit 4; }
[ "$(cat "$CGROUP/memory.swap.max")" = "0" ] || { echo "NOT_MEASURED: swap is not off"; exit 4; }
if [ -n "$HIGH" ]; then
  WANT_HIGH=$(numfmt --from=iec "${HIGH%B}" 2>/dev/null || echo 0)
  GOT_HIGH=$(cat "$CGROUP/memory.high")
  [ "$GOT_HIGH" = "$WANT_HIGH" ] || {
    echo "NOT_MEASURED: memory.high is $GOT_HIGH, asked for $WANT_HIGH"; exit 4; }
fi

for _ in $(seq 1 50); do
  grep -q "accepting OTLP" "$OUT/collecty.log" 2>/dev/null && break
  sleep 0.2
done
grep -q "accepting OTLP" "$OUT/collecty.log" || { echo "NOT_MEASURED: collecty never listened"; tail -5 "$OUT/collecty.log"; exit 4; }

touch "$OUT/RUNNING"
(
  # Anon is the collector. The rest is the queue on disk, which a cgroup
  # charges to whoever wrote it: how much of it is cache, how much of that is
  # still dirty or in writeback and therefore not reclaimable yet, and how it
  # splits between the inactive list reclaim takes first and the active one it
  # does not. `memory.events.local` says whether the kernel ever had to push
  # back, and `memory.pressure` -- PSI -- says what that cost the workload,
  # which "memory is high" on its own cannot.
  echo "t,anon,file,file_dirty,file_writeback,inactive_file,active_file,current,peak,cpu_usec,psi_some_us,psi_full_us,ev_high,ev_max,ev_oom,oom_kills,queue_bytes,segments,alive"
  T0=$(date +%s.%N)
  while [ -f "$OUT/RUNNING" ]; do
    now=$(date +%s.%N)
    mem=$(awk '$1=="anon"{a=$2} $1=="file"{f=$2} $1=="file_dirty"{d=$2}
               $1=="file_writeback"{w=$2} $1=="inactive_file"{i=$2}
               $1=="active_file"{c=$2}
               END{printf "%d,%d,%d,%d,%d,%d", a, f, d, w, i, c}' \
             "$CGROUP/memory.stat" 2>/dev/null)
    psi=$(awk '{for (n = 1; n <= NF; n++) if ($n ~ /^total=/) { sub("total=", "", $n)
                  if ($1 == "some") s = $n; else if ($1 == "full") u = $n }}
               END{printf "%d,%d", s, u}' "$CGROUP/memory.pressure" 2>/dev/null)
    events=$(awk '$1=="high"{h=$2} $1=="max"{m=$2} $1=="oom"{o=$2}
                  $1=="oom_kill"{k=$2} END{printf "%d,%d,%d,%d", h, m, o, k}' \
                 "$CGROUP/memory.events.local" 2>/dev/null)
    cpu=$(awk '$1=="usage_usec"{print $2}' "$CGROUP/cpu.stat" 2>/dev/null)
    printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
      "$(awk -v a="$now" -v b="$T0" 'BEGIN{printf "%.2f", a-b}')" \
      "${mem:-0,0,0,0,0,0}" "$(cat "$CGROUP/memory.current" 2>/dev/null)" \
      "$(cat "$CGROUP/memory.peak" 2>/dev/null)" "${cpu:-0}" \
      "${psi:-0,0}" "${events:-0,0,0,0}" \
      "$(du -sb "$QUEUE_DIR" 2>/dev/null | cut -f1)" \
      "$(find "$QUEUE_DIR" -name '*.seg' 2>/dev/null | wc -l)" \
      "$(kill -0 "$COLLECTY_PID" 2>/dev/null && echo 1 || echo 0)"
    sleep 0.25
  done
) >"$OUT/cgroup.csv" 2>/dev/null &
SAMPLER_PID=$!

OUTAGE_FLAGS=()
[ -n "$OUTAGE_AT" ] && OUTAGE_FLAGS=(--outage-at "$OUTAGE_AT" --outage-for "$OUTAGE_FOR")

echo "load: ${EPS} eps over ${CONNECTIONS} connections for ${SECONDS_CAP}s, cage $LIMIT${HIGH:+, throttle $HIGH}"
"$ROOT/target/release/examples/memrig" \
  --collecty "127.0.0.1:$COLLECTY_PORT" --sink "127.0.0.1:$SINK_PORT" \
  --eps "$EPS" --connections "$CONNECTIONS" --seconds "$SECONDS_CAP" \
  --records-per-export "$RECORDS" --report "$OUT/rig.json" \
  --latency-csv "$OUT/latency.csv" \
  --trace-eps "$TRACE_EPS" --metric-every "$METRIC_EVERY" \
  "${OUTAGE_FLAGS[@]}" >"$OUT/rig.log" 2>&1 &
RIG_PID=$!
wait $RIG_PID
RIG_STATUS=$?

# The settle is where signy's gate found its worst peak: the workload stopping
# is not the process being done.
echo "settling for ${SETTLE}s"
sleep "$SETTLE"

rm -f "$OUT/RUNNING"
sleep 0.5

# From the samples, which were taken while the scope still existed. Read here
# instead, it would be read after the scope may already be gone -- which is how
# a run that the kernel killed first reported itself MEASURED.
# By the header's name and not by a column number: adding a column to the
# sampler once silently turned the queue's size into an OOM count, and the
# run reported OOM_KILLED with nothing killed.
KILLED=$(awk -F, 'NR==1 {for (i = 1; i <= NF; i++) if ($i == "oom_kills") c = i; next}
                  c && $c+0 > k {k = $c} END {print k+0}' "$OUT/cgroup.csv" 2>/dev/null)
kill -TERM "$COLLECTY_PID" 2>/dev/null
sleep 2

python3 - "$OUT" "$LIMIT" "${KILLED:-0}" "$RIG_STATUS" "${OUTAGE_AT:-0}" <<'PY'
import csv, json, sys, os

out, limit, killed, rig_status = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
outage_at = float(sys.argv[5])
MiB = 1048576.0

def rows(name):
    path = os.path.join(out, name)
    if not os.path.exists(path):
        return []
    with open(path) as handle:
        return [r for r in csv.DictReader(handle) if r.get("t")]

cg = rows("cgroup.csv")
mp = rows("memprof.csv")
anon = [float(r["anon"]) for r in cg if r["anon"].isdigit()]
result = {"name": os.path.basename(out), "limit": limit, "oom_kills": killed}

# The collector has to have been alive for the whole run. A trace that stops
# early is a crash, and a crash is not a budget result.
alive = [r for r in cg if r.get("alive") == "1"]
if cg and alive:
    result["alive_until"] = round(float(alive[-1]["t"]), 1)
    result["run_until"] = round(float(cg[-1]["t"]), 1)
    if result["run_until"] - result["alive_until"] > 5.0:
        result["died_early"] = True

if cg:
    current = [float(r["current"]) for r in cg if r["current"].isdigit()]
    if current:
        result["peak_current_mib"] = round(max(current) / MiB, 1)
        result["final_current_mib"] = round(current[-1] / MiB, 1)
    # The queue is on disk, so its pages are page cache, and a cgroup counts
    # those. A sidecar meets its limit through this long before it meets it
    # through the heap: `docs/MEMORY.md` §3 has a run that reached 255.9 MiB of
    # a 256 MiB cage with anon at 105.8.
    cache = [float(r["file"]) for r in cg if r.get("file", "").isdigit()]
    if cache:
        result["peak_file_mib"] = round(max(cache) / MiB, 1)
        result["final_file_mib"] = round(cache[-1] / MiB, 1)
    # What the whole scope spent, sampled from the cgroup rather than from the
    # process, so it counts every thread including the ones tokio made.
    cpu = [float(r["cpu_usec"]) for r in cg if r.get("cpu_usec", "").isdigit()]
    if cpu and max(cpu) > 0:
        result["cpu_seconds"] = round((max(cpu) - min(cpu)) / 1e6, 1)
    # How deep the backlog got. The queue is the collector's answer to a sink
    # that is away, so a run that changed the footprint by holding less of one
    # is a different experiment, and this is the column that says so.
    depth = [float(r["queue_bytes"]) for r in cg if r.get("queue_bytes", "").isdigit()]
    if depth:
        result["peak_queue_mib"] = round(max(depth) / MiB, 1)
    held = [int(r["segments"]) for r in cg if r.get("segments", "").isdigit()]
    if held:
        result["peak_segments"] = max(held)

    # What the kernel had to do about all that cache, and what it cost.
    # `high` counting up without `max` or `oom` is the throttle working; PSI
    # is how long the workload was stopped while it worked.
    def column(name):
        return [int(r[name]) for r in cg if r.get(name, "").isdigit()]

    reclaim = {}
    for field in ("file_dirty", "file_writeback", "inactive_file", "active_file"):
        values = column(field)
        if values:
            reclaim[f"peak_{field}_mib"] = round(max(values) / MiB, 1)
            reclaim[f"final_{field}_mib"] = round(values[-1] / MiB, 1)
    for field in ("ev_high", "ev_max", "ev_oom"):
        values = column(field)
        if values:
            reclaim[field] = max(values)
    for field, name in (("psi_some_us", "psi_some_seconds"), ("psi_full_us", "psi_full_seconds")):
        values = column(field)
        if values:
            reclaim[name] = round((max(values) - min(values)) / 1e6, 2)
    if reclaim:
        result["reclaim"] = reclaim

# A drain is three phases and one number cannot hold them: what the collector
# sat at before the sink went away, what it reached while the backlog came
# back out, and what it kept once everything was gone. The step between the
# first and the last is the one `docs/MEMORY.md` calls permanent.
if cg and outage_at > 0:
    before = [r for r in cg if float(r["t"]) <= outage_at]
    if before:
        result["pre_outage_anon_mib"] = round(float(before[-1]["anon"]) / MiB, 1)
        result["pre_outage_current_mib"] = round(float(before[-1]["current"]) / MiB, 1)

if not anon:
    result["verdict"] = "NOT_MEASURED"
    result["reason"] = "no cgroup samples"
else:
    result["peak_anon_mib"] = round(max(anon) / MiB, 1)
    result["final_anon_mib"] = round(anon[-1] / MiB, 1)
    result["samples"] = len(anon)

try:
    with open(os.path.join(out, "rig.json")) as handle:
        result["load"] = json.load(handle)
except Exception:
    result["load"] = None

if mp:
    last = mp[-1]
    peak_live = max(int(r["tagged_live"]) for r in mp)
    result["engine"] = {
        "peak_tagged_live_mib": round(peak_live / MiB, 1),
        "final_tagged_live_mib": round(int(last["tagged_live"]) / MiB, 1),
        "final_by_arena_mib": {
            a: round(int(last[f"tagged_live_{a}"]) / MiB, 2)
            for a in ("other", "intake", "queue", "send")
        },
        "final_rss_mib": round(int(last["rss"]) / MiB, 1),
        "peak_threads": max(int(r["threads"]) for r in mp),
        "final_libc_in_use_mib": round(int(last["libc_in_use"]) / MiB, 1),
        "final_libc_free_mib": round(int(last["libc_free"]) / MiB, 1),
        "final_libc_mmap_mib": round(int(last["libc_mmap"]) / MiB, 1),
        "peak_inflight_mib": round(max(int(r["inflight_bytes"]) for r in mp) / MiB, 2),
        "header_bytes_mib": round(int(last["header_bytes"]) / MiB, 1),
        "total_allocated_gb": round(int(last["total_bytes"]) / 1e9, 1),
        "total_allocations_m": round(int(last["total_allocs"]) / 1e6, 1),
    }
    # Under glibc the tagged total and the C side share one heap, so this is
    # C-side allocation plus whatever Rust allocation no guard covered -- an
    # approximation, and the column that moves when zstd stops churning.
    result["engine"]["untagged_in_use_mib"] = round(
        (int(last["libc_in_use"]) - int(last["tagged_live"])) / MiB, 1
    )

if "verdict" not in result:
    load = result.get("load") or {}
    offered = load.get("offered_exports", 0)
    accepted = load.get("accepted_exports", 0)
    if killed:
        result["verdict"] = "OOM_KILLED"
    elif result.get("died_early"):
        result["verdict"] = "NOT_MEASURED"
        result["reason"] = (
            f"the collector stopped at t={result['alive_until']}s "
            f"of a {result['run_until']}s run"
        )
    elif rig_status != 0 or offered == 0:
        result["verdict"] = "NOT_MEASURED"
        result["reason"] = "the load did not run"
    elif accepted < offered * 0.9:
        result["verdict"] = "NOT_MEASURED"
        result["reason"] = f"only {accepted} of {offered} exports were accepted"
    else:
        result["verdict"] = "MEASURED"

with open(os.path.join(out, "result.json"), "w") as handle:
    json.dump(result, handle, indent=2)
print(json.dumps(result, indent=2))
PY
