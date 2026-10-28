use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::{Value, json};

pub const EPOCHS: [&str; 4] = ["setup", "measurement", "oracle", "reopen-oracle"];
const MAX_SAMPLES: usize = 4096;

#[derive(Clone, Debug, Serialize)]
pub struct MemoryBudget {
    pub cgroup_root: PathBuf,
    pub max_bytes: u64,
    pub high_bytes: u64,
    pub sample_interval_ms: u64,
}

impl MemoryBudget {
    pub fn validate(&self) -> Result<(), String> {
        if !cfg!(target_os = "linux") {
            return Err("backend memory enforcement requires Linux cgroup v2".to_owned());
        }
        if self.high_bytes == 0 || self.high_bytes > self.max_bytes {
            return Err("require 0 < backend memory.high <= memory.max".to_owned());
        }
        if !self.max_bytes.is_multiple_of(4096) || !self.high_bytes.is_multiple_of(4096) {
            return Err("backend memory limits must be multiples of 4096 bytes".to_owned());
        }
        if !(10..=10_000).contains(&self.sample_interval_ms) {
            return Err("pressure sample interval must be between 10 and 10000 ms".to_owned());
        }
        validate_cgroup_path(&self.cgroup_root)?;
        if !read_text(&self.cgroup_root, "cgroup.procs")?
            .trim()
            .is_empty()
        {
            return Err("backend cgroup root must contain no processes".to_owned());
        }
        if !read_text(&self.cgroup_root, "cgroup.subtree_control")?
            .split_whitespace()
            .any(|controller| controller == "memory")
        {
            return Err("backend cgroup root must delegate the memory controller".to_owned());
        }
        for entry in fs::read_dir(&self.cgroup_root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            if entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                return Err("backend cgroup root must have no existing child groups".to_owned());
            }
        }
        Ok(())
    }
}

pub struct PressureMonitor {
    configuration: MemoryBudget,
    samples: Arc<Mutex<Vec<Value>>>,
    active_epoch: Arc<Mutex<Option<String>>>,
    truncated: Arc<AtomicBool>,
    stop: Arc<(Mutex<bool>, Condvar)>,
    sampler: Option<thread::JoinHandle<()>>,
    initial: BTreeMap<String, Value>,
}

impl PressureMonitor {
    pub fn new(configuration: MemoryBudget) -> Result<Self, String> {
        configuration.validate()?;
        let mut initial = BTreeMap::new();
        configure_group(&configuration.cgroup_root, &configuration)?;
        initial.insert(
            "cell".to_owned(),
            group_snapshot(&configuration.cgroup_root),
        );
        for epoch in EPOCHS {
            let path = configuration.cgroup_root.join(epoch);
            fs::create_dir(&path).map_err(|error| format!("create {}: {error}", path.display()))?;
            configure_group(&path, &configuration)?;
            initial.insert(epoch.to_owned(), group_snapshot(&path));
        }
        let samples = Arc::new(Mutex::new(Vec::new()));
        let active_epoch = Arc::new(Mutex::new(None::<String>));
        let truncated = Arc::new(AtomicBool::new(false));
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let sampler = {
            let samples = Arc::clone(&samples);
            let active_epoch = Arc::clone(&active_epoch);
            let truncated = Arc::clone(&truncated);
            let stop = Arc::clone(&stop);
            let root = configuration.cgroup_root.clone();
            let interval = Duration::from_millis(configuration.sample_interval_ms);
            thread::Builder::new()
                .name("density-pressure".to_owned())
                .spawn(move || {
                    loop {
                        let (lock, wake) = &*stop;
                        let Ok(stopped) = lock.lock() else { break };
                        let Ok((stopped, _)) =
                            wake.wait_timeout_while(stopped, interval, |stopped| !*stopped)
                        else {
                            break;
                        };
                        if *stopped {
                            break;
                        }
                        drop(stopped);
                        let epoch = active_epoch.lock().ok().and_then(|epoch| epoch.clone());
                        if let Some(epoch) = epoch {
                            append_sample(&samples, &truncated, &root, &epoch, "periodic");
                        }
                    }
                })
                .map_err(|error| error.to_string())?
        };
        Ok(Self {
            configuration,
            samples,
            active_epoch,
            truncated,
            stop,
            sampler: Some(sampler),
            initial,
        })
    }

