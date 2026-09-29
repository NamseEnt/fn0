//! What the harness reads about the server rather than from it: resident
//! memory out of `/proc`, and the Prometheus scrape.

use std::collections::HashMap;

/// Resident memory of the **server**.
///
/// `VmHWM` is the kernel's own high-water mark, so a peak between two samples
/// cannot be missed. What this replaces polled `ps` every tenth push and took
/// `unwrap_or(0)` on failure (`bin/load.rs:850-861` before the rewrite), so a
/// server whose peak fell between two polls, or whose `ps` call failed, was
/// recorded as having used less memory than it did — and before that the poll
/// read `std::process::id()`, the harness's own RSS.
pub struct Memory {
    pub vm_rss_bytes: u64,
    pub vm_hwm_bytes: u64,
    /// Anonymous memory, when the source can separate it.
    ///
    /// A cgroup's `memory.current` and `memory.peak` include the **page cache**
    /// the cgroup's own file I/O created, which is reclaimable and is not the
    /// process's footprint. Both of these systems write a write-ahead log and
    /// then large data files, so both accumulate hundreds of megabytes of it,
    /// and a peak reported without this split reads as memory pressure that
    /// the kernel would simply have reclaimed. Measured: an ingest-only run
    /// took `memory.peak` to exactly the 2 GiB limit and was *not* killed,
    /// while the same run with the query workload on was.
    pub anon_bytes: Option<u64>,
    pub file_bytes: Option<u64>,
}

/// Where a run reads the server's resident memory from.
///
/// `/proc` is the M8 path and still the right one when the server is a process
/// beside the harness. A containerised server needs the other: the comparison
/// bed gives both systems the same cgroup memory limit, and `memory.peak` is
/// the kernel's high-water mark for that cgroup — the exact analogue of
/// `VmHWM`, and the only number that is comparable between a Rust process and
/// a Go one, since Go's heap makes RSS a property of when the GC last ran.
pub enum MemorySource {
    Proc(u32),
    Cgroup(String),
}

impl MemorySource {
    pub fn read(&self) -> Result<Memory, String> {
        match self {
            MemorySource::Proc(pid) => read_memory(*pid),
            MemorySource::Cgroup(dir) => read_cgroup_memory(dir),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            MemorySource::Proc(pid) => format!("/proc/{pid}/status VmHWM"),
            MemorySource::Cgroup(dir) => format!("{dir}/memory.peak"),
        }
    }
}

/// cgroup v2 `memory.current` and `memory.peak` for one cgroup directory.
///
/// `memory.peak` needs kernel 5.19 or newer. A kernel without it is an error
/// rather than a fallback to the sampled maximum, for the same reason the
/// `/proc` reader refuses to return zero: a peak that was never measured must
/// not be reported as a peak that was low.
pub fn read_cgroup_memory(dir: &str) -> Result<Memory, String> {
    let read = |name: &str| -> Result<u64, String> {
        let path = format!("{dir}/{name}");
        std::fs::read_to_string(&path)
            .map_err(|error| format!("{path}: {error}"))?
            .trim()
            .parse::<u64>()
            .map_err(|error| format!("{path}: {error}"))
    };
    let stat = std::fs::read_to_string(format!("{dir}/memory.stat")).unwrap_or_default();
    let field = |name: &str| -> Option<u64> {
        stat.lines()
            .find_map(|line| line.strip_prefix(&format!("{name} ")))
            .and_then(|value| value.trim().parse().ok())
    };
    Ok(Memory {
        vm_rss_bytes: read("memory.current")?,
        vm_hwm_bytes: read("memory.peak")?,
        anon_bytes: field("anon"),
        file_bytes: field("file"),
    })
}

pub fn read_memory(pid: u32) -> Result<Memory, String> {
    let path = format!("/proc/{pid}/status");
    let text = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
    let field = |name: &str| -> Option<u64> {
        text.lines()
            .find_map(|line| line.strip_prefix(name))
            .and_then(|value| {
                value
                    .trim()
                    .trim_end_matches("kB")
                    .trim()
                    .parse::<u64>()
                    .ok()
            })
            .and_then(|kilobytes| kilobytes.checked_mul(1024))
    };
    let vm_rss_bytes = field("VmRSS:").ok_or_else(|| format!("{path} has no VmRSS"))?;
    let vm_hwm_bytes = field("VmHWM:").ok_or_else(|| format!("{path} has no VmHWM"))?;
    Ok(Memory {
        vm_rss_bytes,
        vm_hwm_bytes,
        anon_bytes: None,
        file_bytes: None,
    })
}

pub type Metrics = HashMap<String, f64>;

