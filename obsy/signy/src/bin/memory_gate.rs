//! The gate for [`docs/VISION.md`] invariant I: at a declared memory budget,
//! under sustained mixed load, does the engine survive — and does the *cgroup's*
//! anonymous footprint stay under the number the operator declared?
//!
//! **It measures `anon` out of the cgroup's `memory.stat`, and nothing else.**
//! `docs/MEMORY_ATTRIBUTION.md` is the reason: at the instant the kernel killed
//! this engine at a 2 GiB limit, its own live-byte accounting read 669 MiB and
//! `signy_memtable_bytes` read 111 MB, because 44 % of the anonymous
//! footprint was memory the process had already freed. A budget validated
//! against the engine's own view of itself would have reported headroom at the
//! moment of death. The cgroup's `memory.peak` is no better in the other
//! direction: it includes the page cache this engine's own Parquet writes
//! create, so it says the limit was touched by data that was reclaimable.
//! `anon` is what an OOM kill is decided on. Both are recorded; only `anon` is
//! gated.
//!
//! **Why a binary rather than a `#[test]`.** It needs a cgroup scope, a real
//! server process and minutes of wall clock, so it cannot be an ordinary unit
//! test; `cargo test` stays a second long. It is not a shell script either,
//! because `docs/LOAD_RESULTS.md`'s retirement is what happens when a
//! measurement lives in a script and a document and drifts away from the code:
//! the verdict here is typed, it compiles under the same
//! `clippy --all-targets -D warnings` gate as the engine, and there is no
//! second copy of it in `awk`. It is one command with one machine-readable
//! answer:
//!
//! ```text
//! cargo run --release --bin memory_gate -- --budget 2GiB
//! ```
//!
//! **Four outcomes, four exit codes**, because "it failed" is not a finding:
//!
//! | exit | verdict | means |
//! |---|---|---|
//! | 0 | `UNDER_BUDGET` | survived, delivered the workload, peak `anon` ≤ budget |
//! | 2 | `OVER_BUDGET` | survived, but peak `anon` exceeded the budget |
//! | 3 | `OOM_KILLED` | the kernel killed it inside its own declared budget |
//! | 4 | `NOT_MEASURED` | the measurement did not happen |
//!
//! `NOT_MEASURED` is a failure, never a skip: `docs/LOAD_RESULTS.md` §3 records
//! a peak RSS that had never been measured being written down as an engine
//! result, and the rule it arrived at is that **a gate that cannot measure must
//! not pass.** So a missing cgroup, a limit that did not apply, a server that
//! never became ready, a server that crashed for reasons other than the OOM
//! killer, zero samples, or a workload the engine refused most of, are all
//! failures with a stated reason rather than a green light.
//!
//! `OVER_BUDGET` is only reachable when `--limit` is larger than `--budget`.
//! With the two equal — an operator's case, and the default — the kernel
//! enforces the ceiling before `anon` can cross it, so the honest answer at a
//! budget this engine cannot hold is `OOM_KILLED`. Raising `--limit` above
//! `--budget` is how the overshoot is *measured* rather than merely fatal, and
//! it is how the baseline in `docs/MEMORY_BUDGET_GATE.md` was found.
//!
//! **Not in CI.** It needs a cgroup v2 scope, a systemd user manager and
//! minutes per run, and the number it produces is worthless on a shared runner
//! whose neighbours are invisible. CI compiles it — `clippy --all-targets` and
//! `cargo test` both build this target — so it cannot rot, and running it
//! anywhere without a usable cgroup exits 4 rather than passing.
//!
//! `SIGNY_MEMORY_BUDGET` exists now (2026-08-08), and the two numbers
//! meet by construction rather than by being repeated: this gate creates the
//! cgroup scope at `--limit`, and the server inside detects that same scope's
//! `memory.max` and declares 60% of it as its budget, deriving its ceilings
//! from that. `--server-env SIGNY_MEMORY_BUDGET=...` still overrides for
//! an A/B, and `--server-env SIGNY_MEMORY_BUDGET=off` measures the
//! pre-budget behaviour.

use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

const EXIT_UNDER_BUDGET: i32 = 0;
const EXIT_OVER_BUDGET: i32 = 2;
const EXIT_OOM_KILLED: i32 = 3;
const EXIT_NOT_MEASURED: i32 = 4;
const EXIT_USAGE: i32 = 64;

/// 4 Hz, the rate `scripts/run_memprof_local.sh` samples at, so the two
/// instruments miss the same spikes and their numbers stay comparable.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(250);

/// Which workload the gate drives.
///
/// The log scenario is M10's: a paced mixed load whose peak the budget is
/// compared against. The metric one is M14's, and it asks the question the
/// metrics claim rests on — an engine given a budget and then handed more
/// series than that budget can index must contain the churn, not die of it.
/// Its acceptance gate reads the **steady** phase alone: the churn and
/// explosion phases refuse new series *by design*, and gating on their
/// acceptance would fail the engine for doing the thing being measured.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scenario {
    Logs,
    Metrics,
}

impl Scenario {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "logs" | "log" => Ok(Self::Logs),
            "metrics" | "metric" => Ok(Self::Metrics),
            other => Err(format!("--scenario must be logs or metrics, got {other:?}")),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Logs => "logs",
            Self::Metrics => "metrics",
        }
    }

    fn phase(self) -> &'static str {
        match self {
            Self::Logs => "load",
            Self::Metrics => "metric-load",
        }
    }
}