    pub fn worker_arguments(&self, epoch: &str) -> Result<Vec<String>, String> {
        if !EPOCHS.contains(&epoch) {
            return Err(format!("unknown memory budget epoch {epoch}"));
        }
        self.boundary("previous-worker-exited");
        *self
            .active_epoch
            .lock()
            .map_err(|_| "pressure epoch lock poisoned")? = Some(epoch.to_owned());
        self.boundary("before-worker-spawn");
        Ok(vec![
            "--memory-cgroup-path".to_owned(),
            self.configuration
                .cgroup_root
                .join(epoch)
                .display()
                .to_string(),
            "--memory-pressure-interval-ms".to_owned(),
            self.configuration.sample_interval_ms.to_string(),
        ])
    }

    pub fn boundary(&self, event: &str) {
        if let Some(epoch) = self
            .active_epoch
            .lock()
            .ok()
            .and_then(|epoch| epoch.clone())
        {
            append_sample(
                &self.samples,
                &self.truncated,
                &self.configuration.cgroup_root,
                &epoch,
                event,
            );
        }
    }

    pub fn finish(&mut self) -> Value {
        self.stop_sampling();
        self.boundary("after-worker-exit");
        let epochs = EPOCHS.into_iter().map(|epoch| {
            let final_snapshot = group_snapshot(&self.configuration.cgroup_root.join(epoch));
            let events_before = &self.initial[epoch]["memory.events.local"];
            let events_after = &final_snapshot["memory.events.local"];
            let event_delta = counter_delta(events_before, events_after);
            (epoch.to_owned(), json!({
                "initial": self.initial[epoch], "final": final_snapshot,
                "local_event_delta": event_delta,
                "kernel_peak_scope": "entire epoch cgroup lifetime; not reset at offered-load boundaries",
            }))
        }).collect::<serde_json::Map<_, _>>();
        let samples = self
            .samples
            .lock()
            .map(|samples| samples.clone())
            .unwrap_or_default();
        let cell_final = group_snapshot(&self.configuration.cgroup_root);
        json!({
            "enabled": true, "configuration": self.configuration,
            "enforcement": "Linux cgroup v2 memory.max, memory.high, memory.swap.max=0, memory.oom.group=1",
            "scope": "common parent budget spans all backend epochs and retained charges; identical child limits for each worker and descendants; driver excluded; cgroup charges are not RSS or exact retained heap; uncharged ZFS ARC and host caches remain outside this budget",
            "startup_scope": "worker attaches before runtime and backend construction; loader and argument-parsing charges incurred before migration are not retroactively transferred",
            "groups_preserved": true, "epochs": epochs,
            "cell": {"initial": self.initial["cell"],
                "event_delta": counter_delta(&self.initial["cell"]["memory.events"], &cell_final["memory.events"]),
                "final": cell_final},
            "sample_interval_ms": self.configuration.sample_interval_ms,
            "sample_limit": MAX_SAMPLES, "samples_truncated": self.truncated.load(Ordering::Relaxed),
            "samples": samples,
            "sampling_scope": "non-atomic counters; periodic and labelled boundaries; samples may miss transient peaks; inspect timestamps for gaps",
            "max_limit_scope": "kernel enforcement can temporarily overshoot; allocation failure or OOM is a failed capacity outcome, not successful admission control",
        })
    }

    fn stop_sampling(&mut self) {
        let (lock, wake) = &*self.stop;
        if let Ok(mut stopped) = lock.lock() {
            *stopped = true;
            wake.notify_all();
        }
        if let Some(sampler) = self.sampler.take() {
            let _ = sampler.join();
        }
    }
}

impl Drop for PressureMonitor {
    fn drop(&mut self) {
        self.stop_sampling();
    }
}