/// Prometheus text into name -> value, where the name includes the label set
/// so `..._total{kind="put"}` is its own series.
pub fn parse_metrics(body: &[u8]) -> Metrics {
    let mut map = Metrics::new();
    let Ok(text) = std::str::from_utf8(body) else {
        return map;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        if let (Some(name), Some(value)) = (parts.next(), parts.next())
            && let Ok(value) = value.parse::<f64>()
        {
            map.insert(name.to_string(), value);
        }
    }
    map
}

pub fn counter_delta(start: &Metrics, end: &Metrics, name: &str) -> u64 {
    let start_value = start.get(name).copied().unwrap_or(0.0);
    let end_value = end.get(name).copied().unwrap_or(0.0);
    (end_value - start_value).max(0.0) as u64
}

pub fn object_store_op_delta(start: &Metrics, end: &Metrics, kind: &str) -> u64 {
    counter_delta(
        start,
        end,
        &format!("signy_object_store_operations_total{{kind=\"{kind}\"}}"),
    )
}

pub fn gauge(metrics: &Metrics, name: &str) -> u64 {
    metrics.get(name).copied().unwrap_or(0.0) as u64
}

/// Every series whose name starts with `prefix`, summed.
///
/// Loki labels its counters by tenant and by reason, so
/// `loki_discarded_samples_total` is a family rather than a series and the
/// number a comparison needs is the family's total. signy's own metrics
/// are deliberately label-free (`todo.md`, "Per-tenant usage"), so this is only
/// used on the Loki side.
pub fn sum_by_prefix(metrics: &Metrics, prefix: &str) -> f64 {
    metrics
        .iter()
        .filter(|(name, _)| name.as_str() == prefix || name.starts_with(&format!("{prefix}{{")))
        .map(|(_, value)| *value)
        .sum()
}

pub fn sum_delta(start: &Metrics, end: &Metrics, prefix: &str) -> u64 {
    (sum_by_prefix(end, prefix) - sum_by_prefix(start, prefix)).max(0.0) as u64
}

/// `name{label="value",...}` split into its label pairs, for reporting a
/// counter family broken down by one of its labels.
pub fn breakdown(
    metrics: &Metrics,
    prefix: &str,
    label: &str,
) -> std::collections::BTreeMap<String, u64> {
    let mut out = std::collections::BTreeMap::new();
    let needle = format!("{label}=\"");
    for (name, value) in metrics {
        if !name.starts_with(&format!("{prefix}{{")) {
            continue;
        }
        let key = name
            .split_once(&needle)
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(found, _)| found.to_string())
            .unwrap_or_else(|| "unlabelled".to_string());
        *out.entry(key).or_default() += *value as u64;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_high_water_mark_of_a_live_process() {
        let Ok(memory) = read_memory(std::process::id()) else {
            return;
        };
        assert!(memory.vm_rss_bytes > 0);
        assert!(memory.vm_hwm_bytes >= memory.vm_rss_bytes);
    }

    /// The failure that must not be silently zero: an unreadable process.
    #[test]
    fn a_process_that_does_not_exist_is_an_error_not_a_zero() {
        assert!(read_memory(u32::MAX).is_err());
    }

    #[test]
    fn a_counter_family_sums_and_breaks_down_by_label() {
        let metrics = parse_metrics(
            b"loki_discarded_samples_total{reason=\"rate_limited\",tenant=\"a\"} 3\n\
loki_discarded_samples_total{reason=\"rate_limited\",tenant=\"b\"} 4\n\
loki_discarded_samples_total{reason=\"too_old\",tenant=\"a\"} 5\n\
loki_discarded_samples_total_other 99\n",
        );
        assert_eq!(
            sum_by_prefix(&metrics, "loki_discarded_samples_total"),
            12.0
        );
        let by_reason = breakdown(&metrics, "loki_discarded_samples_total", "reason");
        assert_eq!(by_reason.get("rate_limited"), Some(&7));
        assert_eq!(by_reason.get("too_old"), Some(&5));
    }

    /// A cgroup that is not there must not read as a container that used no
    /// memory — the same rule the `/proc` reader is built on.
    #[test]
    fn a_missing_cgroup_is_an_error_not_a_zero() {
        assert!(read_cgroup_memory("/sys/fs/cgroup/signy-does-not-exist").is_err());
    }

    #[test]
    fn keeps_the_label_set_in_the_series_name() {
        let metrics = parse_metrics(
            b"# TYPE x counter\nsigny_object_store_operations_total{kind=\"put\"} 12\n\
signy_wal_backlog_bytes 4096\n",
        );
        assert_eq!(gauge(&metrics, "signy_wal_backlog_bytes"), 4096);
        assert_eq!(object_store_op_delta(&Metrics::new(), &metrics, "put"), 12);
    }
}