/// The workload has to have actually been delivered, or the budget was never
/// exercised. A future budget knob could otherwise pass this gate by refusing
/// every push with a `429` and holding nothing — which is the same class of
/// false green as §3's unmeasured RSS.
const MIN_DELIVERED_FRACTION: f64 = 0.9;

/// How long the harness is given after the server dies before it is killed.
/// Its request timeout is 60 s, so without this a run whose server was killed
/// at t=40 s sits idle for minutes.
const HARNESS_GRACE: Duration = Duration::from_secs(3);

/// Wall clock the run is allowed beyond its own duration cap, covering corpus
/// generation, readiness and teardown. Exceeding it is `NOT_MEASURED`.
const DEADLINE_SLACK: Duration = Duration::from_secs(180);

const MIB: f64 = 1024.0 * 1024.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    UnderBudget,
    OverBudget,
    OomKilled,
    NotMeasured,
}

impl Outcome {
    fn verdict(self) -> &'static str {
        match self {
            Outcome::UnderBudget => "UNDER_BUDGET",
            Outcome::OverBudget => "OVER_BUDGET",
            Outcome::OomKilled => "OOM_KILLED",
            Outcome::NotMeasured => "NOT_MEASURED",
        }
    }

    fn exit_code(self) -> i32 {
        match self {
            Outcome::UnderBudget => EXIT_UNDER_BUDGET,
            Outcome::OverBudget => EXIT_OVER_BUDGET,
            Outcome::OomKilled => EXIT_OOM_KILLED,
            Outcome::NotMeasured => EXIT_NOT_MEASURED,
        }
    }
}

struct Args {
    budget_bytes: u64,
    budget_text: String,
    limit_bytes: u64,
    limit_text: String,
    name: String,
    out_dir: PathBuf,
    seconds: u64,
    events: u64,
    eps: f64,
    connections: usize,
    query_eps: f64,
    query_connections: usize,
    seed: u64,
    port: u16,
    /// Seconds to keep the server running after the workload stops.
    ///
    /// Without this the gate measures the peak of *accepting* load and calls it
    /// the peak. It is not: the comparison bed was OOM-killed fifteen seconds
    /// after its last row was accepted, in the idle settle, while merge
    /// consolidated the parts ingest had left behind. The default matches that
    /// bed's `COMPARE_SETTLE_SECONDS` so the two ask the same question.
    settle: u64,
    server_env: Vec<(String, String)>,
    skip_build: bool,
    keep_data: bool,
    scenario: Scenario,
}