fn configure_group(path: &Path, configuration: &MemoryBudget) -> Result<(), String> {
    for (name, value) in [
        ("memory.max", configuration.max_bytes.to_string()),
        ("memory.high", configuration.high_bytes.to_string()),
        ("memory.swap.max", "0".to_owned()),
        ("memory.oom.group", "1".to_owned()),
    ] {
        fs::write(path.join(name), &value)
            .map_err(|error| format!("write {}: {error}", path.join(name).display()))?;
        if read_text(path, name)?.trim() != value {
            return Err(format!(
                "kernel did not retain requested {name} for {}",
                path.display()
            ));
        }
    }
    Ok(())
}

pub fn attach_worker(path: &Path) -> Result<Value, String> {
    validate_cgroup_path(path)?;
    for name in ["memory.max", "memory.high"] {
        if read_text(path, name)?
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .is_none()
        {
            return Err(format!("worker requires a positive finite {name}"));
        }
    }
    if read_text(path, "memory.swap.max")?.trim() != "0"
        || read_text(path, "memory.oom.group")?.trim() != "1"
        || !read_text(path, "cgroup.procs")?.trim().is_empty()
    {
        return Err(
            "worker requires an empty cgroup with swap disabled and group OOM enabled".to_owned(),
        );
    }
    fs::write(path.join("cgroup.procs"), std::process::id().to_string())
        .map_err(|error| format!("attach worker to {}: {error}", path.display()))?;
    let membership = fs::read_to_string("/proc/self/cgroup").map_err(|error| error.to_string())?;
    let relative_path = path
        .strip_prefix("/sys/fs/cgroup")
        .map_err(|error| error.to_string())?;
    let expected = format!("0::/{}", relative_path.display());
    if !membership.lines().any(|line| line == expected) {
        return Err(format!("worker cgroup membership mismatch: {membership}"));
    }
    Ok(
        json!({"verified": true, "pid": std::process::id(), "path": path,
        "membership": membership.trim(), "before_runtime_and_backend": true,
        "snapshot": group_snapshot(path)}),
    )
}

fn validate_cgroup_path(path: &Path) -> Result<(), String> {
    if !cfg!(target_os = "linux") || !path.is_absolute() || path == Path::new("/sys/fs/cgroup") {
        return Err("memory budget requires a delegated non-root Linux cgroup v2 path".to_owned());
    }
    let canonical =
        fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if canonical != path
        || !path.starts_with("/sys/fs/cgroup")
        || read_text(path, "cgroup.type")?.trim() != "domain"
    {
        return Err("memory budget path must be a canonical cgroup v2 domain".to_owned());
    }
    Ok(())
}

fn read_text(path: &Path, name: &str) -> Result<String, String> {
    fs::read_to_string(path.join(name))
        .map_err(|error| format!("read {}: {error}", path.join(name).display()))
}

pub fn group_snapshot(path: &Path) -> Value {
    let mut snapshot = serde_json::Map::new();
    snapshot.insert("timestamp_unix_ms".to_owned(), json!(unix_time_ms()));
    for name in [
        "memory.current",
        "memory.peak",
        "memory.max",
        "memory.high",
        "memory.swap.current",
        "memory.swap.max",
        "memory.oom.group",
    ] {
        snapshot.insert(
            name.to_owned(),
            match read_text(path, name) {
                Ok(text) => text
                    .trim()
                    .parse::<u64>()
                    .map_or_else(|_| json!(text.trim()), |number| json!(number)),
                Err(reason) => json!({"status": "unavailable", "reason": reason}),
            },
        );
    }
    for name in [
        "memory.stat",
        "memory.events",
        "memory.events.local",
        "memory.swap.events",
        "cgroup.events",
        "cpu.stat",
    ] {
        snapshot.insert(
            name.to_owned(),
            match read_text(path, name) {
                Ok(text) => parse_counters(&text),
                Err(reason) => json!({"status": "unavailable", "reason": reason}),
            },
        );
    }
    for name in ["memory.pressure", "io.pressure", "cpu.pressure"] {
        snapshot.insert(
            name.to_owned(),
            match read_text(path, name) {
                Ok(text) => parse_pressure(&text),
                Err(reason) => json!({"status": "unavailable", "reason": reason}),
            },
        );
    }
    snapshot.insert(
        "cgroup.procs".to_owned(),
        match read_text(path, "cgroup.procs") {
            Ok(text) => json!(
                text.split_whitespace()
                    .filter_map(|pid| pid.parse::<u32>().ok())
                    .collect::<Vec<_>>()
            ),
            Err(reason) => json!({"status": "unavailable", "reason": reason}),
        },
    );
    Value::Object(snapshot)
}