const USAGE: &str = "\
usage: memory_gate --budget <size> [options]

  --budget <size>        the declared budget peak cgroup `anon` is compared
                         against (required), e.g. 2GiB, 3584MiB, 8G
  --limit <size>         the cgroup's memory.max. Defaults to --budget, which is
                         what an operator does. A larger limit makes an overshoot
                         measurable (OVER_BUDGET) instead of fatal (OOM_KILLED).
  --name <name>          run name, used for the output directory and the scope
                         unit (default: budget)
  --out <dir>            output directory (default: target/memory_gate/<name>)
  --seconds <n>          duration cap for the workload (default: 240)
  --events <n>           events to ingest before stopping (default: 1200000)
  --eps <n>              offered ingest rate (default: 20000)
  --connections <n>      ingest connections (default: 8)
  --query-eps <n>        query rate; must be > 0 (default: 5)
  --query-connections <n> query connections, separate from ingest (default: 4)
  --settle <n>           seconds to keep sampling after the workload stops, so
                         the merge backlog load leaves behind is inside the peak
                         (default: 150, the comparison bed's settle). 0 measures
                         only the peak of accepting load, which is what this
                         gate did before it was found to miss a kill.
  --scenario <what>      logs (default) drives the paced mixed log load; metrics
                         drives the M14 churn phases — steady, rolling series
                         replacement, then a cardinality burst — and gates
                         acceptance on the steady phase alone, because the
                         later phases refuse new series by design
  --seed <n>             corpus seed (default: 1592598566, the comparison bed's)
  --port <n>             server HTTP port (default: 3251)
  --server-env K=V       extra environment for the server, repeatable
  --skip-build           do not run cargo build first
  --keep-data            keep the server's data directory

exit codes: 0 under budget, 2 over budget, 3 OOM-killed, 4 not measured,
64 usage. Anything other than 0 is a failure.
";

impl Args {
    fn parse() -> Result<Self, String> {
        let mut budget: Option<(u64, String)> = None;
        let mut limit: Option<(u64, String)> = None;
        let mut name = "budget".to_string();
        let mut out: Option<PathBuf> = None;
        let mut seconds = 240;
        let mut events = 1_200_000;
        let mut eps = 20_000.0;
        let mut connections = 8;
        let mut query_eps = 5.0;
        let mut query_connections = 4;
        // The comparison bed's seed, so this run's corpus is the corpus
        // docs/COMPARISON.md and docs/MEMORY_ATTRIBUTION.md were measured on.
        let mut seed = 1_592_598_566;
        let mut port = 3251;
        let mut settle = 150;
        let mut server_env = Vec::new();
        let mut scenario = Scenario::Logs;
        let mut skip_build = false;
        let mut keep_data = false;

        let mut arguments = std::env::args().skip(1);
        while let Some(flag) = arguments.next() {
            let mut value = || {
                arguments
                    .next()
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag.as_str() {
                "--budget" => budget = Some(parse_size(&value()?)?),
                "--limit" => limit = Some(parse_size(&value()?)?),
                "--name" => name = value()?,
                "--out" => out = Some(PathBuf::from(value()?)),
                "--seconds" => seconds = parse_number(&value()?)?,
                "--events" => events = parse_number(&value()?)?,
                "--eps" => eps = parse_number(&value()?)?,
                "--connections" => connections = parse_number(&value()?)?,
                "--query-eps" => query_eps = parse_number(&value()?)?,
                "--query-connections" => query_connections = parse_number(&value()?)?,
                "--scenario" => scenario = Scenario::parse(&value()?)?,
                "--seed" => seed = parse_number(&value()?)?,
                "--port" => port = parse_number(&value()?)?,
                "--settle" => settle = parse_number(&value()?)?,
                "--server-env" => {
                    let raw = value()?;
                    let (key, val) = raw
                        .split_once('=')
                        .ok_or_else(|| format!("--server-env wants K=V, got {raw:?}"))?;
                    server_env.push((key.to_string(), val.to_string()));
                }
                "--skip-build" => skip_build = true,
                "--keep-data" => keep_data = true,
                "-h" | "--help" => return Err(USAGE.to_string()),
                other => return Err(format!("unknown argument {other:?}\n\n{USAGE}")),
            }
        }

        let (budget_bytes, budget_text) =
            budget.ok_or_else(|| format!("--budget is required\n\n{USAGE}"))?;
        let (limit_bytes, limit_text) = limit.unwrap_or((budget_bytes, budget_text.clone()));
        if limit_bytes < budget_bytes {
            return Err(
                "--limit below --budget would make the kernel enforce a ceiling the \
budget does not claim; raise the budget or the limit"
                    .to_string(),
            );
        }
        // docs/MEMORY_ATTRIBUTION.md: query is 57-77 % of the allocation traffic
        // and an ingest-only run is a different experiment. This gate's workload
        // is reads concurrent with writes, so there is nothing to configure to
        // zero here.
        if query_eps <= 0.0 {
            return Err(
                "--query-eps must be > 0: a run whose reads do not contend with its \
writes is not the workload this gate exists to measure"
                    .to_string(),
            );
        }

        let out_dir =
            out.unwrap_or_else(|| repository_root().join("target/memory_gate").join(&name));
        Ok(Self {
            budget_bytes,
            budget_text,
            limit_bytes,
            limit_text,
            name,
            out_dir,
            seconds,
            events,
            eps,
            connections,
            query_eps,
            query_connections,
            seed,
            port,
            settle,
            server_env,
            skip_build,
            keep_data,
            scenario,
        })
    }
}

fn main() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(EXIT_USAGE);
        }
    };

    let mut facts = Map::new();
    let started = Instant::now();
    let outcome = match measure(&args, &mut facts) {
        Ok(outcome) => outcome,
        Err(reason) => {
            facts.insert("not_measured_reason".to_string(), json!(reason));
            Outcome::NotMeasured
        }
    };

    let mut report = Map::new();
    report.insert("verdict".to_string(), json!(outcome.verdict()));
    report.insert("pass".to_string(), json!(outcome == Outcome::UnderBudget));
    report.insert("exit_code".to_string(), json!(outcome.exit_code()));
    report.insert(
        "gate".to_string(),
        json!({
            "validated_against": "peak `anon` from the cgroup's memory.stat",
            "not_validated_against": [
                "the engine's own arena or memtable accounting, which read 669 MiB and 111 MB \
        at the instant of the kill (docs/MEMORY_ATTRIBUTION.md)",
                "cgroup memory.peak, which includes the page cache this engine's own writes \
        create",
            ],
            "budget": args.budget_text,
            "budget_bytes": args.budget_bytes,
            "cgroup_limit": args.limit_text,
            "cgroup_limit_bytes": args.limit_bytes,
            "declared_by": "--budget, because SIGNY_MEMORY_BUDGET does not exist yet",
        }),
    );
    report.insert(
        "elapsed_seconds".to_string(),
        json!(started.elapsed().as_secs_f64()),
    );
    report.append(&mut facts);

    let rendered = serde_json::to_string_pretty(&Value::Object(report)).expect("report renders");
    let path = args.out_dir.join("gate.json");
    if let Err(error) = fs::write(&path, format!("{rendered}\n")) {
        eprintln!("warning: could not write {}: {error}", path.display());
    }
    println!("{rendered}");
    eprintln!(
        "\n{} at a declared budget of {} (cgroup limit {}) -- {}",
        outcome.verdict(),
        args.budget_text,
        args.limit_text,
        path.display()
    );
    std::process::exit(outcome.exit_code());
}

/// Drives one run and decides. Every `Err` here is a `NOT_MEASURED` with its
/// reason, so no failure path can be mistaken for a budget result.
fn measure(args: &Args, facts: &mut Map<String, Value>) -> Result<Outcome, String> {
    facts.insert(
        "workload".to_string(),
        json!({
            "shape": "the comparison bed's: ingest with reads concurrent with writes",
            "events": args.events,
            "offered_eps": args.eps,
            "ingest_connections": args.connections,
            "query_eps": args.query_eps,
            "query_connections": args.query_connections,
            "seconds_cap": args.seconds,
            "seed": args.seed,
            "otlp_eps": 0,
        }),
    );
    facts.insert(
        "run".to_string(),
        json!({
            "name": args.name,
            "build_revision": build_revision(),
            "machine_profile": machine_profile(),
            "features": "default (no memprof: its 16-byte tag per live allocation is 66-268 MiB \
        in these runs)",
            "out_dir": args.out_dir.display().to_string(),
        }),
    );

    fs::create_dir_all(&args.out_dir)
        .map_err(|error| format!("cannot create {}: {error}", args.out_dir.display()))?;
    require_cgroup_v2_memory()?;

    let server_bin = repository_root().join("target/release/signy");
    let load_bin = repository_root().join("target/release/load");
    if !args.skip_build {
        build(&["--bin", "signy", "--bin", "load"])?;
    }
    for binary in [&server_bin, &load_bin] {
        if !binary.exists() {
            return Err(format!("{} does not exist", binary.display()));
        }
    }

    let data_dir = args.out_dir.join("data");
    let _ = fs::remove_dir_all(&data_dir);
    fs::create_dir_all(&data_dir)
        .map_err(|error| format!("cannot create {}: {error}", data_dir.display()))?;

    let mut scope = Scope::start(args, &server_bin, &data_dir)?;
    let cgroup = scope.cgroup()?;
    verify_limits(&cgroup, args)?;
    facts.insert(
        "cgroup".to_string(),
        json!({
            "path": cgroup.display().to_string(),
            "unit": scope.unit.clone(),
            "memory_max": read_trimmed(&cgroup.join("memory.max")).unwrap_or_default(),
            "memory_swap_max": read_trimmed(&cgroup.join("memory.swap.max")).unwrap_or_default(),
            "mechanism": "systemd-run --user --scope, the same native cgroup v2 scope \
        scripts/run_memprof_local.sh establishes",
        }),
    );

    wait_for_ready(args, &mut scope)?;

    let sampler = Sampler::start(&cgroup);
    let harness_result = args.out_dir.join("load.json");
    let mut harness = spawn_harness(args, &cgroup, &load_bin, &harness_result)?;

    let deadline = Instant::now() + Duration::from_secs(args.seconds) + DEADLINE_SLACK;
    let mut server_exit: Option<ExitStatus> = None;
    let mut server_died_at: Option<Instant> = None;
    let mut scope_result = None;
    let harness_exit: ExitStatus;
    loop {
        match harness.try_wait() {
            Ok(Some(status)) => {
                harness_exit = status;
                break;
            }
            Ok(None) => {}
            Err(error) => return Err(format!("cannot wait on the harness: {error}")),
        }
        if server_exit.is_none()
            && let Ok(Some(status)) = scope.child.try_wait()
        {
            server_exit = Some(status);
            server_died_at = Some(Instant::now());
            // Read now, not at the end: when the OOM killer takes the last
            // process in the scope, systemd tears the cgroup down and both the
            // counter and the unit's own verdict go with it.
            sampler.sample_now();
            scope_result = scope.result();
        }
        if server_died_at.is_some_and(|at| at.elapsed() > HARNESS_GRACE) {
            let _ = harness.kill();
            harness_exit = harness
                .wait()
                .map_err(|error| format!("cannot reap the harness: {error}"))?;
            break;
        }
        if Instant::now() > deadline {
            let _ = harness.kill();
            let _ = harness.wait();
            return Err(format!(
                "the run passed its {:.0} s deadline without finishing",
                (Duration::from_secs(args.seconds) + DEADLINE_SLACK).as_secs_f64()
            ));
        }
        thread::sleep(Duration::from_millis(200));
    }

    // The workload has stopped. Everything above measured the peak of
    // *accepting* load; the settle measures the peak of still holding it.
    //
    // This phase exists because the comparison bed was OOM-killed fifteen
    // seconds after its last row was accepted, while merge consolidated the
    // parts ingest had left behind — and this gate passed the same budget,
    // because it stopped here. `docs/MEMORY_ATTRIBUTION.md` had already
    // measured one merge group's rewrite as the largest single live term.
    let ingest_ended_at = sampler.elapsed_seconds();
    if server_exit.is_none() && args.settle > 0 {
        let settle_until = Instant::now() + Duration::from_secs(args.settle);
        while Instant::now() < settle_until {
            if let Ok(Some(status)) = scope.child.try_wait() {
                server_exit = Some(status);
                // Same reason as above: systemd tears the cgroup down with the
                // last process in it, taking the counter and its own verdict.
                sampler.sample_now();
                scope_result = scope.result();
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }
    }

    let samples = sampler.stop();
    if server_exit.is_none()
        && let Ok(Some(status)) = scope.child.try_wait()
    {
        server_exit = Some(status);
        scope_result = scope.result();
    }
    let alive = server_exit.is_none();
    scope.stop();
    if !args.keep_data {
        let _ = fs::remove_dir_all(&data_dir);
    }

    let series = args.out_dir.join("anon.csv");
    if let Err(error) = write_series(&series, &samples) {
        eprintln!("warning: could not write {}: {error}", series.display());
    }

    let peak = samples
        .iter()
        .max_by_key(|sample| sample.anon)
        .ok_or_else(|| {
            format!(
                "no sample of {} was read, so peak anonymous memory is unknown",
                cgroup.join("memory.stat").display()
            )
        })?;
    let oom_kills = samples
        .iter()
        .map(|sample| sample.oom_kill)
        .max()
        .unwrap_or(0);
    let harness_report = read_json(&harness_result);

    facts.insert(
        "measured".to_string(),
        json!({
            "anon_peak_bytes": peak.anon,
            "anon_peak_mib": peak.anon as f64 / MIB,
            "fraction_of_budget": peak.anon as f64 / args.budget_bytes as f64,
            "at_seconds": peak.at_seconds,
            "file_at_anon_peak_mib": peak.file as f64 / MIB,
            "cgroup_memory_peak_mib": samples
                .iter()
                .map(|sample| sample.memory_peak)
                .max()
                .unwrap_or(0) as f64
                / MIB,
            "cgroup_memory_peak_note": "includes reclaimable page cache; recorded, not gated",
            // Which phase the peak came from is the finding, not a detail. A
            // gate that reports one number cannot say that the engine survived
            // the load and then died cleaning up after it.
            "peak_phase": if peak.at_seconds <= ingest_ended_at { "ingest" } else { "settle" },
            "ingest_ended_at_seconds": ingest_ended_at,
            "settle_seconds_requested": args.settle,
            "ingest_phase_anon_peak_mib": phase_peak_mib(&samples, |at| at <= ingest_ended_at),
            "settle_phase_anon_peak_mib": phase_peak_mib(&samples, |at| at > ingest_ended_at),
            "samples": samples.len(),
            "sample_interval_ms": SAMPLE_INTERVAL.as_millis(),
            "series_csv": series.display().to_string(),
            "cross_check_harness_anon_peak_bytes": harness_report
                .as_ref()
                .and_then(|report| report.pointer("/memory/anon_peak_bytes").cloned())
                .unwrap_or(Value::Null),
            "engine_reported_memtable_peak_mib": harness_report
                .as_ref()
                .and_then(|report| report.pointer("/gauges/memtable_bytes/peak"))
                .and_then(Value::as_f64)
                .map(|bytes| bytes / MIB),
        }),
    );
    facts.insert(
        "server".to_string(),
        json!({
            "alive_at_end": alive,
            "exit_status": server_exit.map(|status| status.to_string()),
            "systemd_scope_result": scope_result,
            "cgroup_oom_kill_events": oom_kills,
            "log_tail": log_tail(&args.out_dir.join("server.log"), 4),
        }),
    );

    // What "delivered" means is the scenario's to say. The log run counts
    // accepted events against the offered target. The metric run reads the
    // **steady** phase's acceptance and nothing else: the churn and explosion
    // phases refuse new series by design, and a gate that counted those
    // refusals as undelivered load would fail the engine for containing the
    // explosion it was handed.
    let (delivered, delivered_fraction) = match args.scenario {
        Scenario::Logs => {
            let delivered = harness_report
                .as_ref()
                .and_then(|report| report.pointer("/ingest/events_accepted"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            (delivered, delivered / args.events.max(1) as f64)
        }
        Scenario::Metrics => {
            let steady = harness_report
                .as_ref()
                .and_then(|report| report.pointer("/load/phases/steady"));
            let delivered = steady
                .and_then(|phase| phase.get("datapoints_accepted"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let fraction = steady
                .and_then(|phase| phase.get("acceptance"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            (delivered, fraction)
        }
    };
    facts.insert(
        "harness".to_string(),
        json!({
            "exit_code": harness_exit.code(),
            "verdict": harness_report
                .as_ref()
                .and_then(|report| report.get("verdict").cloned())
                .unwrap_or(Value::Null),
            "verdict_note": "recorded, not gated: the harness's own targets are latency and RSS \
        gates, and this gate's question is only the anonymous peak",
            "result_path": harness_result.display().to_string(),
            "scenario": args.scenario.name(),
            "events_accepted": delivered,
            "delivered_fraction": delivered_fraction,
            // The ladder's own numbers, so a metrics run publishes what it
            // refused beside what it accepted rather than only a fraction.
            "metric_phases": harness_report
                .as_ref()
                .and_then(|report| report.pointer("/load/phases").cloned())
                .unwrap_or(Value::Null),
            "achieved_eps": pointer_f64(&harness_report, "/ingest/achieved_eps"),
            "throttled_rate": pointer_f64(&harness_report, "/ingest/throttled_rate"),
            "error_rate": pointer_f64(&harness_report, "/ingest/error_rate"),
            "queries_answered": pointer_f64(&harness_report, "/queries/answered"),
            "queries_errors": pointer_f64(&harness_report, "/queries/errors"),
            "achieved_qps": pointer_f64(&harness_report, "/queries/achieved_qps"),
        }),
    );

    // Order matters. A kill is the answer even when the harness also failed,
    // because everything downstream of a dead server is a consequence.
    let killed_by_oom = oom_kills > 0
        || scope_result.as_deref() == Some("oom-kill")
        || (!alive && server_exit.and_then(|status| status.code()) == Some(137));
    if killed_by_oom {
        return Ok(Outcome::OomKilled);
    }
    if !alive {
        return Err(format!(
            "the server exited ({}) with no cgroup OOM event, so this run measured a crash \
rather than a budget; server.log says why",
            server_exit
                .map(|status| status.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ));
    }
    if harness_report.is_none() {
        return Err(format!(
            "the harness produced no result at {} (exit {:?}), so the offered workload is \
unknown",
            harness_result.display(),
            harness_exit.code()
        ));
    }
    if delivered_fraction < MIN_DELIVERED_FRACTION {
        return Err(format!(
            "the engine accepted {delivered:.0} of {} offered events ({:.1} %), so the budget \
was never exercised at the offered rate",
            args.events,
            delivered_fraction * 100.0
        ));
    }
    if peak.anon > args.budget_bytes {
        return Ok(Outcome::OverBudget);
    }
    Ok(Outcome::UnderBudget)
}

/// One reading of the cgroup's own accounting.
struct Sample {
    at_seconds: f64,
    anon: u64,
    file: u64,
    current: u64,
    memory_peak: u64,
    oom_kill: u64,
}

/// Peak `anon` over the samples a phase covers, in MiB, or null when the phase
/// produced none — a settle of zero seconds has no samples, and reporting 0.0
/// for that would read as "it held nothing".
fn phase_peak_mib(samples: &[Sample], in_phase: impl Fn(f64) -> bool) -> Value {
    samples
        .iter()
        .filter(|sample| in_phase(sample.at_seconds))
        .map(|sample| sample.anon)
        .max()
        .map(|anon| json!(anon as f64 / MIB))
        .unwrap_or(Value::Null)
}

struct Sampler {
    stop: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<Sample>>>,
    handle: Option<thread::JoinHandle<()>>,
    cgroup: PathBuf,
    started: Instant,
}

impl Sampler {
    fn start(cgroup: &Path) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let samples = Arc::new(Mutex::new(Vec::new()));
        let started = Instant::now();
        let handle = thread::spawn({
            let stop = stop.clone();
            let samples = samples.clone();
            let cgroup = cgroup.to_path_buf();
            move || {
                while !stop.load(Ordering::Relaxed) {
                    if let Some(sample) = read_cgroup(&cgroup, started) {
                        samples.lock().expect("sample lock").push(sample);
                    }
                    thread::sleep(SAMPLE_INTERVAL);
                }
            }
        });
        Self {
            stop,
            samples,
            handle: Some(handle),
            cgroup: cgroup.to_path_buf(),
            started,
        }
    }

    /// Seconds since sampling began, so a phase boundary can be expressed in
    /// the same clock the samples carry.
    fn elapsed_seconds(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    /// An extra reading taken off the sampler's schedule, for the moment a
    /// death is noticed and the cgroup is about to disappear.
    fn sample_now(&self) {
        if let Some(sample) = read_cgroup(&self.cgroup, self.started) {
            self.samples.lock().expect("sample lock").push(sample);
        }
    }

    fn stop(mut self) -> Vec<Sample> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        std::mem::take(&mut *self.samples.lock().expect("sample lock"))
    }
}

fn read_cgroup(cgroup: &Path, started: Instant) -> Option<Sample> {
    let stat = fs::read_to_string(cgroup.join("memory.stat")).ok()?;
    let field = |name: &str| {
        stat.lines()
            .find_map(|line| line.strip_prefix(name)?.trim().parse::<u64>().ok())
    };
    Some(Sample {
        at_seconds: started.elapsed().as_secs_f64(),
        anon: field("anon ")?,
        file: field("file ").unwrap_or(0),
        current: read_trimmed(&cgroup.join("memory.current"))
            .and_then(|text| text.parse().ok())
            .unwrap_or(0),
        memory_peak: read_trimmed(&cgroup.join("memory.peak"))
            .and_then(|text| text.parse().ok())
            .unwrap_or(0),
        oom_kill: read_trimmed(&cgroup.join("memory.events"))
            .and_then(|text| {
                text.lines()
                    .find_map(|line| line.strip_prefix("oom_kill ")?.trim().parse::<u64>().ok())
            })
            .unwrap_or(0),
    })
}

fn write_series(path: &Path, samples: &[Sample]) -> std::io::Result<()> {
    let mut out = String::from("t,anon,file,current,memory_peak,oom_kill\n");
    for sample in samples {
        out.push_str(&format!(
            "{:.2},{},{},{},{},{}\n",
            sample.at_seconds,
            sample.anon,
            sample.file,
            sample.current,
            sample.memory_peak,
            sample.oom_kill
        ));
    }
    fs::write(path, out)
}

/// The server inside its own transient cgroup scope.
struct Scope {
    unit: String,
    child: Child,
    stopped: bool,
}

impl Scope {
    fn start(args: &Args, server_bin: &Path, data_dir: &Path) -> Result<Self, String> {
        let unit = format!("signy-gate-{}", args.name);
        // A unit left in a failed state from an earlier run cannot be reused,
        // and the failure it reports would be that one, not this one.
        let _ = systemctl(&["reset-failed", &format!("{unit}.scope")]);

        let log = fs::File::create(args.out_dir.join("server.log"))
            .map_err(|error| format!("cannot create server.log: {error}"))?;
        let errors = log
            .try_clone()
            .map_err(|error| format!("cannot dup server.log: {error}"))?;

        let mut command = Command::new("systemd-run");
        command
            .args([
                "--user",
                "--scope",
                "--quiet",
                &format!("--unit={unit}"),
                &format!("-pMemoryMax={}", args.limit_bytes),
                // The comparison bed sets memswap_limit == mem_limit, which is
                // zero swap on cgroup v2. Swap would make the anonymous peak a
                // number about the host's swap policy.
                "-pMemorySwapMax=0",
                "--",
            ])
            .arg(server_bin)
            .env("SIGNY_LISTEN_ADDR", format!("127.0.0.1:{}", args.port))
            .env(
                "SIGNY_OTLP_GRPC_ADDR",
                format!("127.0.0.1:{}", args.port + 1000),
            )
            .env("SIGNY_DATA_DIR", data_dir)
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(errors));
        for (key, value) in &args.server_env {
            command.env(key, value);
        }
        let child = command
            .spawn()
            .map_err(|error| format!("cannot run systemd-run: {error}"))?;
        Ok(Self {
            unit,
            child,
            stopped: false,
        })
    }

    /// Asked of systemd rather than derived from a pid: with `--scope` the
    /// executed command is in the transient scope while its launcher is not, so
    /// a guess would silently sample the calling session's cgroup — which has no
    /// limit at all, and would report a budget that was never applied.
    fn cgroup(&mut self) -> Result<PathBuf, String> {
        let unit = format!("{}.scope", self.unit);
        for _ in 0..100 {
            if let Some(path) = systemctl(&["show", "-p", "ControlGroup", "--value", &unit])
                .filter(|value| value.starts_with('/'))
            {
                let path = PathBuf::from(format!("/sys/fs/cgroup{path}"));
                if path.is_dir() {
                    return Ok(path);
                }
            }
            if let Ok(Some(status)) = self.child.try_wait() {
                return Err(format!(
                    "the server exited ({status}) before its cgroup scope appeared"
                ));
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err(format!("{unit} never reported a cgroup"))
    }

    /// systemd's own verdict on the scope. `oom-kill` here is independent
    /// evidence of the kill, and it survives the cgroup that carried the
    /// counter.
    fn result(&self) -> Option<String> {
        systemctl(&[
            "show",
            "-p",
            "Result",
            "--value",
            &format!("{}.scope", self.unit),
        ])
    }

    fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        // Stopping the unit kills everything in the scope; killing the launcher
        // alone would leave the server running with a port bound.
        let _ = systemctl(&["stop", &format!("{}.scope", self.unit)]);
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = systemctl(&["reset-failed", &format!("{}.scope", self.unit)]);
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        self.stop();
    }
}

fn wait_for_ready(args: &Args, scope: &mut Scope) -> Result<(), String> {
    let address = format!("127.0.0.1:{}", args.port);
    let mut last = "no attempt made".to_string();
    for _ in 0..180 {
        match http_status(&address, "/ready") {
            Ok(200) => return Ok(()),
            Ok(status) => last = format!("/ready answered {status}"),
            Err(error) => last = error,
        }
        if let Ok(Some(status)) = scope.child.try_wait() {
            return Err(format!("the server exited ({status}) before it was ready"));
        }
        thread::sleep(Duration::from_millis(500));
    }
    Err(format!("the server never became ready: {last}"))
}

fn spawn_harness(
    args: &Args,
    cgroup: &Path,
    load_bin: &Path,
    result: &Path,
) -> Result<Child, String> {
    let log = fs::File::create(args.out_dir.join("harness.log"))
        .map_err(|error| format!("cannot create harness.log: {error}"))?;
    let errors = log
        .try_clone()
        .map_err(|error| format!("cannot dup harness.log: {error}"))?;
    let mut command = Command::new(load_bin);
    if args.scenario == Scenario::Metrics {
        // The paced metric phases size themselves from the run's duration:
        // a fifth steady, half churn, the rest burst and recovery, so a
        // longer gate run stretches all three rather than only the tail.
        let steady = (args.seconds / 5).max(30);
        let churn = (args.seconds / 2).max(60);
        let explosion = args.seconds.saturating_sub(steady + churn).max(30);
        command
            .env("SIGNY_LOAD_METRIC_STEADY_SECONDS", steady.to_string())
            .env("SIGNY_LOAD_METRIC_CHURN_SECONDS", churn.to_string())
            .env("SIGNY_LOAD_METRIC_EXPLOSION_SECONDS", explosion.to_string())
            // The anchor is unused by the paced phases but the harness
            // refuses a zero one, for the seeded phases' sake.
            .env("SIGNY_LOAD_METRIC_ANCHOR_NS", "1");
    }
    command
        .env("SIGNY_LOAD_TARGET", "signy")
        .env("SIGNY_LOAD_PHASE", args.scenario.phase())
        .env("SIGNY_LOAD_ADDR", format!("127.0.0.1:{}", args.port))
        .env("SIGNY_LOAD_CGROUP", cgroup)
        .env("SIGNY_LOAD_TIER", "budget-gate")
        .env("SIGNY_LOAD_SEED", args.seed.to_string())
        .env("SIGNY_LOAD_SECONDS", args.seconds.to_string())
        .env("SIGNY_LOAD_EVENTS", args.events.to_string())
        .env("SIGNY_LOAD_TARGET_EPS", args.eps.to_string())
        .env("SIGNY_LOAD_CONNECTIONS", args.connections.to_string())
        .env("SIGNY_LOAD_QUERY_EPS", args.query_eps.to_string())
        .env(
            "SIGNY_LOAD_QUERY_CONNECTIONS",
            args.query_connections.to_string(),
        )
        // Loki has no trace ingest, so the bed this shape comes from runs with
        // OTLP off and so does this.
        .env("SIGNY_LOAD_OTLP_EPS", "0")
        .env("SIGNY_LOAD_RESULT_PATH", result)
        .env("SIGNY_BUILD_REVISION", build_revision())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(errors))
        .spawn()
        .map_err(|error| format!("cannot run {}: {error}", load_bin.display()))
}

/// The limit has to be readable *and* the one that was asked for. This is the
/// check that separates "the engine held 1.6 GiB under a 2 GiB budget" from
/// "nothing was limited and the number means nothing".
fn verify_limits(cgroup: &Path, args: &Args) -> Result<(), String> {
    let max = read_trimmed(&cgroup.join("memory.max"))
        .ok_or_else(|| format!("cannot read {}", cgroup.join("memory.max").display()))?;
    if max.parse::<u64>().ok() != Some(args.limit_bytes) {
        return Err(format!(
            "{} reads {max}, not the {} bytes this run asked for",
            cgroup.join("memory.max").display(),
            args.limit_bytes
        ));
    }
    let swap = read_trimmed(&cgroup.join("memory.swap.max"))
        .ok_or_else(|| format!("cannot read {}", cgroup.join("memory.swap.max").display()))?;
    if swap.parse::<u64>().ok() != Some(0) {
        return Err(format!(
            "{} reads {swap}: with swap available the anonymous peak is a statement about the \
host's swap policy",
            cgroup.join("memory.swap.max").display()
        ));
    }
    Ok(())
}

fn require_cgroup_v2_memory() -> Result<(), String> {
    let controllers = fs::read_to_string("/sys/fs/cgroup/cgroup.controllers")
        .map_err(|error| format!("no cgroup v2 at /sys/fs/cgroup: {error}"))?;
    if !controllers.split_whitespace().any(|name| name == "memory") {
        return Err(format!(
            "the cgroup v2 root does not delegate the memory controller: {}",
            controllers.trim()
        ));
    }
    if systemctl(&["--version"]).is_none() {
        return Err(
            "systemctl --user does not answer, so no transient scope can be created; \
this gate needs a systemd user manager"
                .to_string(),
        );
    }
    Ok(())
}

fn build(target: &[&str]) -> Result<(), String> {
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
        .current_dir(repository_root())
        .args(["build", "--release"])
        .args(target)
        .status()
        .map_err(|error| format!("cannot run cargo: {error}"))?;
    if !status.success() {
        return Err(format!("cargo build --release {target:?} failed: {status}"));
    }
    Ok(())
}

fn systemctl(arguments: &[&str]) -> Option<String> {
    let output = Command::new("systemctl")
        .arg("--user")
        .args(arguments)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn http_status(address: &str, path: &str) -> Result<u16, String> {
    let socket: SocketAddr = address
        .parse()
        .map_err(|error| format!("{address} is not an address: {error}"))?;
    let timeout = Duration::from_secs(5);
    let mut stream = TcpStream::connect_timeout(&socket, timeout).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .map_err(|error| error.to_string())?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|error| error.to_string())?;
    let head = String::from_utf8_lossy(&response[..response.len().min(64)]).to_string();
    head.split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| format!("unparsable status line {head:?}"))
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn pointer_f64(report: &Option<Value>, pointer: &str) -> Value {
    report
        .as_ref()
        .and_then(|value| value.pointer(pointer))
        .cloned()
        .unwrap_or(Value::Null)
}

fn log_tail(path: &Path, lines: usize) -> Value {
    let text = fs::read_to_string(path).unwrap_or_default();
    let tail: Vec<String> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .rev()
        .take(lines)
        .map(strip_ansi)
        .collect();
    json!(tail.into_iter().rev().collect::<Vec<_>>())
}

/// The server writes coloured output when it is not on a terminal too, and the
/// escapes would be quoted verbatim into a JSON verdict nobody can read.
fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut characters = line.chars();
    while let Some(character) = characters.next() {
        if character != '\u{1b}' {
            out.push(character);
            continue;
        }
        for escape in characters.by_ref() {
            if escape.is_ascii_alphabetic() {
                break;
            }
        }
    }
    out
}

/// Baked in at compile time: this is a repository-local tool, and resolving the
/// tree from the binary's own path breaks the moment it is run through
/// `cargo run` from a subdirectory.
fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn build_revision() -> String {
    Command::new("git")
        .current_dir(repository_root())
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn machine_profile() -> String {
    let kernel = read_trimmed(Path::new("/proc/sys/kernel/osrelease"))
        .unwrap_or_else(|| "unknown kernel".to_string());
    let cpus = thread::available_parallelism()
        .map(|count| count.get().to_string())
        .unwrap_or_else(|_| "?".to_string());
    let memory = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("MemTotal:"))
                .and_then(|value| {
                    value
                        .trim()
                        .trim_end_matches(" kB")
                        .trim()
                        .parse::<u64>()
                        .ok()
                })
        })
        .map(|kilobytes| format!("{:.1} GiB RAM", kilobytes as f64 / 1_048_576.0))
        .unwrap_or_else(|| "unknown RAM".to_string());
    format!("{kernel}; {cpus} logical CPUs; {memory}")
}

fn parse_size(raw: &str) -> Result<(u64, String), String> {
    let text = raw.trim();
    let (digits, multiplier) = match text
        .char_indices()
        .find(|(_, character)| !character.is_ascii_digit())
    {
        None => (text, 1),
        Some((index, _)) => {
            let multiplier = match text[index..].trim().to_ascii_lowercase().as_str() {
                "k" | "kib" | "kb" => 1024,
                "m" | "mib" | "mb" => 1024 * 1024,
                "g" | "gib" | "gb" => 1024 * 1024 * 1024,
                "b" | "" => 1,
                other => return Err(format!("unknown size unit {other:?} in {raw:?}")),
            };
            (&text[..index], multiplier)
        }
    };
    let value: u64 = digits
        .parse()
        .map_err(|_| format!("{raw:?} is not a size"))?;
    if value == 0 {
        return Err(format!("{raw:?} is not a usable size"));
    }
    Ok((value * multiplier, text.to_string()))
}

fn parse_number<T: std::str::FromStr>(raw: &str) -> Result<T, String> {
    raw.trim()
        .parse()
        .map_err(|_| format!("{raw:?} is not a number"))
}