fn parse_counters(text: &str) -> Value {
    Value::Object(
        text.lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                Some((
                    fields.next()?.to_owned(),
                    json!(fields.next()?.parse::<u64>().ok()?),
                ))
            })
            .collect(),
    )
}

fn parse_pressure(text: &str) -> Value {
    Value::Object(
        text.lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                let category = fields.next()?.to_owned();
                let values = fields
                    .filter_map(|field| {
                        let (name, value) = field.split_once('=')?;
                        let value = if name == "total" {
                            json!(value.parse::<u64>().ok()?)
                        } else {
                            json!(value.parse::<f64>().ok()?)
                        };
                        Some((name.to_owned(), value))
                    })
                    .collect::<serde_json::Map<_, _>>();
                Some((category, Value::Object(values)))
            })
            .collect(),
    )
}

fn counter_delta(before: &Value, after: &Value) -> Value {
    let Some(before) = before.as_object() else {
        return Value::Null;
    };
    let Some(after) = after.as_object() else {
        return Value::Null;
    };
    Value::Object(
        after
            .iter()
            .filter_map(|(name, value)| {
                let delta = value.as_u64()?.checked_sub(before.get(name)?.as_u64()?);
                Some((name.clone(), json!(delta)))
            })
            .collect(),
    )
}

fn append_sample(
    samples: &Mutex<Vec<Value>>,
    truncated: &AtomicBool,
    root: &Path,
    epoch: &str,
    event: &str,
) {
    let snapshot = group_snapshot(&root.join(epoch));
    if let Ok(mut samples) = samples.lock() {
        if samples.len() < MAX_SAMPLES {
            samples.push(json!({"epoch": epoch, "event": event, "snapshot": snapshot,
                "cell_snapshot": group_snapshot(root)}));
        } else {
            truncated.store(true, Ordering::Relaxed);
        }
    }
}

pub fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_keeps_microsecond_totals_as_integers() {
        let parsed = parse_pressure(
            "some avg10=1.25 avg60=0.00 avg300=0.00 total=9007199254740993\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=17\n",
        );
        assert_eq!(
            parsed["some"]["total"].as_u64(),
            Some(9_007_199_254_740_993)
        );
        assert_eq!(parsed["some"]["avg10"], 1.25);
        assert_eq!(parsed["full"]["total"], 17);
    }

    #[test]
    fn missing_or_reset_events_are_not_reported_as_zero() {
        let before = parse_counters("high 5\noom 1\noom_kill 0\n");
        let after = parse_counters("high 3\noom 2\noom_kill 1\nnew_counter 7\n");
        let delta = counter_delta(&before, &after);
        assert!(delta["high"].is_null());
        assert!(delta.get("new_counter").is_none());
        assert_eq!(delta["oom"], 1);
        assert_eq!(delta["oom_kill"], 1);
    }

    #[test]
    fn rejects_regular_directories_instead_of_simulating_enforcement() {
        let directory = tempfile::TempDir::new().unwrap();
        assert!(attach_worker(directory.path()).is_err());
        let budget = MemoryBudget {
            cgroup_root: directory.path().to_owned(),
            max_bytes: 64 * 1024 * 1024,
            high_bytes: 32 * 1024 * 1024,
            sample_interval_ms: 100,
        };
        assert!(PressureMonitor::new(budget).is_err());
    }
}
